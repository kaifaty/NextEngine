# R7e Linux distribution closure — task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / V6_CPU_GATE_PASS / DESKTOP_EVIDENCE_BLOCKED` |
| Updated | 2026-08-21 |
| Task key | `r7e-linux-distribution-closure` |
| Scope | Close Linux v1 versioning, user documentation, third-party dependency/license inventory, protected-data absence and reproducible package evidence, then run final Linux acceptance |
| Definition of done | A versioned Linux package and strict manifest bind getting-started/troubleshooting, exact selected dependency versions/sources/checksums/licenses, copied license texts, a reproducible protected-data scan and isolated product receipts; two clean packages and the final native acceptance gate pass |
| Authority | Working context only; ADR-090, SPEC-04, SPEC-11, SPEC-12 and `docs/roadmap.md` remain normative |

## Resume in 60 seconds

- **Current conclusion:** PackageManifest V6 is implemented and the complete
  Linux CPU/offline validation matrix passes. Its real staging path passes
  locked/offline dependency and license materialization plus the protected-data
  scan before any display-dependent smoke. Fresh release ELFs contain no
  `/home/`, `/root/`, `/Users/`, private-key or bounded GitHub token marker;
  Cargo source paths are remapped under `/nextengine/build-user`.
- **Why:** The post-fix real package probe reached `next_game` smoke. That call
  is ordered strictly after the complete staging byte scan, and the failed
  build removed staging without publishing the requested package.
- **Next action:** Once a physical connector is OS-visible, build clean package
  A and B, compare every file byte and mode, validate both, then feed the exact
  package receipt into final Linux native acceptance.
- **Current blocker:** `DISPLAY=:0` remains `0x0`; DRM reports DP-1/2/3 and
  HDMI-A-1 disconnected. SDL consequently reports that the video driver added
  no displays. Final desktop acceptance and R7c R2 share this external blocker.
- **Do not retry:** Do not rerun `v1-package` while display state is unchanged;
  the 2026-08-21 real V6 probe already proved CPU/staging/scan progress and
  failed only at the first desktop smoke.
- **Reconsider when:** `xrandr --current` reports a non-zero mode and at least
  one physical DRM connector reports `connected`.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| R7b exact PackageManifest V5 `4ece6d7c…` | `PASS`: reproducible 157-file runtime/source package with copied game/headless/tool | Preserve all V5 controls in the V6 successor |
| `tools/xtask/src/package.rs` and `runtime.rs` | Exact inventory, neutral source, copied-root smoke, ELF/glibc/direct-library checks; no release version or full dependency/license graph | Extend the existing package authority, not a parallel packager |
| Package `next_game` string scan | `FAIL`: absolute `/home/kaifaty/.cargo/...` and repository paths are embedded | Remap build/source paths and make the content scan fail closed |
| Cargo metadata plus Cargo.lock | Every selected crate has pinned version/source/license metadata and registry checksums are locked | Generate a canonical selected release dependency inventory offline |
| V6 packaged documents | Root notices/provenance plus bounded `GETTING_STARTED.md` and `TROUBLESHOOTING.md` exist and are mandatory inventory paths | User startup and recovery guidance now travels with the exact package |
| V6 focused package suite | `PASS`: 22/22 tests, including exact inventory, offline dependency/license corpus, protected-marker rejection, ABI-before-smoke and post-smoke byte identity | The new package boundary is fail closed in isolation |
| `cargo run --locked -q -p xtask -- v1-package --output artifacts/r7e/package-probe` after scan-order/path fix | `PROTECTED_SCAN_PASS / DESKTOP_BLOCKED`: release build and all pre-smoke distribution checks passed; outer `NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING` preserved inner `PLATFORM_DESKTOP_RUNTIME_UNAVAILABLE`; no package or staging survived | Do not weaken or repeat the package path until display state changes |
| Fresh release ELF byte/string scan | `PASS`: all three binaries contain no local-user/private-key/token marker; expected remapped Cargo paths begin `/nextengine/build-user/` | H1 is closed for the current Linux build products |
| Linux display probe | `NOT_AVAILABLE`: X screen `0x0`; all enumerated physical DRM connectors are `disconnected` | Package A/B and final native gate remain honest `NOT_RUN` |
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
- **Uncertainty:** Exact final dependency/license counts until implementation.
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
  are on the selected non-dev closure; the inventory will state that scope.
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
- **Uncertainty:** Native third-party build products may reveal another path
  prefix; the first V6 real build is the discriminator.
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
3. ADR-001, ADR-003, ADR-028, ADR-030, ADR-035 and the R7e successor ADR
4. `docs/roadmap.md` R7/R7e and R7a-R7d task states
5. Current package manifest, inventory, runtime, smoke, native-gate and report code

## Next action

1. Create the coherent V6 implementation/evidence checkpoint in the linked
   worktree while preserving the exact clean R7c main anchor.
2. Wait for an observed physical display-state transition; do not substitute a
   virtual display or repeat the known-negative package probe.
3. Build two exact packages, compare complete bytes/modes and run final native
   Linux acceptance against their bound receipt.

## Do not retry

- PackageManifest V5 as R7e closure — it cannot express the mandatory evidence.
- A content scan that skips binaries or generated project bytes — the observed
  leak is inside an allowed release ELF.
- Network metadata/license lookup during packaging — the locked offline source
  cache is the reproducible build authority.
- `v1-package` while X remains `0x0` and every physical connector is
  `disconnected` — the current code already reaches and fails desktop smoke.

## Handoff

- **Workspace state:** R7e begins in linked worktree
  `/home/kaifaty/Documents/NextEngine-r7d`; main remains the clean R7c anchor.
- **Checks:** R7d CPU/offline matrix, full workspace `host-check`, V6 focused
  package checks, product roots and the real pre-desktop staging/scan path pass.
- **Remaining risk:** Full selected license closure, native path remapping,
  final desktop/R2 availability and total clean package/gate runtime.
- **Promotion needed:** Accepted V6 ADR plus SPEC-04/SPEC-11/roadmap updates in
  the same coherent implementation checkpoint.
