# NSR3-B4E2D7R20R63ZP direct direction-product admission contract

Revision: `3 / COMPLETE_COMMIT_WORK_LAYOUT_FROZEN_BEFORE_CODE / APPARATUS_PASS / NO_ENDPOINT_AUTHORITY`.

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZP` |
| Architecture snapshot | SPEC-38 and ADR-076 `Proposed`; ADR-081 `Accepted`; R63ZM reviewed `GO`; R63ZN `INCONCLUSIVE`; R63ZO tight enclosure rejected |
| Engineering consumer | determine whether one directly evaluated first PCG direction product can cross a small independently checked fixed-artifact boundary |
| Claim class | exact fixed-profile product containment and first-transition feasibility only |
| Budget | one candidate artifact, one structurally independent exact-dyadic checker, one batched repair and one re-review at most |

## Frozen question

Using only the exact reviewed R63ZM cache/artifact/audit, independently derive
the fixed initial direction

```text
r0   = b - H*x0
z0   = B^-1 r0
p0   = z0
rho0 = r0^T z0,
```

then evaluate exactly one direct two-stage tangent product

```text
q0 = sigma T (T^T p0).
```

Can a candidate containing `p0`, the binary128 `q0` primaries, explicit
component error bounds and complete executed-work receipts be admitted by a
separate checker that reconstructs the exact real product with signed dyadic
integer arithmetic and proves:

1. every exact component lies inside the candidate interval;
2. the induced `p0^T q0` interval is strictly positive;
3. the resulting division interval for `rho0/(p0^T q0)` contains the candidate
   scalar; and
4. the compensated update with that scalar reproduces all `102` stored `x1`
   components as a late fixed-profile consequence?

A positive result admits only one fixed direct direction product. It does not
admit `x1` as generated state, `p1`, `H*p1`, a recurrence, a certificate, a
representation family or a runtime solver.

## Trust and causal boundary

The only parent identities are the reviewed R63ZM revision-5 files:

```text
cache     23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84
artifact  ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87
audit     fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80
```

The candidate may read only the frozen cache fields required for `T`, `sigma`,
the exported factor/permutation/inverse, RHS and `x0`, plus the R63ZM role-2
`H*x0` record after independently solved `x0` correspondence. R63ZM roles
`3..5`, cached `x1/x2` and all certificates are unavailable until after the
candidate artifact and its roots are complete. The final `x1` update check is
a consequence-only control and cannot affect `p0`, `q0`, bounds, roots or
route selection before the update gate.

R63ZN and all R63ZJ/K/L DTOs, validators, schedulers, classifiers, receipts and
routes are forbidden inputs. R63ZO intervals may not seed or shrink the direct
product bounds. Whole-file hash admission precedes every decode.

## Candidate arithmetic

The candidate reconstructs `p0` with the frozen factor and compensated
residual/scalar schedules. It computes `q0` only by the fixed two-stage
binary128 tangent action:

1. `315` compensated dots of length `102` for `T^T p0`;
2. `102` compensated dots of length `315` for `T(T^T p0)`;
3. one binary128 scale product per output component; and
4. the published Dot2/scale forward bound, including propagated inner-dot
   errors, for every component.

The candidate serializes the complete `p0`, `q0`, component bounds, `rho0`,
denominator and step primary, their canonical roots, immutable parent roots,
ordered event trace and executed-work receipt. Every field participates in a
domain-separated final root. No observed checker value can alter it.

## Independent checker

The checker is a separate strict-C++20 executable and translation unit. It may
share only the frozen SHA-256 utility and byte-format constants. It must not
include, link or call candidate parsing, factor solve, compensated dot,
product, bound, work, event, root or classification code.

The checker independently:

1. verifies all three parent whole-file roots and candidate length/version;
2. decodes only causally available fields with an independent cursor;
3. derives `x0`, validates role-2 correspondence, then derives `p0` and
   compares all `102` components and the canonical root;
4. decodes every finite binary128 tangent/direction value into a signed integer
   significand and power-of-two exponent;
5. forms exact `T^T p0`, exact `T(T^T p0)` and exact scale multiplication by
   aligned signed-integer dyadic accumulation;
6. checks each exact dyadic value against the candidate primary-plus-bound
   interval without converting the oracle back to binary128;
7. independently encloses the denominator and division, then runs the
   consequence-only compensated update comparison; and
8. compares every semantic field, event and actually executed work counter
   before verifying the final root.

All fixed-size comparison loops execute every comparison unconditionally.
For variable or malformed paths the receipt records only predicates and bytes
actually evaluated. Planned counts must never be published after
short-circuit execution.

## Exact work and layout boundary

Candidate and checker receipts separately own at least:

- file open/read-call/byte/trailing-byte/close/hash counts;
- binary128, binary64 and index decodes;
- factor forward/backward terms and solves;
- residual, rho, denominator and update compensated-dot terms;
- inner/outer tangent terms and scale products;
- exact-dyadic decode, alignment, multiply and accumulation operations;
- interval comparisons and positivity/division predicates;
- canonical root derivations, event comparisons and seal comparisons; and
- bytes serialized and compared.

Each receipt is sealed independently. Full structure layout is asserted from
magic through terminal root with adjacent-field and total-size assertions in
both producer and checker. The checker audit seals candidate work, checker
work, first-specific route, event trace and all comparison results.

## Routes

Routes are first-specific:

1. `R63ZP_APPARATUS_REJECTED` — invocation, identity, length, parse, layout,
   nonfinite or exact-arithmetic apparatus failure;
2. `DIRECTION_INPUT_REJECTED` — `x0`, role-2, residual, solve, `p0` or `rho0`
   mismatch;
3. `DIRECT_PRODUCT_BOUND_REJECTED` — an exact dyadic `q0` component escapes
   its candidate interval;
4. `DIRECT_CURVATURE_REJECTED` — denominator enclosure is not strictly
   positive or does not contain the candidate primary;
5. `DIRECT_STEP_REJECTED` — division enclosure misses the candidate step;
6. `FIXED_UPDATE_CONSEQUENCE_REJECTED` — the sealed product is valid but the
   late compensated update does not reproduce all `102` fixed `x1` values;
7. `DIRECT_DIRECTION_PRODUCT_ADMISSION_CANDIDATE` — all preceding gates pass.

Unknown routes, flags, versions or trailing bytes fail closed.

## Required controls

- cache/artifact/audit stale, truncated, trailing and same-length mutations;
- role-2 input/value/count/root and early-consumption controls;
- direction component/root, rho and factor/RHS/tangent/scale mutations;
- product primary, component bound, roots and exact-oracle escape mutations;
- negative, zero and nonfinite exact components plus exponent-alignment cases;
- denominator lower-bound zero/negative, division miss and update mismatch;
- candidate/checker work fields mutated one at a time after resealing;
- receipt magic/version/size/route/event/order/root and terminal-seal mutations;
- explicit early mismatch controls proving actual, not planned, checker counts;
- oracle-before-candidate-seal and future-state-before-product-seal controls;
- unknown-route fail-safe and first-specific precedence; and
- two clean Release builds, two baseline runs, two complete control runs,
  sanitizer execution and exact R63ZM regression identities.

Every successful negative control has a distinct sealed checker audit.

## Stop rules and claim ceiling

If exact dyadic containment fails, stop direct binary128 Dot2 product admission
and localize the first escaping component; do not widen from observed error.
If product containment passes but curvature or step fails, stop before update
or recurrence work. If the update consequence alone fails, preserve the
product claim separately and do not repair the state transition in this
package. Any author pass remains unreviewed until one independent review; one
batched repair and one re-review are the complete review budget.

Even reviewed `GO` would authorize only this fixed direct `H*p0`
value-plus-error artifact at the exact R63ZM identities. It would not select
binary64/K2/width-three storage, construct `p1`, admit a recurrence or
certificate, generalize across fixtures, run timing, change runtime/Rust/GPU
code or promote SPEC-38/ADR-076. ProductChecks remain `NOT_RUN`.

## Apparatus outcome

The strict-C++20 apparatus selected
`DIRECT_DIRECTION_PRODUCT_ADMISSION_CANDIDATE`. Two Release runs and one
ASan/UBSan run were byte-identical at stdout SHA-256
`2a419ecf...0082fa`; the Release executable was `790352c8...3df423`.

All `102` exact real components were contained. The exact signed-dyadic vector
root is `271facfd...272f2f`; the minimum exact interval slack has `452`
significand bits at exponent `-567`, hence order `2^-116`. Exact `rho0` and
denominator are contained, the denominator lower bound is positive, the
division interval contains the candidate step and the late compensated update
matches `102/102` fixed components.

An independent Python arbitrary-integer calculation over a temporary raw
`p0/q0/bound` sidecar reproduced all `102` containments, the full exact vector
root and the three minimum-slack fields. Same-size cache/artifact/audit
identity mutations reject before output; replacing every component bound by
zero reaches `DIRECT_PRODUCT_BOUND_REJECTED` with first escape `0`.

This passes only the non-serialized apparatus gate. No candidate artifact,
receipt, checker audit, formal review or product admission exists yet.

## Revision-2 fixed binary package

Revision 2 freezes the serialized boundary before package code. All integers
are unsigned big-endian; binary128 payloads are canonical big-endian IEEE
bytes; roots are raw SHA-256 bytes. Padding is literal zero and checked.

The candidate artifact is exactly `5,920` bytes:

| Offset | Bytes | Field |
|---:|---:|---|
| 0 | 8 | magic `NER63ZP` |
| 8 | 4 | version `2` |
| 12 | 4 | total bytes `5920` |
| 16 | 4 | first-specific route |
| 20 | 4 | exact/normal/contained/positive/step/update flags |
| 24 | 96 | R63ZM cache/artifact/audit roots |
| 120 | 64 | role-2 input and value roots |
| 184 | 96 | `p0`, `q0` and component-bound roots |
| 280 | 80 | rho primary/bound, denominator primary/bound and alpha |
| 360 | 8 | dimension `102` |
| 368 | 8 | candidate work-field count `32` |
| 376 | 256 | candidate work fields |
| 632 | 32 | candidate work root |
| 664 | 8 | event count `8` |
| 672 | 256 | ordered event roots |
| 928 | 32 | trace root |
| 960 | 1632 | `p0[102]` |
| 2592 | 1632 | `q0[102]` |
| 4224 | 1632 | component bounds `[102]` |
| 5856 | 32 | pre-`x1` product-body root |
| 5888 | 32 | final result root |

The product-body root is computed before offsets for cached `x1` are decoded.
It seals all parent identities, role-2 roots, `p0/q0/bounds`, scalar fields
available at that point, their roots and the pre-seal work prefix. The final
result root adds the post-seal update result, complete work root and trace.

Candidate work fields, in exact order, are:

```text
file_open_attempts, read_calls, file_bytes, trailing_checks, close_calls,
file_hashes, cache_quad_decodes_preseal, cache_double_decodes,
cache_index_decodes, artifact_quad_decodes, factor_solves, factor_terms,
residual_dots, residual_terms, rho_dots, rho_terms, product_kernels,
inner_dots, inner_terms, outer_dots, outer_terms, propagation_terms,
scale_products, denominator_dots, denominator_terms, preseal_hash_calls,
preseal_hash_bytes, postseal_x1_decodes, update_dots, update_terms,
final_hash_calls, final_hash_bytes
```

The checker audit is exactly `864` bytes:

| Offset | Bytes | Field |
|---:|---:|---|
| 0 | 8 | magic `NER63ZQ` |
| 8 | 4 | version `2` |
| 12 | 4 | total bytes `864` |
| 16 | 4 | first-specific checker route |
| 20 | 4 | verified/contained/positive/step/update flags |
| 24 | 96 | R63ZM cache/artifact/audit roots |
| 120 | 128 | candidate-file, candidate-result, candidate-work and exact-product roots |
| 248 | 256 | checker work fields |
| 504 | 32 | checker work root |
| 536 | 8 | event count `8` |
| 544 | 256 | checker event roots |
| 800 | 32 | checker trace root |
| 832 | 32 | checker result root |

Checker work fields, in exact order, are:

```text
file_open_attempts, read_calls, file_bytes, trailing_checks, close_calls,
file_hashes, header_predicates_executed, candidate_bytes_decoded,
candidate_fixed_comparisons, cache_quad_decodes_preseal,
cache_double_decodes, cache_index_decodes, parent_artifact_quad_decodes,
factor_solves, factor_terms, residual_dots, residual_terms, dyadic_decodes,
exact_multiplies, exact_additions, alignment_shifts, alignment_bits,
interval_comparisons, exact_denominator_terms, postseal_x1_decodes,
update_dots, update_terms, candidate_work_comparisons, event_comparisons,
seal_comparisons, audit_hash_calls, audit_hash_bytes
```

Both translation units assert every adjacent offset and total size. The eight
ordered events are parent admission, `x0` correspondence, role-2 consumption,
direction derivation, product-body seal, scalar verification, update
consequence and final seal. Event 5 is therefore a hard causal firewall.

Numeric route ordinals are `1..7` in the order already frozen in **Routes**.
Zero and unknown ordinals fail closed. The producer writes a full fixed-size
artifact for every reachable route; unavailable semantic fields and events
are literal zero. The checker always writes one `864`-byte audit unless file
creation itself fails.

## Revision-3 complete commit-work package

Revision 2 is preserved as a rejected pre-code layout. Its prose required
serialization bytes to be owned, but neither 32-field ledger contained output
open/write/byte/close postconditions, serialized-field/byte counts, route
predicates or package-allocation ownership. Implementing that layout would
repeat the invisible-work class already found in R63ZK and R63ZN. No revision-2
package code or evidence exists.

Revision 3 changes only the fixed wire/work boundary. The exact question,
parent identities, arithmetic, routes, causal events, controls and claim
ceiling are unchanged.

The candidate artifact is exactly `6,144` bytes:

| Offset | Bytes | Field |
|---:|---:|---|
| 0 | 8 | magic `NER63ZP` |
| 8 | 4 | version `3` |
| 12 | 4 | total bytes `6144` |
| 16 | 4 | first-specific route |
| 20 | 4 | exact/normal/contained/positive/step/update flags |
| 24 | 96 | R63ZM cache/artifact/audit roots |
| 120 | 64 | role-2 input and value roots |
| 184 | 96 | `p0`, `q0` and component-bound roots |
| 280 | 80 | rho primary/bound, denominator primary/bound and alpha |
| 360 | 8 | dimension `102` |
| 368 | 8 | candidate work-field count `60` |
| 376 | 480 | candidate work fields |
| 856 | 32 | candidate work root |
| 888 | 8 | event count `0..8` |
| 896 | 256 | eight ordered event slots; unavailable suffix is zero |
| 1152 | 32 | trace root |
| 1184 | 1632 | `p0[102]` |
| 2816 | 1632 | `q0[102]` |
| 4448 | 1632 | component bounds `[102]` |
| 6080 | 32 | pre-`x1` product-body root |
| 6112 | 32 | final result root |

Candidate work fields, in exact order, are:

```text
rounding_mode_set_calls, rounding_mode_checks, input_open_attempts,
input_read_calls, input_read_bytes, input_trailing_checks, input_close_calls,
input_hash_calls, input_hash_bytes, parent_header_predicates,
cache_quad_decodes_preseal, cache_double_decodes, cache_index_decodes,
parent_artifact_quad_decodes, finite_predicates, range_predicates,
x0_component_comparisons, parent_root_comparisons, factor_solves,
factor_terms, factor_divisions, residual_dots, residual_terms, rho_dots,
rho_terms, product_kernels, inner_dots, inner_terms, outer_dots, outer_terms,
propagation_terms, scale_products, denominator_dots, denominator_terms,
interval_endpoint_operations, positivity_predicates,
division_endpoint_operations, division_predicates, preseal_root_calls,
preseal_root_bytes, postseal_x1_decodes, update_dots, update_terms,
update_comparisons, event_root_calls, event_root_bytes, trace_root_calls,
trace_root_bytes, final_root_calls, final_root_bytes, route_predicates,
receipt_zero_fill_bytes, receipt_fields_serialized,
receipt_bytes_serialized, output_open_attempts, output_write_calls,
output_write_bytes, output_close_calls, package_allocations,
fixed_loop_iterations
```

The pre-`x1` product-body root seals work fields `0..39`, events `0..3`, all
causally available scalar/vector roots and all three `p0/q0/bound` arrays.
Event slot 4 then seals that product-body root. Work field 40 and every later
field are unavailable to the body root. A nonzero post-seal field cannot be
used to construct or route the product.

The checker audit is exactly `1,112` bytes:

| Offset | Bytes | Field |
|---:|---:|---|
| 0 | 8 | magic `NER63ZQ` |
| 8 | 4 | version `3` |
| 12 | 4 | total bytes `1112` |
| 16 | 4 | first-specific checker route |
| 20 | 4 | verified/contained/positive/step/update flags |
| 24 | 96 | R63ZM cache/artifact/audit roots |
| 120 | 128 | candidate-file, candidate-result, candidate-work and exact-product roots |
| 248 | 504 | checker work fields |
| 752 | 32 | checker work root |
| 784 | 8 | event count `0..8` |
| 792 | 256 | eight ordered checker event slots |
| 1048 | 32 | checker trace root |
| 1080 | 32 | checker result root |

Checker work fields, in exact order, are:

```text
rounding_mode_set_calls, rounding_mode_checks, input_open_attempts,
input_read_calls, input_read_bytes, input_trailing_checks, input_close_calls,
input_hash_calls, input_hash_bytes, header_predicates_executed,
candidate_bytes_decoded, candidate_quad_decodes, candidate_u64_decodes,
candidate_digest_decodes, candidate_padding_bytes_checked,
parent_header_predicates, cache_quad_decodes_preseal, cache_double_decodes,
cache_index_decodes, parent_artifact_quad_decodes, finite_predicates,
range_predicates, x0_component_comparisons, parent_root_comparisons,
factor_solves, factor_terms, factor_divisions, residual_dots, residual_terms,
rho_dots, rho_terms, dyadic_decodes, exact_multiplies, exact_additions,
alignment_shifts, alignment_bits, bigint_capacity_predicates,
interval_comparisons, exact_denominator_terms, positivity_predicates,
division_endpoint_operations, division_predicates, postseal_x1_decodes,
update_dots, update_terms, update_comparisons, semantic_field_comparisons,
candidate_work_comparisons, event_comparisons, seal_comparisons,
route_predicates, root_calls, root_bytes, audit_zero_fill_bytes,
audit_fields_serialized, audit_bytes_serialized, output_open_attempts,
output_write_calls, output_write_bytes, output_close_calls,
package_allocations, exact_vector_hash_fields,
candidate_fixed_loop_iterations
```

The producer performs no dynamic allocation in the package path, and the
checker uses a fixed-capacity signed-dyadic representation with fail-closed
capacity predicates. `package_allocations` must therefore be zero and is
cross-checked with the existing process allocation probe.

Output counters are committed postconditions, not unverified predictions. A
candidate/audit root is formed with one exact output open, one exact full-size
write and one close recorded; the executable returns success only after those
three operations actually produce the complete fixed-size file. A failed or
partial write cannot yield an admitted artifact. No retry or alternate sink is
allowed. Serialization writes every byte of the zero-filled fixed buffer and
must report `399/6144` candidate fields/bytes or `87/1112` checker fields/bytes.

All fixed comparison regions evaluate every predicate before result folding.
Malformed and variable input paths increment counters at each executed
predicate/call and never overwrite them with success-path constants. Both
translation units assert the complete revision-3 offset chain from magic
through the terminal root. Any later field, size, ordering or work-scope change
requires revision 4 before code.
