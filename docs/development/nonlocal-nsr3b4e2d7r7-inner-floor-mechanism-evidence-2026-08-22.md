# NSR3-B4E2D7R7 inner-floor mechanism evidence

Date: `2026-08-22`

Status: `PASS / TRUST_MODEL_OR_DERIVATIVE_RECLOSURE / PRIVATE_ONLY`

## Reproducibility

Two clean Release builds produce byte-identical 4,892,616-byte executables at
SHA `945c1a92568692f7e8db15050633793a31243acc7f2f1fe2ef4910985babb9a1`
and Build ID `3498c4f31e1535ea0ebacc1733c884508be49760`.

Both fresh D7R7 processes exit zero with empty stderr and byte-identical
37,619-byte stdout reports at SHA
`3615964074fd477ab384f5710b274d8da4f4d1c77fd3f11ecb08a45eec0039ff`.
The semantic result is
`a29e3f7921f35065b3bcac1ebbf7f6b1d25422e059b4287d3b7a31cf76865588`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r7.WPYmr0`.

D7R5 remains at `7ea5489f...16d7` and the expected-failing D7R6 remains at
`6979ebf9...9f6f` in both builds. The three failed-state replays and the
`1e-10/1e-11/1e-12` common-state control are exact. Inactive, reset and forced
rollback pass; public commits and trajectory steps remain zero.

## Mechanism result

The three exact pre-failure roots are:

```text
eta=1e-8   21ad77e22bdba4520ca231bb78d51947a1b67e4263e08dae33af13b0aafabd05
eta=1e-9   075442656934aeed156642091e9fc1ed41bc739513b5710990cdfb231d8aabc2
eta=1e-10  299e4ce372a8a3d418ac6f354c70c362fd772acdf278464fb603a3881527901c
```

No replay changes its PHR active root. Minimum PHR switching margin remains
about `0.578`, so the current failure is not an inequality active-set kink.

The first `1e-8` and `1e-9` proposals cross one or more boundary-pair support
memberships. Their later trials retain exact topology but still show positive
quadratic models and negative raw/direct actual reductions. Therefore the
pair crossing is real observability, but it does not explain the complete
failure.

The tight `eta=1e-10` failure is stronger:

- all five proposals keep active, fluid-pair and boundary-pair roots exact;
- minimum distance to the compact-support horizon is
  `5.0986992405910314e-14 m`;
- every quadratic model predicts descent;
- every raw and fixed-order direct reduction reports ascent;
- no trial is inside the frozen eight-ULP merit-floor classification.

For the first tight trial:

```text
predicted reduction       +2.8343305480444868e-19
raw actual reduction      -2.1345772371894611e-15
direct actual reduction   -2.1344440162136736e-15
predicted scale            about 0.327 total-energy ULP
direct scale              -2460.846 total-energy ULP
```

By the last trial the direct ascent is still about `1445.875` total-energy
ULPs while the trust radius has contracted below the unchanged minimum.

## Interpretation

Raw total subtraction and factored/direct reduction agree in sign and scale.
The D7R1 raw-cancellation hypothesis therefore does not explain this state.
Exact topology also rules out ordinary neighbor-set discontinuity for the
tight replay.

What remains is a local model/evaluation question close to the smooth kernel
support edge: either binary64 kernel/density/energy accumulation is too poorly
conditioned for the analytic model at this scale, or the local gradient/HVP
formula does not match the representable energy map there. The current report
cannot distinguish those cases.

## Decision

Select `TRUST_MODEL_OR_DERIVATIVE_RECLOSURE`. Do not change raw admission,
trust limits, kernel support, pressure gates, `beta` or solver family.

Freeze D7R8 as a standalone extended-precision energy/sign discriminator. It
must independently reevaluate the exact binary64 current/trial positions under
a frozen Linux x86-64 long-double profile, compare pair membership and energy
signs, and authorize only a later precision remedy or local derivative
reclosure.

