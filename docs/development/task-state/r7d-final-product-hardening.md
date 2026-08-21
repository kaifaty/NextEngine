# R7d final product hardening — task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / CPU_OFFLINE_PASS / DESKTOP_NON_CLAIM` |
| Updated | 2026-08-21 |
| Task key | `r7d-final-product-hardening` |
| Scope | Close release-blocking Linux gameplay, persistence/replay, corrupted-input, long-session, renderer/input/audio, lifecycle/recovery and offline-fallback defects without adding new product scope |
| Definition of done | Existing production checks and focused tests cover every R7d risk class on one coherent Linux candidate; every required runnable check passes, every unavailable check remains an explicit non-claim, and only observed release blockers are changed |
| Authority | Working context only; Accepted architecture, `docs/roadmap.md`, tracked release profiles/manifests and exact evidence remain normative |

## Resume in 60 seconds

- **Current conclusion:** Every runnable CPU/offline R7d row passes on the
  candidate without a release-blocking defect. Only the physical-display
  desktop row remains an explicit non-claim and may be completed with the final
  Linux acceptance run after the connector becomes OS-visible.
- **Why:** Application `26/26`, verification `83/83` plus one declared ignored
  diagnostic, `play`, `persistence-replay`, `audio-scene` and the clean release
  long-session report all pass. The soak completes 3,600 driver/application
  ticks with exact roots and zero diagnostics.
- **Next action:** Continue R7e distribution closure while preserving the
  display-dependent `platform`/R2 boundary for final acceptance.
- **Current blocker:** Physical-display R2/platform evidence is unavailable:
  Xwayland reports 0x0, NVIDIA reports display inactive and every DRM connector
  reports disconnected. This also remains the sole unfinished R7c workload.
- **Do not retry:** Do not substitute a virtual/software display, create a
  duplicate mega-check, or rerun unchanged negative performance evidence.
- **Reconsider when:** Linux publishes an active physical connector, or a
  failing R7d check proves an actual coverage or product defect.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `crates/verification/src/persistence_replay/runner/finalize.rs` | Directly corrupts RPG and physics generations, requires fallback to the prior generation and proves corrupt bytes remain unchanged | `persistence-replay` is the R7d corrupted-input and persistence-recovery authority |
| `crates/verification/src/live_runtime_performance.rs` and `prepared/*` | Long-session workload drives 3,600 ticks, three 1,200-tick windows, 120 state samples and periodic camera events; finalization requires authoritative/application completion and roots | `performance --scenario long-session-soak --mode report` is the bounded soak authority, independent of R7c hard-gate statistics |
| `crates/verification/src/platform_check.rs` | Production desktop candidate requires normalized controls, resize/focus/fullscreen, device-loss recovery, audio-device reopen and UI overlay success | `platform` remains the renderer/input/audio/lifecycle authority once a physical display is visible |
| Current Linux display probes | `NOT_RUN`: Xwayland 0x0, NVIDIA inactive, all DRM connectors disconnected | Preserve as a non-claim; do not fabricate desktop acceptance |
| `cargo test --locked -p next_application` | `PASS`: 26/26 including manual save/load/close, crash resume, pause-menu lifecycle, preference quarantine and close retry | Application lifecycle and recovery rows are closed for this candidate |
| `cargo test --locked -p next_verification` | `PASS`: 83 tests across unit/integration suites, zero failures; one declared report-only history diagnostic ignored | Replay tamper/corruption, production parity and verification cleanup remain green |
| `xtask play` on `fec8d9d…` | `PASS`: 32 ticks, 52 events, 23 RPG events, state `1e1498bd…bf4e`, ledger `e66f0788…3167` | Representative gameplay and offline deterministic composition pass |
| `xtask persistence-replay` on `fec8d9d…` | `PASS`: 20 ticks, two generations, state `62013d24…da6`, ledger `ad6234b7…08b` | Save/load/replay and corrupt-generation fallback pass |
| `xtask audio-scene` on `fec8d9d…` | `PASS`: 12 cues/facts, 13 non-silent windows, repeated run identical | Displayless audio/fallback row passes |
| Clean release `long-session-soak` on `fec8d9d…` | `PASS / REPORT_ONLY`: 3,600 ticks, 3,617 command bodies, windows `4,019,784/4,751,431/5,582,732 us`, state `d51e9841…f10`, peak RSS `52,228,096`, ready boundaries and zero diagnostics | Long-session degradation/root closure passes; this does not claim R7c/B-12 |
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
- **Uncertainty:** Only the physical-desktop row has not executed on this
  candidate; every CPU/offline row is closed by the evidence above.
- **Reconsider when:** A roadmap risk cannot be represented by an existing
  report or focused production-path test.

### D-002 — Keep display-dependent acceptance exact

- **Observation:** The Linux host currently has no OS-visible physical display.
- **Evidence:** `xrandr --current` is 0x0, `nvidia-smi` reports display inactive
  and `/sys/class/drm/card1-*/status` is disconnected.
- **Decision:** Continue CPU/offline R7d rows now and defer only the real Vulkan
  desktop row until hotplug/EDID becomes visible.
- **Rejected alternatives:** Virtual displays and headless substitutions do not
  exercise the production Vulkan presentation path and cannot close R2 or the
  desktop platform row.
- **Consequences:** CPU/offline defects can be found without contaminating the
  exact R7c worktree; desktop remains an explicit non-claim.
- **Uncertainty:** Whether the connector will become visible during this run.
- **Reconsider when:** Any DRM connector reports `connected` and the desktop
  session publishes a non-zero drawable extent.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Existing checks fully cover R7d without new product code | All runnable candidate rows pass and expose the named roots/failure semantics | Physical desktop row is not currently runnable | Resolve with final physical-display platform acceptance |
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

1. Continue R7e without changing gameplay/runtime semantics.
2. When a physical connector becomes visible, run the release desktop
   `platform` row and the remaining R2 evidence rather than a substitute.
3. Repeat a completed R7d row only after a material affected-path change.

## Do not retry

- Virtual/software presentation for R2 or `platform` — it cannot prove the
  physical Vulkan path; reconsider only after an OS-visible physical connector.
- A new aggregate R7d runtime command — existing authorities are more precise;
  reconsider only if one named roadmap risk has no executable owner.
- Unchanged completed R7c negative batches — no-retry evidence policy forbids
  selecting new samples from the same failed candidate/method.

## Handoff

- **Workspace state:** Work continues in linked worktree
  `/home/kaifaty/Documents/NextEngine-r7d` on
  `codex/r7d-final-hardening`; the clean main worktree remains the exact R7c
  evidence anchor.
- **Checks:** Application `26/26`, verification `83/83`, `play`,
  `persistence-replay`, `audio-scene` and clean release long-session report
  pass. Desktop is `NOT_RUN` because no physical connector is visible.
- **Remaining risk:** Physical-display availability and final post-R7e package
  acceptance; no CPU/offline release blocker was found.
- **Promotion needed:** Final physical-desktop receipt and exact post-V6 native
  acceptance; the roadmap now records the CPU/offline pass without promoting
  the remaining `NOT_RUN` row.
