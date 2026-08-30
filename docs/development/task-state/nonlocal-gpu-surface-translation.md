# Nonlocal GPU surface translation floor — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGP2_PHASE_A` |
| Updated | `2026-08-30` |
| Task key | `nonlocal-gpu-surface-translation` |
| Scope | Localize and repair the translated compressed-pair surface precision floor without changing FCR physics or NCGP1 tolerances |
| Definition of done | NCGP2 selects exactly one predeclared numerical mechanism with reproducible evidence, or closes without a repair when neither counterfactual passes |
| Authority | Working context only; SPEC-38, ADR-076/081, FCR0 and the frozen NCGP1 contracts/results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** NCGP1 is verified refuted for the strict global-f32
  profile, while the corrected formulas are not refuted. The next uncertainty
  is translation-dependent surface/state precision.
- **Why:** the pair near `x=0.25` passes, the same pair near `x=0.75` fails at
  `R_x=1.50362650553e-5`, and `gamma=0` at `x=0.75` passes.
- **Next action:** implement and run the NCGP2 Phase-A translation sweep using
  the unchanged CPU/CUDA solvers and seal its input/work/result identities.
- **Current blocker:** full 4k/50k physics and performance remain forbidden
  until the original translated tiny gate passes.
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
| H1 global-state quantization | failure changes with absolute shift although the mathematical pair is translated unchanged | no accepted-step/ULP trace yet | frozen Phase-A translation sweep |
| H2 surface arithmetic | `gamma=0` removes failure | energy already uses binary64 surface distance/potential | `surface-f64` counterfactual after Phase A |
| H3 trust/globalization | failure ends in trust-radius collapse | CPU succeeds under the same conceptual rules | both arithmetic counterfactuals |
| H4 apparatus defect | prior review found and repaired apparatus defects | final re-review found no remaining load-bearing defect | input/permutation/work closure in the new mode |

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

1. Add a separate NCGP2 report mode/target without changing the frozen NCGP1
   command semantics.
2. Require exact reproduction of the primary failure and both controls.
3. Keep the workspace rollback path and all NCGA/NCGP1 regression outputs
   unchanged before implementing Phase B.

## Do not retry

- Pressure-only f64 — NCGP1 evidence localizes the surviving failure to
  surface participation, not the authorized pressure discriminator.
- `gamma=0` as a fix — it changes the physical model and is only a negative
  control.
- Wider `R_x` or state gate — it would erase the question instead of repairing
  the arithmetic representation.
- Full 50k timing — correctness stop still applies.

## Handoff

- **Workspace state:** branch `codex/water-research`; clean parent commit
  `d0679986` before NCGP2 files.
- **Checks:** NCGP1 clean build/review/sanitizer/regression evidence closed;
  NCGP2 checks not run yet.
- **Remaining risk:** a tiny-only repair may not scale to dynamic graphs,
  boundaries or a 240-step trajectory.
- **Promotion needed:** none; this remains report-only Proposed research.
