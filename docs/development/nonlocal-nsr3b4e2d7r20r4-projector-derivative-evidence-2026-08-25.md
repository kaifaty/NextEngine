# NSR3-B4E2D7R20R4 projector derivative evidence

Status: `PASS / SELECTED GENERALIZED JACOBIAN NONSINGULAR CANDIDATE`.

## Frozen result

Implementation `23a25a17` ran twice with byte-stable semantic result:

```text
18d3d90b0f3994bb4fac9d4cddeef961a93830c88c82818766de0f47c6209a4d
```

Route:

```text
FACE_HESSIAN_NONSINGULAR_CANDIDATE
```

The Linux/x86-64 GNU binary128 profile, exact parent roots, analytic controls,
problem roots, operator ownership and workspace lifecycle all pass. The
diagnostic performs zero Newton iterations, multiplier updates and line-search
trials.

## Projector derivative

All 48 frozen tangent-safe central-difference probes preserve their projector
masks. On active-ball cases, reducing the finite-difference step from `2^-20`
to `2^-24` reduces the maximum error by approximately the expected second-order
factor of 256. The inactive-ball excited transfer case uses exactly
representable free-coordinate probes and has zero error at both steps.

Projector KKT residuals are zero to `6.02e-36`, against bounds between
`1.38e-30` and `2.64e-30`. Analytic symmetry, PSD and nonexpansiveness controls
pass, including active/inactive balls, both box clamps, a tangent direction,
duplicate rows and a forced negative pivot.

## Excited face geometry

The selected generalized Jacobian gives:

| development case | positive rows | rank | nullity | minimum pivot | pivot bound |
|---|---:|---:|---:|---:|---:|
| supported column | 16 | 16 | 0 | `1.001e-4` | `7.94e-31` |
| filled edge | 38 | 38 | 0 | `1.238e-5` | `9.44e-31` |
| filled corner | 42 | 42 | 0 | `1.652e-5` | `1.25e-30` |

Face symmetry residual is zero for supported and `3.76e-37` for both filled
cases, below `~2e-32` bounds. Cholesky reconstruction is at most `1.50e-36`.
Thus numerical rank is not marginal at binary128 precision.

This establishes existence of a nonsingular admissible generalized Jacobian at
`lambda=0`; it does not establish strong regularity of every generalized
Jacobian. In particular, the supported case contains weakly clamped box
coordinates with zero normal margin. Newton directions may cross those kinks,
so mask changes and globalization must be observed explicitly.

## Decision

Proceed to one unregularized semismooth-Newton direction at `lambda=0` on the
positive dual-gradient face, with audited Cholesky solve and certified
exact-dual Armijo search. Do not add a diagonal, prune directions, or iterate
until dual feasibility, ascent and mask behaviour are observed.

This is development evidence only. The v2 filled cases are public development
data; no generalization, runtime, GPU or production authority exists.
