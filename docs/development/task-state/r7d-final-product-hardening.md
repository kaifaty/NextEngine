# R7d final product hardening — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE / FINAL_LINUX_PASS` |
| Updated | 2026-08-28 |
| Task key | `r7d-final-product-hardening` |
| Scope | Close release-blocking Linux gameplay, persistence/replay, corrupted-input, long-session, renderer/input/audio, lifecycle/recovery and offline-fallback defects without adding new product scope |
| Definition of done | Existing production checks and focused tests cover every R7d risk class on one coherent Linux candidate; every required runnable check passes, every unavailable check remains an explicit non-claim, and only observed release blockers are changed |
| Authority | Working context only; Accepted architecture, `docs/roadmap.md`, tracked release profiles/manifests and exact evidence remain normative |

## Resume in 60 seconds

- **Current conclusion:** R7d is complete on the current Linux product
  boundary. No release-blocking gameplay, persistence/replay, long-session,
  renderer/input/audio, lifecycle/recovery or offline-fallback defect is known.
- **Why:** The retained application `26/26`, verification `83/83` plus one
  declared ignored diagnostic, `play`, `persistence-replay`, `audio-scene` and
  3,600-tick release soak all pass. Exact code commit `919663ff…` additionally
  passes the physical Vulkan `platform` row and the complete eight-check native
  gate with `release_ready=true`.
- **Next action:** None for R7d; preserve the exact receipts and hand off the
  release-ready Linux candidate.
- **Current blocker:** None.
- **Do not retry:** Do not substitute a virtual/software display, create a
  duplicate mega-check, or rerun unchanged negative performance evidence.
- **Reconsider when:** A new observed release regression or explicitly scoped
  post-v1 change invalidates one of these receipts.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `crates/verification/src/persistence_replay/runner/finalize.rs` | Directly corrupts RPG and physics generations, requires fallback to the prior generation and proves corrupt bytes remain unchanged | `persistence-replay` is the R7d corrupted-input and persistence-recovery authority |
| `crates/verification/src/live_runtime_performance.rs` and `prepared/*` | Long-session workload drives 3,600 ticks, three 1,200-tick windows, 120 state samples and periodic camera events; finalization requires authoritative/application completion and roots | `performance --scenario long-session-soak --mode report` is the bounded soak authority, independent of R7c hard-gate statistics |
| `crates/verification/src/platform_check.rs` | Production desktop candidate requires normalized controls, resize/focus/fullscreen, device-loss recovery, audio-device reopen and UI overlay success | `platform` remains the renderer/input/audio/lifecycle authority once a physical display is visible |
| Final physical Vulkan `platform` on `919663ff…` | `PASS`: production desktop, lifecycle/recovery, input, audio-device reopen and UI overlay close with state `1e1498bd…bf4e`, ledger `e66f0788…3167` and presentation `773df608…a5a` | The only previously open R7d row is closed on the exact release candidate |
| Final native gate on `919663ff…` | `artifacts/r7e/final-native-gate-919663ff/targets/x86_64-unknown-linux-gnu/target-report.json` is `PASS / release_ready=true`: all eight checks pass; SHA-256 `144c3ab2…`; packaged runtime and desktop smoke pass | R7d is complete on the coherent Linux release candidate |
| `cargo test --locked -p next_application` | `PASS`: 26/26 including manual save/load/close, crash resume, pause-menu lifecycle, preference quarantine and close retry | Application lifecycle and recovery rows are closed for this candidate |
| `cargo test --locked -p next_verification` | `PASS`: 83 tests across unit/integration suites, zero failures; one declared report-only history diagnostic ignored | Replay tamper/corruption, production parity and verification cleanup remain green |
| `xtask play` on `fec8d9d…` | `PASS`: 32 ticks, 52 events, 23 RPG events, state `1e1498bd…bf4e`, ledger `e66f0788…3167` | Representative gameplay and offline deterministic composition pass |
| `xtask persistence-replay` on `fec8d9d…` | `PASS`: 20 ticks, two generations, state `62013d24…da6`, ledger `ad6234b7…08b` | Save/load/replay and corrupt-generation fallback pass |
| `xtask audio-scene` on `fec8d9d…` | `PASS`: 12 cues/facts, 13 non-silent windows, repeated run identical | Displayless audio/fallback row passes |
| Final retained `long-session-soak` on `0ca1f2e9…` | `target/perf/r7d-0ca1f2e9-long-session-final/performance-report-v6.json` is `PASS / REPORT_ONLY`: 3,600 ticks, 3,617 command bodies, state `d51e9841…f10`, peak RSS `58,384,384` and zero diagnostics | Long-session degradation/root closure passes; later release changes do not alter the exercised hot path |
| Post-V6 workspace `host-check` | `PASS`: Linux format, Clippy, workspace tests and doc-tests on pinned Rust 1.97.1 | Distribution/version integration introduces no CPU/offline hardening regression |

