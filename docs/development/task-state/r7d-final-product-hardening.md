# R7d final product hardening — task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / ACCEPTANCE_MATRIX_LOCKED / EVIDENCE_NEXT` |
| Updated | 2026-08-21 |
| Task key | `r7d-final-product-hardening` |
| Scope | Close release-blocking Linux gameplay, persistence/replay, corrupted-input, long-session, renderer/input/audio, lifecycle/recovery and offline-fallback defects without adding new product scope |
| Definition of done | Existing production checks and focused tests cover every R7d risk class on one coherent Linux candidate; every required runnable check passes, every unavailable check remains an explicit non-claim, and only observed release blockers are changed |
| Authority | Working context only; Accepted architecture, `docs/roadmap.md`, tracked release profiles/manifests and exact evidence remain normative |

## Resume in 60 seconds

- **Current conclusion:** R7d is primarily an evidence-and-defect-closure step,
  not a request for a new aggregate runtime or subsystem. Existing production
  paths already contain direct coverage for representative gameplay,
  save/load/replay, corrupt-generation fallback without source rewriting,
  long-session input/state roots, audio, desktop device recovery and
  game/headless parity.
- **Why:** `persistence-replay` injects corrupt RPG and physics generations and
  verifies exact fallback/preservation; `platform` injects device and audio
  loss through the production desktop adapter; `long-session-soak` drives
  3,600 authoritative and application ticks in three windows with periodic
  camera input and root closure.
- **Next action:** Run the locked R7d acceptance matrix from the exact Linux
  candidate and fix only a concrete failing boundary.
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
- **Uncertainty:** The matrix has not yet been executed on this candidate.
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
| H1: Existing checks fully cover R7d without new product code | Source audit covers every named risk class | Candidate-wide evidence has not run yet | Execute the locked matrix and inspect typed failures/reports |
| H2: Long-session behavior remains bounded and root-stable | Workload and finalization require exact tick/sample/root closure | No report from the current candidate yet | Run the release long-session report once |

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

1. Run focused application/persistence/platform tests and the CPU/offline
   product-check rows.
2. Treat the first typed failure as the only implementation target; otherwise
   record the passing exact reports.
3. Re-run the affected row plus its routed non-regression check after any fix.

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
- **Checks:** Source audit complete; candidate execution pending. Desktop is
  `NOT_RUN` because no physical connector is visible.
- **Remaining risk:** Candidate-wide failures and physical-display availability.
- **Promotion needed:** Exact R7d evidence and roadmap status after the matrix
  completes; no new ADR is currently justified.
