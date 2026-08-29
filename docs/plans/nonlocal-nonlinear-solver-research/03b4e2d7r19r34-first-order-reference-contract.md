# NSR3-B4E2D7R19R34 -- 32-step first-order reference contract

Date: `2026-08-24`

Status: `FROZEN / REPORT ONLY`

Parent: `827e6092`, exact R33 stdout SHA-256
`6bab6bfb7e203ca45b387bbad04eefbf62aa03fdf187761c70f46d5fd7bc5464`,
semantic result
`61f21b041257c92222d61b4c454e44aa69c30b3da3e45d8ecc70f952db3e9ec0`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r34-first-order-reference-extension|v1|parent=827e6092:6bab6bfb7e203ca45b387bbad04eefbf62aa03fdf187761c70f46d5fd7bc5464:61f21b041257c92222d61b4c454e44aa69c30b3da3e45d8ecc70f952db3e9ec0|source=checkpoint-e949afa7039e76b79deba5cacbbc3d87b29b68bb1a90324e0984b443b0183e55;response-7dd519932782f9177be354418c9ecaaa1fed3baac1e8255305675da7cb59dff3;gradient-295d66fc0b9b5b74e06200db48b11792005720477ad5291b2f6ea1147570679f;projected-mapping-11bade99d29986e5dc2a945bf00ffae3a7b5a69eb334143e69954dff570e1939;particles6000;pairs340340;max-degree113|model=phi(v)=0.5*norm(max(c+A*v,0))2;A=SPACING*Jc;all-rows;support-fixed|trust=global-l2;delta0.25;fixed|continuation=exact-r33-iterate-response;prefix-fresh-jvp;iterations9-through32;projected-normalized-steepest;exact-r32-piecewise-line;checkpoints16,32|slope=phi16/phi8;phi32/phi16;violation16/violation8;violation32/violation16;geometric-per-step;report-only|terminal=fresh-jvp-v;fresh-vjp-positive;maintained-direct-response-relative<=1e-12;projected-mapping;strict-monotone|controls=dense-conditioned-slope;dense-early-stationary;dense-boundary-stationary;parent;source;workspace;prefix;iteration;direct;work;rollback|observations=checkpoint-phi-violation-active-step;accepted-total;terminal-response-gradient-mapping-roots;norm-ratios;trust-use;block-contractions|routes=extended-parent-rejected;extended-source-rejected;extended-dense-rejected;extended-workspace-rejected;extended-prefix-rejected;extended-iteration-rejected;extended-direct-rejected;extended-work-rejected;all-inequality-projected-stationary;extended-first-order-saturated;extended-first-order-reference-candidate|precedence=parent,source,dense,workspace,prefix,iteration,direct,work,classification|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-r33-once;diagnostic-workspaces1;new-pair-passes<=51;new-hvp0;new-model0;new-trial0;new-outer0|correction=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-32-step-first-order-reference-classification-only
```

SHA-256:
`f5350515798dfe3fd14cd533a2dae25516c7432d918fa7ef9863903f89bedbd2`.

## Command and hard gates

Add `--nonlocal-al-first-order-reference-extension`. Reproduce exact R33 and
capture its iteration-eight state. Build one new read-only sparse workspace.

1. Identity, R33 bytes/semantic and frozen terminal/checkpoint roots pass.
2. Three analytic dense continuation controls pass.
3. Source workspace has 6,000 particles, 340,340 pairs, maximum degree 113.
4. A fresh prefix JVP of captured `v8` exactly reproduces the frozen R33
   terminal response root; captured maintained response agrees directly.
5. Continue only the unchanged R33 recurrence to total iteration 32. Every
   accepted chord remains in the radius-`0.25` ball and every line passes
   strict decrease plus inherited direct line KKT.
6. Record exact checkpoints 16 and 32 and block/geometric contractions. Do not
   gate on their observed magnitude.
7. Fresh terminal JVP/VJP pass; maintained/direct response relative defect is
   `<=1e-12`; projected mapping and all report values are finite.
8. Exactly one diagnostic workspace is built/released; new pair passes are at
   most 51; HVP/model/trial/outer counts are zero.
9. Position, dual, constraint and violated-mask roots roll back exactly.

Hard-gate PASS classifies projected stationarity, binary64 saturation or a
strictly improving 32-step reference.

## Proof and stop boundary

Require two clean Release builds and one fresh process from each with
byte-exact binary/stdout equality. These are not timing runs.

Do not apply the iterate, evaluate nonlinear moved state, change
penalty/trust/cap policy, execute another outer/substep/macro/trajectory,
mutate public schema/runtime or claim production readiness.

Rationale is frozen in the
[D7R19R34 research](../../development/nonlocal-nsr3b4e2d7r19r34-first-order-reference-research-2026-08-24.md).
