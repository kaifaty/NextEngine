# NSR3-B4E2D7R19R48 -- restoration-certificate contract

Date: `2026-08-25`

Status: `FROZEN V2 / GEOMETRY RECLOSURE / IMPLEMENTATION NEXT /
ROLLBACK ONLY`.

Parent: `33e6ebfd`, R47 stdout SHA-256
`dac0ffd871c927cb62253f64a078866d9515faefda59e48a54d923089dbc371d`,
semantic `cbc082189cf1813be74123be818fa55649c7b2feac07d87f31626063b046286c`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r48-restoration-certificate|v2|parent=33e6ebfd:dac0ffd871c927cb62253f64a078866d9515faefda59e48a54d923089dbc371d:cbc082189cf1813be74123be818fa55649c7b2feac07d87f31626063b046286c:RESTORATION_COMPATIBILITY_REQUIRED|source=r43-position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;endpoint-30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960;trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01|problem=y=r43-projected-trial;c=fresh-density-inequality-at-y;A=spacing*Jc-at-y;phi(d)=0.5*norm2(max(c+A*d,0));all-rows;fixed-stable-superset|geometry=next-trust-radius0.0625;normal-reserve-half;normal-radius0.03125;global-l2;current-penetrating-face-nonworsening;current-interior-first-face-bounds;zero-in-every-interval;source-half-skin-implied|generator=malitsky-pock-pdal;d0=0;lambda1=max(c,0);tau0=1;beta1;mu0.5;delta0.99;maximum-growth;main-cap128;per-main-backtrack-cap16;total-extra-backtrack-cap256;early-primal-certificate|prox=euclidean-ball-intersect-box;monotone-multiplier;bracket-and-inward-endpoint;maximum128-bisections;direct-feasibility-bracket-kkt-audit|dual=lambda-nonnegative;D=lambdaTc-0.5*norm2lambda-supportC(-Atlambda);support-upper=lagrangian-eta-separable-box-max;outward-long-double;binary128-if-sign-unresolved-or-disagrees;positive-lower-bound-only|certificate=primal-row-upper=c+A*d+gamma-directed-absolute<=0-all-rows;or-dual-lower>0;mutually-exclusive;positive-hinge-stationarity-insufficient;otherwise-unresolved|controls=dense-feasible;dense-infeasible;ball-projection;box-projection;support-upper;weak-duality;rounding;route-precedence;parent;source;workspace;geometry;projection;generator;primal;dual;work;rollback|routes=restoration-certificate-parent-rejected;restoration-certificate-source-rejected;restoration-certificate-workspace-rejected;restoration-certificate-geometry-rejected;restoration-certificate-projection-rejected;restoration-certificate-generator-rejected;restoration-certificate-primal-audit-rejected;restoration-certificate-dual-audit-rejected;restoration-certificate-work-rejected;restoration-next-trqp-compatible-candidate;restoration-radius-infeasible-certificate;restoration-certificate-unresolved|precedence=parent,source,workspace,geometry,projection,generator,primal-audit,dual-audit,work,classification|work=parent-r47-replays1;static-index1;superset-build1;moved-workspace1;releases1;jvp<=128;vjp<=385;projection-scans-bounded;support-audits<=8;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-next-trqp-restoration-certificate-only
```

SHA-256: `b20c6c6b530bd3078de7707e87c8deacf8eae0dab581584cf30421e5a1d52332`.

## V1 geometry reclosure

V1 stopped at `RESTORATION_CERTIFICATE_GEOMETRY_REJECTED` before executing
PDAL. It assumed a dimensionless half-skin radius `0.3`; the exact inherited
constants produce `0.059999999999940004`. V2 uses dyadic next/normal radii
`1/16` and `1/32`. Together with the R43 displacement, the normal witness ball
fits the frozen source-anchored skin without changing the operator, contact
rules, generator or certificate. V1 receives no solver credit; see the
[v1 diagnostic](../../development/nonlocal-nsr3b4e2d7r19r48-restoration-certificate-v1-diagnostic-2026-08-25.md).

## Hard gates

1. Reproduce exact R47 bytes/semantic/route and immutable R43 source,
   endpoint, trial, contact-owner and R42 superset roots.
2. Rebuild exactly one normalized workspace at the R43 projected trial and
   prove canonical correspondence to the state-local mask of the inherited
   stable superset. The fresh nonlinear hinge is distinct from R47's inherited
   source-linear metric.
3. Freeze next radius `0.0625` and normal radius `0.03125`. Rebuild current contact
   intervals, require zero in every interval and prove the source-anchored skin
   bound from the R43 displacement plus normal radius.
4. Audit the ball-box projection on every use: finite, component-feasible,
   inward trust-feasible and bracket/KKT-consistent. A projection failure cannot
   be reclassified as an unresolved solver result.
5. Run only the frozen PDAL line search. No nominal-outcome tuning, algorithm
   tournament, fitted stopping tolerance or unbounded retry is admitted.
6. A primal route requires independent rowwise directed-JVP forward upper
   bounds at the selected witness, exact trust/contact feasibility and zero
   positive certified upper bounds. Generator residual alone is insufficient.
7. A dual infeasibility route requires `lambda>=0` and a strictly positive
   lower bound after an outward support upper bound and precision audit. It
   proves infeasibility only for the frozen local model and radius.
8. Primal and dual certificates are mutually exclusive. If neither closes,
   report `RESTORATION_CERTIFICATE_UNRESOLVED` without changing policy.
9. Dense feasible/infeasible, projection, support, weak-duality, rounding and
   all route-precedence controls must pass.
10. Exact work, release and rollback. No witness/state/filter commit,
    restoration exit, trust update, following outer or timing.

Require two clean Release builds and byte-exact outputs. Any PASS is one
private next-TRQP compatibility classification only, not a completed
restoration transaction, runtime permission or production evidence.

Rationale:
[R48 research](../../development/nonlocal-nsr3b4e2d7r19r48-restoration-certificate-research-2026-08-25.md).
