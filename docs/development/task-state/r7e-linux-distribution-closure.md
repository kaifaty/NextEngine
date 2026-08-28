# R7e Linux distribution closure — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE / V6_REPRODUCIBLE_PASS / NATIVE_RELEASE_READY` |
| Updated | 2026-08-28 |
| Task key | `r7e-linux-distribution-closure` |
| Scope | Close Linux v1 versioning, user documentation, third-party dependency/license inventory, protected-data absence and reproducible package evidence, then run final Linux acceptance |
| Definition of done | A versioned Linux package and strict manifest bind getting-started/troubleshooting, exact selected dependency versions/sources/checksums/licenses, copied license texts, a reproducible protected-data scan and isolated product receipts; two clean packages and the final native acceptance gate pass |
| Authority | Working context only; ADR-090, SPEC-04, SPEC-11, SPEC-12 and `docs/roadmap.md` remain normative |

## Resume in 60 seconds

- **Current conclusion:** R7e is complete under ADR-097. Exact code commit
  `919663ff…` produces two byte/mode-identical PackageManifest V6 trees and a
  final Linux native report with all eight checks `PASS` and
  `release_ready=true`.
- **Why:** Independent package A/B each contain 266 files and bind the same
  manifest `47beaf91…`, composition lock `2c5b466d…` and copied binaries. The
  protected-data scan, dependency/license inventory, all three copied-root
  launches, physical desktop smoke and native acceptance pass.
- **Next action:** None for R7e; preserve the exact receipts and hand off the
  release-ready Linux package.
- **Current blocker:** None.
- **Do not retry:** Preserve the 2026-08-21 display-unavailable V6 probe as
  historical negative evidence; the final successor receipt supersedes it, so
  no unchanged rerun is useful.
- **Reconsider when:** A dependency, release feature, toolchain or package input
  changes, or a concrete distribution regression is observed.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| R7b exact PackageManifest V5 `4ece6d7c…` | `PASS`: reproducible 157-file runtime/source package with copied game/headless/tool | Preserve all V5 controls in the V6 successor |
