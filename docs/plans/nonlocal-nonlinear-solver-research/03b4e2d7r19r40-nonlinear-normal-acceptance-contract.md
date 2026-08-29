# NSR3-B4E2D7R19R40 -- nonlinear normal-step acceptance contract

Date: `2026-08-24`

Status: `FROZEN / IMPLEMENTATION NEXT / ROLLBACK ONLY`.

Parent: `56ddb2ac`, R39 stdout SHA-256
`06812cb75177cd4666b103101f6bc9238fa20760179bc71563f194fe476db252`,
semantic `ce86c4266d224165fec4a3de32786e986f29bae95afe96dc41937f895157203b`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r40-nonlinear-normal-acceptance|v1|parent=56ddb2ac:06812cb75177cd4666b103101f6bc9238fa20760179bc71563f194fe476db252:ce86c4266d224165fec4a3de32786e986f29bae95afe96dc41937f895157203b|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;predicted-36112dde1e0b274c5b9216f4818b82977a0c80dc257478c111b0f0a9390d2d7e;dual-cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d;constraint-c567599060d7d00ffec4928009772327d081d3ef294d9030bfc5499b82e533c9;topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;theta0x3fc5cccccccccccd;particles6000;pairs340340;max-degree113|candidate=r39-hz-trajectory-73fe00202b08e17458fe0c9e495d6c9e077a93918c53d2d5d6ee1702dd8fefd4;terminal-checkpoint-9da21927cfeee2fddf448306843fe306410a2e8ee0e124aee5f0c56789ae6986;terminal-response-2dcc7b8d136864b3b7bf33d3bb617f7f7a9fb531add1e2965d6b58d4248d4bbf;terminal-mapping-167f62a1419b96ff117a9e3749f0bda8fb61b27232685b3ba1623cd2668aa3ce;linear-phi9.1332887060949100e-27;dimensionless-global-l2<=0.25|mapping=trial-position=current-position+SPACING*hz-endpoint;spacing0x3fa999999999999a;physical-global-l2<=0.0125;rollback-only|nonlinear=fresh-current-and-trial-normalized-static-sparse;constraint-psi=0.5*norm2(max(0,c));pair-membership-exact;contact=dam-box-no-increased-penetration-over-max-source-or1e-12|accept=feasibility-predicted=source-psi-minus-linear-psi;actual=source-psi-minus-trial-psi;both-positive;rho=actual/predicted;rho>=0.1;full-merit=normalized-inertia-plus-phr;precancelled-reduction-positive;precision-resolved-positive|precision=always-long-double-binary64-owned;binary128-only-if-long-unresolved-or-sign-disagrees;topology-precision-mismatch-report-only|controls=analytic-nonlinear-accept;feasibility-pass-full-merit-reject;route-precedence;parent;source;endpoint;mapping;workspace;topology;contact;divided-repeat;precision;work;rollback|routes=nonlinear-parent-rejected;nonlinear-source-rejected;nonlinear-dense-rejected;nonlinear-endpoint-rejected;nonlinear-mapping-rejected;nonlinear-workspace-rejected;nonlinear-topology-rejected;nonlinear-contact-rejected;nonlinear-feasibility-model-rejected;nonlinear-merit-precision-required;composite-normal-tangential-step-required;nonlinear-normal-step-acceptance-candidate|precedence=parent,source,dense,endpoint,mapping,workspace,topology,contact,feasibility,precision,full-merit,candidate|work=parent-r39-replays1;diagnostic-workspaces2;releases2;maximum-live2;nonlinear-trials1;divided-repeats2;long-audits1;binary128-audits<=1;new-jvp0;new-vjp0;new-hvp0;new-model0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|correction=none;candidate-commit=none;following-outer=none;tolerance=none;penalty-change=none;public-state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-nonlinear-normal-step-classification-only
```

SHA-256:
`5dbe3c14fa231eab01f176c7fd2423c08485c786cfff0075670a65c7b6b1431a`.

## Hard gates

1. Exact R39 parent bytes/semantic result, transitive R30 state/position/
   predicted/dual/constraint/topology roots and exact terminal HZ trajectory,
   checkpoint, response and mapping roots.
2. Fixed analytic controls for a nonlinear feasibility acceptance and for a
   feasibility-pass/full-merit-reject case, plus exact route precedence.
3. The captured HZ endpoint is finite, its terminal linear objective is
   exactly `9.13328870609491e-27`, and its dimensionless global L2 norm is at
   most `0.25`.
4. Trial mapping is exactly `current + SPACING*endpoint`, with finite physical
   global L2 displacement at most `0.0125`; no clipping or projection.
5. Exactly two fresh normalized static-sparse workspaces. Current reproduces
   the frozen topology/constraint; trial is evaluated at its own nonlinear
   density field with the same support binding, dual, theta and prediction.
6. Ordered pair-membership roots are exact and equal. Trial box penetration
   is at most `max(source penetration, 1e-12)`.
7. Constraint agreement uses `psi=0.5*||[c]_+||^2`, R39 terminal `psi` as
   prediction, finite positive predicted/actual reductions and `rho>=0.1`.
8. Full normalized merit reduction uses the pairwise-precancelled evaluator
   twice exactly. One binary64-owned long-double audit is mandatory;
   binary128 runs only when long double is unresolved or disagrees with the
   binary64 sign. Extended-precision topology mismatch is reported and does
   not replace binary64-owned membership.
9. A candidate requires a precision-resolved positive full-merit reduction.
   Feasibility acceptance with nonpositive resolved full merit selects
   `COMPOSITE_NORMAL_TANGENTIAL_STEP_REQUIRED`; unresolved precision selects
   `NONLINEAR_MERIT_PRECISION_REQUIRED`.
10. Work is exactly one R39 replay, two workspace builds/releases, maximum two
    live workspaces, one private nonlinear trial, two divided evaluations,
    one long-double audit, at most one binary128 audit and zero new JVP, VJP,
    HVP, model or outer work. Exact rollback and every route case are required.

Require two clean Release builds and byte-exact fresh outputs. Do not time,
apply/commit the candidate, update a dual, run contact resolution, change
penalty/trust/tolerance, execute a following outer/substep/macro/trajectory,
mutate public state or claim runtime/production authority.

Rationale:
[R40 research](../../development/nonlocal-nsr3b4e2d7r19r40-nonlinear-normal-acceptance-research-2026-08-24.md).
