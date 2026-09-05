# ADR-111: Final-weight corrected walking evaluation

| Field | Value |
| --- | --- |
| ID | ADR-111 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-110](110-applied-command-stop-window.md) |
| Supersedes | ADR-110's exclusion of corrected learned evaluation only; raw contact-presence support proxy only in the separately identified matrix below; no old run or result changes |
| Superseded by | [ADR-112](112-prospective-validated-walking-training.md), successor optimizer/selection exclusion only; this final-only matrix remains frozen |

## Evidence and bounded decision

The [native investigation](../../development/r8b-native-lift-return-2026-09-05.md)
finds speculative contact presence masking exclusive loaded support. Adding
actual free-foot release preserves 24 qualifying switches in diagnostic 3999;
zero, grounded-unloading, partial-substep and short-run controls do not qualify.
The independently identified V8 fixes the impossible applied-command stop tail.
Neither defect warrants restarting the optimizer before final-weight evaluation.

Admit one deterministic, weights-only **inference transfer**, not initialization
or resume, from the completed V7 generation to V8. Freeze
[the evaluation profile](../../../lab/profiles/canonical-walking-corrected-evaluation.v1.json)
before final weights are inspected. Its source is the exact admitted generation,
training profile and **model_9999.pt** only. Resolve its checkpoint hash from the
completed run's verified artifact closure. Missing final weights or incomplete
closure fails; intermediate models, optimizer updates and selection are forbidden.

Require source/target body, joints, actuator order/scales, observation layout and
normalization, controller, safety, reset, reward formula/coefficients, phase and
horizon compatibility. Only ADR-110's command and environment-dependent identities
may differ. Record both descriptors, checkpoint, source manifest, evaluation
profile, repository commit and evaluator/native executable hashes. Use a fresh
external output directory and never replace the active learner's executable.

## Separate matrix, unchanged physical task

All five seeds 1001..1005 must complete 1,200 safe ticks, travel >=3 m,
track forward velocity with MAE <=0.2 m/s, receive exactly zero commands for
the last 180 **applied actions**, and have planar speed MAE <=0.1 m/s there.
Retain >=8 consecutive single-support ticks per side and >=2 switches between
qualified runs. Nonqualifying ticks break a run; they do not erase the previous
qualified side. Short opposite-side runs cannot count as a switch.

Single-support measurement in this new matrix requires positive absolute
vertical impulse in existing canonical `SoleSupport` contacts on exactly the
same one foot in **all four actual physical substeps**, and strictly positive
post-step whole-box minimum Y (canonical integer um) on the other foot.
No positive-force tuning threshold, contact-offset change, all-phase flat-foot
constraint or safety relaxation is introduced. Post-step clearance does not
claim four-substep geometric clearance. Verify its raw height with an exact
Q30/ties-even/floor geometry oracle, not a floating visualization tolerance.

Retain raw presence-based measurements and their failed/passed gates alongside
the corrected report. The old V7 final matrix remains a distinct frozen result,
including its 179-zero defect. Five identical nominal resets are not independent
robustness trials. Passing grants bounded forward/start-stop evidence only,
not natural style, terrain/recovery, Isaac correspondence or runtime/export.

## Verification and rollback

Before inference require focused compatibility/closure/final-selection failures,
exact action conversion, 88-channel V8 adapter reset/step/timeout controls,
support/geometry positive and failure tests, and the unchanged V7 adapter tests.
For each evaluated trajectory replay the exact action tape through the native
auditor and require exact recorded poses, joint order, velocities, commands and
contact flags before accepting derived support. Preserve full terminal frames.
Require the completed five-episode matrix, not a selected successful episode.

Use focused Python/native boundary checks; ADR-110's passing native routing and
Linux host-check remain evidence for its unchanged plant. New public contracts
or broader changes still trigger host-check. Rollback retires this matrix while
preserving V7/V8 and all source evidence. No new optimization is admitted.
Analysis is O(ticks × fixed body/contact bounds), at most 1,200 ticks per episode.
