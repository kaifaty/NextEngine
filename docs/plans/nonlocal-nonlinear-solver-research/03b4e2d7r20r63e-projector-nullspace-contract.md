# NSR3-B4E2D7R20R63E projector nullspace contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63E` |
| Architecture snapshot | SPEC-38 Proposed; ADR-076 Proposed; ADR-081 Accepted; R63D semantic `f9a5f0d8...45b8d`; exact source/projected rank 102; projected numerical rank 101 |
| Engineering consumer | Select clamp-representative versus ball-tangent range-space research |
| Claim class | Profile-bound projector range/nullspace correspondence and witness decomposition |
| Claim status target | First projector component creating the inherited numerical rank loss, or exact apparatus boundary |
| Budget | One capture-only parent replay; three reconstructed projected Gram blocks; inherited rank profiles; one pivot witness; no RHS, rank action, row drop, center, trajectory or timing |

## Exact claim

The unique R63 matrix-selected inverse audit captures both its ordered 102
source row IDs and the exact `ALR20ProjectorState` used to form that NNQP
principal block. Reconstructing the full derivative Gram from those objects
matches the captured matrix root and every binary128 entry exactly.

Clamp-only, clamp-plus-ball-tangent and full-scaled projected Gram blocks are
formed in fixed binary128 order. Exact modular ranks and normalized
binary64/binary128 pivot profiles reuse the frozen R63D definitions. The first
binary128 profile whose rank falls below 102 classifies the responsible
projector component.

The inherited normalized full-block pivot witness is converted to the
unscaled row combination and decomposed into clamped, free-radial and retained
tangent components. Fixed bounds validate vector reconstruction, JVP,
Gram-image and energy identities.

## Exact negation

The first platform, parent, hook lifecycle, selected-call, projector-state,
source-row, matrix reconstruction, modular, normalization, inherited-rank,
witness, identity, work, root or control gate fails. An unmatched projector
state, absent witness or rank-deficient modular image is not evidence for a
clamp/ball mechanism.

## Frozen capture and dimensions

- Case and matrix are the immutable R63D oblique-jet/twist block.
- Require `105` particles, `315` scalars and `102` selected source rows.
- The verified-inverse capture gains a private projector field. A thread-local
  context points to `current.projector` only around the existing NNQP call and
  is copied only when the pre-frozen matrix root selects the audit.
- Require one solver execution, 125 observed inverse audits, one matrix-
  selected audit and 102 selected identity columns.
- Require projector `exact`, mask/value sizes `315/105`, component counts that
  sum to 315, and a root that is bound before any decomposition.
- The context slot must be clear before and after replay. No public solver root
  or report may include the private context.

## Projector blocks

For every selected source row vector `b_j`, form in stable scalar order:

1. `q_j=Q b_j`: copy only mask-zero components;
2. if the ball is active, `t_j=q_j-y_f*(y_f^T q_j)/free_norm_squared`, else
   `t_j=q_j`;
3. `j_j=t_j/(1+eta)`.

Form symmetric Grams `G_Q[i,j]=b_i^T q_j`,
`G_T[i,j]=b_i^T t_j`, and `G_J[i,j]=b_i^T j_j`, using both directed dots and
the same half-symmetrization as the parent. Require finite values, exact work
counts and symmetry roots.

`G_J` must equal the captured 102x102 matrix entry-for-entry and by root.
Additionally, direct `al_r20_projector_jvp(projector,b_j)` must equal every
constructed `j_j` entry-for-entry. This is the primary state/correspondence
gate.

## Rank profiles and route

- Run R63D modular rank for both fixed primes on `G_Q`, `G_T`, and `G_J`.
  Require exact-full rank 102 under at least one prime for every represented
  block. Otherwise route to the first named modular apparatus boundary.
- Diagonally normalize each block and run the frozen binary64 and existing
  binary128 pivot diagnostics without changing either threshold.
- Require the reconstructed `G_J` profiles to repeat R63D: binary64 rank 101,
  binary128 rank 101.
- Route precedence after all apparatus gates:
  1. `CLAMP_METRIC_NUMERICAL_RANK_LOSS` if binary128 `G_Q` rank is below 102;
  2. `BALL_TANGENT_NUMERICAL_RANK_LOSS` if `G_Q` rank is 102, the ball is
     active and `G_T` rank is below 102;
  3. `PROJECTOR_SCALE_ARITHMETIC_BOUNDARY` if `G_Q/G_T` rank 102 and `G_J`
     rank 101;
  4. `PROJECTOR_COMPONENT_RANK_UNRESOLVED` otherwise.
- Binary64 ranks and witness energy dominance are report-only corroboration and
  cannot override this order.

## Near-null witness

- Extend the existing `ALR20PivotedRank` diagnostic to retain the same witness
  vector already hashed by `dependency_root`; do not change its calculation or
  root material.
- Require the full normalized binary128 rank to be 101 and witness size 102.
- With `D_i=sqrt(G_J[i,i])`, set `alpha_i=w_i/D_i`. Preserve the deterministic
  pivot sign/scale; do not renormalize or rotate the witness.
- Form `v=B^T alpha` with the existing binary128 accumulator, then `c`, `q`,
  `r`, `t`, and `j` exactly as defined in the research note.
- Publish squared norms, radial coefficient, `eta`, component counts, maximum
  identity residuals, bounds and roots.
- Require:
  - `v=c+r+t` componentwise within a gamma bound;
  - constructed `j` agrees with `al_r20_projector_jvp(state,v)` within a gamma
    bound and by finite work;
  - captured `G_J alpha` agrees with every `b_i^T j` within a gamma bound;
  - `alpha^T G_J alpha` agrees with `||t||^2/(1+eta)` within a gamma bound;
  - clamped, radial and tangent energies close `||v||^2` within a gamma bound.
- No witness scalar or ratio may choose a rank threshold or modify a row.

## Controls

1. A literal finite projector with one clamped component and an active ball
   closes the clamp/radial/tangent decomposition and production-form JVP.
2. Literal two-row operators distinguish clamp-created and ball-tangent-created
   rank-one Gram blocks under the same classifier.
3. A literal normalized rank-one matrix returns the already hashed pivot
   witness; converting through a nonuniform diagonal reproduces its original
   Gram image.
4. Mutating one projector mask item, one free value or one source row after
   root binding is rejected before classification.
5. Hook/context lifecycle, selected-call uniqueness and all vector/Gram work
   counts close. R63B, R63C and R63D public stdout remain byte-exact in separate
   regression invocations.

## Resolution firewall

- Positive scientific routes are exactly the four names in the rank-profile
  precedence above.
- First apparatus failure takes precedence and yields no projector attribution.
- `rank 101` remains a numerical diagnostic for this state, not exact or
  physical rank.
- The 2026 polyhedral extreme-representative method is related research only;
  R63E cannot claim it applies to the active Euclidean ball.

## Does not count

QR/SVD/LSQR implementation; selecting or dropping a row; fitting a tolerance;
regularizing the Gram; changing masks, ball radius or `eta`; solving an NNQP
RHS; constructing another inverse; center/refinement/trajectory work; timing,
runtime/GPU or production inference.

## Stop and reconsider

- Clamp route: freeze projection-equivalent representative semantics before a
  rank-revealing factorization.
- Ball route: freeze a tangent/range-space formulation and its equivalence
  proof before solving any RHS.
- Scale route: isolate normalized tangent/full rounding before wider precision.
- Unresolved or apparatus route: repair only capture/correspondence; do not
  choose a solver response.
- R64 and R65 remain blocked for every R63E result.
