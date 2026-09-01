# Physical sound V22 F1r — resource-bounded tournament result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Decision | `SINGLE_RUN_IMPLEMENTATION_REJECT / NO_ARTIFACT / RUN_B_NOT_RUN / QUALITY_UNOBSERVED / FAMILY_CLOSED` |
| Frozen protocol | [P1r](physical-sound-v22-p1r-resource-bounded-continuous-field-protocol-2026-09-01.md), SHA-256 `9352e678508550518c384f5b38decd766cb95b86707c6809d16c8e5055ded9e3` |
| Implementation commit | `e0c44d87a08dff69677b2dbdb9128f3d6f16a562` |
| Implementation files | common `c97288eac3ba988da712b8a6cf8c21c1870939026c70b9b6531d1fc9fecff1a8`; model `36aea212d92861ff2643ed4bad14369c5e26e0131d42c25c0e0d12f53fc1fe36`; tournament `503fae494af10aa819617b269dd55b93aecaae863b281d452ffb2c462a46c18f`; tests `ce1f5b817b8273c19e7b41e765c8e9ca8c44cc6b8d21b964b3cb8c72345e42ae` |
| Equivalence fixture | `35/35`, max absolute/scaled difference `4.212116766488805e-13 <= 5e-12` |
| Published tree | None; both official output paths are absent |
| Product effect | None; authored clips remain complete fallback and runtime ML remains unauthorized |

## Execution

The metadata/equivalence suite passed together with its V21 predecessor suite:
`13/13`. After the clean implementation commit, official F1r run A opened only
fresh train `2201…2224` and development `2301…2312`. The batched process remained
CPU-active at about `1,037,880 KiB` RSS during an observation near 5.6 minutes.

After roughly 26 minutes, training and initial evaluation reached the inherited
structural-corruption path and failed with:

```text
AttributeError: module 'physical_sound_v22_f1r_model'
has no attribute 'validate_request'
```

The V22 model wrapper exported prediction/training/serialization functions but
omitted the inherited fail-closed `validate_request` symbol required by the V21
evaluator. Metadata tests checked evaluator provenance but did not close the
complete callable API surface. This is an implementation-conformance failure,
not evidence about the continuous model.

The runner had not reached atomic publication. It removed staging; official
run-A and run-B targets are absent. Run B was not started.

## Evidence boundary

- `2201…2312` are spent. Their internal training/evaluation state and abandoned
  staging cannot select or validate a repair.
- No metric row, model, prediction, selection, report or tree was published or
  inspected; field quality remains unobserved.
- One-shot test `2401…2412`, integration `2501…2512`, real, protected,
  waveform, force, source-body, dataset/checkpoint and network roles were not
  opened by the implementation.
- V22 does show that the batched path remains near the resource boundary and
  that fixture-only numeric equivalence is insufficient without a closed-world
  evaluator API contract.

## Decision

P1r explicitly states that any exception closes the execution family before
test. Adding the missing alias and replaying the spent role would violate that
boundary. V22 therefore closes with no retry and no quality claim.

A successor requires a genuinely smaller F2 family, fresh identities and an
implementation contract that enumerates and invokes every evaluator callback on
a value-independent synthetic smoke fixture before commit. The next bounded
research compares closed-form feature ridge, another iterative neural retry and
large neural operators; it cannot use V21/V22 quality because none exists.