| `tools/xtask/src/package.rs` and `runtime.rs` | Exact inventory, neutral source, copied-root smoke, ELF/glibc/direct-library checks; no release version or full dependency/license graph | Extend the existing package authority, not a parallel packager |
| Package `next_game` string scan | `FAIL`: absolute `/home/kaifaty/.cargo/...` and repository paths are embedded | Remap build/source paths and make the content scan fail closed |
| Cargo metadata plus Cargo.lock | Every selected crate has pinned version/source/license metadata and registry checksums are locked | Generate a canonical selected release dependency inventory offline |
| V6 packaged documents | Root notices/provenance plus bounded `GETTING_STARTED.md` and `TROUBLESHOOTING.md` exist and are mandatory inventory paths | User startup and recovery guidance now travels with the exact package |
| V6 focused package suite | `PASS`: 22/22 tests, including exact inventory, offline dependency/license corpus, protected-marker rejection, ABI-before-smoke and post-smoke byte identity | The new package boundary is fail closed in isolation |
| Current-line integration plus lock repair | ADR-097 preserves ADR-095 generative authoring and ADR-096 active-kernel authority; `cargo fmt`, locked offline metadata, package `22/22`, xtask-bin `45/45` and strict xtask Clippy historically pass | V6 is ready for final post-R7c package/native evidence on the current architecture line |
| Post-R7c native gate on `b14a2e73…` | First attempt exhausted rebuildable Cargo disk state; the bounded retry with incremental disabled reached host-check and exposed `PROTECTED_DATA_DETECTED` only in the package test tool fixture because gate-local `TMPDIR` was under `/home/...` | Preserve both failures as typed pre-publication evidence; keep the scanner strict and remap the generated fixture source prefix instead of weakening protected-data detection |
| Package fixture under repository-local `TMPDIR` after remap | `PASS`: exact pipeline test compiles and scans the fixture with `/nextengine/test-fixture`; real A/B packages on the predecessor were already protected-scan `PASS` | The fixture fix is retained in the final passing candidate |
| Native host-check on `dfa9d6ed…` | The remapped package fixture passed, then 11 external-input tests rejected gate-local manifests because host-check still exported its repository-local state directory as `TMPDIR` | Treat this as one native-gate isolation defect, not 11 test defects: keep check state under staging but create and clean a unique child temporary under the external OS temp base |
| Native host-check isolation fix on `919663ff…` | Unique child `TMPDIR` is created below the external OS temp base, rejected if repository-local, passed to child checks and removed afterward; xtask-bin `218/218`, strict Clippy and boundary scan pass | The systematic host-check failure is fixed without weakening any external-input or protected-data rule |
| Final PackageManifest V6 A/B on `919663ff…` | `PASS`: `artifacts/r7e/final-919663ff-package-a` and `-b` are byte/mode-identical, 266 files each, manifest `47beaf91…`, composition lock `2c5b466d…`; copied `game`, `headless` and `next` launch successfully | Reproducible clean distribution closure is complete |
| Final native gate on `919663ff…` | `artifacts/r7e/final-native-gate-919663ff/targets/x86_64-unknown-linux-gnu/target-report.json` is `PASS / release_ready=true`: all eight checks, target runtime and desktop smoke pass; SHA-256 `144c3ab2…`, package descriptor `3e689039…`, closure `72105952…` | Linux native acceptance and R7e are complete |
| `cargo run --locked -q -p xtask -- v1-package --output artifacts/r7e/package-probe` after scan-order/path fix | `PROTECTED_SCAN_PASS / DESKTOP_BLOCKED`: release build and all pre-smoke distribution checks passed; outer `NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING` preserved inner `PLATFORM_DESKTOP_RUNTIME_UNAVAILABLE`; no package or staging survived | Historical negative control; the unchanged strict path later passes on the final candidate |
| Fresh release ELF byte/string scan | `PASS`: all three binaries contain no local-user/private-key/token marker; expected remapped Cargo paths begin `/nextengine/build-user/` | H1 is closed for the current Linux build products |
| Linux display probe | `AVAILABLE`: physical 1920×1080 output and the production Wayland/Vulkan path are active | Final package and native-gate desktop smoke pass on that production path |
| `cargo test --locked -p xtask --lib` / `--bin xtask` | `PASS`: 114/114 lib and 45/45 bin; focused package subset is 22/22 | V6/report/native-gate positive, tamper, retired-schema and closure-oracle coverage pass |
| `cargo clippy --locked -p xtask --all-targets -- -D warnings`; `boundary-scan` | `PASS` | New package modules preserve lint and repository boundaries |
| `content-package`; `play`; `persistence-replay` | `PASS`: 123 records/64 chunks; state `1e1498bd…bf4e`; replay state `62013d24…da6` | Distribution/version work preserves exact content, gameplay and persistence roots |
| `cargo run --locked -q -p xtask -- host-check` | `PASS`: full Linux workspace format, Clippy, tests and doc-tests on Rust 1.97.1 | All runnable broad CPU/offline gates are green |

## Decisions that still constrain the work

### D-001 — Advance the strict package wire

- **Observation:** Adding mandatory documents, dependency/license evidence,
  release version and scan receipt changes what a complete package means.
- **Evidence:** V5 denies unknown fields and its validator accepts a package
  without every R7e artifact.
- **Decision:** Emit and accept only PackageManifest V6 for the current Linux
  release; retain V5 only as exact historical evidence.
- **Rejected alternatives:** Optional V5 fields or unbound sidecars let old
  validators accept an incomplete distribution under the current version.
- **Consequences:** Package report parsing, native gate validation, SPEC-04 and
  the package fixture tests advance coherently.
- **Uncertainty:** None for the frozen release closure: 44 selected dependency
  records and 105 copied license files are bound in the final package.
- **Reconsider when:** No current shipped V5 compatibility consumer exists;
  any future need requires an explicit reader/migration contract.

### D-002 — Inventory the selected Cargo release closure offline

- **Observation:** Direct ELF libraries describe system runtime prerequisites,
  not statically linked Rust/SDL/Luau/Wasm dependency provenance.
- **Evidence:** V5 reports libc/Vulkan/etc., while Cargo metadata/lock contains
  the selected package graph, exact versions, sources, checksums and licenses.
