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
- verified route-1 partial receipts for every invalid parent-input class;
- selector-1 observed-`x0` mismatch proving route 2 and zero product/oracle/
  future-state work;
- selector-2 zero component bound proving an exact-oracle escape and route 3;
- selector-3/4 denominator-lower zero and negative controls proving route 4;
- selector-5 division miss proving route 5;
- selector-6 late `x1` mismatch proving route 6 after a valid product seal;
- exact zero, negative, nonfinite, exponent-alignment and fixed-capacity cases
  through the checker's allocation-free `--exact-controls` mode;
- candidate/checker work fields mutated one at a time after resealing;
- receipt magic/version/size/route/event/order/root and terminal-seal mutations;
- a downstream-resealed event-2 mismatch proving zero body/oracle/`x1` work;
- a downstream-resealed body/event-5 mismatch proving zero exact/oracle/`x1`
  work;
- a downstream-resealed event-6 mismatch proving exact work may be nonzero but
  every `x1`/update counter is zero;
- explicit malformed and first-predicate mismatch controls proving actual, not
  planned, checker counts;
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

## Revision-6 single batched repair package

Revision 2 is preserved as a rejected pre-code layout. Its prose required
serialization bytes to be owned, but neither 32-field ledger contained output
open/write/byte/close postconditions, serialized-field/byte counts, route
predicates or package-allocation ownership. Implementing that layout would
repeat the invisible-work class already found in R63ZK and R63ZN. No revision-2
package code or evidence exists.

Revision 3 is also preserved as rejected before an executable package. The
first independent-checker compilation exposed seven unowned rounded-product
paths: one kernel, `315` inner dots, `32,130` inner terms, `102` outer dots,
`32,130` outer terms, `32,130` propagated-radius terms and `102` scale
products. Revision 4 added those fields, but the first baseline comparison
showed that its pre-`x1` body sealed the global `finite_predicates` field and
then incremented that same field during 102 post-seal `x1` checks. Revision 4
is therefore rejected before evidence too. Revision 5 added one candidate
`postseal_x1_finite_predicates` field, but formal review rejected it because
checker oracle/`x1` work crossed the body seal, routes 1--5 had no independent
partial receipts, fixed predicates overstated short-circuit work and the
nominal control count omitted required arithmetic/causal branches.

Revision 6 is the single permitted batched repair. It changes causal staging,
partial-route fixtures and the wire/work boundary, but not the direct-product
question, parent identities, baseline arithmetic or claim ceiling.

The candidate artifact is exactly `6,176` bytes:

| Offset | Bytes | Field |
|---:|---:|---|
| 0 | 8 | magic `NER63ZP6` |
| 8 | 4 | version `6` |
| 12 | 4 | total bytes `6176` |
| 16 | 4 | first-specific route |
| 20 | 4 | exact/normal/contained/positive/step/update flags |
| 24 | 96 | R63ZM cache/artifact/audit roots |
| 120 | 64 | role-2 input and value roots |
| 184 | 96 | `p0`, `q0` and component-bound roots |
| 280 | 80 | rho primary/bound, denominator primary/bound and alpha |
| 360 | 8 | dimension `102` |
| 368 | 8 | candidate work-field count `64` |
| 376 | 512 | candidate work fields |
| 888 | 32 | candidate work root |
| 920 | 8 | event count `0..8` |
| 928 | 256 | eight ordered event slots; unavailable suffix is zero |
| 1184 | 32 | trace root |
| 1216 | 1632 | `p0[102]` |
| 2848 | 1632 | `q0[102]` |
| 4480 | 1632 | component bounds `[102]` |
| 6112 | 32 | pre-oracle/pre-`x1` product-body root |
| 6144 | 32 | final result root |

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
division_endpoint_operations, division_predicates,
control_selector_predicates, preseal_control_injections, preseal_root_calls,
preseal_root_bytes, postseal_x1_decodes, postseal_x1_finite_predicates,
postseal_control_injections, update_dots, update_terms,
update_comparisons, event_root_calls, event_root_bytes, trace_root_calls,
trace_root_bytes, final_root_calls, final_root_bytes, route_predicates,
receipt_zero_fill_bytes, receipt_fields_serialized,
receipt_bytes_serialized, output_open_attempts, output_write_calls,
output_write_bytes, output_close_calls, package_allocations,
fixed_loop_iterations
```

The pre-oracle/pre-`x1` product-body root seals work fields `0..41`, events
`0..3`, the negative-control selector, all
causally available scalar/vector roots and all three `p0/q0/bound` arrays.
Event slot 4 then seals that product-body root. Work field 42 and every later
field are unavailable to the body root. A nonzero post-seal field cannot be
used to construct or route the product.

All serialized package/control roots except the retained apparatus exact-dyadic
root use a one-byte tag plus unsigned-big-endian `u64` byte length before each
field. Tags are `1=raw bytes`, `2=u32`, `3=u64`, `4=binary128`, `5=digest`,
`6=u64 array`, `7=digest array`, `8=binary128 array`. A domain is the first
tag-1 field. Integers inside a typed field are big-endian; binary128 is
canonical IEEE big-endian. No host padding, C++ object bytes or implicit
terminator participates.

The producer domains and ordered fields are:

```text
nextengine.nonlocal.r63zp.quad-vector.v1:
  dimension, values
