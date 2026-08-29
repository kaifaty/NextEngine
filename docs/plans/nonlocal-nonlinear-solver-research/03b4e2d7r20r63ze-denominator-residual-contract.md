# NSR3-B4E2D7R20R63ZE denominator/residual factorial contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZE` |
| Parent | reviewed R63ZD result `1fec1e31...149f` |
| Engineering consumer | selection of one exact quadratic-discrepancy audit |
| Claim class | fixed final-product consumer factorial |
| Arithmetic | frozen Linux x86-64 strict binary128 profile |
| Budget | four equal-work lanes, two updates, states `0..2`, no timing |

## Exact claim and negation

Replay the frozen tangent recurrence through state 1. On its identical `p1`,
compute tangent product `t`, common product `c`, tangent denominator
`d_t=p1^Tt` and common denominator `d_c=p1^Tc`. Cross the denominator selector
with the terminal residual-product selector in four equal-work lanes.

The positive result is an identity-, endpoint-, alias- and work-closed pass
pattern that selects one declared causal class. The negation is the first
fixture, arithmetic, positivity, recurrence, endpoint, alias, selector,
certificate, work or observability failure.

## Frozen identity

- Parent cache schema/root:
  `nextengine.nonlocal.formula_probe_parent_fixture.r63zc.v1`,
  `7780543a...4553`.
- Reviewed R63ZD result/stdout:
  `1fec1e31...149f`, `90c73a8a...9461`.
- Original RHS/inverse scale, tangent/common operators, exported factor,
  permutation and R63Y profile are unchanged.
- R63ZD mask `0` supplies the required tangent state-1 trajectory; mask `4`
  supplies the common-final-product endpoint.

## Recurrence and selectors

Every lane executes exactly:

1. factor solve `x0=M^-1 b`;
2. tangent `Hx0`, `r0`, factor solve `z0`, positive `rho0`;
3. tangent `Hp0`, positive denominator, `x1/r1`, factor solve `z1`, positive
   `rho1`, `p1`;
4. both `t=H_t p1` and `c=H_c p1` in fixed tangent/common order;
5. both positive scalar dots `d_t=p1^Tt` and `d_c=p1^Tc` in the same order;
6. select `d_D`, compute `alpha1=rho1/d_D`, update
   `x2=x1+alpha1*p1`;
7. select product `q_R`, update `r2=r1-alpha1*q_R`;
8. seal states and run immutable R63Y certificates for `x0,x1,x2`.

Lane order is `TT, TC, CT, CC`, where the first symbol selects denominator and
the second selects residual product. Both unselected values remain computed,
rooted and counted.

## Endpoint and alias gates

1. `TT` must reproduce R63ZD `TTT`: reject/reject/pass, state-2 solution root
   `38f8d0b2c99126ff046d2bd6471c3c0a018b50fba66f38b418a8e3eb0600baea`.
2. `CC` must reproduce R63ZD `TTC`: reject/reject/reject, state-2 solution
   root
   `f7e28b44d35ee1e0bd0581425bded4f784f507be7b5bcce3605cc8b659bf59e8`.
3. `TT/TC` must share the state-2 solution root but have distinct residual and
   state roots.
4. `CT/CC` must share the state-2 solution root but have distinct residual and
   state roots.
5. All four lanes must share every state-0/state-1 solution, residual,
   direction and certificate root.

State-2 certificate-root equality within each denominator pair is the audited
solution-only isolation property. It is required for a positive result but is
not treated as a pre-classification apparatus gate: a mismatch publishes an
explicit isolation rejection.

## Classification

After all preceding gates:

1. `DENOMINATOR_RESIDUAL_APPARATUS_REJECTED`;
2. `DENOMINATOR_RESIDUAL_IDENTITY_REJECTED`;
3. `DENOMINATOR_RESIDUAL_WORK_REJECTED`;
4. `TANGENT_ENDPOINT_REJECTED`;
5. `COMMON_ENDPOINT_REJECTED`;
6. `DENOMINATOR_STEP_SUFFICIENT` for pass pattern `1100` with equal
   state-2 certificate roots inside both denominator pairs;
7. `CERTIFICATE_RESIDUAL_ISOLATION_REJECTED` for `1010`;
8. `CERTIFICATE_INDEPENDENT_ISOLATION_REJECTED` for `1000`;
9. `CERTIFICATE_INTERACTION_ISOLATION_REJECTED` for `1110`;
10. `CERTIFICATE_ROOT_ISOLATION_REJECTED` if pass pattern `1100` has unequal
    state-2 certificate roots inside a denominator pair;
11. otherwise `DENOMINATOR_RESIDUAL_PATTERN_REJECTED`.

The classifier is exhaustively checked over all 16 synthetic pass patterns.
Only `DENOMINATOR_STEP_SUFFICIENT` is a positive causal result; routes 7--11
stop further operator mathematics for an isolation/apparatus audit.

## Fixed work

Across all four lanes:

```text
certificates / products / solves        12 / 16 / 12
factor terms / divisions                123624 / 2448
scalar dots / divisions                 20 / 12
solution / residual / direction updates 816 / 816 / 408
tangent inner / outer dots              3780 / 1224
tangent inner / outer terms             385560 / 385560
tangent scale products                  1224
common dots / terms                     408 / 41616
adaptive stops                          0
```

## Controls

1. Validate the complete frozen parent fixture before any lane.
2. Bind selector pair, original input, both ordered final product roots, both
   denominator roots, all solve/scalar/state roots and full lane work into the
   transaction root.
3. Reject duplicate, missing, out-of-range and relabelled selectors.
4. Mutating either computed final-product or denominator identity must change
   the transaction/result root. A value not selected as the solution
   denominator cannot change the selected solution.
5. Certificate mutation changes only certificate/lane/result identity.
6. Invalid selector, finite product overflow and nonpositive selected
   denominator fail before state publication with exact work prefixes.
7. Require endpoint and solution/residual alias gates before reading the pass
   pattern; audit state-2 certificate aliases as part of classification.
8. Freeze source/binary/cache/command/stdout hashes; execute twice and require
   one bounded independent review before interpretation.

## Stop and ceiling

Stop `INCONCLUSIVE` on an endpoint, alias or load-bearing review defect. Apply
at most one batched repair followed by one re-review.

A positive result localizes only the final-slot certificate loss to the scalar
denominator/step length on this exact recurrence. Residual-dependent routes
are isolation rejections, not alternative causal success. R63ZE does not
establish which operator is physically authoritative, choose a correction,
fit a homotopy/tolerance, add iterations, measure performance, validate a
corpus or authorize CPU/GPU/runtime/production integration.
