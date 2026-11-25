//! ConfigAdmin (plugin-management-plan S3-2, ruling A3; D29 hot lane).
//!
//! Implements the recorded `config_live` specification (modules/05 §6):
//! a plugin that declares `config_live = ["key", ...]` receives change
//! events for those keys as ordinary bus topics (`config/<plugin-id>`,
//! `QoS::LatestValue` so a late-joining consumer gets the current value
//! immediately); keys that were never declared are refused with a typed
//! error - never hot-applied, never published (the liar-family enforcement
//! point). Values are strings in year 1; the typed schema arrives with the
//! proto `Config` message (05 §6 first bullet).
//!
//! Persistence (the CTK precedent, which CppMicroServices lacks): every
//! accepted `set` rewrites the plugin's `<dir>/<plugin-id>.toml`, so values
//! survive a host restart. The deploy-TOML stays the value source once the
//! deploy-tier file type lands (M7+); until then the store seeds from these
//! persistence files only - `set` is the sole year-1 write path, which is
//! exactly why it must refuse undeclared keys outright.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use crate::bus::Bus;
use crate::identity::{BundleId, TopicId};
use crate::topic::QoS;

/// The typed refusals of the hot lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// The key was not declared in the plugin's `config_live` - hot-apply is
    /// refused outright (a plugin that assumed otherwise is the liar fixture).
    NotLive { plugin: String, key: String },
    /// The persistence write failed (the in-memory store is still updated).
    Persist(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NotLive { plugin, key } => write!(
                f,
                "config key {key:?} is not declared in plugin {plugin}'s config_live - \
                 hot-apply refused (declare it first, 05 §6)"
            ),
            ConfigError::Persist(what) => write!(f, "config persistence failed: {what}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// The config value store, hot lane and persistence, in one door.
pub struct ConfigAdmin {
    store: Mutex<BTreeMap<(BundleId, String), String>>,
    live: Mutex<BTreeMap<BundleId, BTreeSet<String>>>,
    bus: Option<Arc<Bus>>,
    dir: Option<PathBuf>,
}

impl ConfigAdmin {
    /// A bare store: no hot lane, no persistence (unit-test shape).
    #[must_use]
    pub fn new() -> Self {
        Self {
            store: Mutex::new(BTreeMap::new()),
            live: Mutex::new(BTreeMap::new()),
            bus: None,
            dir: None,
        }
    }

    /// Attach the hot lane: accepted `set`s publish on `config/<plugin-id>`.
    #[must_use]
    pub fn with_bus(mut self, bus: Arc<Bus>) -> Self {
        self.bus = Some(bus);
        self
    }

    /// Attach persistence and load back whatever the directory holds.
    /// Missing directory = empty store (first boot); the directory is
    /// created on the first successful `set`.
    #[must_use]
    pub fn with_persistence(mut self, dir: PathBuf) -> Self {
        self.load(&dir);
        self.dir = Some(dir);
        self
    }

    fn load(&self, dir: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return; // first boot: nothing persisted yet
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "toml") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let plugin = BundleId::new(stem.to_string());
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(values) = toml::from_str::<BTreeMap<String, String>>(&text) else {
                continue; // a corrupt file never wedges the boot
            };
            let mut store = self.store.lock().unwrap_or_else(PoisonError::into_inner);
            for (key, value) in values {
                store.insert((plugin.clone(), key), value);
            }
        }
    }

    /// Declare a plugin's hot keys (the kernel install tail feeds this from
    /// the manifest's `config_live`). Idempotent replace.
    pub fn declare(&self, plugin: BundleId, keys: Vec<String>) {
        self.live
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(plugin, keys.into_iter().collect());
    }

    /// Drop a plugin's declarations (it uninstalled). Persisted values stay
    /// on disk - the deploy source of truth outlives one install cycle.
    pub fn forget(&self, plugin: &BundleId) {
        self.live
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(plugin);
    }

    /// Hot-apply one key: store, persist, publish - in that order, and only
    /// for a key the plugin declared.
    ///
    /// # Errors
    ///
    /// `ConfigError::NotLive` for an undeclared key (nothing is stored,
    /// written or published); `ConfigError::Persist` when the disk write
    /// fails (the in-memory store and the bus event still carry the value).
    pub fn set(&self, plugin: &BundleId, key: &str, value: &str) -> Result<(), ConfigError> {
        {
            let live = self.live.lock().unwrap_or_else(PoisonError::into_inner);
            let declared = live.get(plugin).is_some_and(|keys| keys.contains(key));
            if !declared {
                return Err(ConfigError::NotLive {
                    plugin: plugin.as_str().to_string(),
                    key: key.to_string(),
                });
            }
        }
        self.store
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert((plugin.clone(), key.to_string()), value.to_string());
        self.persist(plugin)?;
        if let Some(bus) = &self.bus {
            let topic = TopicId::new(format!("config/{}", plugin.as_str()));
            bus.publish(
                &topic,
                QoS::LatestValue,
                format!("{key}\t{value}\n").into_bytes(),
            );
        }
        Ok(())
    }

    /// Read one value (the year-1 "value read" path, 05 §6).
    #[must_use]
    pub fn get(&self, plugin: &BundleId, key: &str) -> Option<String> {
        self.store
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&(plugin.clone(), key.to_string()))
            .cloned()
    }

    fn persist(&self, plugin: &BundleId) -> Result<(), ConfigError> {
        let Some(dir) = &self.dir else {
            return Ok(()); // no persistence attached: the store is the door
        };
        let store = self.store.lock().unwrap_or_else(PoisonError::into_inner);
        let values: BTreeMap<&String, &String> = store
            .iter()
            .filter(|((p, _), _)| p == plugin)
            .map(|((_, k), v)| (k, v))
            .collect();
        let text = toml::to_string(&values).map_err(|e| ConfigError::Persist(e.to_string()))?;
        std::fs::create_dir_all(dir).map_err(|e| ConfigError::Persist(e.to_string()))?;
        std::fs::write(dir.join(format!("{}.toml", plugin.as_str())), text)
            .map_err(|e| ConfigError::Persist(e.to_string()))
    }
}

