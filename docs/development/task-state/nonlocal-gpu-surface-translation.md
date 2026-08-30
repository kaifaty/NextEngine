# Nonlocal GPU surface translation floor — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGP2_REVIEW_PENDING` |
| Updated | `2026-08-30` |
| Task key | `nonlocal-gpu-surface-translation` |
| Scope | Localize and repair the translated compressed-pair surface precision floor without changing FCR physics or NCGP1 tolerances |
| Definition of done | NCGP2 selects exactly one predeclared numerical mechanism with reproducible evidence, or closes without a repair when neither counterfactual passes |
| Authority | Working context only; SPEC-38, ADR-076/081, FCR0 and the frozen NCGP1 contracts/results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** the author evidence selects H1, a global binary32
  state-update floor. `surface-f64` leaves the failure unchanged, while the
  shared-anchor `hi+lo` tiny discriminator passes all nine translations.
- **Why:** the original `x=0.75` route remains at
  `R_x=1.50362650553e-5` under surface-only binary64, but reaches
  `1.62124633789e-6` with the local binary32 part and matches CPU within
  `3.85e-8 m`.
- **Next action:** receive the independent NCGP2 review; if it accepts the
  representation semantics and receipts, freeze a scalable per-cell/per-point
  state contract before any 4k/50k run.
- **Current blocker:** full 4k/50k physics and performance remain forbidden
  because the passing representation is a boundary-free shared-anchor
  specialization, not a scalable graph/boundary implementation.
- **Do not retry:** pressure-only f64 promotion, tolerance widening, disabling
  surface, or timing the graph-only path; none answers the failing surface
  state route.
- **Reconsider when:** Phase A or the two frozen counterfactuals falsify the
  current translation-floor explanation.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/development/nonlocal-gpu-full-step-evidence-2026-08-30.md` | `VERIFIED_PHYSICS_REFUTED` | strict global-f32 full-step profile cannot proceed to performance |
| independent reviewer counterfactuals retained in NCGP1 evidence | `x=0.25 PASS`; `x=0.75,gamma=0 PASS` | absolute translation and surface participation are jointly causal clues, not yet a mechanism proof |
| NVIDIA CUDA floating-point guide and IEEE note | `REPORT_ONLY` | binary32 spacing and subtractive cancellation are plausible; local experiments remain authority |
| `docs/development/nonlocal-gpu-surface-translation-evidence-2026-08-30.md` | author `H1 SUPPORTED`, review pending | surface-only f64 is falsified; shared-anchor local state closes the retained pair and sweep |

## Decisions that still constrain the work

### D-001 — Preserve the NCGP1 negative result

- **Observation:** the repaired author package and single re-review agree on
  the same typed failure, work/result roots and route.
- **Evidence:** commits `3bc5e81d`, `0bd26ad6`, `d0679986`.
- **Decision:** NCGP2 is a successor discriminator. It changes no NCGP1 result,
  tolerance, work ceiling or product status.
- **Rejected alternatives:** accepting `1.5036e-5` as close enough or reporting
  graph timing as full water timing.
- **Consequences:** performance and trajectory work remain `NOT_RUN`.
- **Uncertainty:** whether the lost precision is in state updates or surface
  operator products.
- **Reconsider when:** a frozen successor arithmetic representation passes the
  original input and retained controls.

### D-002 — Test representation before broad solver changes

- **Observation:** CPU long double succeeds from the same initial binary32
  bytes, while removing only surface tension makes the CUDA route succeed.
- **Decision:** first measure endpoint ULP, encoded separation, CPU correction
  and CUDA route over a fixed translation sweep; then evaluate only
  `surface-f64` and `compensated-state-f32` in that order.
- **Rejected alternatives:** another pressure discriminator, a new solver,
  blended precision changes or 50k optimization before tiny admission.
- **Consequences:** one first-specific mechanism is selected or the package
  closes without implementation authority.
- **Uncertainty:** a translation-correlated failure can still arise from the
  trust controller rather than either arithmetic boundary.
- **Reconsider when:** both counterfactuals fail with exact apparatus.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1 global-state quantization | shared-anchor local state passes all translations; original and surface-f64 routes fail identically | scalable per-particle/per-cell form is not implemented | independent review, then successor storage contract |
| H2 surface arithmetic | `gamma=0` removes failure | surface-f64 does not change any failing route | falsified for the retained pair |
| H3 trust/globalization | original failure ends in trust-radius collapse | local representation restores the frozen solver route | falsified for the retained pair |
| H4 apparatus defect | shared-anchor factoring is a specialized representation that needs independent scrutiny | CPU/input/permutation/work identities and three sanitizers pass | independent review at commit `2a0c38a2` |

## Required context

Read these sources in precedence order before acting:

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/plans/nonlocal-continuum-formula-reclosure/00-formula-contract.md`.
3. `docs/plans/nonlocal-corrected-gpu-full-step/00-physical-performance-contract.md`
   through `03-review-repair-closure.md`.
4. `docs/development/task-state/nonlocal-gpu-full-step.md` and its final
   evidence.
5. `docs/plans/nonlocal-gpu-surface-translation/00-surface-translation-contract.md`.

## Next action

1. Await the exact read-only independent review of commit `2a0c38a2`.
2. If accepted, freeze anchor ownership, renormalization, graph transitions,
   boundaries, rollback and work/memory receipts for a scalable successor.
3. Reopen 4k correctness only after that successor passes the original tiny
   route; keep 50k timing stopped until 4k trajectories pass.

## Do not retry

- Pressure-only f64 — NCGP1 evidence localizes the surviving failure to
  surface participation, not the authorized pressure discriminator.
- `gamma=0` as a fix — it changes the physical model and is only a negative
  control.
- Wider `R_x` or state gate — it would erase the question instead of repairing
  the arithmetic representation.
- Full 50k timing — correctness stop still applies.

## Handoff

- **Workspace state:** branch `codex/water-research`; NCGP2 candidate commit
  `2a0c38a2`, evidence/task-state update not yet committed.
- **Checks:** two byte-identical clean Release builds/runs, retained
  graph/operator/solver controls and memcheck/initcheck/synccheck pass; review
  is running.
- **Remaining risk:** the shared-anchor tiny representation may not satisfy
  the frozen canonical `hi+lo` meaning or scale to dynamic graphs, boundaries
  and a 240-step trajectory.
- **Promotion needed:** none; this remains report-only Proposed research.
