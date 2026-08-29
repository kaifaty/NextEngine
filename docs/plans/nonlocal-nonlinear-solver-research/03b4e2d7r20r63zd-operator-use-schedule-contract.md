# NSR3-B4E2D7R20R63ZD operator-use schedule contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZD` |
| Parent | reviewed R63ZC result `cdf13230...c9b4` |
| Engineering consumer | selection of one later operator-repair discriminator |
| Claim class | exhaustive finite three-product schedule |
| Arithmetic | frozen Linux x86-64 strict binary128 profile |
| Budget | eight lanes, two updates, states `0..2`, no timing |

## Exact claim and negation

Let `P0=Hx0`, `P1=Hp0` and `P2=Hp1` be the three operator applications in the
frozen original-input recurrence. For each subset `S` of `{P0,P1,P2}`, use the
common-block product exactly at slots in `S` and the tangent product elsewhere.

The positive result is a complete, identity- and work-closed execution of all
eight subsets, including the required passing empty-set endpoint and rejecting
full-set endpoint, followed by the exact eight-bit pass mask and every
inclusion-minimal rejecting subset. The negation is the first apparatus,
identity, work, arithmetic, endpoint or observability failure that prevents
that finite classification.

No monotonicity over subsets is assumed.

## Frozen model

- Parent fixture schema/root:
  `nextengine.nonlocal.formula_probe_parent_fixture.r63zc.v1`,
  `7780543a...4553` after the reviewed sealing repair.
- Dimension 102; tangent columns 315; original frozen tangent and `sigma`.
- Original binary128 RHS root `64be4951...92b1` and original inverse scale.
- Exported factor root `a7a85789...4f85`, permutation root
  `335235cb...26f`.
- Common twofold component root `9ecbea11...c20e` and frozen common artifact
  root `38cd1871...de1`.
- R63Y twofold verifier profile `b1c43044...a291`.
- Same PCG signs/order as R63ZC; no early certificate stop.

Bit order is `bit0=P0`, `bit1=P1`, `bit2=P2`; zero means tangent and one means
common. Lane order is integer mask `0..7`, so endpoints are `TTT=0` and
`CCC=7`.

## Recurrence

Each lane executes exactly:

1. factor solve `x0=M^-1 b`;
2. selected `P0`, `r0=b-P0`, factor solve `z0`, positive `rho0=r0^Tz0`;
3. selected `P1`, positive denominator, update `x1/r1`, factor solve `z1`,
   positive `rho1`, update `p1`;
4. selected `P2`, positive denominator, update `x2/r2`;
5. seal all states and transaction roots;
6. execute immutable R63Y certificates for `x0,x1,x2`.

## Endpoint correspondence

- `TTT` must reproduce baseline raw solution-set root
  `d0b42562f5ed45efdf2d454f1c1f6ec93f587fe8045943517f9af076d279c98e`
  and reject/reject/pass with state-2 `24+/78-/0?`.
- `CCC` raw solution roots must be exactly:
  `11170afba73f379a19c02f4a9a1514944cc8f9f18bf7c79872d77abc3f478292`,
  `7d5ee1b8c6c9f2a9496e52ddaa5d08dcc2ccf7f419ec8d30c690ee1ccf103bb5`,
  `1341b62cb0d8ab3e561682b1560e951d3e7c4d5faa4684ffd7c48db3838cdb12`,
  and all three certificates must reject at `12+/24-/66?`.

Failure of either endpoint rejects before schedule interpretation.

## Classification

Publish the pass bit for all eight masks and the set of rejecting masks that
have no proper rejecting subset. Classification route is:

1. `OPERATOR_USE_SCHEDULE_APPARATUS_REJECTED`;
2. `OPERATOR_USE_SCHEDULE_IDENTITY_REJECTED`;
3. `OPERATOR_USE_SCHEDULE_WORK_REJECTED`;
4. `TANGENT_ENDPOINT_REJECTED`;
5. `COMMON_ENDPOINT_REJECTED`;
6. `INITIAL_PRODUCT_MINIMAL_SUFFICIENT` if mask `1` is minimal rejecting;
7. `FIRST_KRYLOV_PRODUCT_MINIMAL_SUFFICIENT` if mask `2` is minimal rejecting;
8. `SECOND_KRYLOV_PRODUCT_MINIMAL_SUFFICIENT` if mask `4` is minimal rejecting;
9. `MULTIPLE_MINIMAL_SCHEDULES` if more than one minimal rejecting schedule
   remains after the ordered singleton checks;
10. `PAIRWISE_OPERATOR_INTERACTION_REQUIRED` if a size-two mask is the first
    minimal reject;
11. `THREE_PRODUCT_INTERACTION_REQUIRED` when only mask `7` rejects.

The complete minimal-set list is authoritative; the route is a compact first
summary and cannot hide additional minimal sets or non-monotonicity.

## Fixed work

Across all eight lanes:

```text
certificates / products / solves       24 / 24 / 24
factor terms / divisions               247248 / 4896
scalar dots / divisions                32 / 24
solution / residual / direction updates 1632 / 1632 / 816
tangent inner / outer dots             3780 / 1224
tangent inner / outer terms            385560 / 385560
tangent scale products                 1224
common dots / terms                    1224 / 124848
adaptive stops                         0
```

## Controls

1. Validate the complete repaired fixture before any lane and require each
   product/factor/certificate kernel to reject its relevant resealed payload.
2. Execute all masks exactly once in fixed order; reject duplicate, missing,
   out-of-range or relabelled schedules.
3. Bind mask, selected product type/root at every slot, ordered solve/scalar/
   state roots and full per-lane work into each transaction root.
4. Bind all eight transaction/certificate roots and aggregate work into the
   result root.
5. Require both endpoint correspondences before reading hybrid pass bits.
6. Verify the minimal-reject algorithm exhaustively over every synthetic
   eight-bit pass pattern consistent with passing mask 0 and rejecting mask 7.
7. Certificate mutation changes only the certificate/lane/result boundary and
   cannot alter transaction roots.
8. Invalid dimension, nonfinite/overflow product and nonpositive scalar
   boundaries reject before partial state publication with exact work prefix.
9. Freeze commit/source/binary/cache/command/stdout hashes; execute twice and
   require fresh independent code/evidence review before interpretation.

## Stop and ceiling

Stop `INCONCLUSIVE` on an endpoint or load-bearing review defect. Do not repair
more than one batched review surface under this revision.

Success localizes only the product-use subset responsible on this exact
recurrence. It authorizes no operator correction, regularization, homotopy,
precision, extra iteration, tolerance, solver replacement, timing, corpus,
GPU/runtime path, nonlinear commit or production claim.
