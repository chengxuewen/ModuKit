# ModuKit — History Archive

> Full chronological History log, moved out of `status.md` on 2026-10-14 (the per-turn-loaded file must stay small). Append NEW entries at the TOP of this file; status.md keeps only a pointer. Live state lives in status.md Phase/Open Items.

- 2026-10-21 (Stage-4 M3 LANDED, the service-call door): wave-2 opened and its first candidate closed the SAME DAY on maintainer word - three rulings walked A/A/A in plain speech (PIT-21 landing earned by that register): door-only scope, one verb modukit_sup_service_call mirroring the dl face contract (embed.h 23->24, D83 tail-add, ABI major stays 1), fidelity stage inside the existing m2 demo (no new script, no new leg per W3). The mechanism is the reuse the ruling demanded: acquire::<ProxyService> on the supervisor kernel's registry - the same parent-side door dual_placement proves on the rs side, no second dispatch path. The FIDELITY stanza makes a real design point: the parent now WAITS the fidelity marker before the kill, which is what makes post-READY words observable at all - and the first thing it observed was a lie: the M2 demo's dial-REFUSAL assertion sat on a never-observed dead branch (old parent killed at READY before those lines; runner boots Step), corrected in-place to the typed-turn truth (PIT-22 minted from the finding). DEMO_STAGE4_M3_SABOTAGE planted red once (evidence kept at /tmp/m3-sabotage-red.log; the M2 hook re-verified red beside it); orchestrator re-measured everything: 226 tests / 47 suites from redirected file, fresh-tree ctest 9/9, check.sh all green (24-verb parity, ledger 24/40, doc-refs 59/0), zero strays. Lane note on record: regen-golden.sh never passes --embed_header (pre-existing gap, registered in plan section 6). Commits: code + ledger pair, ahead grew with them (push stays maintainer word).

