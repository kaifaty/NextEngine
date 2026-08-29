# NSR3-B4E2D7R19R33 -- iterated all-inequality normal-step contract

Date: `2026-08-24`

Status: `FROZEN / REPORT ONLY`

Parent: `f535f109`, exact R32 stdout SHA-256
`243a115d66a33667d57f2fec73ca58553c57f00d83a235dcb8640d04f1d701db`,
semantic result
`2417db008424792e6b000beebf50999c2e4b71493d0cfb0d7df69ae2e14ba695`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r33-iterated-all-inequality-normal-step|v1|parent=f535f109:243a115d66a33667d57f2fec73ca58553c57f00d83a235dcb8640d04f1d701db:2417db008424792e6b000beebf50999c2e4b71493d0cfb0d7df69ae2e14ba695|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;constraint-c567599060d7d00ffec4928009772327d081d3ef294d9030bfc5499b82e533c9;topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;particles6000;pairs340340;max-degree113|model=phi(v)=0.5*norm(max(c+A*v,0))2;A=SPACING*Jc;all-rows;support-fixed|trust=global-l2;delta0.25;fixed|iteration=projected-normalized-steepest;z=project-ball(v-g/norm(g),delta);d=(z-v)/norm(z-v);alpha-in-[0,norm(z-v)];exact-r32-piecewise-line;update-v-response;max8;checkpoints1,2,4,8|terminal=fresh-jvp-v;fresh-vjp-positive;maintained-direct-response-relative<=1e-12;projected-mapping;strict-monotone;first-step-r32-exact|controls=dense-conditioned;dense-active-switch;dense-boundary-stationary;dense-feasible-stationary;parent;source;workspace;first-step;iteration;direct;work;rollback|observations=checkpoint-phi-violation-active-step;accepted-iterations;terminal-response-gradient-mapping-roots;norm-ratios;trust-use|routes=iterated-parent-rejected;iterated-source-rejected;iterated-dense-rejected;iterated-workspace-rejected;iterated-first-step-rejected;iterated-iteration-rejected;iterated-direct-rejected;iterated-work-rejected;all-inequality-projected-stationary;iterated-cauchy-no-additional-progress;iterated-all-inequality-normal-step-candidate|precedence=parent,source,dense,workspace,first-step,iteration,direct,work,classification|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-r32-once;diagnostic-workspaces1;new-pair-passes<=18;new-hvp0;new-model0;new-trial0;new-outer0|correction=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-iterated-all-inequality-normal-step-classification-only
```

SHA-256:
`7e4a70ffc56b82d73f8244a1f7096135419638842b27fd17004e7bea8e3f6aab`.

## Command and recurrence

Add `--nonlocal-al-iterated-all-inequality-normal-step`. Reproduce exact R32,
capture its frozen source and first step, then build one new read-only sparse
workspace.

Use the recurrence in the frozen identity for at most eight accepted
iterations. Projection is Euclidean onto the fixed radius-`0.25` ball. The
line interval is the complete feasible chord from current `v` to projected
target `z`; reuse the R32 binary128-ordered piecewise hinge minimizer. Record
checkpoints after accepted iterations 1, 2, 4 and 8.

## Hard gates

1. Identity, exact R32 parent stdout/semantic and all frozen source roots.
2. Four analytic dense controls from the research note.
3. One exact workspace: 6,000 particles, 340,340 pairs, maximum degree 113.
4. Iteration one exactly reproduces R32 gradient, direction, response and line
   roots and binary64 alpha/objective.
5. Every accepted chord is finite and inside the trust ball; every line has
   strict objective reduction and satisfies the inherited direct line KKT
   rule (`relative <= 1e-10` interior, derivative `<=0` at its endpoint).
6. A fresh terminal JVP of `v` and VJP of its positive residual pass. The
   maintained/direct response relative defect is `<=1e-12`; all direct
   terminal values are finite.
7. Exactly one diagnostic workspace is built and released. New work is at
   most 18 pair passes, zero HVPs/models/trials/outers.
8. Position, dual, constraint and violated-mask roots roll back exactly.

Hard-gate PASS classifies projected stationarity, no additional progress, or
an iterated candidate. Candidate requires at least two accepted iterations
and terminal objective strictly below the exact R32 first-step objective.

## Proof and stop boundary

Require two clean Release builds and one fresh process from each with
byte-exact binary and stdout equality. These are correctness/reproducibility
runs, not timing measurements.

Do not apply the step, evaluate a moved nonlinear state, change penalty,
trust/cap policy, execute another outer/substep/macro/trajectory, mutate
public schema/runtime, or claim production readiness.

Rationale is frozen in the
[D7R19R33 research](../../development/nonlocal-nsr3b4e2d7r19r33-iterated-normal-step-research-2026-08-24.md).
