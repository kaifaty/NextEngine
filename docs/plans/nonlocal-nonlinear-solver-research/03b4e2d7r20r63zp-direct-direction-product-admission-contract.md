# NSR3-B4E2D7R20R63ZP direct direction-product admission contract

Revision: `1 / FROZEN_BEFORE_CODE / APPARATUS_PENDING / NO_ENDPOINT_AUTHORITY`.

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
