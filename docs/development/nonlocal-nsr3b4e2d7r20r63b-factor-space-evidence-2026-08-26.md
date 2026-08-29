# NSR3-B4E2D7R20R63B factor-space evidence

Status: `REFUTED_BOUNDED / BINARY64_FACTOR_LEFT_NONCONTRACTIVE`.

## Outcome

R63B is a valid asymmetric negative result. The existing binary128 Cholesky
factor reconstructs the captured 102-dimensional matrix within its inherited
bound, and the newly constructed binary128 triangular inverse contracts on
both sides. After independent projection to binary64, all 20,808 `Dot2Err`
intervals still contain their independent exact-dyadic values. The right
factor defect remains excellent, but the left factor defect exceeds one:

```text
                         exact projected       Dot2Err candidate
||I - L64 Z64||inf       1.24231041023e-14      1.28569305279e-14
||I - Z64 L64||inf       1.57913154409e+00      1.57913154409e+00
```

The frozen requirement was two-sided contraction. The resulting candidate is
therefore rejected at `BINARY64_FACTOR_LEFT_NONCONTRACTIVE`; the successful
right side may not repair or relabel this result.

## Frozen execution

```text
command:
  /tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
    --nonlocal-al-generalization-v5-factor-space

status: FAIL
route: BINARY64_FACTOR_LEFT_NONCONTRACTIVE
semantic SHA-256:
  28bfb856cf7b537b0c00acf39626f703c1007c7e59e20707aca58b4e2ae160fc
stdout byte SHA-256, both credited executions:
  320d6ea1842d49fef30b6821586b189dc923ef5bcfe9e3c22e86f4a38bc24216
```

The two credited executions are byte-identical. Their nonzero exit is the
expected representation of the frozen negative route. An earlier apparatus-
only execution stopped at `FACTOR_SPACE_CAPTURE_REJECTED`, because the capture
hook observed all 125 parent inverse audits. Contract revision 2 selects the
unique already frozen R63 matrix root and requires exactly one selected audit;
that rejected execution exposed no factor arithmetic result and has no
scientific credit.

## Capture, reconstruction and exact factor

The capture observes 125 parent audits but selects exactly one audit and 102
existing inverse columns at the immutable matrix root. No new factorization is
performed.

```text
matrix root  aa401d0191ad53b7caa7837c827913fa711e1fdc433bc200c020719d297fb09d
inverse root 467e815a4813e7774523699147db38fbcda06dea9b93bfb1696b410e0df0cfd7
factor root  782588f764834bf498b2a25ad4ab20e84083b51a3d9016786e0c411227c70358

||A - L L^T||inf outward       9.76256139804e-36
maximum reconstruction entry  1.28120133361e-36
inherited maximum bound        2.16065255408e-33
```

The fixed-order binary128 forward substitution executes exactly 102 columns,
176,851 multiply-subtract terms and 5,253 divisions. Its diagonal spans
`3.03182322199e-16` to `9.05302745911e-2`. Before binary64 projection, both
factor defects are contractive:

```text
||I - LZ||inf outward = 8.09851057768e-33
||I - ZL||inf outward = 8.01341608801e-18
```

Thus the negative result is specifically a binary64 representation/order
boundary, not a failure of the captured factor or triangular construction.

## Correspondence and conditioning

Right and left audits each contain `10404/10404` exact entries and execute
10,404 candidate dots. All literal well-conditioned, underflow, deliberately
shrunk-interval and reconstruction-corruption controls pass.

The directed lower bound retained from the matrix/inverse relation is:

```text
cond_inf(A) lower bound       2.57538142950e32
u64 * lower bound             2.85924776023e16
```

This establishes severe conditioning of the Gram matrix, but is not an exact
condition number and does not prove that every rank-aware, scaled or directed
algorithm must fail.

## Interpretation and next question

R63B refutes an ordinary two-sided binary64 inverse representation for this
factor. It does not yet refute a directed triangular solution certificate.
For a square factor, a bound `||I-LZ||<1` can establish nonsingularity and an
inverse-norm bound without also requiring `||I-ZL||<1`; whether that can close
the actual forward/backward solution and NNQP sign decisions must be frozen
and tested separately with its own rounding and work ledger.

The next research therefore compares two bounded routes before implementation:

1. a one-sided, a-posteriori triangular solve certificate that never forms
   the full inverse of `A`;
2. rank-revealing treatment of the underlying active projector rows if the
   transpose solve or sign enclosure cannot be certified.

No NNQP right-hand side, rank threshold, row removal, center, trajectory,
timing, runtime/GPU path or production authority is admitted by R63B. R64 and
R65 remain blocked.

## Parent regression

- R63A remains byte-exact at `b0dba1bc...5b4c9`, semantic
  `c08b7abe...7639b`, route `MIXED_PROJECTION_CONTRIBUTION`.
- R63 remains byte-exact at `b54cd01e...9d472`, semantic
  `70f298d4...11496`, route `BINARY64_RIGHT_NONCONTRACTIVE`.
