# NSR3-B4E2D7R20R63P finite-projector idempotence contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63P` |
| Architecture snapshot | R63O semantic `4b2434fa...2f270`; bounded direct-product correspondence and inverse-semantic rejection |
| Engineering consumer | Select the mathematical matrix-free operator for a later PCG replay |
| Claim class | Exact dyadic/rational operator-semantic arbitration on the inherited weak witness |
| Claim status target | apparatus boundary, idempotent boundary, structural rejection or source/JVP/source candidate |
| Budget | One capture-only replay; one exact weak-witness reconstruction; no RHS solve, factor solve, PCG, timing or state update |

## Exact parent

Require the immutable R63O capture, R63E full matrix and witness, R63F physical
tangent rows, and R63O defect:

```text
R63O semantic        4b2434fa485f6d3f5362169587da44ffc744d0b448a9384ebe769f3f3dc2f270
R63E full root       aa401d0191ad53b7caa7837c827913fa711e1fdc433bc200c020719d297fb09d
R63E witness root    507258b6c55abcb9261c7c7e45f0834ad2189cab139feac941d290d994dce864
R63F tangent values  114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1
R63O defect root     e531f06788e134c1fa65e39dd8e4f0be8e6350b23b781c990d6e7d72ac666f94
```

The source rows, projector value/mask, stored `d`, stored `eta`, dense `H`,
tangent coefficients and witness `alpha` are immutable.

## Exact arithmetic model

Convert every consumed binary128 value to its exact dyadic integer/exponent
representation. Reject any failed conversion. Define, in increasing frozen
index order:

```text
s      = sum_free y_l^2
z_l    = sum_i alpha_i A_il
z2     = sum_free z_l^2
g      = sum_free y_l z_l
d      = exact dyadic image of stored free_norm_squared
sigma  = exact dyadic image of stored 1/(1+eta)
```

Construct exact rational models by numerator/positive-denominator pairs:

```text
h*_num = sigma (z2 d - g^2),             h*_den = d
delta_num = sigma (s-d) g^2,             delta_den = d^2
k*_num = h*_num d + delta_num,           k*_den = d^2.
```

Require exact identity

```text
k* - h* = delta
```

by integer cross-products. Do not evaluate this identity through rounded
division.

## Stored-operator observations

Evaluate exactly as dyadics:

```text
h_obs = alpha^T H alpha
u_l   = sum_i alpha_i T_il
k_obs = sigma sum_l u_l^2.
```

Compare absolute rational errors only by multiplying positive denominators.
Require all four strict semantic relations:

```text
|h_obs-h*| < |h_obs-k*|
|k_obs-k*| < |k_obs-h*|
sign(k_obs-h_obs) = sign(delta) != 0
|k_obs-h_obs-delta| < min(|k_obs-h_obs|, |delta|).
```

The last relation is a scale-free dominance check: subtracting the derived
term must reduce the exact observed gap, and the residual must be smaller than
both quantities. No fitted ratio is permitted.

Publish `s`, stored `d`, `s-d`, numerical projections of `h*`, `k*`, `delta`,
both observations and all exact comparison/root identities. Numerical text is
diagnostic; only exact integer relations gate the route.

## Fixed work

- 174 free-vector square terms for `s`;
- 32,130 exact source-combination products for `z`;
- 174 terms each for `z2` and `g`;
- 10,404 exact dense quadratic-form products;
- 32,130 exact tangent-combination products and 315 norm terms;
- zero RHS, inverse, factor, triangular, PCG, sparse, timing or state work.

The implementation must publish and gate every count.

## Controls

1. An exactly normalized dyadic projector has `s=d` and selects the idempotent
   boundary.
2. A deliberately non-normalized dyadic projector satisfies the rank-one
   identity exactly for both signs of `s-d`.
3. Replacing `d` by `s` makes the derived defect exactly zero.
4. Swapping the source/JVP/source and tangent-Gram models is detected by the
   strict closeness relations.
5. Mutation of `s`, `d`, `g`, either observation or a comparison changes its
   bound root.
6. Classifier precedence covers every route.
7. R63B--R63O byte regressions pass.

## Resolution precedence

1. `PROJECTOR_IDEMPOTENCE_APPARATUS_REJECTED`.
2. `STORED_PROJECTOR_IDEMPOTENT_BOUNDARY`.
3. `PROJECTOR_RANK_ONE_EXPLANATION_REJECTED`.
4. `SOURCE_JVP_SOURCE_OPERATOR_CANDIDATE`.

## Does not count

Rounded high-precision-only evidence; a decimal tolerance; a fitted dominance
ratio; changing stored `d`; redefining the projection; applying a rank-one
repair; solving the immutable RHS; more PCG; sparse/precision work; timing,
state replacement, runtime/GPU or production inference.

## Stop and reconsider

- Candidate: freeze R63Q around direct `A DPi(A^T p)` canonical
  correspondence, then replay the unchanged two PCG lanes and original dense
  certificates.
- Idempotent boundary: the hypothesis is false for the frozen data; build a
  broader exact operator oracle before another solve.
- Structural rejection: locate the first failed exact relation and audit model
  orientation/finite construction; do not patch the weak direction.
- R64 and R65 remain blocked for every route.