## Decisions that still constrain the work

### D-001 — Reuse native product authorities

- **Observation:** R7d risk classes already map to multiple production checks
  whose reports expose subsystem-specific roots, counters and typed failures.
- **Evidence:** `play`, `audio-scene`, `persistence-replay`, `platform`,
  `long-session-soak` and `v1-closure` execute the real reference project and
  production composition roots.
- **Decision:** Use those checks as one explicit acceptance matrix and add or
  change executable code only when a row fails or a required risk is absent.
- **Rejected alternatives:** A new monolithic R7d command would duplicate
  existing authorities, add a new schema surface and hide which product
  boundary failed.
- **Consequences:** Evidence remains composable and failures remain locally
  actionable; final release closure can cite the exact report for each risk.
- **Uncertainty:** None within the bounded R7d risk matrix.
- **Reconsider when:** A roadmap risk cannot be represented by an existing
  report or focused production-path test.

### D-002 — Keep display-dependent acceptance exact

- **Observation:** The Linux host exposes a physical `1920×1080` display and
  the production Wayland/Vulkan path is runnable.
- **Evidence:** The final `platform` report on `919663ff…` passes the real Vulkan
  desktop path; earlier disconnected probes remain exact historical facts.
- **Decision:** Accept the real Vulkan desktop receipt as final R7d evidence.
- **Rejected alternatives:** Virtual displays and headless substitutions do not
  exercise the production Vulkan presentation path and cannot close R2 or the
  desktop platform row.
- **Consequences:** R7d closes without virtual/software substitution.
- **Uncertainty:** None for the current release candidate.
- **Reconsider when:** A future production `platform` run reports a concrete
  regression.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Existing checks fully cover R7d without new product code | Closed: retained CPU/offline rows and final physical `platform` all pass | None observed | Reopen only if a named risk loses its production-path owner |
| H2: Long-session behavior remains bounded and root-stable | Clean release report completes 3,600 ticks with exact root and zero diagnostics | None observed | Closed for this candidate; repeat only after a material hot-path change |

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md`, `docs/architecture/README.md`,
   `docs/architecture/agent-routing.md` and `docs/architecture/glossary.md`
2. SPEC-00, SPEC-04, SPEC-08, SPEC-09, SPEC-11, SPEC-12, SPEC-15,
   SPEC-17, SPEC-18, SPEC-20 and SPEC-29
3. ADR-001, ADR-003, ADR-019, ADR-028, ADR-030, ADR-035, ADR-044,
   ADR-047 and ADR-090
4. `docs/roadmap.md` R7/R7d and the R7a-R7c task states
5. Current product-check, application, persistence, platform and packaging code

## Next action

None. Preserve the final exact-commit reports and hand off the release-ready
Linux candidate.

## Do not retry

- Virtual/software presentation for R2 or `platform` — it cannot prove the
  physical Vulkan path; the real connector is available and must be used.
- A new aggregate R7d runtime command — existing authorities are more precise;
  reconsider only if one named roadmap risk has no executable owner.
- Unchanged completed R7c negative batches — no-retry evidence policy forbids
  selecting new samples from the same failed candidate/method.

## Handoff

- **Workspace state:** Exact release code is commit `919663ff…` on
  `codex/r7c-active-kernel-authority`; the following documentation-only commit
  records completion without changing the validated binaries.
- **Checks:** Retained application/verification/gameplay/replay/audio/soak
  evidence and final physical `platform` pass; the native gate reports all
  eight checks `PASS` and `release_ready=true`.
- **Remaining risk:** None observed within R7d scope.
- **Promotion needed:** None; R7d is complete.
