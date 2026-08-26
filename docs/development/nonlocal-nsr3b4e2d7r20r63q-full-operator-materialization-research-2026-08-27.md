# NSR3-B4E2D7R20R63Q full-operator materialization research

Status: `FROZEN / IMPLEMENTATION_NEXT`.

## Question

R63P refutes finite-projector non-idempotence as the cause of the R63O inverse
split and shows that tangent Gram is dramatically closer to the common exact
operator on one inherited weak witness. Is that advantage a full-operator
property, or an isolated cancellation result?

## Common exact operator

For frozen source rows `a_i`, free projector vector `y`, stored denominator
`d`, stored scale `sigma` and exact dyadic products, define

```text
c_ij = sum_free a_i,l a_j,l
g_i  = sum_free a_i,l y_l

H*_ij = sigma (c_ij d - g_i g_j) / d.
```

All entries share the positive exact dyadic denominator `d`. R63Q therefore
stores exact dyadic numerators and never rounds the division while making a
semantic decision.

Compare two immutable finite materializations:

```text
H_dense,ij = captured binary128 matrix entry
K_tangent,ij = sigma sum_l T_i,l T_j,l
```

The second quadratic form is evaluated as an exact dyadic sum over the stored
binary128 tangent coefficients; it is not the rounded R63N product center.

## Full-matrix discriminator

For every ordered entry, construct exact common-denominator error numerators:

```text
eH_ij = abs(H_dense,ij d - H*_num,ij)
eK_ij = abs(K_tangent,ij d - H*_num,ij).
```

Derive without floating comparison:

- maximum entry error;
- infinity norm as the maximum exact row sum;
- squared Frobenius error;
- dense/tangent/equal per-entry winner counts;
- exact common-operator infinity norm;
- the full ideal rank-one `P_d^2-P_d` infinity norm;
- inherited R63P weak-form identities and errors.

Exploit symmetry only as a work reduction: calculate 5,253 upper-triangle
source and tangent dots, mirror them, then audit all 10,404 ordered entries and
exact symmetry roots.

The tangent representation advances only if both global comparisons are
strict:

```text
||K_tangent-H*||_inf < ||H_dense-H*||_inf
||K_tangent-H*||_F^2 < ||H_dense-H*||_F^2.
```

No relative tolerance or required winner percentage is fitted. Maximum error
and per-entry counts are diagnostic and root-bound.

## Interpretation

A pass selects tangent Gram as the better finite *operator representation* for
the complete captured matrix, and authorizes a later structured-RHS
certificate against `H*`. It does not prove that the existing direct PCG
solution is correct, does not erase R63O's dense-system result, and does not
grant runtime or production authority.

A failure confines R63P to its weak witness and requires source/JVP/source or a
higher-precision dense construction before any new solve.

## Scope ceiling

No RHS, inverse, factor, triangular solve, PCG, rank change, sparse
realization, timing, state update, runtime/GPU or production work is admitted.
R64 and R65 remain blocked.