nextengine.nonlocal.r63zp.candidate-work.v1:
  route, all 64 work fields
nextengine.nonlocal.r63zp.product-body.v1:
  three parent roots, two role roots, three semantic roots, five scalars,
  dimension, control selector, work fields 0..41, event slots 0..3,
  p0, q0, bounds
nextengine.nonlocal.r63zp.candidate-event.v1:
  event ordinal, stage-local candidate route, then the event-specific fields
  below
nextengine.nonlocal.r63zp.candidate-trace.v1:
  event_count, all eight event slots
nextengine.nonlocal.r63zp.candidate-result.v1:
  route, flags, three parent roots, two role roots, three semantic roots,
  five scalars, dimension, candidate work root, event_count, trace root,
  product-body root, update-match count
```

Candidate events are exactly:

1. control selector and three observed parent roots;
2. parent role-2 input root, independently derived `x0` root and observed
   cache-`x0` root;
3. role-2 input and value roots;
4. `p0` root plus rho primary/bound;
5. product-body root;
6. `q0`/bound roots plus denominator primary/bound and alpha;
7. post-seal derived `x1` root and update-match count;
8. product-body root and candidate work root.

The stage-local route is the first rejection known at that event, or ordinal
`7` when no rejection is known yet. Event counts are exactly `1/2/5/6/6/8/8`
for candidate routes `1..7`. Route 3 seals the body with unavailable
denominator/step fields zero; routes 4 and 5 seal the body plus event 6; routes
6 and 7 additionally own the late `x1` event and final work seal. Events 1
through 6 are constructed before any `x1` decode and therefore cannot contain
or change in response to final route 6 versus 7. Event 7 first owns that
distinction; event 8 repeats the final route. This rule applies even when an
externally mutated candidate later claims another final route.

The candidate flag bits are `0=input/direction exact`, `1=product arithmetic
exact/normal`, `2=candidate bounds finite/nonnegative`, `3=curvature positive`,
`4=step interval contains alpha`, `5=102/102 late updates`; bits `8..15` hold
the negative-control selector and all other bits are zero. Selector `0` is the
only admissible path. Selectors `1..6` are respectively observed-`x0`
mismatch, zero component bound, denominator lower zero, denominator negative,
division miss and late `x1` mismatch. They deterministically exercise routes
`2/3/4/4/5/6`, are sealed in body/event/result roots, and can never produce
route 7. Invalid selectors fail as malformed before replay. Candidate route
precedence remains `1..7`.

The production producer invocation has no selector argument and therefore
always seals selector 0. The same executable accepts one optional decimal
selector only for the frozen control corpus. Selector parsing and the actual
stage-local injection have distinct sealed counters. The checker never
receives a selector argument: it reads the sealed candidate selector, applies
the independently implemented fixture and requires the named arithmetic gate
to produce the claimed negative route. Thus a selector is not shared route
authority, and any nonzero selector is structurally incapable of admission.

The checker audit is exactly `1,176` bytes:

| Offset | Bytes | Field |
|---:|---:|---|
| 0 | 8 | magic `NER63ZQ6` |
| 8 | 4 | version `6` |
| 12 | 4 | total bytes `1176` |
| 16 | 4 | first-specific checker route |
| 20 | 4 | verified/contained/positive/step/update flags |
| 24 | 96 | R63ZM cache/artifact/audit roots |
| 120 | 128 | candidate-file, candidate-result, candidate-work and exact-product roots |
| 248 | 568 | checker work fields |
| 816 | 32 | checker work root |
| 848 | 8 | event count `0..8` |
| 856 | 256 | eight ordered checker event slots |
| 1112 | 32 | checker trace root |
| 1144 | 32 | checker result root |

Checker work fields, in exact order, are:

```text
rounding_mode_set_calls, rounding_mode_checks, input_open_attempts,
input_read_calls, input_read_bytes, input_trailing_checks, input_close_calls,
input_hash_calls, input_hash_bytes, header_predicates_executed,
candidate_bytes_decoded, candidate_quad_decodes, candidate_u64_decodes,
candidate_digest_decodes, candidate_padding_bytes_checked,
parent_header_predicates, cache_quad_decodes_preseal, cache_double_decodes,
cache_index_decodes, parent_artifact_quad_decodes, finite_predicates,
range_predicates, control_injections, x0_component_comparisons,
parent_root_comparisons,
factor_solves, factor_terms, factor_divisions, residual_dots, residual_terms,
rho_dots, rho_terms, rounded_product_calls, rounded_inner_dots,
rounded_inner_terms, rounded_outer_dots, rounded_outer_terms,
rounded_propagation_terms, rounded_scale_products, dyadic_decodes,
exact_multiplies, exact_additions,
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

