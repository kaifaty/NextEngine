# NSR3-B4E2D7R20R63ZG certificate-detail decomposition contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZG` |
| Parent | reviewed R63ZF result `0770e6cf...a38dd` |
| Engineering consumer | select image-center, image-radius, mixed-bound or margin audit |
| Claim class | fixed binary64 R63Y certificate-detail factorial |
| Arithmetic | unchanged R63Y strict binary64 Dot2Err/outward bounds |
| Budget | one R63ZF prefix, two immutable detail certificates, 18 synthetic sign cells, no timing |

## Exact claim and negation

On the frozen R63ZF tangent/common final solutions, reproduce both immutable
R63Y certificates, expose and independently seal their existing internal
image/solution components, recompute the native scalar identities, and cross
the two solution representations with tangent/common center-only,
radius-only, unamplified-image and native global error budgets.

The positive result is the first identity-, detail-, work-, correspondence-,
native-cell- and control-closed route selecting one next certificate audit.
The negation is the first parent, DTO, finite arithmetic, scalar identity,
native reproduction, work, mutation-sealing or incomplete-publication failure.

## Frozen identity

- Parent commit/result/stdout:
  `7dea7ddbcd98b2c489cb224396fed7dfb182463e`,
  `0770e6cf...a38dd`, `be74e412...de0e`.
- Parent cache/fixture:
  `23dbf605...bb84`, `7780543a...4553`.
- Tangent/common raw solution roots:
  `38f8d0b2...0baea`, `f7e28b44...f59e8`.
- Tangent/common certificate roots:
  `d4ba011a...b8ecd`, `59170126...e5d4`.
- Tangent/common certificate `error_upper` bits:
  `0x3f508e20150de982`, `0x42bc23fa1e984623`.
- Selected profile root remains `b1c43044...a291`; profile width/dimension are
  `2/102`; `rho_upper` bits are `0x3f828ed8e0fe9914` and the derived
  denominator bits are `0x3fefb5c49c7c059b`.

R63ZF recurrence, products, denominators, endpoints, private scalar bound and
all work/controls remain unchanged.

## Private detail DTO

`FormulaProbeCertificateDetail` is private and report-only. It contains:

```text
exact, width, dimension
dots, dot_products, radius_terms, solution_dots, sign_comparisons
image_infinity_upper, denominator_lower, error_upper, minimum_separation
image_center[102], image_radius[102]
solution_center[102], solution_local_radius[102]
minimal_certificate, profile_root
component_root, radius_root, root
```

The detail producer must:

1. validate the complete frozen fixture and solution shape first;
2. construct the unchanged width-two R63Y solution and certificate once;
3. copy image data without recomputing it;
4. recompute only each solution expansion's fixed-order Dot2 center and local
   radius `up(center.error + solution.radius[i])`;
5. bind every scalar bit identity and vector/root into the detail root;
6. return non-exact without partial publication on invalid/nonfinite input.

Minimal certificate roots and parent cache serialization remain unchanged.

## Fixed execution and scalar identities

1. Execute the complete reviewed R63ZF Primary once and require all parent
   roots, work and route.
2. Produce tangent then common details in that order.
3. For each detail recompute with the same outward primitives:

```text
center_inf = max_i abs(image_center[i])
radius_inf = max_i image_radius[i]
image_inf  = max_i up(abs(image_center[i]) + image_radius[i])
denom      = down(1 - rho_upper)
amp        = up(1 / denom)
center_G   = up(center_inf / denom)
radius_G   = up(radius_inf / denom)
native_G   = up(image_inf / denom)
```

4. Require bit equality of `image_inf`, `denom` and `native_G` with the
   immutable detail certificate.
5. Construct two local-only sign cells and the 16 fixed cells ordered by
   solution `T,C`, budget origin `T,C`, mode `U,GC,GR,G`.
6. Each cell applies the unchanged outward sign rule:

```text
total = up(solution_local_radius[i] + selected_global_budget)
lower = down(solution_center[i] - total)
upper = up(solution_center[i] + total)
```

No cell may stop a later cell.

## Native and decomposition gates

