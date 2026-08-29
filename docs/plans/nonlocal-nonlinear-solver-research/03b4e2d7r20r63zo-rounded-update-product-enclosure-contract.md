# NSR3-B4E2D7R20R63ZO rounded-update product-enclosure contract

Revision: `1 / FROZEN_BEFORE_CODE / PREFLIGHT_NEGATIVE / CLOSED_BEFORE_PACKAGE / NO_ENDPOINT_AUTHORITY`.

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZO` |
| Architecture snapshot | SPEC-38 and ADR-076 `Proposed`; ADR-081 `Accepted`; R63ZM reviewed `GO`; R63ZN `INCONCLUSIVE` |
| Engineering consumer | decide whether reviewed adjacent state products plus exact update-rounding cells can certify the first direction product without applying the tangent operator to that direction |
| Claim class | fixed-profile mathematical enclosure feasibility; no recurrence, representation or production claim |
| Budget | one first transition, two frozen envelope constructions, one late direct-product oracle, exact controls, two repeats, then stop before a checker package |

## Frozen question

Let the exact R63ZM cache and reviewed binary artifact provide dimension
`n=102`, tangent width `m=315`, binary128 tangent `T`, positive scale `sigma`,
exported factor/permutation/inverse, original RHS `b`, adjacent baseline states
`x0,x1` and independently checked product primaries `y0,y1` for `H*x0,H*x1`,
where

```text
H = sigma T T^T.
```

Independently derive the frozen first preconditioned direction

```text
r0   = b - y0
z0   = B^-1 r0
p0   = z0
rho0 = r0^T z0.
```

Can the set of real scalars that round `x0 + alpha*p0` componentwise to the
stored binary128 `x1`, together with enclosures for the two reviewed state
products and `H` applied only to the update-rounding error, produce a rigorous
enclosure of `H*p0` that:

1. contains an independently computed direct-product oracle;
2. keeps `p0^T H*p0` strictly positive; and
3. makes the rounded PCG step scalar a subset of the update-consistency set?

The positive answer would establish only that this fixed first transition has
a non-circular post-state enclosure certificate. It would not create `x1`,
admit R63ZN, recover `H*p1`, select a portable representation or authorize a
runtime algorithm.

## Trust boundary

The only parent authority is exact R63ZM revision 5:

```text
cache     23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84
artifact  ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87
audit     fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80
```

R63ZN candidate/checker receipts, R63ZJ/K/L DTOs and their routes are forbidden
inputs. Their numerical fields may not seed, bound or classify R63ZO. A direct
tangent application to `p0` is computed only after the candidate enclosure is
sealed and serves solely as an oracle containment control.

## Exact rounding-cell model

Arithmetic mode is IEEE binary128 round-to-nearest, ties-to-even. For a finite
stored component `x1[i]`, define its exact real rounding preimage `C_i` from
the midpoints to `pred(x1[i])` and `succ(x1[i])`, with endpoint inclusion
determined by the even significand. Signed zero, subnormal, overflow,
nonfinite and midpoint-tie cases are explicit; no implicit half-ulp formula or
decimal epsilon is allowed.

For nonzero `p0[i]`, invert

```text
x0[i] + alpha*p0[i] in C_i
```

with exact dyadic endpoints, reversing bounds when `p0[i] < 0`. For zero
`p0[i]`, require `x0[i]` itself to lie in `C_i`. Intersect all component
constraints to obtain `A0`. Reject if `A0` is empty, nonfinite or contains
zero.

For every `alpha in A0`, define

```text
delta(alpha) = x1 - (x0 + alpha*p0).
```

Each component receives the tight affine interval over the two endpoints of
`A0`; no observed direct product may shrink it.

## Two candidate envelopes

R63ZM product primaries are not treated as exact real products. Independently
reconstruct their published product-bound roots and use the resulting
component bounds `e0,e1` so that

```text
H*xk in [yk-ek, yk+ek],  k in {0,1}.
```

Construct both frozen bounds for `H*delta`:

1. tight dense bound: form the exact dyadic `H`, then
   `u_H = |H| * maxabs(delta)`;
2. rectangular bound: without cancellation, form
   `u_T = sigma * |T| * (|T|^T * maxabs(delta))`.

For each bound `u` and each `alpha in A0`, enclose

```text
H*p0 = (H*x1 - H*x0 - H*delta(alpha)) / alpha.
```

All interval endpoints use exact dyadics or explicit outward rounding. A
binary128 primary without its error term is never called exact.

## Scalar and transition gates

For each envelope, enclose the binary128 Dot2 evaluation of
`d0 = p0^T q0`, including product/sum rounding. The envelope passes only when:

- every component interval is finite and the late direct `q0` primary plus
  its independently computed bound is contained;
- `lower(d0) > 0`;
- the binary128 division enclosure for `rho0/d0` is finite, excludes zero and
  is a subset of `A0`; and
- every rounding-cell, state-product, operator-bound, dot and division work
  count equals the frozen schedule.

The direct oracle is a falsifier only. It cannot change `A0`, either candidate
envelope, the denominator bound or the selected route.

## Routes

Routes are first-specific:

1. `R63ZO_APPARATUS_REJECTED` — identity, parse, parent, exact-arithmetic,
   nonfinite or oracle-containment failure;
2. `UPDATE_PREIMAGE_EMPTY` — no common nonzero `A0` exists;
3. `TIGHT_PRODUCT_ENCLOSURE_REJECTED` — even exact `|H|` cannot preserve the
   positive denominator and step subset;
4. `DENSE_ABSOLUTE_ENVELOPE_REQUIRED` — exact `|H|` passes but the
   cancellation-free rectangular bound does not;
5. `RECTANGULAR_ROUNDING_ENVELOPE_CANDIDATE` — both envelopes pass.

No route claims a generated iterate, complete recurrence or production
representation.

## Competing hypotheses

| ID | Hypothesis | Prediction |
|---|---|---|
| H1 | update-rounding error is small enough for the exact dense absolute operator | tight envelope preserves denominator positivity and its step interval lies inside `A0` |
| H2 | cancellation loss in `|T||T|^T` is decisive | tight envelope passes but rectangular envelope rejects |
| H3 | conditioning amplifies even the exact rounding cell too much | tight envelope rejects before any representation choice |
| H4 | state products or update witnesses are inconsistent | `A0` is empty or the late direct oracle escapes the enclosure, producing apparatus rejection |

## Required controls

- one-dimensional exact update, absorption and halfway ties with even and odd
  result significands;
- positive and negative direction components plus a valid zero component;
- empty scalar intersection and an interval containing zero;
- literal diagonal SPD matrices where tight and rectangular envelopes agree;
- a cancellation matrix where tight passes and rectangular deliberately
  widens;
- nonfinite/subnormal/overflow rejection;
- state/product/bound-root, factor, RHS, tangent, scale and direct-oracle
  mutations;
- oracle-before-seal and oracle-fed-bound controls;
- every declared work counter mutated independently after resealing; and
- unknown-route fail-safe plus first-specific precedence.

## Evidence and stop rules

The first implementation is a bounded feasibility preflight. It must publish
exact endpoints, widths, worst components, denominator/step intervals, oracle
containment, both work ledgers and stable hashes twice. It grants no scientific
claim and must stop before a receipt/checker package.

If the tight envelope rejects, close the state-product reconstruction path and
return to a direct independently admitted direction-product representation.
If only the rectangular envelope rejects, research a portable dense absolute
bound before any recurrence. If both pass, freeze a separate verifier-first
package that treats the future state only as a witness and cannot generate it.

Do not repair or consume R63ZN, fit a tolerance, import R63ZJ/K/L routes,
change the factor/RHS/operator, add an iteration, select width three, build a
dynamic producer, run timing, or claim runtime/Rust/GPU/production authority.
SPEC-38/ADR-076 remain `Proposed`, ADR-081 remains binding and ProductChecks
remain `NOT_RUN`.

## Preflight outcome

The strict-C++20 offset-only preflight selected
`TIGHT_PRODUCT_ENCLOSURE_REJECTED`. Two Release executions were byte-identical
at stdout SHA-256 `d3cfe83e...d5ce3`; the executable was
`5bcffaf7...62245`. It reproduced the independently solved `x0`, both reviewed
state-product values and bound roots, and all `102/102` compensated update
components. The late direct product remained contained and both denominator
lower bounds remained positive.

The common update-consistency enclosure had width `2^-112`, while the tight
step enclosure was approximately `[0.89746, 0.91741]` and therefore about
`2^106.35` times wider. Sensitivity accounting localized the loss to the
absolute image of update rounding: state-product uncertainty contributed only
about `2^-54` to the denominator, while the tight `|H|` update term contributed
about `2^-3`. The cancellation-free rectangular term was wider still.

This was a feasibility preflight, not the exact-rational receipt/checker
package described above. It grants no positive enclosure theorem. The margin
is nevertheless sufficient for the frozen stop rule: do not build that
package or infer `H*p0` from adjacent rounded states. Any successor must admit
a direct direction product independently and must not consume R63ZN.