- **Decision:** Traverse non-dev dependencies from `next_game` with its desktop
  feature, `next_headless` and `next_cli`; package canonical records and copied
  upstream license/copyright files, all bound by hashes.
- **Rejected alternatives:** All-workspace metadata is over-inclusive and raw
  `Cargo.lock` alone is not a human-usable selected dependency inventory.
- **Consequences:** Missing checksum, license expression or license file blocks
  publication before atomic rename.
- **Uncertainty:** Build-only tools may be conservatively included when they
  are on the selected non-dev closure; the inventory states that scope.
- **Reconsider when:** Cargo gains a more precise stable shipped-object graph
  that preserves the same offline, locked evidence.

### D-003 — Make protected-data absence reproducible

- **Observation:** Current path allowlists prevent copied operational files but
  do not detect secrets or builder-local paths embedded inside allowed files.
- **Evidence:** Accepted package binaries visibly contain the builder's home and
  registry paths.
- **Decision:** Remap repository/Cargo/toolchain prefixes during release build,
  then scan every packaged byte for high-confidence private-key/credential and
  absolute-user-path markers. Bind scanner version, file/byte counts and PASS
  receipt in V6; validation reruns the scan.
- **Rejected alternatives:** A repository-only grep cannot prove generated
  binaries and cooked package bytes; a human attestation is not reproducible.
- **Consequences:** A hit aborts staging and publishes no package.
- **Uncertainty:** None observed in the final V6 A/B and native-gate products.
- **Reconsider when:** A stronger reproducible binary provenance scanner
  supersedes the bounded marker scan.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Rust path remapping removes every local-user path from all three ELFs | Closed: real staging scan passed and an independent fresh-ELF scan found no protected marker | None on the current build | Reopen only if build inputs/toolchain or scanner marker set changes |
| H2: Every selected external dependency has exact lock checksum, declared license and distributable license text in its Cargo source | Closed for the locked Linux closure: real offline V6 materialization passed; 22 focused tests cover omission/tamper | None for the current lock | Reopen whenever `Cargo.lock` or selected release features change |

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, ADR-090
2. SPEC-00, SPEC-04, SPEC-09, SPEC-11, SPEC-12, SPEC-15 and SPEC-29
3. ADR-001, ADR-003, ADR-028, ADR-030, ADR-035 and ADR-097
4. `docs/roadmap.md` R7/R7e and R7a-R7d task states
5. Current package manifest, inventory, runtime, smoke, native-gate and report code

## Next action

None. Preserve the final A/B packages and exact-commit native report; rebuild
only after a release input changes or a distribution regression is observed.

## Do not retry

- PackageManifest V5 as R7e closure — it cannot express the mandatory evidence.
- A content scan that skips binaries or generated project bytes — the observed
  leak is inside an allowed release ELF.
- Network metadata/license lookup during packaging — the locked offline source
  cache is the reproducible build authority.
- The former `v1-package` probe while X was `0x0` and every physical connector
  was disconnected — preserve that exact negative result instead of rerunning it.
- A full native gate with default incremental workspace rebuild state on a
  nearly full volume. Keep `CARGO_INCREMENTAL=0`, bound Cargo jobs and preserve
  `target/perf` while reclaiming only regenerable build cache.
- A package test fixture compiled from an unremapped gate-local temporary path.
  The protected-data scanner correctly rejects that path; the fixture compiler
  must retain its neutral source-prefix remap.
- Per-test exemptions for repository-local physical-sound inputs. Those tests
  correctly enforce the external evidence boundary; native host-check owns the
  responsibility to provide an external child `TMPDIR`.

## Handoff

- **Workspace state:** Exact release code is commit `919663ff…` on
  `codex/r7c-active-kernel-authority`; the following documentation-only commit
  records completion without changing the validated package inputs.
- **Checks:** V6 package `22/22`, xtask-bin `218/218`, strict Clippy,
  boundary-scan, complete A/B comparison, copied-root launches and final
  eight-check native acceptance pass.
- **Remaining risk:** None observed within R7e scope.
- **Promotion needed:** None; R7e and R7 are complete.
