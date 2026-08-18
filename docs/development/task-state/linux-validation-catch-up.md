# Linux validation catch-up — complete task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE / LINUX_TARGET_PASS / WINDOWS_DEFERRED` |
| Updated | 2026-08-18 |
| Task key | `linux-validation-catch-up` |
| Scope | Restore current native Linux performance-host and Ubuntu 22.04/glibc 2.35 package compatibility, then close accumulated hardware-GPU, render-content and player/session Linux checks with one full clean-commit native gate |
| Definition of done | A native `x86_64-unknown-linux-gnu` hardware-GPU checkpoint publishes a valid `PASS` target bundle with all eight checks, compatible packaged binaries and exact evidence hashes; retired allocator work is classified without resurrection; Windows pairing remains an explicit non-claim |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, the deferred Windows backlog and exact native-gate reports outrank this file; the retired Linux backlog preserves only historical procedure |

## Resume in 60 seconds

- **Current conclusion:** Linux catch-up is complete on clean commit
  `d15c11a23f9b62a91fa2cc7e400f6ab8409e1766`. The full native target report,
  hardware SDL/ash path, replay/content closure and packaged `game`/`headless`
  launches pass on RTX 3080/Wayland.
- **Why:** The missing Linux Performance V5 host probe now emits complete typed
  fingerprint/preflight evidence, and the Linux package build isolates SDL
  under a C17/glibc-2.35 compatibility configuration. Both packaged binaries
  require at most GLIBC 2.34 on the Ubuntu 26.04 build host.
- **Next action:** Preserve the complete Linux target bundle as exact-commit
  evidence and continue the roadmap on Linux. Windows pairing now belongs to
  the deferred pre-R7 backlog; this general native gate is not the PhysX Stage
  0 platform/replay/correspondence gate.
- **Current blocker:** None for current Linux feature development. B-01 remains
  a deferred R1/R7 release blocker because no same-commit Windows report or
  `native_gate_ready` claim exists.
- **Do not retry:** Do not overwrite or relabel the preserved `de92e37…`
  source-size FAIL or the first `d15c11a…` no-display FAIL. Do not relax the
  `<40%` CPU/GPU preflight or the glibc 2.35 package baseline.
- **Reconsider when:** A later material Linux platform/package/runtime change
  requires a new native checkpoint, or an explicit pre-R7 ADR activates the
  deferred Windows bundle.

## Locked boundary

1. This task closes only the current Linux half of native platform/package
   correctness and accumulated LNX-003/004/005 checks.
2. Linux performance remains `REPORT_ONLY`. The native gate outer performance
   check is `PASS` because the observed-host report and environment evidence
   are valid; its workload verdict is not a hard timing PASS and does not close
   B-12.
3. The package contract remains Ubuntu 22.04 / glibc 2.35. Building on newer
   Linux may select only compatible SDL fallbacks; it may not raise the ABI
   declaration or hide imports from the auditor.
4. Windows correctness, same-commit comparable roots and
   `native_gate_ready = true` are not claimed.
5. The general desktop native gate does not promote the PhysX Stage 0,
   Isaac correspondence, R5 representative hard-performance or TRAIN route.
6. Generated reports, packages and caches remain ignored local artifacts and
   are not committed.

## Sequential implementation order

1. **Completed:** reproduced the current Linux native gate and retained the
   first package ABI failure instead of masking it with the later report
   validator diagnostic.
2. **Completed:** implemented native Linux CPU/RAM/load/clock/storage/OS/GPU
   fingerprint and ready-preflight collection for Performance V5.
3. **Completed:** isolated Linux package build output and configured vendored
   SDL to avoid post-glibc-2.35 symbol versions while retaining GNU APIs.
4. **Completed:** split package build helpers into a bounded module after the
   first committed gate exposed the repository 1000-line source limit.
5. **Completed:** preserved the no-display failure, confirmed all DRM outputs
   were physically disconnected, then reran in a new output only after the
   monitor and hardware platform smoke were restored.
6. **Completed:** verified the final target report, performance environment,
   package manifest, binary GLIBC imports and clean worktree; updated roadmap
   and backlog without widening Windows, B-12 or PhysX claims.

## Decisions

### D-001 — Linux host evidence uses native typed sources

- **Decision:** Read CPU, RAM, load, clock, storage, OS/kernel/BIOS and governor
  from Linux procfs/sysfs, while retaining the existing `nvidia-smi` GPU,
  driver, utilization and thermal probe.
- **Reason:** Native-gate validation requires complete fingerprint and ready
  preflight evidence on Linux; returning unsupported-host made a required
  correctness target structurally impossible.