1. Detail minimal certificate, profile and solution roots reproduce R63ZF.
2. Native `T/G_t` and `C/G_c` cells reproduce certificate sign roots,
   pass bits and `24+/78-/0?`, `12+/24-/66?` counts exactly.
3. Both local-only cells pass and reproduce the shared raw sign vector.
4. The four native-global cross cells determine whether budget columns are
   solution-independent:

```text
T/G_t pass   C/G_t pass
T/G_c reject C/G_c reject
```

5. `U` versus `G` cells bind whether contraction amplification changes any
   sign classification; scalar magnitude alone is not a classification gate.
6. Common-origin `GC`, `GR` and `G` cells bind center/radius sufficiency only
   when gate 4 holds.

## Classification

After apparatus, identity, work, detail, scalar correspondence and native-cell
gates:

1. `CERTIFICATE_DETAIL_APPARATUS_REJECTED`;
2. `CERTIFICATE_DETAIL_IDENTITY_REJECTED`;
3. `CERTIFICATE_DETAIL_WORK_REJECTED`;
4. `CERTIFICATE_DETAIL_CORRESPONDENCE_REJECTED`;
5. `CERTIFICATE_DETAIL_NATIVE_CELL_REJECTED`;
6. `SOLUTION_MARGIN_INTERACTION_REQUIRED` when native global-budget columns
   are not solution-independent;
7. `AFFINE_IMAGE_CENTER_AND_RADIUS_INDEPENDENTLY_SUFFICIENT` when common `GC`
   and `GR` both reproduce rejection for both solutions;
8. `AFFINE_IMAGE_CENTER_RESIDUAL_SUFFICIENT` when only common `GC` does;
9. `AFFINE_IMAGE_RADIUS_SUFFICIENT` when only common `GR` does;
10. `AFFINE_IMAGE_MIXED_BOUND_REQUIRED` when common `G` rejects both but
    neither component budget does;
11. otherwise `CERTIFICATE_DETAIL_FURTHER_FACTORIAL_REQUIRED`.

An orthogonal `contraction_boundary_changed` bit records any `U/G` sign-cell
difference and binds the result; it does not override routes 6--11.

## Fixed work

The parent R63ZF Primary work remains exactly:

```text
certificates / products / solves        2 / 4 / 3
factor terms / divisions                30906 / 612
recurrence scalar dots / divisions      5 / 4
quadratic dots                           1
```

The detail audit adds exactly:

```text
detail certificates                     2
image dots / dot products                204 / 83640
image radius terms                       62424
native solution dots / sign comparisons  204 / 204
detail center dots / products             204 / 408
derived max component visits             612
derived budget divisions                   8
synthetic sign cells                     18
synthetic sign comparisons               1836
candidate/operator/solver updates        0 / 0 / 0
```

The detail producer's native sign comparisons are observationally counted even
though the immutable parent certificates already exist; no work is hidden or
credited twice within either ledger.

## Controls

1. Parent cache bytes, R63ZE regression, R63ZF stdout and kernel smoke remain
   byte-identical.
2. Bind every detail vector/scalar/count, every cell, ordered cell-set, work
   ledger and route into Primary/result roots.
3. Exhaustively enumerate classifier precedence and all pass/reject
   combinations used by routes 6--11.
4. Resealed center-only, radius-only and selected-budget mutations change
   detail/cell/result identities; a component swap without resealing rejects.
5. Invalid dimension, finite overflow, noncontractive denominator and
   incomplete detail reject before cell publication with exact work prefixes.
6. A native error mutation must fail native-cell correspondence even when its
   synthetic sign counts happen to alias.
7. Freeze commit/source/binary/cache/command/stdout hashes; run Release twice,
   require byte identity and one bounded independent review.

## Stop and ceiling

Stop `INCONCLUSIVE` on a detail, scalar-identity, native-cell or load-bearing
review defect. Apply at most one batched repair and one re-review.

Success localizes one certificate boundary on one fixed Linux x86-64 R63Y
profile. It does not validate a modified budget, prove a correction, select an
operator/profile/tolerance, measure performance, validate a corpus or
authorize CPU/GPU/runtime/production integration.
