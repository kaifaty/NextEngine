# NSR3-B4E2D7R20R63ZF quadratic-step/sign contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZF` |
| Parent | reviewed R63ZE result `2a291006...731c` |
| Engineering consumer | choose raw-sign audit or certificate-enclosure factorial |
| Claim class | fixed binary128/twofold quadratic and sign-boundary audit |
| Arithmetic | frozen Linux x86-64 strict binary128 plus existing Dot2 bounds |
| Budget | one shared prefix, two final endpoints, no timing |

## Exact claim and negation

On the frozen tangent trajectory through `p1`, reproduce the reviewed tangent
and common final products, denominators, step lengths, solution endpoints and
R63Y certificates. Enclose the executed quadratic discrepancy in two algebraic
forms, enclose the induced componentwise solution displacement, and classify
whether the raw binary128 solution signs differ before certificate uncertainty
is applied.

The positive result is an identity-, endpoint-, algebra-, work- and
control-closed route selecting exactly one next certificate-boundary audit.
The negation is the first fixture, arithmetic, positivity, endpoint,
containment, component-correspondence, raw-zero, certificate or work failure.

## Frozen identity

- Parent cache schema/root:
  `nextengine.nonlocal.formula_probe_parent_fixture.r63zc.v1`,
  `7780543a...4553`.
- Reviewed R63ZE stdout/result:
  `4fcb94cd...f4a4`, `2a291006...731c`.
- Final product roots:
  tangent `c24382a3...d29ff7`, common `0edf282e...22112b`.
- Denominator roots:
  tangent `13ddc46e...e7347e`, common `b2b6d749...c7a730`.
- Tangent/common state-2 solution roots:
  `38f8d0b2...0baea`, `f7e28b44...f59e8`.
- Tangent/common certificate roots:
  `d4ba011a...b8ecd`, `59170126...e5d4`.
- Original RHS, inverse scale, tangent/common operators, factor, permutation,
  profile, update order and depth are unchanged.

The exact reviewed denominator/step hex values are gates, not fitted inputs:

```text
d_t     0x1.0aa2a45ce0a757d6d8d6a216fb75p-2
d_c     0x1.0d0965b4944f15fb2a261d799f16p-2
alpha_t 0x1.12a78d1467ea7a19c272f3cdbfeep+0
alpha_c 0x1.1033f5858aac3dfb3b0ae944a492p+0
```

## Private scalar correspondence extension

`FormulaProbeScalar` adds the binary128 `bound` already produced by the
unchanged `ALR20R38Dot2` kernel. Its existing `exact`, `positive`, `value` and
root semantics remain unchanged. Parent cache serialization contains no
`FormulaProbeScalar` and must remain byte-identical. The extension has no
runtime/public-schema authority.

Every scalar audit binds center, bound, lower/upper projection and producer
root. Bound mutation must change the child/result identity while leaving the
center unchanged.

## Fixed execution

1. factor solve `x0=M^-1b`; tangent `Hx0`; build `r0,z0,rho0,p0`;
2. tangent `Hp0`; build `x1,r1,z1,rho1,p1`;
3. compute ordered tangent/common final products `t,c`;
4. compute ordered twofold denominators `d_t,d_c` and both positive steps;
5. build `x_t,r_t` then `x_c,r_c` in fixed endpoint order;
6. run immutable R63Y certificates for `x_t,x_c`;
7. build `c-t` with two-term bounds and audit `d_c-d_t` against direct
   `Dot2(p1,c-t)` with propagated input radii;
8. audit every `x_c-x_t` against `(alpha_c-alpha_t)p1` under outward twofold
   bounds;
9. count/root raw `-1/0/+1` signs for both solutions and changed indices.

No certificate or raw-sign result can stop an earlier operation.

## Endpoint and algebra gates

1. Product, denominator, hex, solution and certificate roots reproduce R63ZE.
2. Tangent certificate is `24+/78-/0?`; common is `12+/24-/66?`.
3. The two quadratic-discrepancy intervals overlap, exclude zero and are
   strictly positive; reversed order is strictly negative.
4. `alpha_c-alpha_t` excludes zero and is strictly negative.
5. All 102 componentwise displacement relations are contained and no raw
   solution component is zero.
6. Raw sign roots, changed-index root, both certificate `error_upper` bit
   identities and all algebra roots bind the result semantic.

## Classification

After apparatus, identity, work, endpoint, quadratic and component gates:

1. `QUADRATIC_STEP_SIGN_APPARATUS_REJECTED`;
2. `QUADRATIC_STEP_SIGN_IDENTITY_REJECTED`;
3. `QUADRATIC_STEP_SIGN_WORK_REJECTED`;
4. `QUADRATIC_STEP_SIGN_ENDPOINT_REJECTED`;
5. `QUADRATIC_DISCREPANCY_CONTAINMENT_REJECTED`;
6. `STEP_DISPLACEMENT_CONTAINMENT_REJECTED`;
7. `RAW_SOLUTION_ZERO_BOUNDARY_REJECTED`;
8. `RAW_SIGN_CHANGE_CONTRIBUTES` when at least one nonzero raw sign changes;
9. `CERTIFICATE_ENCLOSURE_AMPLIFICATION_CANDIDATE` when raw signs are
   identical, tangent passes, common rejects and common `error_upper` is
   strictly larger;
10. otherwise `CERTIFICATE_MARGIN_REPRESENTATION_AUDIT_REQUIRED`.

All synthetic raw-sign/change/error-order combinations are classified without
using the observed result. Routes 8--10 select research only.

## Fixed primary work

```text
certificates / products / solves        2 / 4 / 3
factor terms / divisions                30906 / 612
recurrence scalar dots / divisions      5 / 4
quadratic discrepancy dots              1
solution / residual / direction updates 306 / 306 / 102
product-difference updates               102
solution-difference updates              102
predicted-displacement products          102
scalar-difference updates                 2
raw sign comparisons                     204
tangent inner / outer dots               945 / 306
tangent inner / outer terms              96390 / 96390
tangent scale products                   306
common dots / terms                      102 / 10404
adaptive stops                           0
```

Two-term update internals remain classified as updates, matching the R63ZD/E
ledger convention; they are not double-counted as recurrence scalar dots.

## Controls

1. Validate the complete frozen fixture before any work and require the cache
   bytes to remain identical after the scalar DTO extension.
2. Bind every solve/product/scalar/update/endpoint/certificate/algebra/sign
   root and complete work ledger into the transaction/result roots.
3. Reproduce both R63ZE endpoints before interpreting discrepancy or signs.
4. Exercise bound-only mutation, reversed discrepancy, raw sign mutation and
   raw-zero route controls.
5. Invalid dimension, finite overflow and nonpositive scalar controls reject
   before endpoint publication with exact work prefixes.
6. An injected incomplete endpoint passes through the same guarded evaluator
   and returns apparatus rejection without partial publication.
7. Freeze commit/source/binary/cache/command/stdout hashes; run Release twice,
   require byte identity and one bounded independent review.

## Stop and ceiling

Stop `INCONCLUSIVE` on an endpoint, algebra, raw-zero or load-bearing review
defect. Apply at most one batched repair and one re-review.

Success classifies only whether raw sign change contributes on one exact
Linux x86-64 strict-binary128/twofold recurrence and chooses the next bounded
certificate audit. It does not identify a physically authoritative operator,
prove global-error sufficiency, choose a correction, change tolerance or
iterations, measure performance, validate a corpus or authorize
CPU/GPU/runtime/production integration.
