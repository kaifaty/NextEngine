# NSR3-B4E2D7R19R29 linearized-feasibility operator evidence

Date: `2026-08-24`

Status: `PASS / LINEARIZED_FEASIBILITY_OPERATOR_CANDIDATE / REPORT ONLY`

## Outcome

The exact R28 successor now has a closed dimensionless constraint-Jacobian
operator

```text
A = SPACING * J_c
```

and a closed adjoint over the binary64-owned joint pair topology. Static
support remains fixed; only the `3N` fluid coordinates are variables. The
candidate is read-only and does not execute LSQR, classify a feasibility
floor, form a correction or mutate solver state.

```text
particles          6000
owned pairs        340340
maximum degree     113
violated rows      1420
PHR-active rows    2432
boundary rows      3142
interior rows      2858
pair passes        6 / 8
```

## Operator controls

The pair-once JVP and separately folded directed-row reference are byte-exact:

```text
JVP root       0da680021bb4899de02f800ced5c6955019cefd6e3793cf6ba397e0e69c20382
reference      0da680021bb4899de02f800ced5c6955019cefd6e3793cf6ba397e0e69c20382
bound failures 0
max ratio      0
```

The remaining independent controls pass their frozen bounds:

```text
centered finite-difference relative L2   4.0292050312385699e-10  <= 1e-6
adjoint identity relative error          2.1073640541875535e-14  <= 1e-12
translation interior image               exact zero
translation boundary image               nonzero and localized
```

The topology root is
`070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5`.
Exact violated, active, boundary and interior membership roots are present in
the emitted report. All `11/11` route cases, workspace lifecycle and rollback
controls pass. No new solver HVP, model evaluation, trial, precision audit or
outer update occurs.

## Interpretation

R29 closes the prerequisite for a causal range diagnostic. It does not prove
that R28 has reached a discretization floor and does not make the Nonlocal
solver admissible or production-ready.

The next useful question is whether the violated constraint vector contains a
substantial component outside the linearized image of this exact `A`. R30 may
therefore research and freeze a matrix-free least-squares range projection.
Its projected residual and normal residual must be reported without choosing
a post-observation floor threshold. Any nonlinear correction/refinement test
is a separate later experiment.

## Reproducibility

Research/contract commit: `205033b4`.

Implementation commit: `ce328fba`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r29-a.sIIyNZ`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r29-b.sHXfRc`.

Both binaries are `6,908,472` bytes, have SHA-256
`96493348604af67f495b387bac065967f50b7b3c82d791f80d41fae32169c732`
and GNU build ID `3f7014274858d98c13084c5452011e086ba165fa`.

Fresh one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r29-a.EAIqcC`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r29-b.LWC6qs`.

Both exit `0` and reproduce `1,950` stdout bytes exactly:

```text
stdout SHA-256  596c81d4979bed80181d65429e5d253a6464d61f39ec76d8330fe0ad464dfe7b
semantic        f6736b14927c8daedf751649a32cdf3f5ac527116b3037c9367ed921c369c835
route           LINEARIZED_FEASIBILITY_OPERATOR_CANDIDATE
```

These are correctness/reproducibility runs, not timing or performance
measurements.

## Next action

Research and freeze R30 as a zero-state-mutation matrix-free range projection
over the exact R29 operator and frozen R28 residual. Do not execute another
outer update, form or apply a nonlinear correction, tune penalty/cap/policy,
or claim a feasibility floor before that independent contract is frozen and
executed.
