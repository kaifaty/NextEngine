# NSR3-B4E2D7R20R63R common-operator inverse contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63R` |
| Architecture snapshot | R63Q semantic `48ec75fe...e345`; tangent Gram selected by full exact operator norms |
| Engineering consumer | Supply a common-operator inverse verifier for a later structured RHS certificate |
| Claim class | Exact two-sided inverse-defect contraction over rational `H*` |
| Claim status target | apparatus/construction boundary, left/right contraction boundary or common-inverse candidate |
| Budget | One original binary128 tangent QR; 102 canonical two-triangular solves; two exact 102-cubed defect products; no physical RHS or PCG |

## Exact parent

Require:

```text
R63Q semantic       48ec75fe0a173dabbfae012372dd43c39d8d8c34ed5f2dddc39fea39ebd0e345
R63Q audit          b24da3fed653f7583b2b49a47a698c3421aa09ba68ae2342f60a1d061c7eae09
R63Q matrix         1e935dbe8a29210c3601038812c7b6b18bcb73b7d82c3f64bbfaf628c982998a
R63Q oracle         221b475869ab34ee919d1d3add4aa993ab361e2dd914be308722f03726c41266
R63G original QR    fde9aef5ee0233c920ab76c855c612cc722422c67f44d18598211ae901082ec4
R63G permutation    335235cbaf9a93c805a2bfdbd17c593eb3ae44c9a20ea8a752782fa10f15826f
```

Reconstruct exact R63Q and its strict tangent infinity/Frobenius wins. Build
the original, not binary64-stored, transposed/permuted `315 x 102` tangent
factor and require the frozen R63G root.

## Inverse construction

For canonical column `j=0..101`, solve with the same fixed permutation and
original binary128 `R`:

```text
R^T w = (1/sigma) P^T e_j
R u   = w
Z[:,j] = P u.
```

This is the existing `al_r20_r63i_q_solve` with
`inverse_scale=1/sigma=1+eta`. Require per column:

- 5,151 forward terms;
- 5,151 backward terms;
- 204 divisions;
- finite values and exact solve root.

Assemble exactly 10,404 values without symmetrization. Bind column roots,
matrix root, maximum exact asymmetry and row infinity norm of `Z`.

## Exact two-sided defects

Convert every `Z` value once to exact dyadic form. For each ordered `(i,j)`:

```text
left_num_ij  = (i==j ? d : 0) - sum_k Z_ik H_num_kj
right_num_ij = (i==j ? d : 0) - sum_k H_num_ik Z_kj.
```

Require exactly `2 * 102^3 = 2,122,416` exact products, 20,808 residual
entries and 20,808 absolute row-sum terms. Compute exact maximum entry and
infinity norm numerators for each side.

Candidate requires strict exact comparisons:

```text
left_inf_num  < d
right_inf_num < d.
```

Publish `||Z||inf` and the exact left inverse-amplification expression
`||Z||inf/(1-rho_left)` as a rational diagnostic; do not apply it to an RHS.

## Controls

1. A small exact diagonal SPD operator and its triangular factor yield zero
   two-sided defects.
2. A nonsymmetric approximate inverse distinguishes left and right products.
3. A perturbed diagonal inverse makes the left defect noncontractive.
4. A construction with left contraction and right noncontraction exercises
   classifier order.
5. Transpose reversal, column permutation and raw-inverse asymmetry are
   observable.
6. Mutation of a solve, inverse entry, defect or norm changes its root.
7. Classifier precedence covers every route.
8. R63B--R63Q byte regressions pass.

## Resolution precedence

1. `COMMON_OPERATOR_INVERSE_APPARATUS_REJECTED`.
2. `COMMON_OPERATOR_INVERSE_CONSTRUCTION_REJECTED`.
3. `COMMON_OPERATOR_LEFT_NONCONTRACTIVE`.
4. `COMMON_OPERATOR_RIGHT_NONCONTRACTIVE`.
5. `COMMON_OPERATOR_TWO_SIDED_CANDIDATE`.

## Does not count

Using dense `X`; binary64/exported factor substitution; inverse
symmetrization; iterative refinement; an analytic QR error estimate instead
of exact defects; relative tolerance; immutable RHS/signs/PCG; sparse/timing/
state/runtime/GPU/production work.

## Stop and reconsider

- Two-sided candidate: freeze R63S around the immutable RHS and direct tangent
  PCG with exact `H*` residual/sign certification using `Z` only as verifier.
- Left failure: research wider QR/triangular arithmetic or exact-rational
  factor verification before any RHS.
- Right-only failure: test explicit symmetric `R^-1 R^-T` formation under a
  new contract; do not silently symmetrize this result.
- Construction/apparatus failure: locate the first root/count/orientation
  failure and preserve R63Q.
- R64 and R65 remain blocked for every route.