- **Consequence:** Missing mandatory Linux evidence fails closed. Current
  thresholds and Performance V5 wire format remain unchanged.

### D-002 — Package compatibility is fixed at build time

- **Decision:** Build Linux package binaries in an isolated target cache with a
  repository CMake toolchain that pins vendored SDL to GNU C17, disables libc
  APIs introduced after glibc 2.35 when SDL has fallbacks, and prevents glibc
  2.38+ C23 redirects.
- **Reason:** The runtime auditor correctly rejected GLIBC 2.43 imports; raising
  the declared baseline would violate SPEC-04.
- **Consequence:** The ordinary developer build remains untouched, while
  package output is deterministic with respect to the baseline configuration.

### D-003 — Failed evidence remains evidence

- **Decision:** Use a new commit after the source-size defect and a new output
  after the external display condition changed. Never overwrite either failed
  report.
- **Reason:** Native-gate and performance policy forbid retry-to-green or
  mutation of published evidence.
- **Consequence:** The final PASS is attributable to explicit code and host
  changes, not selective repetition.

## Evidence log

| Evidence | Result | Consequence |
| --- | --- | --- |
| Initial clean gate on `fc5c6b82ba5ae1c5bb259378042d8a2f22693c4a` | `FAIL`: Linux performance host unsupported; package `next_game` imported GLIBC 2.43; failure publication then exposed a secondary report-validation diagnostic | Identified two independent implementation defects; no target claim |
| Scoped xtask tests and clippy | `PASS`: Linux host parser tests, package build isolation test, full xtask tests, `cargo clippy -D warnings`, formatting and diff checks | Probe and package helpers are covered without changing public contracts |
| Standalone package probe | `PASS`: manifest `cd4f49d2bbf901275254f05f1f6d1da4a3561d2ce7287c7ead977d7e03bdea41`; packaged game/headless launch and root parity pass | Build-time compatibility fix satisfies the existing runtime auditor |
| Gate on `de92e37c4347a78706860a913f6674d33e1de204` | Preserved `FAIL`: `SOURCE_FILE_TOO_LARGE`, target-report SHA-256 `82e8d41766dede45f917476a991845ffdd2279983e2a503983f79ed818896ac3` | Required bounded module split; no retry or relabel |
| First gate output on `d15c11a…` | Preserved `FAIL`: hardware display unavailable during `v1_closure`; target-report SHA-256 `9c6ccadd54a7276079f670ad5f1b50b2608e441423f99bd9753ca9d3f2cdfccd` | Sysfs and Mutter showed zero outputs; external condition was fixed before a new output |
| Focused hardware platform smoke after monitor restoration | `PASS`: NVIDIA/Wayland SDL/ash candidate, 4 normalized events, 8 rendered objects, presentation hash `1574d2bf7bce23c6b3b0fd4598ec7e6fe41f3bce5b6cb960a50da54d4fc31283` | Real hardware path is available; no virtual/displayless substitute used |
| Full native gate on clean `d15c11a…` | `PASS`: all eight checks; target-report SHA-256 `f22745ba5f9aacf8a3e940c005ddc969e2c1bee9e5a8e63e4dc0c4228f916930` | LNX-003/004/005 and current Linux native correctness are complete |
| Performance report | Outer `PASS`, inner `REPORT_ONLY`; report SHA-256 `e11aac351ed471d71053ffe1f1614d9455eef3b3671417bdd71d1d1e54f92dd7`; preflight CPU/GPU `18/10%`, postflight `39/23%`, both ready | Linux fingerprint/preflight implementation works; no B-12 hard claim |
| Final package audit | `PASS`: actual maximum GLIBC import 2.34 for both binaries; game `77907999fb6f32ee4196878b023c0fed336074dcf31f6b8e3de66723dbc69ebd`, headless `83d175e065d64be2f87e4d3f0aa8bc82406d29a7ae355b9f21866acb31d2ced8`; identical packaged state/ledger roots | Ubuntu 22.04 / glibc 2.35 baseline and game/headless authority parity hold |

## Explicitly remaining outside this completed task

- A future matching Windows/Linux target pair and `native-gate-compare` on a
  newly selected clean pre-R7 commit for B-01/R1; `d15c11a…` remains historical
  Linux evidence and is not the required future candidate.
- Representative Windows Performance V5 ten-run baselines and fixed three-run
  hard gates after bring-up; representative Linux timing remains report-only
  and B-12 stays open for R7.
- PhysX-only Windows/Linux Stage 0 platform/replay matrix, R5 hard performance
  and Isaac GPU correspondence.
- The R5 completion audit after completed R5g pose correctives and cadence/LOD
  isolation.