- 2026-10-20 (Stage-4 M2 CLOSED, exe rung): the C++ stack now supervises children - seven sup_* verbs tail-added to HostEmbedV1 over a second ambient Supervisor (two rungs, two registries, wall asserted from both faces), and host_sup_demo proves ORPHAN-PROOF under a C++-authored host (SIGKILL -> runner self-reap census; sabotage red observed with the survivor pid named). M2a ran inline after the delegation channel died twice more (session-abort signature; provider deaths total 7 today). Three first-contact landmines of the second language family, all root-caused with socketpair repros and temp probes (each fixed at the shared source, not patched around): RTLD_NODELETE on the dlopen door (probe-drop + reload re-ran C++ static initializers -> protobuf descriptor-pool SIGABRT), to_snake wire naming single-sourced into the cpp emitter (it said "Echo" where cabi+runner said "echo"), and the runner self-check guard de-shrined (empty-probe + pass-on-Ok: byte-equality echo was the raw-bytes twin's law pretending to be universal; typed replies are legitimately non-empty; fidelity belongs to the parent-side proxy round-trip - CORRECTED by the gate review: that sentence overclaimed; the rs family IS gated there (dual_placement) but cpp x exe has NO byte-fidelity gate (no service-call door on the sup face) - recorded as plan §6 candidate). The same review caught the first "ctest 9/9" measured on a stale build tree (fresh = 8/9: the wire sweep had missed probe.cc "Echo" - now fixed, second-pass sweep single-sourced the rust skeleton too); post-fix fresh ctest is 9/9 measured clean. Ledger/parity/goldens moved with the tail-add as designed. 225 tests, check.sh green; all pushed 2026-10-20.

- 2026-10-19 (Stage-4 M1 CLOSED, same-process rung): six-lane build-out in one day despite five provider channel deaths (librarian plan-trace-only x2, rps x3) - all absorbed by the PIT-14 rule (verdict from tree + own gates; session-resume retries; zero work lost). Landed: probe (protoc C++ + fdset byte-equal) -> embed.h/plugin.h contract originals + three-way parity legs -> C++ emitters both faces (goldens) -> cpp fixtures + bindings/cpp Visia-grammar shelf + live two-arm probe -> HostEmbedV1 Rust impl (16 verbs, process-global kernel, catch_unwind at every FFI edge, codec split) -> host_demo full v1 word list + demo-stage4-m1 gate leg (sabotage RED observed) -> capability ledger + gate. Cargo.toml first structural change (crate-type staticlib - C ABI exit reaching a C++ exe; judged in-policy, recorded in commit+plan). History-repair catch of the day: TWO older entries had swallowed headers (replace-without-anchor-rejoin, 2026-10-19 write-back batch) - the assert verified the anchor existed, never that it came back; fixed + entry-continuity grep added to the habit. Gaps on record: registry acquire-by-id absent, bus size-probe pending-pop, single global kernel. ctest 8/8, 219 tests, check.sh all green, 8 unpushed commits.
- 2026-10-19 (M3S-6 holdings lane): the last "adjudicated but unlanded" pair on the books - stage-2 §9 frictions 1-2 - landed on maintainer sanction. Manifest::dl_internal (D83 tail-add; equivalence test pins field-for-field match to a minimal TOML parse) + DlopenExecutor::probe_self_report/SelfReport (guards MOVED verbatim into shared open_validated - load keeps kind-check+box; runner's placeholder-manifest throwaway at the probe site deleted; install-site manifest deliberately untouched because team/version differ from dl_internal scope). Deliberately NOT done: double-dlopen elimination (hearing-grade shape surgery), bundle.rs probe re-plumb, test-helper conversions (TOML witnesses stay). Agent 9m run + full orchestrator re-verify: 198 tests (45 suites), clippy -D clean, fmt clean, both demo gates PASS, files all under cap. Commits a77b712 + 2ff7c97. Second agent-delivery truncation this box (cut mid-work-stream at the return channel) - tree truth + own gates decided, not the report.

- 2026-10-19 (D92 / static-placement brainstorm): maintainer brainstorm on build-time plugins (no artifact shelf, everything linked at build) -> prior-art survey -> six adjudications walked one by one, all ruled (see D92). Evidence honesty: two librarian background runs died model-side (planning trace only, zero tool calls); box has no egress (curl + webfetch both dead) - survey landed with confidence tiers, local .refinfo/CppMicroServices pinned three claims to primary (static_bundles.rst "no API differences"; Bundle.cpp:148 Uninstall has no static special case; BundlePrivate.cpp:727 manifest parsed at runtime from embedded resource). Roadmap gained item 6 (candidate, not a stage); numbering gate widened to D1-D92 with observed red/green before minting. Zero code touched; the 5 pending commits and the K7 note remain maintainer calls.

- 2026-10-16 (M8.6): demo shelf - maintainer's second layout finding; stage1_demo moved from modukit-core's pocket into workspace member examples/ (package modukit-examples; rs band per VisiaEngine grammar; lane retargeted to FOLDER examples/rs, C1 scan scope gained examples/, repo_root depth fixed; first-red captured on the old path; graph 7 edges + ctest 6/6 unchanged).

- 2026-10-16 (PIT-7 / CMP0174): maintainer's VSCode configure explosion traced to PolyOrch's six un-guarded cmake_policy(SET CMP0174) calls (policy born in cmake 3.31; the declared 3.25 embed floor was a lie no suite exercised) + ambient-tool drift (IDE ran /usr/bin/cmake 3.30 while every scripted gate used pixi's 4.4). Fixed on dev-modukit, control-proven: 4.4 suite identical 74/5/29 (5 fails pre-existing at tip on this env), 3.30 improves 27/56 -> 35/48; maintainer push + main-merge + pin bump pending per C2. Host: .vscode/settings.json pins IDE to the pixi cmake + out/build (its .gitignore whitelist fixed: dir-exclusion vs /*-exclusion); user repro command now configures AND builds green on 3.30.7. PIT-7 recorded; edit-safety annotated (fragment-replace trap hit this session's own status.md twice).

- 2026-10-16 (M8.5): lane wiring landed - D66 fulfilled after the maintainer's zero-target finding; W0 shelf split (first-red: cross-shelf relative paths broke), W1 pixi translation live (configure prints route+cargo), W2-W4 five crate lanes + example lane + 6 ctest bridges on the graph; codemodel proves the 5 FOLDER shelves exactly (aggregate FOLDER-less per T-H10); run button inherits shell cargo (mise-shim red documented, ctest immune via CARGO injection); file-API query lands next-configure (noted); ctest 6/6 + clean-cycle green; drifts logged: codegen = 2 handles, plugins/ still grammar-only.

- 2026-10-16 (M8 + Stage-1 close): acceptance landed - end-to-end demo as a real gate (public APIs only, full chain from cdylib build to negatives incl. tampered-refuses-pre-root + ABI-mismatch FAILED with D27 record), falsifiability proven via fake-cargo probe; coverage 78.6% vs 60% D50 soft gate MET through a dedicated pixi cov env (own solve-group; cargo-llvm-cov from conda-forge, no cargo-install, workspace deps untouched); §10 In-list mapped 1:1, GAP-1 (named dual-placement contract suite) ruled satisfied-by-structure - exe half ships with Stage-2's ProcessExecutor; STAGE-1 COMPLETE: M1-M8 all landed on master, 108 tests, first-red evidence per milestone.

- 2026-10-16 (M7): bundle install protocol landed - hand-written SHA-256 (NIST vectors caught a real K[26] constant typo), D28 staging->verify-all-hashes->inode-dedupe(444)->atomic-rename, strict D7 cross-check via the untouched DlopenExecutor (missing self-report = fail), shared reverse-DNS grammar (publish hot path free), fdset-equality seam keeping dump() frozen; 108 tests, 6 first-reds; ambiguities into the plan row (integrity codes ride CONTRACT_VIOLATION until a D51 hearing; provides-summary cross-check waits for a descriptor that carries one; zip = D28 CLI trigger).

- 2026-10-15/16 (stage-1 build-out M2-M6 + governance): C2 corrected per maintainer (ModuKit lands on master; branch discipline = PolyOrch upstream dev-modukit -> main -> test; PIN POLICY later same day: gitlink follows dev-modukit tip, kept fresh - three bumps landed a246861/a620933 as first exercises, ff-only, dirty=0). C3 commit-shape rule adopted (subject <=60, ' - ' body bullets <=100; negative proof bit even the rule's own first attempt - missing blank line + wrapped bullets caught by its own gate; 24-commit local history reworded via filter-branch msg-filter with tree-hash-identical proof + zero-MISS dry run). M5 landed (topic bus D10 with kernel-owned QoS + drop-oldest ring receivers; std-only modukit-transport D11/D14: Dummy/Socketpair contract suite, memfd shim, PoolStub EPOCH_EXPIRED two-stage views; vendor-lock test; 60->88 with M6). M6 landed (clock/events/logger/edge-ledger/counters/introspect-trio/testhost; timers POST never call; testhost step-clock zero-sleeps; acceptance RED-fixture = bad plugin FAILED + zero ModuleLoaded; 8 floors-of-evidence first-reds x6; 8 ambiguities adjudicated in plan row - ledger=projection not call-table, fdset-equality rides M7/M8). fast-model family balance deaths again at audit-team time (fallback chain recovered). check.sh ran RED once mid-flight while M6 agent worked the tree (PIT-14 near-miss: orchestrator ran full gates over a half-written tree - rule going forward: agent in flight = C1/skill-lint class only, full gates at acceptance).

- 2026-10-15 (audit): doc-audit FULL round 3 (team mode, 4 premium members + lead inline dimension-5; clean closure). Merged findings 0 CRIT / 3 HIGH / 5 MED / 8 LOW, all user-approved as batch, fixed in 5 thematic commits (fac3d1c..c6af6b2). Headlines: card-11 dead link (lead self-inflicted, repaired + table integrity re-probed), doc-audit SKILL's own stale scope/claims (00-10, superpowers floats, gate.sh-only scripts, 4-member example), register strikes (A6 -> D63, §5 fork -> D25, metadata-first -> D63 were still listed pending across status/backlog/00-overview), index fold-in IDs D63-D73 across modules rows, dated audit notes on profile-snapshot drift (iceoryx2 'unadjudicated fork', accessbase gate.sh open-item), PIT-11 provenance qualification, check.sh wrapper-intent comment, D53 tombstone correction, D69 on-host/off-host qualifier (the 'ANY placement vs D15 tail-arm' wording collision found by decision-validator). F16 (probe 'empty set' wording) verified NO LIVE TARGET - the line had already left status.md; naive re-run legitimately returns modukit-dj (whitepaper:55 name-collision warning, initial commit, git log -S proved). Gates green before AND after (check.sh all green, skill-lint 12/0/0, C1, numbering, size). Tree clean, local ahead 14, unpushed.
- 2026-10-15 (adjudication): two walkthrough batches, 7 items, user-adjudicated one by one. PolyOrch batch (votes A/A/A/A, D64-D67): immediate-policy adoption as build-orchestration layer (physical gitlink submodule on maintainer call), mise/pixi environment split (mirrors D57 deploy-side), full-shape VisiaEngine root assembly (five-part, embed floor 3.25, zero-intrusion keeps crates pure Cargo), gitlink-single-authority pinning + naming-grammar adoption + two-surface FFI refusal; profile docs/reference/polyorch.md written (measured root 5b5ad89 + consumer wiring 3cbc155). Media-server base batch (votes A/A''/C/C/C/C/B, D68-D73): control-plane P1 posture (industry 7:1 against frames crossing plugin ABIs; Janus/OBS costed counter-examples), frame clause D69 promoted from modules/01 rules 3/7 + 07 §3 (pool+descriptor channel only; GStreamer refcounted-handle shape absorbed as pool handles, raw-pointer arguments refused), node factories ride plugin contracts with ephemeral work-unit registration (registry refusal line stands), parallel_ok coexistence stays inert with doorbell wording, capability-vocab audit passed (device entrances ride resources{} + soft words, iron rule intact), integration posture parallel-not-wired (intent declared, library-first direction when kernel API stabilizes); doctrine card docs/modules/11-media-server-base.md (thin, pointers only). fast-model family balance deaths x3 chains recovered via fallback + one Aborted; VisiaEngine build/codegen sub-questions answered inline in sandbox.
- 2026-10-14 (later): maintainer approved fix batches + stale-task cleanup. H1: this History block moved verbatim from status.md (47924 -> 4144 bytes, 51 bullets; new size-discipline check added to status Verification, red 47924 / green 4144 both observed). H2: rust rules fmt contradiction fixed in BOTH coding-style + hooks (per-file rustfmt --edition 2024; workspace cargo fmt forbidden per edit-safety #12). Phantoms: ecosystem-scan items 7/8/14/15 re-scoped (SKILL.md registry via skill-lint + status/backlog pointer sync replace the nonexistent AGENTS SKILL DIRECTORY + context-engineering routing refs); doc-audit D1-D61 hardcount replaced by live probe command. Plus skill-router phantom section re-scoped during residual-pointer scan (context-engineering routing table absent in this repo; passive matching happens via skill frontmatter + SKILL.md registry). LOW: edit-safety #10 duplicate Blocking line removed; content-auditor's git-workflow.md dead-link finding DISPROVED at verification (file absent repo-wide) and dropped - finding-level false positive, honest note per verification-before-claims. Plugins: opencode.json now ponytail@5.2.0, oh-my-openagent@5.1.33, superpowers pinned @v7.0.0 (tag existence verified via refs API); context-mode left at 1.0.169 (1.19.0 figure unverified for this package line). All pending restart = backlog R. P1: vendored ECC rust-patterns (13500 B) + rust-testing (11798 B) via SHA-pinned raw fetch @b92f181b3b74, byte-verified frontmatter + CJK-clean; registered in SKILL.md (10 -> 12 skills); See-skill rust pointer now live; ECC repo-trust flagged star-growth anomalous but files gate-passed. Residual (trigger-gated, not applied): rule-pack runnable-command real execution (needs crates/), per-turn instruction budget ~10-15k tokens (Stage-1 review), jscpd + addyosmani cherry-picks (Stage-1), eval harness (Stage-2), S2 global apiKey rotation (user-side).
- 2026-10-14: ecosystem-scan FULL rerun (team mode: 4 premium members + lead synthesis; backlog P5 closed; closure sweep clean, force:true needed post-approvals - PIT-3 tail). Installed system: NO BLOCK, 4 WARN - (S2 top) user-global ~/.config/opencode/opencode.jsonc carries plaintext apiKey mode 0664 + http no-TLS gateway (machine-level, repo clean; fix = env-interpolation + chmod 600 + rotate, user action); codegraph installer .sh curl-pipe-sh at main + .bat irm-iex (binary pinned v1.6.2, installer ref floating); arxiv MCP uvx unpinned; superpowers floating git-ref measured moved 6.4.2 -> 7.0.0 silently under us. External: Rust FFI/C-ABI skill gap RE-CONFIRMED unfilled (ours to write); affaan-m/ECC finally reachable and profiled (278.8k stars MIT active; rust-patterns + rust-testing top live vendor candidates; star-growth flagged anomalous); addyosmani/agent-skills (106.4k MIT) x3 cherry-picks = Stage-1 trigger; jscpd (0 deps) = Stage-1 check.sh addition; skill-behavior-eval 3-source convergence = Stage-2 rewrite-to-adapt; boltffi = watch lane (fits D63 emitter-template framing); no MCPs for zenoh/iceoryx2/Foxglove/ROS (DIY confirmed); doc-audit-tools dead niche. Lead ran the 6-item gate inline (auditor idle through 2 delivered nudges - mailbox wake unreliable): all 5 candidate bodies fetched clean, zero danger patterns, frontmatter OK. Local HIGH x2: status.md History = 91% of file (~10.4k tokens/turn) violating its own size-discipline header - archive proposed; rust coding-style ALWAYS-cargo-fmt line contradicts edit-safety #12 (latent until Stage-1). MED: plugin updates available (ponytail 5.2.0, omo 5.1.33, context-mode plugin 1.19.0), doc-audit SKILL hardcount D1-D61 stale (fix = dynamic probe), ecosystem-scan items 7/15 reference nonexistent routing tables (PolyOrch-era phantoms), rule-pack checks non-runnable (vacuous without crates/), per-turn instruction budget ~10-15k tokens. LOW: git-workflow dead link x1, duplicate-passage clusters x4, edit-safety #10 duplicated line. No fixes applied - maintainer adjudication pending (execution gate).

- 2026-10-13 (continued 3): queue-zero brainstorm closed (two adjudications walked one by one, votes C / B; second re-walked after maintainer challenge): D62 fallback semantics = error-carrying stub (ServiceRef stays callable while dep absent, every call returns D27 SERVICE_UNAVAILABLE record, never blocks never fabricates; stale-cache and default-mapping demoted to consumer-side patterns per D46 routes-not-interprets + D16 honest-absence family; absence counted in D21 so degradation alarms; required+fallback still fails boot resolve; closes the 02 open item, corrects 03 'serves stale' line); D63 bindings three-tier doctrine (A6 ratified REFINED after the iceoryx2 usability challenge - generated floor with hand-written substitutes forbidden, ergonomics fixed only in emitter templates, thin per-language sugar allowed above the floor under floor-only/semantics-free/fixture-tested rules; per-language lag becomes 'sugar arrives late' never 'floor lies'; plus metadata-first ratified - .proto truth, C ABI generated projection, D13 direction made explicit, uniffi polarity challenge closed). Architecture §11 queue line rewritten (4 settled items removed, 2 remainder named trigger-gated); modules/04 gained section 6 + polarity note line; 00-overview section 4 items 4/5 marked ratified. Repo-profile evidence underpinning the re-walk: iceoryx2.md:71,82,83,106 (cbindgen floor, ffi-macros tames the footgun, python/cxx lag, cxx = sugar above floor). High-water D63, fresh D64 probe.

- 2026-10-13 (continued 2): doc-audit FULL round 2 (team mode, 4 members, five dimensions; 20 findings merged from 7+7+7+6 with dedup: 1 CRIT / 2 HIGH / 4 MED / 13 LOW / 1 INFO; all maintainer-adjudicated one by one, batch fix): F1 CRIT D50 phantom realized at last - scripts/check.sh created (five checks, skip-when-absent) + staged-coverage law across testing/code-review/development-workflow/rules-README + gate.sh->check.sh citations in edit-safety + inline secret grep; D50 record rescoped honestly (M7 family partially retired - ~/.claude and See-skill subfamilies stay open; the failed 10-12 batch commit had swallowed the artifacts, audit F1 restored only records). F2 HIGH high-water rot CURED BY REMOVAL (maintainer vote): README/AGENTS x4/modules-00/ref-README/reference-00 counts de-numeralized per the AGENTS git-line precedent 'volatile counts deliberately not recorded'; status.md Phase/Open Items keep accurate D1-D61 counts. F3 D55 index row added + D53 RESERVED footnote under architecture section 11. F4-F7 MED: 05 section 8 deploy row gains [net] listen + [ros2] fields, section 9 bridge-reads-table clause; modules-00 rows 05/07/09/10 synced (D51 misattribution fixed, D56-D61 landings listed); reference roster counts de-numeralized; edit-safety port-provenance header added (14 PolyOrch-era PIT citations registered as pointers to the reference checkout, audience line fixed PolyOrch->ModuKit). F8-F19 LOW batch: anchors retargeted (D52 to section 11 row; D16/D17 06 section 3->4 after D42 renumber), 05 6.6 recipe clause (D58 landing point), architecture date + four section-13 subjects, DeepSeek row Celix copy-paste cells removed x2, stale C9/C4-C6 citations in lesson-memory/edit-safety/think-before-act/lesson-review, pitfalls 'no entries' line + path prefix, conventions seed line rescoped, status History reordered (D55 line back to chronology) + D53/D54 line added, maintainer-calls line reads 'among others'. F20 INFO no-action (audesys frozen tense, profile-freeze convention). Gates after fix: check.sh all green, numbering D1-D61 clean, contiguity OK, dead links 0, C1 OK. Registry: SKILL.md doc-audit description untouched (skill-lint sync holds).

- 2026-10-13 (continued): Foxglove interop brainstorm closed (four rulings walked one by one, votes A / A / A / C, plus two no-contest boundaries, single record D61 - four faces of one bridge): (a) recording unchanged - D16 recorder ours over the neutral mcap crate, foxglove-sdk never enters the recording path, our three-part self-describing header cannot be carried by the SDK log format, Studio stays consumer-never-producer (industry: ros2 bag record/Autoware/gstreamer all keep own recorders over open containers); (b) live plane - foxglove-ws bridge plugin embeds the foxglove-sdk (MIT Rust) WebSocket server, app connects unchanged ws://host:port with all stock panels lit, shape pre-fabricated by D11/D35 (manifest-gated net-listen + export allowlist, copy boundary, ring audit), chained zenoh-ros2dds->foxglove-bridge rejected (double DDS tax + ROS-only + resurrects D25 worldview tax; SDK server IS the mainstream non-ROS path per Foxglove docs); (c) writeback - publish/service-call/parameters cross as ordinary clients through existing gates (D10+D26 / D4+D9 / D37 four-gate), no debug exemption (D44 no-second-channel), undeclared bridge = read-only by construction (ROS2/K8s/DB-GUI industry consensus: tools hold identities, writes go through the front door); (d) mapping - per-bridge type-mapping tables per D11 verbatim, dictionary-pack promotion (modukit-types-foxglove, D45 form) pinned at second bridge/ecosystem before the third copy (OTel extract-after-duplication sequence), central pack now rejected as YAGNI (D52 freeze would apply); (e) sdk residence (no-contest) - foxglove-sdk only inside bridge plugins, never the host link domain (D60 mirrored; host-side WS server would bypass D26 net-listen = audit blindness); (f) upload (no-contest) - Foxlet/Data Platform are site-side deployments (data-exfiltration = ops decision, Grafana/Loki never-self-built law), recorded mcap is an ordinary file outside D30's window. External facts fetched same day: Foxglove docs (components table, SDK stream+log, Foxglove WebSocket = custom-stack path, feature matrix publish/service/parameters); github fetches timed out (network), docs-site over reports. modules/07 bridge entry, 09 upload line, 10 consumer row, architecture 8 + index synced. High-water D61, fresh D62 probe.

- 2026-10-13: ROS 2 interop brainstorm closed (six adjudications walked one by one, votes A / A+C-dual-track / A / C / A / A; sixth = archive shape): D56 net-listen grammar extension (port ranges + multicast joins for DDS discovery 7400-7600 / 239.255.0.1, no new word, vocabulary stays 13; Docker -p ranges / K8s endPort cross-check; coarse per-middleware words rejected as audit porridge + floodgate); D57 pixi/conda environments (D55 class four -> vendor/, pixi.lock = the pin satisfying D33 by construction; activation = static injection default - single prefix makes it feasible - plus env_mode=activate-script opt-in escape, wrapper in the plugin process, ring-audited; host NEVER executes bundle-borne shell, permanent exclusion with container-boundary isomorphism); D58 build recipe delegation (modukit.toml [build] recipe strings, ecosystem knowledge lives in D23 templates, the framework executes strings it does not understand); D59 ROS site settings (deploy-TOML [ros2]: ROS_DOMAIN_ID/RMW_IMPLEMENTATION/ROS_LOCALHOST_ONLY = site facts, host_id logic applied to the network domain, one host one domain host-guaranteed, bundles carry no runtime values); D60 lifecycle note + discipline (stop_timeout 3s recommended rclcpp override - suggestion not mechanism, D34 untouched; ROS knowledge lives in templates/recipes/declarations, host middleware-illiterate by construction). External facts fetched same day: RoboStack linux/macos/windows distro matrix (Jazzy/Kilted/Lyrical), pixi-pack env/+activate.sh relocation without conda at target, rmw_zenoh Tier-1 packages + router-gossip discovery (multicast off by default). modules/05 gained 9 (+2.2 grammar note), 07 interop cross-ref, architecture 8 + index synced. High-water D60, fresh D61 probe. Librarian subagent died 3x (account balance); recovered via direct fetch (files over reports).

- 2026-10-12 (continued 14): build/packaging brainstorm closed (D55, direct vote): build endpoint = deploy-ready bundle (`modukit build` split from D23 wagon as first-class: detect builder -> compile -> assemble standard layout -> $ORIGIN RPATH -> dependency collection -> content-addressed dedup >1MB into .modukit/objects + hardlinks -> hash manifest -> installs via D28); dependency privatization four classes (private in / OS whitelist / per-bundle lib/ with RPATH / vendor/ venv-node_modules), no version ranges; framework = packager never builder; setup.bash env-chaining deliberately absent. High-water D55, fresh D56 probe.

- 2026-10-12 (continued 13+): Node/Web integration brainstorm closed (D54, vote C: net-listen/net-connect/workdir capability words - D26 vocabulary 10->13, napi-rs clarified binding-not-embedding, workdir boundary fixed/uninstall removes; commits b1d79a1/2dad023) + D53 RESERVED tombstone (audit-F1 numbering gap closed mechanically). High-water D54, fresh D55 probe.

- 2026-10-12 (continued 13): doc-audit FULL (team mode, five dimensions; findings 10, merged from 5 auditors + lead verification; all maintainer-adjudicated item by item): F1 CRIT five-topic phantom re-filed as D48-D52 (votes had happened; batch commit had failed; records restored verbatim + D42->api attribution fixed to D52 + status history line corrected); F2 stale high-water counts synced (AGENTS x3, README D2-D5 line, status Phase/Open Items -> D1-D52); F3 doc-audit skill re-scope block rewritten (architecture/modules DO exist; memory enumeration refreshed); F4 D41 entry vendor-leftover lines rewritten to port wording; F5 = F1 attribution fix; F6 modules/03 duplicate section 6 renumbered (Update path -> 9); F7 Celix index rows repaired (README 4th cell + 00-overview 5-column fill); F8 Open Items P2-P5 line synced with backlog truth; F9 backlog Pending table blockquote moved above header; F10 minor batch (conventions trailing line, README roadmap pointer, 07 table cell, 08 count, last-pushed-hash marker). Gates: skill-lint 10/0/0, C1 OK fail-closed, dead links 0, D-contiguity D1-D52, verification row raised to D1-D52/flag D53+. three named items (adjudications 48-50, votes C/C/C): D45 companion type packs (modukit-types-vision/motion as plain proto packages, opt-in, kernel never embeds - dictionary not grammar), D46 job-progress discipline (one latest-value topic per JobToken in org.modukit.job.*, kernel routes not interprets), D47 failover pairs (hot standby free today via D4 ranking, cold standby named trigger-gated from stock parts, general condition-startup DSL rejected per D38). modules 02/03 + architecture/index synced. High-water D47, fresh D48 probe. (D44, direct vote B): runtime control panel as ledger-native service (start/stop/restart/reap - hand-cranked twins of D3/D34 automations; pause-publish storm-triggered; gates D37 identity + D26 org.modukit.control + D35 ring audit; no second control channel); monitor ladder (year-1 `modukit status` + ctl, TUI `modukit watch` ratatui at the D32 bell, web bays at D39); monitors are pure readers of existing ledgers (dl n/a shown honestly); monitor obeys host lifecycle (oxmgr lesson inverted). modules/03 gained section 8, 10 §4 ladder; architecture/index synced. High-water D44, fresh D45 probe. (D42/D43, votes C/C): point-tags = topic usage discipline (TagEnvelope in modukit-api, voluntary, uniform rendering; run-value downlinks = publishes through D26/D37; config vs run-values two lanes never merge; deadbands publisher-side; central TagServer rejected), observation = D39 bays + `modukit history get` CLI tool over mcap (no resident history in host), tag breakpoints = debugger domain (D17), interlocks = watchdog plugins not rule engines (D38), single-tag trace first step + full chains trigger-gated. modules/06 gained section 3 (renumbered), architecture/index synced; verification-line regex rewritten whole-line (third occurrence of the [x-9][0-9] range-includes-high-water bug class). High-water D43, fresh D44 probe. (five-topic menu walked one by one, votes C/C/C/C/C: protocol-docs automation, recording storage tiers, CI/test gates + scripts/check.sh, kernel error-code dictionary, SDK three-crate layout - BATCH COMMIT FAILED, records lost; re-filed same day as D48-D52 per audit ruling, phantom D41-D45 numbers in this line corrected; the DSL bridge PORT correction of that batch did land as the real D41). High-water D41, fresh D42 probe. (D40): platform extension points = pre-registered platform service interfaces (host as ordinary consumer, no second channel; side-effect deps as services; key-value blackboard rejected); STOPPING drain semantics (refuse new / finish old / stop_timeout bound - Java/gRPC/K8s/Rust converged answer); start-approval hook named trigger-gated; manifest team label (audit/accounting aggregation, NO permission semantics); org-level platform mode never self-built. OSGi verdict recorded: lacks are deliberate rejections or filed triggers; supersets (placements/tiers/contracts/determinism) are what OSGi cannot do. modules 02/03/05 + architecture/index synced. High-water D40, fresh D41 probe. (D39): unified seven-section blueprint (plugins/communication/services/debugging/logs/audit/settings) designed as the D37 hearing exhibit - IDE skeleton + cockpit bays, shell Svelte 5, bays separate-page, three-protocol data law (introspect/EventStream/ConfigService), multi-host switcher, trigger ladder (D32 bell / remote-debugging / ops team / Loki / compliance export); zero year-1 work. modules/05 §6.5; architecture/index synced. High-water D39, fresh D40 probe. landed (docs/reference/) and study adoptions (D38): config dump layer-annotated, id-addressed overlay patches, structured validation errors, minimal as named profile, settings-UI into cfg-web hearing, generated config catalogs (D23 candidate); cautionary: config never mounts code. High-water D38, fresh D39 probe. - configuration management (D37): layout (runtime/deploy/overlay tree, precedence, directory-as-instance-boundary), effect atomic unit = startup of what it describes (no file watching), multi-version rules (missing->default/stale->warn/type->fail, migration = author's), hot-lane ConfigService design (set gates + atomic snapshot swap + async overlay write-back, file stays single truth), thin RBAC (bridge-declared identity, role-x-prefix, no accounts, ABAC trigger-gated, ~200 lines self-build no framework), web admin stance (read = D32 tabs, write = files+git only, full admin = standalone hearing; AccessBase: no submodule/no hosted plugin/audit-format adopted/external-authority mode = future candidate), stack (Svelte 5 forms, Monaco+react-flow IDE cabin, Grafana/Loki never self-built, separate-page coexistence, React collapse allowed). modules/05 gained 6.1-6.4; architecture/00-overview synced. High-water D37, fresh D38 probe. - host identity and boundaries (D36): deploy-level host_id (default hostname, carrier-collision warning forcing explicit ids), contract names untouched (D9/D13), cross-host addressing = host_id/name door-plate composition, resolution local-first + cross-host explicit; ROS-style runtime renaming rejected (contract softness counter-example; DDS Domain/K8s namespace/systemd @-instances = mainstream identity-in-deploy-config confirmation); communication boundaries = name discipline zero machinery (component prefixes, unqualified fleet-wide names, app-prefix clusters), deliberate violation = malice handled by D26/D2 gates. modules 02/05/06 + architecture + index synced. High-water D36, fresh D37 probe. - boundary terms (D35): gate/audit split (checks at own gates, decisions on the ring, callback into lower tier passes its gate; unified policy engine rejected), ModuKit has no gateway (cross-world = bridge plugins, external entry = bridge plugins, central policy = more bridges never kernel), cluster identity (per-machine sovereignty, carrier-transparent cross-machine calls, fleet orchestration belongs to the orchestrator via manifest + idempotent install; no logical multi-machine host). modules 01/02/07 + architecture + index synced. High-water D35, fresh D36 probe. - lifecycle audit closures (D34): stop_timeout (default 2s; expiry per placement: exe SIGKILL / wasm cancel / dl mark-and-move-on); host lifecycle in three statements (boot DAG cold start / graceful shutdown reverse order + budgets + total cap + unclean exit code / self-healing = OS with exit-code vocabulary + crash testament); plus seven patches: capability freeze at install, duplicate-id rejection, reverse-order shutdown explicit, rate limits year-1 observe-only (storm trigger), same-machine multi-host via zenoh loopback, service-level access control named (multi-vendor trigger), hot-update state handover explicitly undefined (cold-state D8). modules 02/03/05/06/07 + architecture + index synced. High-water D34, fresh D35 probe. - language strategy (D33): three layers with one dividing line (who authors the artifact): handwritten languages on their own engines in exe-runners (Lua/JS/TS/Py, no ordering, skeleton at second runner, TS = dev-time only); graph/generated languages all lower to the bridged AUDESYS hal-ir VM (five IEC frontends measured, scan cycle = D20 step clock, VM Rust so wasm-tier eligible - the only DSL path into untrusted); blueprints = generalized FBD frontends on the same IR (script downgrade = editor export option); never self-built engines/VMs/LabVIEW scheduler. Opened from the polyglot brainstorm; corrected twice in discussion (AUDESYS asset discovery, maintainer authorship-line challenge). - dependency graph tool (D32): three views over the D18 ledgers (declared/observed/diff, diff default), year-1 minimal analyzer bin, full subcommand on the Stage-2 CLI wagon; live animated view trigger-gated (plugins > 20 or second team) with fixed bell-day shape (browser page as ordinary EventStream client, Foxglove probe first, cytoscape.js single-file fallback, never framework-embedded/wasm/hand-rolled layout). modules/10 §6 amended, architecture/00-overview synced. Opened from the rqt_graph brainstorm. - resource tree (D31): directory-as-route (`resources/` tree IS the route table, single window prefix, no .qrc alias layer), verification checklist in the manifest `[resources]` section (same D7 envelope, tool-fillable via toml_edit-preserving scan; separate .idx and central-directory-as-index rejected), manifest format = TOML (INI/JSON/YAML rejected with deployment-floor rationale). modules/05 §5 + §8 amended; high-water D31, fresh D32 probe with regex fixed to D(3[1-9]|[4-9][0-9]|[0-9]{3,}). - resources (D30): bundle members (D28) with manifest declaration + D7 cross-check; ResourceService window for exe/wasm/untrusted/archive readers, fd fast lane for trusted in-process dl only (exe excluded: isolation semantics + irrevocable fds + D22 accounting bypass); wasm-pack resource containers evaluated and rejected (four taxes, zero industry adoption); compression = packaging-CLI parameter (same trigger as archive format); pool behind the window gate = implementation note on measured latency. modules/05 gained section 5 (6-8 renumbered), architecture/00-overview synced. High-water D30, fresh D31 probe. Opened from the resource-file brainstorm; i18n fallback chains and scoped exe fd passes left as triggers. - configuration (D29): schema in the plugin's own .proto, values in deploy-TOML verified by the D7 cross-check, change = restart by default (D8); `config_live` declared keys opt into change events, hot-lane spec trigger-gated on first such plugin; full OSGi Config Admin push-everything and blank-defer rejected. modules/05 gained section 5 (section 6 renumbered to 7), 06 noted reserved system names, architecture/00-overview synced. High-water D29, fresh D30 probe. Opened from the OSGi/CTK/CppMS brainstorm; lazy activation + version ranges left trigger-gated, system-event naming note folded into 06. — bundle distribution (D28): logical install unit + staging/cross-check/atomic-rename protocol, directory and archive both accepted on one pipeline, verification ladder (hash now / signature at untrusted trigger / repository at OTA trigger) each with an owner; archive format and rollback explicitly deferred with owners. modules/05 §4 rewritten + architecture/index synced. High-water D28, fresh D30 probe. The adjudication queue is now EMPTY — remainder are trigger-gated, not discussion-gated.

- 2026-10-10 (continued 23): twenty-third adjudication — error model (D27): two code spaces (kernel enum in runtime.proto, first ledger-native vocabulary; author-owned service enums under sentinel rule), one shared record {code,domain,source,message,kv,hint}; hints feed D3/D6 policies, codes become D21 count dimensions; string-only, universal-table and gRPC-codex options rejected. modules 04/08 + architecture/index synced. High-water D27, fresh D29 probe. Queue left: bundle distribution only.

- 2026-10-10 (continued 22): twenty-second adjudication — capability vocabulary (D26): org.modukit.* hard-checked initial 8 words each mapped to an adjudicated mechanism, list lives in the ledger; com.vendor.* soft self-registration consumed by export/quota/trust policies; autodetection and closed-list rejected. modules/05 §3 + architecture/index synced. High-water D26, fresh D28 probe.

- 2026-10-10 (continued 21): post-DX-walk pool question walked (21 + house-rule 21.5) — D25: iceoryx2 as first Stage-2 pool backend with exact-pinned stable tag and three fallback triggers; self-built ring demoted to trigger-gated option; slots-house-porters-only rule ratified (worldview vendors bridge-only, FastDDS included); vendor vocabulary quarantined in the adapter. modules/07 + reference §2 sheet + architecture/index synced. High-water D25, fresh D27 probe. Rulings renumbered: remaining queue = capability vocabulary, error model, bundle distribution.

- 2026-10-10 (continued 20): twentieth adjudication closed the DX walk — testhost (D24): public test-support face = in-process host + dummy transport + step clock + fold-checker + fixture feed, three-piece surface discipline, true-process e2e stays Stage-2. modules 08/10 + architecture/index synced. High-water D24, fresh D26 probe. DX theme (adjudications 14-20) closed: 7 decisions D18-D24.

- 2026-10-10 (continued 19): nineteenth adjudication — scaffolding (D23): example/bad fixtures gain template standing (changes = compatibility-weighted doc acts; CI keeps them fresh), modukit new generator = Stage-2 CLI subcommand, ledger-driven, dialect-forbidden; B-before-C order rule recorded. modules/08 §9 + architecture/index synced. High-water D23, fresh D25 probe.

- 2026-10-10 (continued 18): eighteenth adjudication — resource books (D22): exe-only supervisor sampling + wait4 accounting, Snapshot fields + ResourceTick events, dl = n/a by design, cgroup/rlimit enforcement to P2 trigger, eBPF attribution rejected. modules 10 + architecture/index synced. High-water D22, fresh D24 probe.

- 2026-10-10 (continued 17): seventeenth adjudication — gate counters (D21): edge latency accumulators + topic rate/high-water as atomic adds at the D13/D18 doorways; reads via Snapshot + CountersTick ring events; in-kernel quantiles and direct OTel adoption rejected (P2 subscriber lane). modules 02/09/10 + architecture/index synced. High-water D21, fresh D23 probe.

- 2026-10-10 (continued 16): sixteenth adjudication — unified clock + timers-as-jobs (D20): now()/timer(deadline->JobToken) across real|sim|step; kernel timers ride it; replay gains dial authority; hot paths use capture_ts; LD_PRELOAD-hijack and test-mock-clock rejected. modules 02/03/09 + architecture/index synced. High-water D20, fresh D22 probe.

- 2026-10-10 (continued 15): fifteenth adjudication — logger (D19): kernel Logger service + LogEvent on the shared monotonic ring; sinks = subscribers; OTLP demoted to future subscriber (P2); B-vs-C dissection recorded (async smuggling / no plugin C ABI / no bus-seq). modules 09/10 (+ D18 numbering-order slip fixed) + architecture/index synced. High-water D19, fresh D21 probe.

- 2026-10-10 (continued 14): DX gap walk opened (items 14-20 proposed from the tooling matrix). Fourteenth adjudication — edge ledger + analysis (D18): live (caller,service,method) counts recorded at the generated stub doorway + declared ledger => callers/uses/impact/orphans/diff as pure walks; undeclared use warn-by-default, reject as policy knob (B chosen over B+); tracing deferred to counters/OTel. modules 02/05/10 + architecture/index synced. High-water D18, fresh D20 probe.

- 2026-10-10 (continued 13): final adjudication of the communication arc — D17 observability & debugging: introspect trio as read-only services + ModuleLoaded event (year 1), churn/replay/ledger-equality acceptance tests, DAP via off-the-shelf adapters for native dl/exe (scripts at Stage-5 via runner hook), Foxglove Studio as D16 adoption target, tool constitution (no bespoke protocol, no nested UI framework, no back doors). New modules/10; 09/architecture/index synced. High-water D17, fresh D19 probe. Walkthrough totals: 16 decisions D2-D17, theme closed.

- 2026-10-10 (continued 12): twelfth adjudication — recording & replay (D16): recorder = ordinary export-gated subscriber (deep kernel probe rejected), three-part self-describing header (fdset+manifest+ABI stamps), gap-stats-as-honest-absence, dual-encoding archive/quality separation (MediaServo lesson), replay via virtual publisher (content deterministic, timing not; realtime|burst|step clocks; burst feeds contract suite), mcap-first container w/ own-format fallback; new modules/09; 06/architecture/index synced. High-water D16, fresh D18 probe.

- 2026-10-10 (continued 11): walkthrough finale landed — D15 cargo oneof (local-ticket / off-host-tail arms, bridge ticket-to-tail, three-sheet definition law, pixels never in .proto) + docs/modules/08-end-to-end-walkthrough.md (151-line video-frame worked example); modules 04/05/07 + architecture §11/§13 + index synced. High-water D15, fresh D17 probe.

- 2026-10-10 (continued 10): round-15 consolidation adopted — D14 descriptor access protocol + carrier-transparent API charter (self-sufficient slot/epoch/layout tokens, attach-once UDS transfer, typed liveness with EPOCH_EXPIRED, two-stage meta/data retrieval, carrier=deployment-not-type). modules 07 gained §3-§4 (renumbered ≤§7), 04 view row + fuse rule, 05 carrier note, architecture §10/§11 + index synced. High-water D14, fresh D16 probe.

- 2026-10-10 (continued 9): maintainer challenged the authoring direction (diagram review) — D13 adopted: hand-written .proto is the single contract door; protoc plugin emits Rust skeleton/C ABI/fdset (exporter risk retired; fdset becomes protoc-native; whitepaper §3.2 literal via same-source C generation). D12’s multi-entry door clause revised; modules 04/05/06 + architecture synced; high-water D13, fresh D15 probe.

- 2026-10-10 (continued 8): Arrow Flight profiled to docs/reference/arrow-flight.md (104 lines) from same-day arrow-rs checkout (dfef34e6, 60.0.0+214, 19.7k LoC, 132 tests) + upstream Flight.proto. Findings: D12 sidecar rule confirmed at spec level (FlightData body deliberately at tag 1000 = numbering-as-architecture); Ticket/PollFlightInfo/DoExchange map onto D4 handles / D6 job tokens / D10 stream channels; gRPC substrate re-rejected (D6/D11); Flight SQL classed as product-layer not kernel debt; encoder "one-copy" honesty noted vs zero-copy marketing. README + 00-overview remote-round matrix updated.

- 2026-10-10 (continued 7): eleventh adjudication landed after the longest interrogation (B/A/D/E variants, Python-barrier, proto-vs-Arrow layers, Flight sidecar technique, video-frame lane) — contract registry = derived sidecar .proto + fdset, three registry rules, two-lane payloads, multi-entry doctrine, AUDESYS 14-type whitelist inherited, Flight gRPC substrate explicitly rejected. D12 recorded; modules 04/06/07 + architecture + index synced. User-supplied corroborating material folded in (transcoder libs logged as unverified bridge-side tools). High-water D12, fresh D14 probe. §2 decision-sheet composition amendment (iceoryx2 local pool / zenoh remote carrier wording) held pending maintainer word.

- 2026-10-10 (continued 6): AUDESYS profiled to docs/reference/audesys.md (170 lines) — provenance: same-maintainer SISTER project (gitee origin verified), Apache-2.0, 24 crates/~36.5k LoC/826 tests. Findings cross-check the open adjudications: their D10/D11/D12/D17 decisions = Signal/StreamChannel/RPC trichotomy (converges with our D10), rmw-shaped three-pole middleware trait trio w/ zenoh-scaffold backend (contrast for our D11 seam + dummy-drift flag), cycle-boundary hot-swap prepare/commit/rollback (pattern twin of D8), closed 14-type protocol (strengthens pending adjudication 11 C-protobuf), hand-written-JSON ipc.rs as the cautionary debt, 297-line supervisor (adjudication-1 cost proof), 32 napi exports (JS product-face-2 precedent). README + 00-overview matrices updated.

- 2026-10-10 (continued 5): communication theme opened and two more adjudications landed — topic bus as the second object kind of the unified handle table (D10) and transport architecture: narrow seam + per-plane slots + bridge-not-heart-swap (D11, rmw worldview-tax analyzed). New modules 06/07; architecture/04/05/index/reference-§2 synced; high-water D11, fresh D13 probe.

- 2026-10-10 (continued 4): eighth adjudication closed the walkthrough — service identity (D9): canonical reverse-DNS + SERVICE_ID consts + bundle-scoped aliases as sugar; modules 02/04/05 amended. Walkthrough total: 8 decisions D2-D9; numbering regex simplified (flags any two-digit D, fresh D10 probe observed red/green).

- 2026-10-10 (continued 3): seventh adjudication — update semantics (D8): stop-based year-1 (dl activate-on-boot / exe swap-in-restart), `parallel_ok` declared escape hatch inert until Stage-2 routing; closes whitepaper §4.1 update vs state-machine gap found in self-audit. Modules 02/03/05 + architecture amended; high-water D8, fresh probe (planted `## D9:` -> red, removed -> green).

- 2026-10-10 (continued 2): sixth adjudication — manifest residence (option D: three tiers deploy-TOML / self-report descriptor / stage-5 binary verification; mandatory cross-check + liar fixture; TOML-only, boot DAG from files alone). D7 recorded; modules 04/05 + architecture + index amended; header-first-vs-metadata-first queue item narrowed. High-water D7, fresh probe.

- 2026-10-10 (continued): fifth adjudication walked after the docs landed — async model (synchronous boundary + job tokens + event completion; actor messaging demoted to exe-transport mechanics; uniffi async-withdrawal as costed precedent). D6 recorded; modules 02/03/04 + architecture + index amended same round (assistant pre-written "Async-first" line withdrawn by adjudication). Numbering high-water extended to D6 with fresh red/green probe (planted `## D8:` -> red, removed -> green).

- 2026-10-10: architecture brainstorm (architectural path): four adjudications walked one-by-one (placement hybrid+shadow-runner; availability B tier; service model C+ with six invariants; solo+AI thin Stage-1 cut). Architecture docs landed at `docs/architecture.md` + `docs/modules/01-05` (MediaServo-style structure per maintainer redirect away from the spec path) with whitepaper delta table; D2-D5 recorded. Task store threw internal_error during walkthrough (PIT-3 recurrence). Numbering invariant in Verification extended (allow D1-D5, flag D6+) with fresh red/green proof.

- 2026-10-09: agent/config scaffolding ported from PolyOrch (plan A). PolyOrch-era memory (status / conventions / decisions / pitfalls) reset to templates; `.gitignore` PolyOrch-specific entries dropped (`PolyOrch` ignore line kept for the local reference checkout); rules tree and skills otherwise kept as-is.

- 2026-10-09: whitepaper v1.0 landed as `docs/whitepaper.md` (user-provided, kept verbatim); root `README.md` generated from it; status Project/Phase/Modules filled.
- 2026-10-09: C1 recorded (all artifacts English; chat follows the user). README.md and docs/whitepaper.md regenerated in English (whitepaper content unchanged); functional-CJK exceptions in `.agents/skills/**` documented and excluded from the C1 scan.
- 2026-10-09: /init-deep generated `AGENTS.md` (root) + `.agents/AGENTS.md` hierarchy; C1 gate scope extended to cover both; two toothless ported references in rules/common recorded as open item.
- 2026-10-09: root `SKILL.md` skills registry added (pattern ported from `.refinfo/MediaServo/SKILL.md`, English per C1; content reflects ModuKit's actual 10 project skills — MediaServo-only entries like openspec-propose/explore/test-harness deliberately absent and noted). C1 gate scope extended to SKILL.md in both gate copies.
- 2026-10-09: plugin audit — superpowers/ponytail/context-mode already latest; upgraded oh-my-openagent 4.19.4 -> 5.1.27 in opencode.json (5.x fixes unbounded auto-continuation and opaque child-task errors observed this session). Pending: session restart to load; omo.jsonc schema recheck after restart.
- 2026-10-09: design doc `docs/codegraph-bootstrap.md` written + reviewed inline (3 perspectives, time-boxed; parallel team mode cancelled by user). Verdict: POSIX .sh stays the only repo launcher; the v1 .cjs cross-platform proposal was WITHDRAWN (win32 launcher is a codegraph.cmd shim, install.ps1:145; opencode guarantees neither node nor sh on native win32). Windows path = WSL or per-user global-config .ps1 twin, trigger-gated. Implementation triggers recorded in doc section 8.
- 2026-10-09: win32 discussion overturned doc v2's global-config remedy (R-6: project config overrides global, user caught it); implemented `.opencode/init-mcp-codegraph.bat` twin (cmd+powershell, pinned v1.6.2, CRLF-verified; install.ps1 has no CODEGRAPH_BIN_DIR, shim fixed at <INSTALL_DIR>\current\bin\codegraph.cmd). WSL now optional, not required. Doc at REVIEWED v3; first real-hardware run pending.
- 2026-10-09: LSP root-cause cleanup (user-approved): replaced 3 custom `lsp` wrapper entries with product built-ins `"lsp": true` (panel error was node-v10 ESM crash in init-lsp-wrap.mjs on every .md open via the self-invented remark entry); deleted dead `init-lsp-wrap.mjs` (`init-mcp-openspace.mjs` was already absent at cleanup time — recorded honestly); rust LSP trigger = Stage 1 rustup component; remark dropped (not built-in; CLI-first per opencode best practices). Bootstrap doc section 2 gained the MCP-vs-LSP asymmetry note.
- 2026-10-09: mise installed user-level (2026.10.7), `mise.toml` pins node 22.23.3 (file pins major "22"; 22.23.3 = resolved version then — system node v10 untouched; shims beat it via ~/.bashrc activation). Fixes OMO's bundled `lsp` MCP (-32000 was lsp-tools-mcp needing node>=20, unrelated to our own local-codegraph). C1 gate scope extended to mise.toml in both copies. Pending: restart opencode from an activated shell to light the lsp row; Stage-1 rust-analyzer trigger unchanged.
- 2026-10-09: bootstrap twins added (option C, user-approved): `bootstrap.sh` (live-verified idempotent on this box) + `bootstrap.bat` (GitHub-zip mise, .NET User-PATH API — no winget id gamble, no setx 1024 truncation; unverified until first real win run). mise itself deliberately NOT auto-installed per opencode session (PATH prerequisite, no config hook; silent rc-edit plugin rejected as invasive). README gained the Development status section; AGENTS.md structure/where-to-look/commands + both C1 gate copies synced.

- 2026-10-09: docs/reference series delivered (user-specified location; earlier docs/reference/visiaengine mis-scope pulled and superseded): 8 external-project profiles + 00-overview decision matrix, 1,280 lines total, C1 green. Method: 4-lane research team (auto-closed) -> hyperplan tribunal (5 adversarial lenses adjudicated a 30-item VisiaEngine port-candidate list; closed early on user pivot; verdicts distilled into visiaengine.md §8 + cross-cutting rules) -> 7-member profile team (fast-model balance deaths recovered via auto-retry + premium-channel substitutes; see PIT-2). Key outputs: Stage-2 fork converges on composition (iceoryx2 local plane + zenoh carrier, decision sheet 00-overview §2, NOT yet a D-entry); I420 confirmed as own design term (smuggled-residue suspicion withdrawn); A6 hand-written-bindings REJECT wording drafted for ratification. Task store proved unwritable mid-run (PIT-3) — files are truth.

- 2026-10-09: remote-round profiles added (user pivot beyond .refinfo): 9 GitHub projects profiled from blobless clones under ~/.cache/modukit-research/ (rutis, zellij, CLAP, uniffi-rs, wasmtime+Extism survey, ipc-channel, smithay, sysplugin-existence-audit, webrtc-rust-state audit) + whitepaper-citation audit (7 named-but-unvendored refs: 1 strong rutis, 1 toy vnrit, 1 misfire rustbridge, 4 unresolvable). docs/reference/ now 20 files (17 profiles + audits + index + 00-overview), all C1-verified; README + 00-overview carry both rounds, Stage-2 fork sheet gained a third option (typed control plane via ipc-channel shape) and Stage-4 decision axis (sans-I/O dual vs libwebrtc-FFI vs vendor; getstream-rtc = naming+category error). Team modukit-ref-remote survived a premium-family Bad Gateway storm via 5 recovery substitutes reusing on-disk clones (PIT-2 recovery recipe held: files over reports, never trust member completion notices); all teams closed and deleted. New maintainer canon questions queued (00-overview §4): header-first vs metadata-first contract (uniffi finding), whitepaper reference corrections list, Stage-4 engine tiering.

- 2026-10-09: license decided (user "landed" approval) — D1 recorded: MIT OR Apache-2.0 dual; LICENSE-MIT + LICENSE-APACHE at root (Apache text from family-standard full text, 202 lines verified); README License section + status open item closed; both C1 gate copies extended to scan the license files; DCO required from first external contribution (relicensing-space protection). Cargo workspace `license` metadata = recorded Stage-1 trigger.

- 2026-10-09: ecosystem-scan (Full, first run): 15-item local audit + 2-way external scan. Landed same session — R1 AGENTS.md C1 gate copy drift fixed (was missing crates/ dynamic scope); R2 openspec-* x3 flagged inoperative (CLI absent); R4 G10 executed (ts/python/web/golang/cpp rule files unmounted: instructions 26->16; 7 never-mounted dirs (csharp/dart/java/kotlin/perl/php/swift) initially deleted then RESTORED 35/35 per maintainer veto — rule files stay on disk, mount-only slimming); plugins pinned (ponytail@5.1.0, context-mode@1.0.169, omo 5.1.27->5.1.28; superpowers git-ref left floating by design); PIT-4 recorded. External verdicts: ADR-MCP class dead, rust-analyzer-MCP dup with built-in LSP, serena/playwright-MCP no-value-at-stage, npm spdx/osv trio provenance smell — all REJECT; github-MCP / arxiv-MCP / skill-lint+evals pattern = conditional P2 pending; Rust FFI/C-ABI skill = community gap (nothing to adopt, will be ours to write). Uncovered honestly: affaan-m/everything-claude-code timed out x3, not profiled.

- 2026-10-09: A bundle committed (`2909081`, landed 2026-10-10 09:17). Backlog planning doc added at `.agents/memorys/backlog.md` (opencode-plugin/config/skill/MCP track only; Stage-1 is a separate track resumable on explicit instruction; rules-asset "mount, never delete" ruling recorded). B landed same session: `scripts/skill-lint.py` (7 checks over .agents/skills + registry sync), C0 negative-proved via /tmp fixture (3 FAIL, exit 1) then repo green (10 skills, 0 fail); C1 gate scope extended to scripts/ in both copies; skill-lint registered in AGENTS.md COMMANDS.

- 2026-10-09: D1 landed (user invoked /doc-audit right after plan doc): doc-audit SKILL.md re-scoped to ModuKit canon — 18 anchor replacements (decision/consistency/agent/gap/phase dimensions re-targeted to whitepaper + docs/reference series + gate copies; PolyOrch xmake/C2-C6/D2-D29 targets removed; skill-lint wired as the mechanical first pass). Rescope process itself surfaced+fixed two stale claims in AGENTS.md (NOTES "License TBD" post-D1; header "Commit: none yet" post-commits) — exactly the finding-class dimension 4 hunts. C1 + skill-lint green on all edits.

- 2026-10-10: doc-audit FULL first run (4 fresh-eyes background auditors): 18 findings (0 CRIT / 5 HIGH / 9 MEDIUM / 4 LOW), ALL user-adjudicated one-by-one (A×16 / B×2). Fix batch landed across 16 files per the adjudication table (H1 git-pointer line, H2 five-place doc-audit sync, H3 backlog D->P rename with retro-mapping note, H5 codegraph live-state lines, M1 replaced the always-green ERE check with a falsifiable ported-numbering invariant, M2 six license-TBD annotations, M3 §4.3 EXECUTED, M4 child-AGENTS line-by-line sweep incl. measured file count, M5 five-part template normalization + PIT-4/PIT-5 completion, M6 awk quoting+guard, M7 dangling-refs family registered, M8 STRUCTURE +4 entries, M9 PORT_NOW as §4 item 8, L1 eight, L2 empty docs/modules rmdir'd, L3 edit-safety second #11 -> #25, L4 date/pin annotations). C0 instant payoff: M1's own negative proof caught a SAME-SESSION regression (I re-wrote `\|` in the new ERE — planted fixture stayed green, fixed to `|`, then red->green observed). Gates: C1 OK, skill-lint 10/0/0, gate copies identical.

- 2026-10-10: backlog P2-P4 executed (user issued an explicit "get to work" on the three). P3/P4 done: uv pinned via mise.toml (installed 0.12.24; bootstrap twins auto-cover), arxiv-mcp-server smoke-tested cold (uvx exit 0), config `arxiv` enabled; openspec-* trio ruled KEEP per mount-never-delete doctrine (vetoable). P2 half: github-mcp-server v2.0.2 Linux x86_64 installed to ~/.local/bin with sha256 verification against official checksums; `github` config entry present with enabled:false + {env:GITHUB_PAT} interpolation — activation needs a user-created fine-grained read-only PAT, then flip to true. Both new MCPs load at the R restart. C1 OK, opencode.json valid.

## 2026-10-16 - Stage-2 plan of record drafted (docs/stage-2-plan.md)
- Maintainer opened the §10 Out spine (ProcessExecutor / shadow-runner / cross-
  process bus / iceoryx2 first entry); plan anchors architecture.md §12 item 2,
  quotes §10 lists, milestones S2-M0..M5 + optional M6a/b, every gate falsifiable.
- Scope honesty: D25 (iceoryx2-first) is EXECUTION not re-debate; Zenoh cross-host
  stays trigger-gated per D11 (first cross-machine link).
- Call sheet A1-A7 for kickoff adjudication: clock-over-wire (extends D20),
  declined external-consumer round as freeze-hearing input (D5), vendor-lock
  amendment for the crates.io ban (A3), freeze-hearing scope (D51/M5, integrity/
  M7, hint/M8 deferrals), D16/D17 lane in-vs-defer (canon schedules both - defer
  = supersession), coverage 80% hard-gate switch (D50), and A7: parallel_ok
  routing contradiction (D8/§11/modules-03 schedule it with THIS stage; my draft
  had excluded it citing D71's different-scenario wording).
- Momus review verdict OKAY (1 MAJOR + 5 MINOR): MAJOR = the parallel_ok
  misquotation -> A7 row; MINORs = watchdog/ASIL attribution (belongs to D3
  trigger + §10 Out, not modules/01 §7), restart-policy cite (D3 not D4), runner
  bin naming vs card `modukit-run <plugin.so> --fd=N`, MemfdLane->MemfdBlob drift,
  "mcat" typo, M0 omissions (numbering-gate widen for first D74, push-decision
  OPEN per Stage-1 precedent). All fixed in-draft before landing.
- Execution gate CLOSED: nothing of Stage-2 runs until maintainer go; next step
  is the A1-A7 walkthrough (adjudication-walkthrough protocol) at kickoff.

## 2026-10-16 - Stage-2 kickoff walkthrough, all seven verdicts (D74-D80)
- adjudication-walkthrough protocol run one-at-a-time on the plan call sheet;
  maintainer asked for plainer language twice mid-walk (fixed in-flight, kept).
- A1 clock: supervisor owns the cross-process dial (D74) - rejected Real-clock
  relaxation and field-reserved-but-unbuilt. A2: runner = first non-kernel
  consumer, dual run pays D5's declined round (D75).
- A3 vendor boundary: maintainer amended the draft - switch renamed to the
  neutral `vendor-shm`, factory confined to one module + one pinned dep line,
  equivalence suite as the machine proof the abstraction is not captive (D76).
- A4 hearing: door-by-door freeze, three IOUs + ABI bump policy as agenda (D77).
- A5: recorder + CLI both superseded out of Stage-2 (D78) - honest deferral beat
  half-built side lanes; no mcap entry. A6: 80% hard gate at S2-M5 workspace-wide
  with an anti-gaming clause (D79). A7: Momus-caught parallel_ok contradiction
  resolved by supersession + typed deferral replacing the silent-ignore (D80).
- write-back: D74-D80 minted in decisions.md; architecture §11 +7 rows; plan
  verdict block + milestone re-anchors; numbering gate widened to D1-D80 with
  negative proof observed (planted D81 -> red, removed -> green); pin re-check
  fresh (5945f2a == origin tip). Execution gate stays CLOSED until go.
- honesty footnote: the FIRST gate-proof attempt was invalid (regex held in a
  shell variable split on its embedded space -> grep got junk args; fake green
  AND fake red both observed before scrutiny caught it). Re-run with the command
  typed directly: real green -> planted D81 red -> removed green. PIT-19 class,
  caught by eyeballing the error lines the broken run emitted.

## 2026-10-16 - S2-M1 landed: ProcessExecutor + supervisor loop
- maintainer go after kickoff; one deep-low agent (rust skills), 1h07m;
  orchestrator independently re-verified BEFORE commit: 116 passed (108+8),
  clippy all-targets -D warnings, check.sh green, STAGE1-DEMO PASS regression.
- design accepted as built: kernel stays synchronous, Supervisor poll/revive
  external (host loop = M2/M3); fd-0 stdin control channel per 01 §1
  socketpair/stdio (arbitrary-fd/SCM_RIGHTS deferred to M3 by plan); placement
  gate moved from kernel into each executor load (kernel placement-agnostic).
- E3 crash code rides CONTRACT_VIOLATION -> IOU-4 added to the D77 hearing
  agenda (S2-M5 row).
- process hygiene landed with the milestone: /proc/self/fd before/after probe
  caught a real parent-socket leak (reap now take()-closes the control half);
  zero-zombie scan with bounded retry for the shared-process race (documented).
- self-caught en route: continuation-line C3 body bullets (amended single-line);
  two python anchor drifts after the adjudication rewrite were caught by the
  count==1 asserts BEFORE any write - no half-state shipped, completed on the
  third pass with anchors copied verbatim from grep output.

## 2026-10-16 - S2-M2 + F1 landed: shadow-runner, dual-placement table, first-consumer catch
- modukit-runtime/modukit-run (card name; embedded-Kernel host design - zero
  duplicated ABI machinery); 7-row dual-run table on the untouched fixture;
  runner bin committed a3205be, F1 fix separate (correct bugfix hygiene).
- F1 = the D75 round's first payoff: the non-kernel consumer caught the parent
  silently holding the injected child-end fd (Command lifetime, not kernel
  weirdness - probe4 isolated builder-lifetime as the mechanism; block-scoped
  spawn, std-only, no unsafe). Silent pre-HELLO death now = immediate EOF ->
  ProcessCrashed; the sub-second bound doubles as the deterministic
  retention probe (count probes missed it: fd numbers recycle).
- F2 (kernel-side echo parity) stays M3 by design; runner self-check gates the
  START ack meanwhile (shortcut comment in place).
- friction log 7 items -> plan section 9 (hearing material).
- process notes: one C3 subject catch (72 -> 58, amended pre-push); agent
  disclosed a prior-turn git-diff slip against the zero-git rule (read-only,
  logged honestly).

## 2026-10-16 - C3 machine-gate birth saga (three traps in one shift)
- commit-shape leg landed in check.sh (pending range origin/master..HEAD);
  eight pending commits reworded via filter-branch (msg-only, trees
  identical by construction; refs/original cleaned after).
- trap 1: quoted regex in a shell variable split on its embedded space -
  first negative proof ran against junk args (fake red AND fake green).
- trap 2: mawk has no {n,m} intervals - regex leg flagged every legit
  bullet (false flood); predicates moved to length+substr (isbullet).
- trap 3: the author's own messages carried continuation lines twice; the
  new leg caught its own creator live - first real reds were self-caught.
- five pushed legacy subjects stay outside scope by policy (immutable).
- C0 for the leg: planted SUBJ-VIOL(63) + BODY-VIOL red observed, pending
  range silent green, check.sh green including the leg. PIT-8 the class.

## 2026-10-16 - S2-M3a landed: TL frames, SCM_RIGHTS crossing, F2 closed
- 41m agent run; independent verify 127/0 + surface match (15 files); zero
  ESCALATE (04 §2 / 07 §3 cross-read before building matched the proxy shape).
- behaviour red taught the real semantics: SCM_RIGHTS passes the WHOLE open
  file description incl offset - receiver positions from the meta token.
- F2 closure evidence: dual_placement acquires ProxyService kernel-side;
  echo + unknown-method rejections byte-equal dl-vs-exe.
- ops: 293 stale test-scratch dirs had /tmp at 100% (the "flaky copy" was
  ENOSPC); scratch patterns now cleaned; mk-* dirs only ever mine.
- M3b (bus/QoS/dial/D80/host-loop) dispatched immediately after.

## 2026-10-16 - S2-M3b landed: wire dial closes D17 test-2; bus clause escalated
- 1h16m continuation run; independent verify 133/0 (headline step_replay test
  pins the transcript [(0.5s,[]),(1.5s,[alpha@1]),(4s,[beta@2])] BEFORE the
  byte-equality assert - and that assert caught a real fired_at ordering bug,
  proving non-vacuity).
- D74 verdict landed: MSG_DIAL/MSG_DUE, supervisor-driven dial, Real-domain
  typed refusal (ClockDialRefused), named DIAL_TIMEOUT kill guard; runner
  clock default flipped Real->Step (A1 test/replay track) with --clock=real
  production opt-out; one existing hygiene test switched to Supervisor::step.
- D80 landed: RoutingDeferred at the co-host swap door; undeclared paths
  byte-identical to the old InvalidTransition.
- THE CATCH of the milestone: the cross-process bus clause could not be
  honestly built - ABI v1 gives plugins exactly one door (register_service);
  no bus verbs, no acquire, not even in-process (the bus was always a
  kernel-side face). Agent refused the machinery-to-machinery self-talk
  bridge (D79 anti-gaming class) and escalated with quotes: the clause
  waits for the D77 hearing as IOU-5 (ABI-v2 bus/clock verbs vs runner-
  bridge SDK posture; either needs a bus-speaking fixture). D10 trigger
  stays unexercised - recorded honestly in the plan row.
- D74 also surfaced friction #9: plugins lack clock verbs; the runner
  schedules replay markers itself (shortcut on-site).

## 2026-10-16 - D81: bus clause transfers to the hearing (adjudication C)
- maintainer ruled the single-option walkthrough: neither ABI-v2 pulled
  forward nor the self-talk bridge; clause rides the S2-M5 agenda as IOU-5
  with friction items 8-9 as evidence; D10 trigger recorded unexercised.
- numbering gate widened to D1-D81 (second widening same day; negative
  proof observed red->green both times); §11 index +D81 row.
- M4 (A3-ready: vendor-shm boundary adjudicated) awaits its own go.

## 2026-10-18 - S2-M4 landed: vendor-shm / iceoryx2 first entry
- 16m run; independent verify: default 134/0 (41 suites), vendor lane
  19/0, clippy green BOTH feature shapes, check.sh fast-gate untouched
  (D50), confinement grep = 3 files only (module, Cargo.toml, the gate).
- vendor_lock amended honestly: frozen-19 baseline UNION lock-graph
  closure of iceoryx2 only; +4th proof that no workspace member's
  transitive closure reaches the vendor (core std-only, mechanically).
- measured vendor cold wall 17.8s / warm 0.25s; task `pixi run vendor-test`.
- pre-existing flake to know: modukit-core hung_peer timing test failed
  once under first full-parallel load, green isolated and in every
  re-run since - logged, not masked (M5 acceptance re-runs it many times).
- orchestrator counting lesson x2: concurrent cargo interleaves and
  corrupts terminal lines - totals must be parsed from a redirected
  file, never a pipe through a busy tty.

## 2026-10-18 - ETXTBSY flake class eliminated (not just logged)
- the hung_peer "timing flake" and the crash_injection failure share one
  root: full-parallel runs caught exec racing a half-copied standin file
  (Text file busy). Standin builds now stage (.new/.tmp) and land via
  rename; the complete-marker checks the LAST stem. Defense hardened in
  both builder copies. Workspace x2 full-parallel 134/0, demo PASS.
- new orchestrator habit proven: amend commits get verified by building
  the COMMITTED tree in a throwaway worktree (caught the base-definition
  ordering bug that the working tree alone would have hidden).

## 2026-10-18 - S2-M5a landed: the host loop finds its home; hearing evidence complete
- 57m run; NO new constructor needed (Supervisor::with_clock already wires
  the pair) - the E1 feared seam did not exist.
- ONE deviation, escalated not swallowed: bundle cross_check probe now
  rides kind=Dl whatever the bundle declares (probe door = dlopen read,
  no code runs; mirrors the runner internal_manifest posture). Existing
  bundle tests untouched and green. Hearing decides native exe admission
  vs InstalledBundle-level runner field (IOU-6 candidate).
- ORPHAN-PROOF is the milestone's new evidence class: SIGKILL the host,
  the runner grandchild dies on its own (control EOF), /proc census = 0.
  The process tree's crash-blast-radius promise now has a live demo.
- falsifiability: DEMO_STAGE2_SABOTAGE=1 -> STAGE2-DEMO FAIL exit 1,
  observed and kept documented (single named read point in the demo).
- coverage (D79 hearing input, no chasing): TOTAL 71.76 vs 80 hard;
  core 78.42 / codegen 73.19 (M8 debt stands) / transport 80.78 /
  runtime bins 0 pct (instrumentation boundary, M8 fixture precedent).
- ops: sab-run first read showed exit 0 - pipe-tail echo artifact;
  true exit is 1 (PIT-17 class, caught by re-running with captured rc).

## 2026-10-18 - THE FREEZE HEARING: eight rulings, Stage-2 closed (D82-D88)
- walked one by one, plain-language on demand (two re-renders mid-walk).
1 v2 DIRECTION admitted (bus/clock verbs, shapes not drawn) - D82.
2 amendment policy: tail-add by record, break/semantic-edit by hearing - D83.
3 coverage split: bins = ruler boundary (M8 precedent), core+codegen debt
  calendared to the Stage-3 kickoff gate, second-delay hook - D84.
4 dictionary 14->19: supervision domain tags 23-27; INTEGRITY_FAILED at the
  bundle tier; 7b/c/e folded in (five true names, one execution) - D85.
5 hint() paid + probe posture promoted to rule (reads-as-dl,
  verifies-at-HELLO); native exe admission refused per the new policy - D86.
6 THE GAVEL: v1 STABLE (host table, wire v1, install protocol, supervisor
  verbs, proxy face, dictionary-as-amended); v2 PLANNED signage - D87.
7 7a dictionary-origin=kernel, 7d friction 1-2 = testimony received - D85/D86.
8 Stage-2 ACCEPTED against all five DoD points - D88.
- execution same-day: proto supervision domain + fdset + goldens recast,
  error remap, INTEGRITY_FAILED split, hint() shipped, tests assert true
  codes; 134/0, check.sh green, both demos PASS. Red/green proof for the
  widened numbering gate observed (planted D89 red, removed green).
- self-caught x3 en route: goldens caught the enum growth by design (recast
  per its own instruction), produced-set needed samples for new variants,
  pipe-tail exit-code artifact again (checked rc directly).

## 2026-10-18 - Stage-3 kickoff walked one by one (K1-K7 -> D89)
- protocol-corrected start: orchestrator first listed all seven in one
  message (anti-pattern), user invoked the walkthrough skill, restart
  one-at-a-time with plain-language re-renders on demand (x3).
- verdicts: K1 serial gate (A), K2 UI out (A), K3 Zenoh gated (A), K4 v2
  signatures stay planned (A), K5 minimal truth-set (A), K6 pin unchanged
  (A), K7 = C - the D73 trigger resolved as the middle path: inform the
  fact, defer the hearing proposal, proposal trigger stays armed.
- the recurring principle, now three rulings deep: work with no creditor
  does not enter the stage.
- numbering gate widened D1-D89 (red/green observed: planted D90 red,
  removed green); D89 consolidated one entry for the seven (K1-K6 are
  confirmations of existing doctrine; only K7 carries new information).

## 2026-10-18 - D84 gate measured and filed (no execution, maintainer order)
- summary-only run: TOTAL 71.70% (stable vs M5a 71.76, jitter 0.06).
- per-crate split run aborted mid-run by the ETXTBSY-family flake
  (process_executor.rs:251, standin copy busy under llvm-cov load);
  maintainer ordered measure-and-file ONLY - nothing executed, nothing
  fixed. Debt-crate numbers quoted from the M5a table (78.42 / 73.19).
- verdict per D84: both debt crates miss; hook ARMED; gate NOT cleared.
- M3S-G row carries the observation verbatim; the flake stays untouched.

## 2026-10-18 - profraw litter swept at maintainer word
- 61 files (root + crates/*, all default_<ts>_<pid>_<seq>.profraw from
  instrumented runs) deleted; .gitignore gained the *.profraw rule so
  spawned-child and uninstrumented-exit litter can never be staged.
- root cause pointer stays in the debt-lane report (LLVM_PROFILE_FILE
  not exported to test-spawned children; proper redirect = hearing
  candidate if the litter regrows at scale).

## 2026-10-18 - D90 hook lifted; demo D85 drift paid (same-day follow-through)
- maintainer ruled 1+2: hook LIFT (D90 minted; numbering gate widened
  D1-D90 with red/green proof) and the demo fix as the first unfrozen
  work.
- demo fix honesty note: the first attempt assumed both stage1 negatives
  were version-lies; the live demo corrected it - bad-plugin lies about
  its ABI (both doors ABI_MISMATCH); only stage2's manifest-version lie
  is a true IDENTITY_MISMATCH. Reality over memory, again.
- end state: both demos PASS, 192/0, check.sh green. The check.sh
  gate-coverage gap (demos not in the gate) stays an open hearing item.

## 2026-10-18 - leftover sweep: demo leg in check.sh, PIT-10 closed
- check.sh gained the demo leg (ok/FAIL per demo PASS-line gate,
  skip-when-absent per D50) - the D85 drift hid two days because demos
  were in no gate; red/green proof observed (sabotaged PASS line ->
  exit 1 with FAIL row; restored -> all green).
- PIT-10 CLOSED: unique dirs (done with the debt lane) + the cov env
  redirect (LLVM_PROFILE_FILE via pixi activation env). Clean-sweep
  run: zero fresh litter, zero ETXTBSY, three post-fix cov runs exit 0.
  The writer class is fully accounted for; the historical shared-path
  site never recurred.
- both items landed without touching any frozen surface.

## 2026-10-19 - Stage-3 accepted (D91): the quiet closing of a quiet stage
- final gate fired: coverage re-verified post-lanes (core 85.80 / codegen
  85.24 / transport 80.78; runtime boundary 0% per D84a), check.sh all
  green, pin fresh; zero ETXTBSY under the instrumented final run = the
  spawn-door retry's last word.
- the stage's shape, in one line per lane: hearing rulings minted (D82-D88
  were the previous day's close-out), gate-first honored (K1 order kept -
  debt paid before anything else), docs brought to as-built, gallery added,
  flake proven-to-death and absorbed at the door, doc-liveness leg caught
  a real (reserved-shelf) citation on day one and got prose, not a waiver.
- numbering gate widened D1-D91 (red/green observed); UI tier stays behind
  its named trigger (K2); Stage-4 remains remote WebRTC with untouched
  engine-tiering question; push decision: maintainer, as ever.

- 2026-10-20 (Stage-4 M1 CLOSED