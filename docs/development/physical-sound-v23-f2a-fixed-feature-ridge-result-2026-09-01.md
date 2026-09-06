# Physical sound V23 F2a — fixed-feature ridge result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `SINGLE_RUN_IMPLEMENTATION_CONFORMANCE_REJECT / NO_ARTIFACT / QUALITY_UNOBSERVED / TRAIN_DEVELOPMENT_SPENT / RUN_B_NOT_STARTED / TEST_INTEGRATION_RETIRED_UNOPENED` |
| Protocol | [P2a](physical-sound-v23-p2a-fixed-feature-ridge-protocol-2026-09-01.md) |
| Implementation commit | `da07d6f49b0297d0f32bfc6ba274ec496e613ffe` |
| Planned output A | `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v23-f2-fixed-feature-ridge-run-a` |
| Published artifact | None; final and owned hidden staging targets were both absent after failure |
| Scientific claim | None; candidate quality was not published or observed |

## Result

The value-independent E2a suite passed five focused tests, including two
byte-identical whole-API smoke executions, before implementation commit. The
clean-commit guard then resolved exactly to `da07d6f…3ffe`.

Official run A opened the frozen `2601…2712` train/development meshes and truth,
fit the closed-form candidates and entered the inherited remesh evaluator. It
then failed before publication with the exact stable CLI diagnostic:

```text
error: module 'physical_sound_v23_f2_model' has no attribute 'analytic_surface'
```

The missing callback is used by
`physical_sound_v21_f1_tournament._truth_at_probes`, reached through
`_continuous_remesh`. `analytic_surface` was not included in P2a's
`REQUIRED_MODEL_API`; the miniature smoke invoked direct-probe prediction and
the inherited gates/selection adapter but did not execute that inherited truth
probe path. The smoke therefore proved its declared API, corruptions and
serialization, but not the entire effective evaluator dependency closure.

No nine-file artifact, partial final directory or owned hidden staging
directory exists. Run B was not started. The reserved `2801…2912` test and
integration values were never generated and retire unopened under Roadmap V24.

## Interpretation

- This is an implementation-conformance reject, not evidence for or against
  fixed-feature ridge, continuous residual quality or the frozen thresholds.
- `2601…2712` are permanently spent because their values were opened before the
  exception. Adding the trivial alias and replaying would violate P2a's
  no-repair stop rule and create selection from an opened role.
- V23 closes. It must not be repaired, replayed, reconstructed from process
  state or used as a quality baseline.
- The reusable lesson for V24 is stronger than a manually enumerated callback
  list: before any candidate values, its value-independent fixture must execute
  the complete owning pipeline entry point, including every inherited control,
  remesh/truth probe, mutation, serializer and publication/abandon branch.

## Decision

Proceed to Roadmap V24 D0. The next protocol will freeze one external evidence
record/builder and an end-to-end no-role-value contract fixture before any
teacher, real-object or neural candidate value is opened. It will not reuse the
V23 bands or repair the V23 wrapper.
