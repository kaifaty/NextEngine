# NSR3-B4E2D7R19R4 Krylov model-image evidence

Date: `2026-08-23`

Status: `PASS / KRYLOV_MODEL_IMAGE_RESIDUAL_CANDIDATE / REPLAY ONLY / NO TRIAL`

## Outcome

The final recursive CG residual supplies a valid zero-HVP model image for the
exact sixth trust solve:

```text
H(step) ~= r_final - g
```

Against one direct sparse `H(step)` oracle, the residual-derived image has
relative L2 error `1.24463574613552e-15` and maximum component error
`1.58127455314667e-13` relative to the direct summation scale. More
importantly, `step·H(step)` and the predicted reduction are bit-exact to the
direct oracle.

The accumulated `sum(alpha*H(d))` lane also passes every `1e-10` bound, but
its quadratic/model scalars differ slightly and it adds vector accumulation
on every iteration. Frozen precedence therefore correctly selects the cheaper
and more accurate residual-derived lane.

No trial is formed and no recurrence grace or production cap is selected.

## Parent and target correspondence

D7R19R3 remains exact after exposing the internal step/residual/image vectors:

```text
R3 stdout SHA-256  a1038937496f31ed64008eb8e366763e2da9875e3935194955d68604a7b23771
R3 semantic        156782481d783abc500c1a1b888b693d158f8cd30503412d41285f4c725df6d8
R2 retained SHA    3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0
```

The exact target remains:

```text
current root       54bafbf48d0798438fd9baad9fb91e67c7b5cf6b12e37c4bb1d49694384ddf8a
predicted root     36112dde1e0b274c5b9216f4818b82977a0c80dc257478c111b0f0a9390d2d7e
prefix root        f478832923673956bc98d8067fff9bdeb5c3dab109c0a8e239ad12c6844d60dd
trust radius bits  0x3f8999999999999a
recurrence         33 HVP / FORCING_CONVERGED
step root          74a9b58726d5d0279699498c41a70fb0e199f45e531d11befd2bc2dfe692d2bd
residual root      63a5d61b013fd82a291e1ff2821226888fa33bc911d11c3bdbaec4cebd92f3cc
```

## Direct oracle

The direct model image root is
`d23c1f362768627bd0e457871313fdd1115281fa1c47eb6c53e84d227646dd66`.
Its model scalars are:

```text
g·step               -1.5662997029469477e-20
step·H(step)           1.5662997029469495e-20
predicted reduction    7.8314985147347295e-21
```

All are finite and the predicted reduction is positive.

## Residual-derived lane

```text
image root                    ac1a994e1fcbaa92888ce43862441175fc2ac7fb835ef6d798b8aea8c33e76f5
relative L2 image error       1.2446357461355193e-15
maximum scaled component      1.5812745531466691e-13
quadratic relative error      0
predicted relative error      0
quadratic bits                0x3bd27dd9b2871eb3 (direct exact)
predicted bits                0x3bc27dd9b2871ea7 (direct exact)
```

The recursive residual gap is therefore observable in the vector root but
cannot affect the exact quadratic model scalar at this state.

## Accumulated lane

```text
image root                    fd33dbf386545dac72b70821559473064b71e8f867de11a8fe52ca39fda38092
relative L2 image error       1.2891954296294596e-15
maximum scaled component      1.5818634091123394e-13
quadratic relative error      3.8425156212993021e-16
predicted relative error      3.8425156212993125e-16
```

This lane is numerically safe but inferior under the frozen preference:
slightly larger model-scalar error and O(N) accumulation each Krylov
iteration instead of one final subtraction.

## Work and safety

New R4 work is exactly one direct oracle HVP and one workspace build/release.
There are zero precision audits, formed trials, acceptances or all-pair calls.
Static binding, lifecycle and rollback pass. R3's parent recurrence remains
33 HVPs and is not candidate work.

## Reproducibility

Implementation commit:
`9182d78f` (`research: select residual-derived model image`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r4-a.SFzpQw`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r4-b.D0xobU`.

Both binaries are `5,842,504` bytes, have SHA-256
`35bcba9463e72ad43e14a0f1c31f2b76d419bbbe4cf1821c1359bfc8323978da`
and GNU build ID `93ec05996b2f2d84c7b102f572329b80b6b04171`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r4-a.bFwPkp`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r4-b.IoEEUC`.

Both exit `0`, emit empty stderr and reproduce:

```text
stdout-with-LF bytes  2,875
stdout SHA-256        798e8005aaa503eedb7a90ed9fd3590230cec7bbadf85536b39ac135ac7cb3a0
semantic result       99b77c1e0d4750911af11a5cc9933d512dc22ee3041a873eabccc2a8389f2cd0
route                 KRYLOV_MODEL_IMAGE_RESIDUAL_CANDIDATE
```

## Decision

Select residual-derived `H(step)` as the only candidate for the next bounded
stage. Keep the direct HVP as an oracle/control, not candidate work. Do not
carry the per-iteration accumulated lane forward.

Research/freeze D7R19R5 as a sixth-trial-only reclosure:

1. retain the base 32-recurrence-HVP watchdog;
2. permit exactly one guarded near-convergence recurrence grace HVP when the
   residual is within `1.25*eta`, the last eight ratios decrease, curvature is
   positive and the iterate is interior;
3. use `r_final-g` for the model image, with the direct oracle retained only
   as correspondence control;
4. evaluate the exact sixth trial's predicted/divided reduction, precision
   audit, acceptance and radius result in shadow form;
5. do not commit the accepted state or continue the transaction.

Only that trial-level evidence can authorize a later full candidate
transaction.
