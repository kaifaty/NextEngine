# NSR3-B4E2D7R2 -- topology/step discriminator contract

Status: `FROZEN / NOT_RUN / REPLAY_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r2-topology-step-discriminator|v1|parent=8902a9417b2dad68c51fab118892767e2f21be5191cbea0fcbe2e87e9079d862:3ba8ab9c999a63b72fa42ca4dad98062aea976977945fa16a60ba49bfcc6c51a:9a9582d09897f63ed7cd9953cc2fb6153b4266813fb6797ba71a1de98140cbfe|state=d7-prefix:04a9c03308662d6102b165d8109b23c1b60a3145314603909b7dbfda8385d95e:9bffc61a943f50cab449c052bd0a6791c40128e894d98d9a9589b0969dbb82c2;post-outer9-forced-private:31840abd2f10907491360d75d57f6ecbbe6b0cf94672fa1b703bcb5ffa9d5830|step=first-rejected-newton;trust0=0.0125m;quarter-shrink;reject-cap=8;no-accept|sets=sorted-active-centers;sorted-fluid-pairs;sorted-boundary-pairs;roots;added;removed;changed-pair-detail<=2048|horizon=0.15m;epsilon-scale=epsilon*h;report-radius-margin;normalized-w-w1-w2;require-zero-at-h|ladder=alpha=2^-e;e=0..20;rounded-step;coordinate-moved;live-roots-deltas;model;raw;direct;direct-ratio|continuation=current-radial-branch-per-relation;current-phr-branch-per-center;fixed-order-factored-difference;diagnostic-only|routes=active-constraint-branch-first;horizon-topology-derivative-if-continuation-admits-live-rejects;trust-reject-policy-if-smaller-live-admits;al-hessian-model-otherwise|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;commit=none;physics-mutation=none|credit=one-selected-remediation-research-only
```

Identity SHA-256:
`7b201ab9d281f8ffa3e866e88b067682d116ef3cf38a6507da4a3e4dea99561e`.

## Required command

Add `--nonlocal-al-topology-step-discriminator`. It must:

1. reproduce the exact D7R/D7R1 failed private state and reconstruct only its
   first rejected Newton direction;
2. emit exact current/full-trial roots and sorted differences for active
   centres, unique fluid pairs and boundary pairs;
3. emit every changed pair, capped at 2,048, with type/indices, current/trial
   radius and signed horizon margin, margin in `epsilon*h`, and normalized
   kernel value/first/second derivative;
4. prove binary64 kernel value/first/second derivative are exactly zero at the
   horizon;
5. emit the first quarter-radius shrink count that would bind the original
   physical step and the number still missing after the frozen nine trials;
6. emit all 21 alpha rows with rounded step, coordinate movement, live set
   roots/deltas, quadratic prediction, raw/direct live reduction, direct
   ratio and current-branch direct reduction/ratio;
7. select exactly one route without accepting, committing or publishing a
   trial.

## Gates

- parent state and failure roots reproduce exactly;
- initial gradient/direction, all set entries, radii, kernels, reductions and
  ratios are finite;
- full-step norm and reductions reproduce D7R1 exactly;
- sorted set-difference accounting is exact and changed-pair count is at most
  2,048;
- `W(h)`, `W'(h)` and `W''(h)` are positive-zero bit patterns;
- all exponents `0..20` occur once in order; alpha is exactly `ldexp(1,-e)`;
- at least one row changes a rounded coordinate and the directional model has
  positive first-order descent;
- replay/public state, commit count, parent command bytes and rollback remain
  exact.

## Frozen routes

Route precedence is normative:

1. `ACTIVE_CONSTRAINT_BRANCH_RESEARCH_REQUIRED` when any moved ladder row
   changes the pressure-active set;
2. `HORIZON_TOPOLOGY_DERIVATIVE_RECLOSURE_REQUIRED` when no active set changes
   and at least one moved row has positive predicted reduction plus
   current-branch direct ratio `>=0.1`, while the corresponding live direct
   reduction is nonpositive or its live ratio is `<0.1`;
3. `TRUST_REJECT_POLICY_RECLOSURE_REQUIRED` when neither earlier route applies
   and some exponent `e>0` has a moved live trial with positive predicted and
   direct reduction and direct ratio `>=0.1`;
4. `AL_HESSIAN_MODEL_RESEARCH_REQUIRED` otherwise, if all gates pass.

Reproduction, finiteness, set accounting, kernel closure, ladder, parent-byte
or rollback mismatch is hard FAIL with no route authority. PASS classifies
the first failing step and authorizes research for exactly one emitted route.
It grants no solver PASS, tolerance/beta/formula/policy change, state commit,
trajectory, performance, runtime, GPU/PhysX or production authority.
