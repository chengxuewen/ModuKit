use modukit_core::bundle::BundleError;
use modukit_core::error::{KernelError, ManifestError};
use modukit_core::registry::RegistryError;

// ---------------------------------------------------------------------------
// Error trait plumbing: std::error::Error impls and From conversions are
// adjudicated behavior too.
// ---------------------------------------------------------------------------

#[test]
fn kernel_and_bundle_errors_are_std_errors_and_cloneable() {
    fn assert_error<E: std::error::Error + Clone + PartialEq + std::fmt::Debug>(_e: &E) {}
    let k = KernelError::HandshakeNull;
    let b = BundleError::NotInstalled { id: "x".into() };
    assert_error(&k);
    assert_error(&b);
    assert_eq!(k.clone(), k);
    assert_eq!(b.clone(), b);
}

#[test]
fn from_impls_wrap_without_loss() {
    let m = ManifestError::BadDuration("9z".into());
    assert_eq!(
        KernelError::from(m.clone()),
        KernelError::Manifest(m.clone())
    );
    assert_eq!(
        BundleError::from(m),
        BundleError::Manifest(ManifestError::BadDuration("9z".into()))
    );
    let r = RegistryError::AlreadyReleased { id: 5 };
    assert_eq!(
        KernelError::from(r),
        KernelError::Registry(RegistryError::AlreadyReleased { id: 5 })
    );
}