The checker domains and ordered fields are:

```text
nextengine.nonlocal.r63zp.exact-dyadic-vector.v1:
  the apparatus encoding is preserved exactly: untagged domain length u64,
  domain bytes, count u64, then for each normalized value sign byte, signed
  exponent encoded as two's-complement u64, minimal big-endian magnitude
  length u64 and magnitude; this must reproduce `271facfd...272f2f`
nextengine.nonlocal.r63zp.checker-work.v1:
  checker route, all 71 work fields
nextengine.nonlocal.r63zp.checker-event.v1:
  event ordinal, checker route, candidate event root, independently derived
  event root, equality flag
nextengine.nonlocal.r63zp.checker-trace.v1:
  event_count, all eight checker event slots
nextengine.nonlocal.r63zp.checker-result.v1:
  checker route, flags, three parent roots, candidate file/result/work and
  exact-product roots, checker work root, event_count and checker trace root
```

Checker flag bits are `0=semantic verified`, `1=exact product contained`,
`2=curvature verified positive`, `3=step verified`, `4=update consequence
verified`; bits `8..15` repeat the candidate control selector and all other
bits are zero. Checker routes `1..7` verify the corresponding candidate route;
verified negative routes return process success but never admission. Checker-
only first-specific routes are `8=CANDIDATE_MALFORMED`,
`9=SEMANTIC_MISMATCH`, `10=WORK_MISMATCH`, `11=EVENT_MISMATCH`,
`12=SEAL_MISMATCH`, `13=CHECKER_APPARATUS_REJECTED`. Zero and ordinals above
13 fail closed before semantic replay.

Checker replay is a mandatory three-stage schedule:

1. reconstruct rounded inputs/direction/product and the expected partial
   producer receipt; compare all available semantics, producer work fields
   `0..41`, events 1--5, the independently derived body and the candidate's
   body self-seal;
2. only after stage 1 passes, run the exact signed-dyadic oracle, classify
   product/curvature/step routes and compare event 6 when available;
3. only after the route-3/4/5 gates and event 6 pass, decode any `x1`, execute
   the late update and compare events 7--8 plus terminal work/trace/result.

A semantic, work, event or seal failure at a stage returns immediately to the
matching checker route. Body/event-5 failure has zero dyadic/exact and zero
`x1` work; event-6 failure may have exact work but has zero `x1` work. Invalid
parents are independently reconstructed as verified candidate route 1 rather
than checker route 13. Route 13 is reserved for a failure of the checker's own
rounding, fixed-capacity exact apparatus or root construction after readable
inputs.

With the sole argument `--exact-controls`, the checker runs no package replay
and writes one canonical compact JSON object with booleans
`zero_valid`, `negative_valid`, `nonfinite_rejected`, `alignment_valid` and
`capacity_rejected`, plus the actual dyadic decode/multiply/add/shift/capacity
counters. Success requires all five booleans true, no allocation and no file
input. This control mode is not an admission path and cannot emit an audit.

The producer performs no dynamic allocation in the package path, and the
checker uses exactly 64 little-endian 32-bit magnitude limbs per signed dyadic
with fail-closed capacity predicates. `package_allocations` must therefore be
zero and is cross-checked with the existing process allocation probe.

Output counters are committed postconditions, not unverified predictions. A
candidate/audit root is formed with one exact output open, one exact full-size
write and one close recorded; the executable returns success only after those
three operations actually produce the complete fixed-size file. A failed or
partial write cannot yield an admitted artifact. No retry or alternate sink is
allowed. Serialization writes every byte of the zero-filled fixed buffer and
must report `403/6176` candidate fields/bytes or `95/1176` checker fields/bytes.

All fixed comparison regions first evaluate every predicate into named
temporaries, increment the matching count, and only then fold the results. No
counted `&&`/`||` expression may short-circuit a later predicate. Malformed
and variable input paths increment counters at each executed predicate/call
and never overwrite them with success-path constants. Both
`fixed_loop_iterations` fields count predicate-bearing fixed vector
comparison loops only; serialization and hash traversal are owned by their
separate field/byte counters. Both
translation units assert the complete revision-6 offset chain from magic
through the terminal root. Any later field, size, ordering or work-scope change
closes R63ZP `INCONCLUSIVE`; revision 7 is forbidden by the exhausted repair
budget.
