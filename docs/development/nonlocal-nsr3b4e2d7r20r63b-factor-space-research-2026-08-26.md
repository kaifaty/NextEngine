# NSR3-B4E2D7R20R63B factor-space research

Status: `COMPLETE / BINARY64_FACTOR_LEFT_NONCONTRACTIVE`.

## Question

Does binary64 fail already on one triangular factor of the captured principal
matrix, or only after the Gram/normal-equation composition squares the
conditioning?

The principal system is built as a symmetrized Gram form and solved by
Cholesky. Its exact matrix condition is extreme, while a factor should carry
roughly the unsquared condition. The existing failed inverse audit already owns
the exact represented lower factor, so this can be tested without changing the
solver or factorization.

## Primary-source basis

LAPACK Working Note 149 states the relevant numerical distinction directly:
normal equations can effectively square a least-squares condition number and
lose twice as much accuracy; QR avoids that composition, while higher internal
precision is another possible remedy.

Source: [Design, Implementation and Testing of Extended and Mixed Precision BLAS, LAPACK Working Note 149](https://www.netlib.org/lapack/lawnspdf/lawn149.pdf).

Rank-revealing QR is a recognized way to solve rank-deficient least-squares and
subset-selection problems, while pivoted Cholesky provides a rank diagnostic
for positive-semidefinite matrices. Neither source authorizes an arbitrary row
drop or threshold in our engine; those would need separate model semantics.

Sources:

- [Chan and Hansen, Some Applications of the Rank Revealing QR Factorization](https://epubs.siam.org/doi/abs/10.1137/0913043)
- [LAPACK Working Note 161, complete-pivoted Cholesky](https://netlib.org/lapack/lawnspdf/lawn161.pdf)

## Selected discriminator

Capture the already constructed lower factor `L` for the exact R60/R63A
system. Construct a represented binary128 triangular inverse `Z` by forward
substitution only, then independently certify both `I-LZ` and `I-ZL` with exact
dyadics. Project `L,Z` to binary64 and apply the already validated R63
`Dot2Err` machinery to both defects.

```text
A ~= L L^T                full normal/Gram space
Z ~= L^-1                 one factor space

if L64/Z64 contract:
  full inverse projection failed because factor composition squared the barrier
else:
  even one factor is beyond this binary64 representation
```

This is a representation discriminator, not a replacement solve. It also
computes a directed lower bound for `cond_inf(A)*u64` from R63A's exact matrix,
inverse and left defect. The bound establishes severity but does not prove that
every possible scaling or algorithm must fail.

## Branch after the result

- factor-space contraction: research a QR/triangular active-set formulation
  over the underlying projector rows, with canonical rank handling and full
  KKT/trajectory correspondence;
- factor-space noncontraction: do not build an ordinary binary64 triangular
  solver; compare rank-revealing reduction and a rare software extended-
  precision path;
- factor reconstruction/capture failure: stop at apparatus; make no rank claim.

The observed result occupies a narrower branch than the original binary
classification: `I-L64*Z64` contracts by a wide margin, but `I-Z64*L64` does
not. The frozen two-sided candidate is therefore rejected. Before selecting
rank removal or extended precision, research whether the successful one-sided
defect is sufficient for a fail-closed a-posteriori triangular solution bound;
that is a new claim and may not retroactively change R63B.

## Ceiling

R63B applies no triangular solution to the NNQP right-hand side, chooses no
rank threshold, removes no row, changes no objective and authorizes no center,
trajectory, runtime/GPU, timing or production path.
