# NSR3-B4E2D7R20R63G rectangular-factor precision contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63G` |
| Architecture snapshot | R63F semantic `048601d3...e9acf`; direct tangent rank 102 under both primes; Gram numerical rank 101 |
| Engineering consumer | Select binary64 or bounded wider arithmetic for the first no-drop rectangular RHS experiment |
| Claim class | Fixed-order orthogonal-factor correspondence plus immutable weak-direction signal separation |
| Claim status target | binary64 storage floor, factor floor, no-drop factor candidate, reference rejection or exact apparatus boundary |
| Budget | One capture-only parent replay; one `315 x 102` transpose; two complete fixed-order Householder factors; inherited permutation/witness only; no RHS, rank threshold, row action, center, trajectory or timing |

## Exact claim

The R63F parent, direct tangent matrix and inherited witness repeat exactly.
For `C=A^T` and the complete frozen R63E permutation `P`, a binary128
Householder reference and a strict binary64 Householder candidate both retain
all 102 columns. Their stored factors are independently audited in binary128.

The binary128 lane closes thin-orthogonality, reconstruction, Gram and weak-
direction identities. Binary64 coefficient storage and factorization are then
classified by whether their error for the immutable direct witness lies
strictly inside its nonzero signal norm.

## Exact negation

The first platform, parent, capture, tangent, permutation, witness, control,
factor construction, finiteness, nonzero-diagonal, orthogonality,
reconstruction, Gram, weak-direction, root, work or lifecycle gate fails.
Small nonzero diagonals cannot be relabelled rank deficiency or removed.

## Frozen subject and orientation

- Reuse the R63F `102 x 315` tangent values and source row order.
- Materialize `C=A^T` as 315 rows by 102 columns.
- Reuse all 102 entries of the R63E binary128 normalized-pivot permutation.
  Require a true permutation of `0..101` and bind its root.
- Form `C P` once per arithmetic lane. No lane selects or changes a pivot.
- Retain every column; `rank_actions=0` and `row_drops=0` are hard gates.

## Householder construction

- Unblocked left-looking Householder QR, columns `0..101`, deterministic
  increasing row/column reductions, no FMA contraction and no fast math.
- Binary128 lane reads exact tangent coefficients. Binary64 lane rounds every
  coefficient once with `static_cast<double>` before any factor work.
- Use the stable signed-norm reflector construction; require finite norm,
  reflector, beta and update values at every step.
- Materialize thin `Q` by applying the stored reflectors to the first 102
  identity columns in reverse factor order. Retain complete upper-triangular
  `R` including all 102 finite, strictly nonzero diagonal entries.
- Bind input, permutation, reflector, `Q`, `R`, diagonal and work roots.

## Independent factor audits

All audit accumulation is binary128 and uses independently materialized
original inputs.

1. `Q^T Q-I`: 10,404 entries and `315` products per entry.
2. `Q R-C P`: 32,130 entries and at most `102` products per entry.
3. `R^T R-P^T G_T P`: 10,404 entries and at most `102` products per entry.
4. Require every binary128 reference residual inside its frozen
   dimension-derived gamma bound. Binary64 global residuals are published and
   must be finite; they do not by themselves decide weak-signal survival.

The exact work ledger records reflector norms, reflector updates, Q updates,
orthogonality products, reconstruction products and Gram products separately.

## Weak-direction lanes

Reuse the R63E `alpha` and R63F direct tangent image `u` without
renormalization. Require their roots and `||u||^2` to repeat.

Compute in binary128 audit arithmetic:

- `u_c64=C64*alpha` from once-rounded binary64 coefficients;
- `u_a64=C*alpha64` from once-rounded binary64 coefficients of `alpha`;
- `u_qr128=Q128*(R128*(P^T alpha))`;
- `u_qr64=Q64*(R64*(P^T alpha))` from stored binary64 factors.

For each vector publish value root, error root, squared norm, squared error and
componentwise maximum residual. Binary128 factor survival requires
`||u_qr128-u||^2 < ||u||^2`. Define:

```text
storage_survives = ||u_c64-u||^2 < ||u||^2
factor_survives  = ||u_qr64-u||^2 < ||u||^2
```

The rounded-alpha lane is report-only attribution. No decimal relative-error
threshold, effective rank or singular-value cutoff exists.

## Controls

1. A literal tall dyadic full-rank matrix closes binary64/binary128 QR,
   orthogonality, reconstruction and Gram identities.
2. A literal full-rank near-dependent dyadic matrix retains its nonzero last
   direction; an exact duplicate-column mutation is rejected by the nonzero-
   diagonal gate.
3. A fixed nonidentity complete permutation reconstructs the original Gram.
4. A literal weak nonzero vector passes the strict open-ball signal gate;
   its zero and outside-ball mutations do not.
5. Mutating one input, permutation, reflector or factor scalar changes the
   corresponding root.
6. Deliberately shrink one reference correspondence bound; the exact value
   escapes it.
7. R63B--R63F byte regressions pass. RHS, LSQR/LSMR/SVD, rank action, row
   drop, regularization, inverse, center, replacement, state and timing counts
   are zero.

## Resolution precedence

1. First apparatus/control/reference/correspondence failure.
2. `RECTANGULAR_BINARY128_REFERENCE_REJECTED` if the complete binary128 factor
   or its weak-direction signal gate fails.
3. `BINARY64_RECTANGULAR_STORAGE_FLOOR` if binary128 closes and
   `storage_survives=false`.
4. `BINARY64_RECTANGULAR_FACTOR_FLOOR` if storage survives but
   `factor_survives=false`.
5. `BINARY64_NO_DROP_FACTOR_CANDIDATE` if both survive.

## Does not count

Global uniqueness or production stability; a numerical rank decision;
dropping/replacing a constraint; a QRCP pivot search; SVD/LSQR/LSMR; an NNQP
RHS or triangular solve; regularization; default runtime binary128; center,
trajectory, timing, runtime/GPU or production inference.

## Stop and reconsider

- Binary64 candidate: freeze one RHS solve plus independent original-system
  residual/sign certificate; do not integrate it directly.
- Storage floor: research representation or mixed/wider operator arithmetic
  before factor policy.
- Factor floor: preserve binary64 storage and compare bounded wider factor or
  matrix-free reorthogonalized Golub--Kahan arithmetic.
- Reference rejection: repair the binary128 factor/correspondence apparatus or
  add a Jacobi-SVD oracle; do not weaken the signal gate.
- R64 and R65 remain blocked for every route.
