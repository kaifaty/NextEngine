# NSR3-B4E2D7R20R63I exported-factor RHS contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63I` |
| Architecture snapshot | R63H semantic `34452faf...56708`; wide-built factor survives one-time binary64 export |
| Engineering consumer | Select triangular consumption precision for the first no-drop rectangular NNQP solve |
| Claim class | One immutable RHS plus independently enclosed original-system error/sign certificate |
| Claim status target | wide rejection, export-wide rejection, binary64-consumption rejection, binary64 RHS candidate or exact apparatus boundary |
| Budget | One capture-only parent replay; exact R63H reconstruction; three fixed two-triangular solves; one R60 depth-4 reference and three residual/sign certificates; no refinement, following transition, trajectory or timing |

## Exact claim

R63H repeats at semantic `34452faf...56708` with wide factor root
`bf0a9e1b...779b14` and exported `R` root `a7a85789...3a4f85`. The immutable
R60 tuple repeats at exact matrix/inverse/RHS/solution roots.

The retained-wide, exported-factor/wide-consumption and exported-factor/
strict-binary64 lanes each solve the same scaled `R^T R` system once. Every
candidate is audited against the original captured Gram and RHS. The existing
contractive R60 left inverse converts the enclosed original residual into a
rigorous infinity error radius, and the resulting 102 component intervals are
compared with the exact R60 depth-4 sign vector.

## Exact negation

The first platform, parent, capture, R63H root, factor, scale, RHS, triangular
construction, finiteness, original residual, left-inverse contraction, error
radius, sign correspondence, work or lifecycle gate fails. Stored-system
residual alone cannot pass a lane.

## Frozen roots and tuple

Require:

```text
R63H semantic 34452faf4640780badf441f0f3d22e12d0e5797194e568cda22c1c0a79156708
matrix        aa401d0191ad53b7caa7837c827913fa711e1fdc433bc200c020719d297fb09d
inverse       467e815a4813e7774523699147db38fbcda06dea9b93bfb1696b410e0df0cfd7
rhs           64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1
solution      2c3a68b39bc82c4de5c09d9f4d4a091cbfd41024b0cdc3a8e509da7effcfe6e3
wide R        contained in factor bf0a9e1b9e2fd4525d267ec63d63640675479ab55c5b6cd3bb3b0c4ea4779b14
export R      a7a85789364f5af91aaf71e1531759713c95ec085e48b7ca614daa524c3a4f85
```

Require dimension 102, complete inherited permutation and exact projector
`eta/scale` roots. Bind scaled/permuted RHS separately per arithmetic lane.

## Fixed triangular solves

For `R^T y=c`, iterate rows `0..101`; for `R z=y`, iterate rows `101..0`.
Every inner reduction is increasing index order. Binary128 lanes use the
existing accumulator and binary128 division. The strict-binary64 lane uses
ordinary non-FMA binary64 multiply/add/subtract/divide under the frozen
compiler profile. Fail on every nonfinite or zero diagonal/intermediate.

Each lane executes exactly:

- one RHS scale and permutation;
- `5,151` multiply-subtract terms in the forward solve;
- `5,151` multiply-subtract terms in the backward solve;
- `204` divisions;
- one inverse permutation.

Bind intermediate `y`, permuted solution and original-order solution roots.
No solution is reused as another lane's input.

## R60 reference certificate

Recompute the exact R60 depth-4 practical certificate from the frozen tuple.
Require its existing root, `rho_bound<1`, `24/78/0` signs, error
`2.7127883223498731e-6` and minimum separation
`32.42340332375679`. Derive and bind the componentwise reference sign vector
from its final solution and error radius.

## Candidate residual/error certificate

For every lane:

1. Compute all 102 components of `r=b-H lambda` using `al_r20_r38_dot2` with
   the RHS as the leading term. Require exact/no-underflow and bind value/bound
   roots.
2. Compute all 102 components of `w=Xr` with Dot2. Propagate each input
   residual interval through `sum_j |X_ij| r_bound_j` using frozen upward
   operations.
3. Let `w_inf` be the outward maximum of `|w_i|+w_bound_i`. Let
   `denominator=down(1-rho_bound)>0` and
   `error=up(w_inf/denominator)`.
4. Require every `lambda_i +/- error` to exclude zero and match the component
   sign from the R60 depth-4 certificate.

Publish residual infinity bound, backward-error observation, `w_inf`, error
radius, positive/negative/unresolved counts, minimum separation and all roots.
Only the rigorous interval/sign gate selects a route.

## Controls

1. Literal `H=[2]`, `X=[1/2]`, `b=[2]`, `R=[sqrt(2/s)]` closes all three
   lanes and yields one certified positive sign.
2. Perturb the strict-binary64 literal solution across zero while retained-wide
   remains separated; classifier selects binary64-consumption rejection.
3. A noncontractive `X=[0]` rejects the error certificate.
4. Forward/back orientation and a nonidentity permutation reconstruct a
   literal known solution.
5. Zero diagonal and nonfinite intermediate fail closed.
6. Mutating one RHS, factor, intermediate, residual or sign bit changes its
   root.
7. Shrinking an outward residual/error interval rejects containment.
8. Classifier precedence covers all five routes.
9. R63B--R63H byte regressions pass. New inverse/refinement/iteration, rank
   action, row drop, regularization, replacement, state, following transition
   and timing counts are zero.

## Resolution precedence

1. First apparatus/control/parent/certificate failure.
2. `WIDE_RHS_SIGN_CERTIFICATE_REJECTED` if retained-wide has any unresolved or
   mismatched sign.
3. `EXPORTED_FACTOR_WIDE_CONSUMPTION_REJECTED` if retained-wide passes and
   export/wide does not.
4. `EXPORTED_FACTOR_BINARY64_CONSUMPTION_REJECTED` if both wide-consumption
   lanes pass and strict binary64 does not.
5. `EXPORTED_FACTOR_BINARY64_RHS_CANDIDATE` if all three pass.

## Does not count

Replacing the parent principal solution; executing a following NNQP decision;
iterative refinement, inverse construction, LSQR/LSMR/SVD; effective-rank or
row policy; regularization; default runtime-wide arithmetic; center mutation,
trajectory, timing, runtime/GPU or production inference.

## Stop and reconsider

- Binary64 candidate: freeze a single private replacement/decision parity
  transaction against the unchanged parent, still without public state.
- Binary64 consumption rejected: retain exported factor storage and research
  wider/compensated triangular consumption only.
- Export-wide rejected: retain wide factor through consumption or research
  factor export with error-carrying representation.
- Retained-wide rejected: research one original-residual refinement from the
  stored-operator solution; do not weaken signs or change rows.
- R64 and R65 remain blocked for every route.
