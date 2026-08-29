# NSR3-B4E2D7R19R65 best-model dyadic research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / EXHAUSTIVE COMPOSED-MODEL PROBE SELECTED`.

## Hypothesis

v5 proves that a projected-path candidate can be screened through the real
joint objective, but it accepts the largest safe dyadic value. Positive
reduction relative to the frozen normal reference is a production gate, not
a line minimizer. A smaller candidate may yield a lower composed inertia and
a better active face.

Use the exact same 15-HVP face direction. Before testing it, project the
post-Hildreth no-PCG state

```text
baseline input = correction + q
```

through the same box-ball operator and measure `baseline_inertia`. Then
evaluate all 16 frozen dyadic candidates. Admit a candidate only when:

```text
dual change                         < 0
baseline_inertia - candidate_inertia > 0
normal_inertia - candidate_inertia   > 0
```

Select the candidate with the largest strict baseline reduction, equivalent
to minimum candidate inertia. The fixed descending enumeration gives the
larger alpha deterministic priority on an exact tie. Fresh reconstruction,
committed projection and the v5 gamma model-agreement gate remain unchanged.

This is a finite exact search over a predeclared set, not an adaptive line
fit. It follows the globalization requirement of projected Newton while
using the composed Dykstra/inertia model exposed by v5.

- [Bertsekas, Projected Newton methods](https://doi.org/10.1137/0320018)
- [Moré--Toraldo GPCG](https://doi.org/10.1137/0801008)

## Frozen exploratory schedule

```text
outer blocks                         16
identification sweep                  1 per outer / omega=1
face PCG products                    15 per outer / 240 maximum
baseline joint projections            1 per outer
dyadic candidates                    16 per outer / exactly 256
candidate order                       1,1/2,...,2^-15
selection                             maximum strict baseline reduction
normal-reference gate                 strict positive reduction
candidate/commit agreement            frozen v5 gamma bound
tolerance                             none
dense Gram                            none
timing                                none
```

At frozen topology, even the conservative bound of 528 transposes, 272
actions, 17 audits and maximum-degree coordinate work is below the FISTA
596,971,680-term reference. Candidate and baseline projections are counted
separately even though they add no sparse terms.

## Predeclared classifications

```text
BEST_MODEL_DYADIC_ACCELERATION_CANDIDATE
BEST_MODEL_DYADIC_NO_COMPOSED_DESCENT
BEST_MODEL_DYADIC_NORMAL_MODEL_REJECTED
BEST_MODEL_DYADIC_MODEL_AGREEMENT_REJECTED
BEST_MODEL_DYADIC_CURVATURE_REJECTED
BEST_MODEL_DYADIC_RECURRENCE_REJECTED
BEST_MODEL_DYADIC_CHECKPOINT_MODEL_REJECTED
BEST_MODEL_DYADIC_REFERENCE_RETAINED
```

Strict acceleration still requires terminal raw and projected-gradient
dominance over FISTA at lower counted sparse work. Do not change direction
depth, dyadic set, production gates or runtime state.
