# NSR3-B4E2D7R19R32 -- all-inequality Cauchy normal-step contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r32-all-inequality-cauchy-normal-step|v1|parent=546035f6:67adbee532f615b9014e93cb44fccd053bc887c54f5d2ffcd76a19f2c771f3cb:d57f2712573d8e8dda52bdf44a11f40e2a6cbca03b1ac30e07c13b20af8aaa98|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;constraint-c567599060d7d00ffec4928009772327d081d3ef294d9030bfc5499b82e533c9;topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;violated-bd06f7b7c5e1f1943e5c269bbfb192229504091381b8f20f0f4988eecb021c70;particles6000;pairs340340;max-degree113|model=phi(v)=0.5*norm(max(c+A*v,0))2;A=SPACING*Jc;all-rows;support-fixed|trust=global-l2;delta0.25;inherited-inner-initial-radius0.25*SPACING|direction=g=At*max(c,0);d=-g/norm(g);unit-l2|line=alpha-in-[0,delta];events=-c/(A*d);binary128-order;stable-row-tie;fixed-long-double-fold;exact-piecewise-quadratic-minimum|direct=fresh-gradient-response;phi0;phialpha;directional-derivative;interior-kkt-relative<=1e-10;boundary-derivative<=0;positive-reduction|controls=dense-interior;dense-boundary;dense-inactive-entry;dense-simultaneous-event;parent;source;workspace;gradient;event-order;direct-kkt;work;rollback|observations=gradient-root;direction-root;response-root;events-root;alpha;trust-active;phi-reduction;violation-norm-ratio;initial-final-active;entered-left;selected-inactive-positive;step-rms-max|routes=cauchy-parent-rejected;cauchy-source-rejected;cauchy-dense-rejected;cauchy-workspace-rejected;cauchy-gradient-rejected;cauchy-line-rejected;cauchy-kkt-rejected;cauchy-work-rejected;all-inequality-linearized-stationary;all-inequality-cauchy-normal-step-candidate|precedence=parent,source,dense,workspace,gradient,line,kkt,work,classification|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-r31-once;diagnostic-workspaces1;new-pair-passes2;new-hvp0;new-model0;new-trial0;new-outer0|correction=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-all-inequality-cauchy-normal-step-classification-only
```

Identity SHA-256:
`5e69a197102f8b4e1d86d68ec49f1da55f03bf754fb96d25d056a8edd5588b1d`.

## Required command

Add `--nonlocal-al-all-inequality-cauchy-normal-step`. Reproduce exact R31,
rebuild the exact R29 workspace, pass four dense event-sweep controls, compute
`g`, unit `d`, `A d` and the exact trust-bounded piecewise line minimum, then
emit direct hinge/KKT/work/rollback evidence.

## Hard failures

Identity/parent/source/workspace/topology, any dense control, nonfinite or
nonunit direction, invalid event order/tie handling, nonpositive reduction for
nonzero gradient, failed direct KKT, work/rollback or two-build/process
mismatch is hard FAIL. Exact positive-violation zero-gradient stationarity is
a successful diagnostic route, not production convergence.

## Authority boundary

Read-only private diagnostic only. No correction, nonlinear moved state,
floor classification, following outer, policy change, timing, state/public/
world mutation, runtime or production authority.

Research basis:
[D7R19R32 research](../../development/nonlocal-nsr3b4e2d7r19r32-all-inequality-cauchy-research-2026-08-24.md).
