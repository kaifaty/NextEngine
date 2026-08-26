# NSR3-B4E2D7R20R63G rectangular-factor precision research

Status: `FROZEN FOR IMPLEMENTATION`.

## Question

R63F proves that the `102 x 315` tangent operator has represented row rank
102. Can a no-drop orthogonal factorization preserve its weakest observed
direction in binary64, or does coefficient/factor rounding erase that signal
before any NNQP right-hand side is solved?

This is now an arithmetic question, not a rank-policy question.

## Direct formulation

Let `A` contain the 102 tangent rows and set `C=A^T`, so `C` is `315 x 102`.
For the frozen projector scale `s=1/(1+eta)`, the R63 matrix is

```text
H = s A A^T = s C^T C.
```

With a full no-drop QR factorization

```text
C P = Q R,
```

the same matrix is

```text
H = s P R^T R P^T.
```

No constraint is removed. A later solve could use two triangular solves in
the permuted coordinates without explicitly forming the Gram matrix. This
does not remove the intrinsic sensitivity of `H`; it only avoids introducing
the normal-equation matrix as the primary floating-point representation.

LAPACK uses QR/LQ for full-rank least squares, while its rank-revealing driver
uses QR with column pivoting plus a complete orthogonal factorization. The
orthogonal transformations preserve the 2-norm and do not amplify errors.

Sources:

- [LAPACK Users' Guide: linear least squares](https://www.netlib.org/lapack/lug/node27.html)
- [LAPACK Users' Guide: orthogonal factorizations](https://www.netlib.org/lapack/lug/node39.html)
- [LAPACK `DGELSY`](https://www.netlib.org/lapack/explore-html/dc/d8b/group__gelsy.html)

## Why ordinary QR is not assumed sufficient

The R63E witness has tangent norm squared only about `3.95e-30`. That places
its norm near `2e-15`, close enough to binary64 roundoff at unit scale that a
globally backward-stable factor can still fail to preserve this particular
direction. Global reconstruction error alone is therefore not a sufficient
gate.

The normal-equation relation squares condition number. LAPACK Working Note
149 explicitly motivates wider formation/solution of normal equations as one
way to recover accuracy, while direct QR is the ordinary alternative. For our
research this means that wider arithmetic remains a bounded comparison lane,
not an automatic runtime choice.

Source: [LAPACK Working Note 149, extended and mixed-precision BLAS](https://www.netlib.org/lapack/lawnspdf/lawn149.pdf).

Column-pivoted QR also needs care: norm downdates can mis-pivot and
underestimate numerical rank. R63G therefore does not ask a binary64 routine
to rediscover rank or choose a threshold. It reuses the already immutable
R63E permutation, retains all 102 columns, and runs fixed-order Householder QR
in both arithmetic lanes.

Sources:

- [LAPACK Working Note 176, instability in QR with column pivoting](https://www.netlib.org/lapack/lawnspdf/lawn176.pdf)
- [LAPACK Working Note 296, robust parallel QRCP](https://www.netlib.org/lapack/lawnspdf/lawn296.pdf)

## Frozen experiment

Replay the exact R63F subject and form `C=A^T` in scalar-major/row order. Use
the complete R63E normalized-pivot permutation as a fixed no-drop column
ordering. Do not perform a new rank decision.

Run the same unblocked Householder construction twice:

1. binary128 input and arithmetic as the bounded reference;
2. tangent coefficients rounded once to binary64, strict binary64 factor
   arithmetic, and binary128 audit of the stored binary64 result.

For each lane reconstruct the thin `Q`, retain the complete `R`, and audit:

- `Q^T Q=I`;
- `Q R=C P`;
- `R^T R=P^T(A A^T)P`;
- every one of the 102 diagonal factors is finite and nonzero;
- exact construction/work/root correspondence.

No diagonal magnitude is interpreted as an effective-rank threshold.

## Weak-direction signal gate

Reuse the immutable R63E coefficient vector `alpha`. Let

```text
x = P^T alpha
u = C alpha
u_factor = Q (R x).
```

The direct R63F `u` is the reference. Evaluate three isolated perturbations in
binary128 audit arithmetic:

1. coefficient storage only: rounded-binary64 `C64` times exact `alpha`;
2. vector storage only: exact `C` times rounded-binary64 `alpha64`;
3. stored binary64 factor: exact evaluation of `Q64(R64 P^T alpha)`.

For each lane compute the squared error against `u`. A lane preserves the
observed signal only when

```text
||u_lane-u||^2 < ||u||^2.
```

This strict separation uses no fitted decimal tolerance. A result outside the
open signal ball cannot certify even the sign/direction of the immutable weak
mode. Componentwise gamma bounds still audit each deterministic dot and
factor reconstruction; they are correspondence gates, not rank thresholds.

The binary128 factor must preserve the signal and close global orthogonality,
reconstruction and Gram identities before any binary64 result is interpreted.

## Alternatives retained, not executed

LSQR/LSMR use only forward and adjoint rectangular products and avoid
explicitly storing the normal equations. The project already has a controlled
LSMR implementation, so this is the scalable follow-up if the factor probe
closes. It is not executed in R63G because an actual right-hand side and stop
policy belong to the next frozen stage.

Sources:

- [Paige and Saunders, LSQR](https://stanford.edu/group/SOL/software/lsqr/lsqr-toms82a.pdf)
- [Fong and Saunders, LSMR](https://web.stanford.edu/group/SOL/software/lsmr/LSMR-SISC-2011.pdf)

Preconditioned Jacobi SVD can sometimes compute tiny singular values more
accurately and remains a diagnostic fallback. Its high-relative-accuracy
guarantees depend on matrix scaling structure that has not been established
for this clamp-cancellation operator, so it is not the first solver candidate.

Source: [LAPACK `DGEJSV`](https://www.netlib.org/lapack/explore-html/d8/d78/group__gejsv_gaca7ba7f1e8002c7a1d5bffa4ccbb541f.html).

## Routes

- binary128 factor/reference failure:
  `RECTANGULAR_BINARY128_REFERENCE_REJECTED`;
- coefficient rounding alone leaves the signal ball:
  `BINARY64_RECTANGULAR_STORAGE_FLOOR`;
- storage survives but the binary64 factor leaves the signal ball:
  `BINARY64_RECTANGULAR_FACTOR_FLOOR`;
- both binary64 storage and factor preserve the signal:
  `BINARY64_NO_DROP_FACTOR_CANDIDATE`;
- any parent/control/work/correspondence failure: apparatus rejection.

## Continuation and ceiling

A binary64 candidate permits a separately frozen RHS solve/certificate test;
a storage or factor floor permits a bounded mixed/wider-accumulation design
study. Neither route selects production arithmetic.

R63G performs no NNQP RHS, inverse, rank action, row drop, regularization,
center, replacement, state update, trajectory or timing. It cannot authorize
runtime, GPU or production work. R64 and R65 remain blocked.