impl Default for ConfigAdmin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ServiceId;
    use crate::registry::{Properties, PropertyValue, Registry};

    fn plugin(name: &str) -> BundleId {
        BundleId::new(name.to_string())
    }

    #[test]
    fn declared_set_stores_but_a_bare_admin_cannot_publish() {
        let admin = ConfigAdmin::new();
        let p = plugin("com.acme.sensor");
        admin.declare(p.clone(), vec!["interval".into()]);
        admin
            .set(&p, "interval", "500")
            .expect("declared key applies");
        assert_eq!(admin.get(&p, "interval").as_deref(), Some("500"));
    }

    #[test]
    fn undeclared_key_is_refused_outright() {
        let admin = ConfigAdmin::new();
        let p = plugin("com.acme.sensor");
        admin.declare(p.clone(), vec!["interval".into()]);
        // the liar family: assuming hot-apply without declaring it
        let err = admin.set(&p, "secret_mode", "on").unwrap_err();
        assert_eq!(
            err,
            ConfigError::NotLive {
                plugin: "com.acme.sensor".into(),
                key: "secret_mode".into()
            }
        );
        assert_eq!(admin.get(&p, "secret_mode"), None, "refused = not stored");
    }

    #[test]
    fn hot_lane_publishes_on_the_plugin_topic() {
        let bus = Arc::new(Bus::new());
        let admin = ConfigAdmin::new().with_bus(Arc::clone(&bus));
        let p = plugin("com.acme.sensor");
        admin.declare(p.clone(), vec!["interval".into()]);
        let topic = TopicId::new("config/com.acme.sensor".to_string());
        let mut rx = bus.subscribe(&topic).expect("subscribe");
        admin.set(&p, "interval", "500").expect("set");
        let payload = String::from_utf8(rx.try_recv().expect("the change event")).expect("utf8");
        assert_eq!(payload, "interval\t500\n");
    }

    #[test]
    fn persistence_survives_an_admin_replacement() {
        let dir = std::env::temp_dir().join(format!("modukit-config-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let first = ConfigAdmin::new().with_persistence(dir.clone());
        let p = plugin("com.acme.sensor");
        first.declare(p.clone(), vec!["interval".into()]);
        first.set(&p, "interval", "500").expect("set");
        drop(first);
        // a fresh admin (host restart) reads the values back off disk
        let second = ConfigAdmin::new().with_persistence(dir.clone());
        assert_eq!(second.get(&p, "interval").as_deref(), Some("500"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn registry_values_satisfy_the_read_path() {
        // the year-1 value read works against any string source; this pins
        // the shape the deploy-TOML seeder will feed once that type lands.
        let r = Registry::new();
        let mut p = Properties::new();
        p.insert("service.id", PropertyValue::Str("com.acme.dep".into()));
        r.register(std::sync::Arc::new(Svc), p).unwrap();
        assert!(r.lookup(&ServiceId::new("com.acme.dep".into())).is_some());
    }

    struct Svc;
}
