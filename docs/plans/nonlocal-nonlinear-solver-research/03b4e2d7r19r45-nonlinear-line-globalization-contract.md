# NSR3-B4E2D7R19R45 -- bounded nonlinear line-globalization contract

Date: `2026-08-24`

Status: `FROZEN / IMPLEMENTATION NEXT / ROLLBACK ONLY`.

Parent: `d774b7ac`, R44 stdout SHA-256
`2fb4299081627e36752d13b4d3ca53046b3e19e8b72596bc318bb4ab85355292`,
semantic `b1485fd07b22ce57c291be9842cfcab9e3993c1ddb5aec730b732c3836c7a9a1`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r45-nonlinear-line-globalization|v1|parent=d774b7ac:2fb4299081627e36752d13b4d3ca53046b3e19e8b72596bc318bb4ab85355292:b1485fd07b22ce57c291be9842cfcab9e3993c1ddb5aec730b732c3836c7a9a1|source=r44-route-contact-feasible-common-descent-candidate;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;projected-endpoint-30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960;projected-trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01;direction-71765e02ba9965489e79d86f3399d1c6401f25fa873d04d664146e4308d45b93;direction-norm5.4585174082622525e-9|parameter=u=d/norm2d;unit-global-l2;composite=n+alpha*u;trial=x+spacing*composite;alpha>=0|domain=minimum-of-trust,skin,contact;trust=0.25-norm2n;skin=(0.5*0.006*sqrt(1-2e-12)/spacing-max-particle-norm-n)/max-particle-norm-u;contact=exact-source-inactive-face-crossing;source-active-tangent-unbounded;alpha0=nextafter(domain,0);direct-audit|ladder=largest-first;alpha-k=ldexp(alpha0,-k);k0through24;maximum25;no-outcome-fit|prediction=unit-r44-long-double-slopes;hinge-and-merit-local-reduction-ratio>=0.1|nonlinear=fresh-exact-r<=h-mask-and-workspace-per-trial;trust;certificate;coverage;all-36000-contact-faces;local-hinge-positive;source-hinge-positive;local-merit-positive;source-merit-positive|precision=selected-binary64-precancelled-local-and-source;long-double-binary64-owned-both;binary128-conditional-on-unresolved-or-disagreement;accepted-signs-resolved-positive|selection=first-largest-all-gates;selected-independent-canonical-correspondence;fresh-selected-mapping;no-stationarity-tolerance|controls=dense-domain-trust;dense-domain-skin;dense-domain-contact;dyadic-order;local-pass-source-reject;precision-route;route-precedence;parent;source;workspace;direction;domain;trial;topology;contact;feasibility;merit;precision;correspondence;work;rollback|routes=line-globalization-parent-rejected;line-globalization-source-rejected;line-globalization-workspace-rejected;line-globalization-direction-rejected;line-globalization-domain-rejected;line-globalization-topology-rejected;line-globalization-contact-rejected;finite-common-descent-not-observed;line-globalization-precision-required;composite-merit-recovery-required;line-globalization-correspondence-rejected;line-globalization-work-rejected;nonlinear-line-globalization-candidate|precedence=parent,source,workspace,direction,domain,topology,contact,local-common-descent,precision,source-merit,correspondence,work,candidate|work=parent-r44-replays1;static-index1;superset-build1;source-and-base-workspaces2;line-trials<=25;masked-trial-workspaces<=25;selected-canonical-workspaces<=1;workspace-releases-match;maximum-live4;contact-tests<=900000;divided-evaluations<=2*trials+2;long-audits<=2*trials;binary128-audits<=2*trials;fresh-selected-vjp<=1;new-hvp0;new-model0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44=unchanged;correction=none;candidate-commit=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-finite-composite-line-globalization-classification-only
```

SHA-256: `37da4b9713e015a3aabe102207ca28e1fe71835dc348041b01d5333199e07bc3`.

## Hard gates

1. Exact R44 parent bytes/semantic/route and exact source, projected endpoint,
   projected trial, owner, superset and direction roots/norm.
2. Dense domain, dyadic-order, local/source-merit, precision and every route
   control before nominal selection.
3. Normalize the exact nonzero R44 direction. Derive positive finite trust,
   skin and contact bounds exactly as frozen; use their minimum and one
   downward `nextafter`. Directly audit the resulting composite endpoint.
4. Generate at most 25 largest-first dyadic candidates. Each visited candidate
   must be finite, inside the inherited global-L2 trust ball, satisfy the R42
   displacement certificate, be covered by the unchanged source superset and
   have no new or worsened box penetration.
5. Fresh nonlinear density and complete-merit reductions from the R43 trial
   must be positive with actual/predicted ratios at least `0.1`. The candidate
   must also improve both quantities from the original source.
6. Confirm both accepted complete-merit signs in normalized long double with
   binary64-owned membership. Binary128 executes only on unresolved or
   disagreeing long double. No unresolved/disagreeing sign may select state.
7. The first largest candidate closing every prior gate is selected. Its
   superset-masked workspace must match one independently rebuilt canonical
   workspace bit-for-bit; root its composite endpoint, physical step, trial,
   exact active mask and fresh projected mapping.
8. Exact bounded work and rollback. No HVP/model/outer, correction commit,
   state mutation, policy/penalty/tolerance change or timing.

Require two clean Release builds and byte-exact outputs. A candidate is one
private finite composite-step classification only; it is not a production
solver or permission to run a following outer.

Rationale:
[R45 research](../../development/nonlocal-nsr3b4e2d7r19r45-nonlinear-line-globalization-research-2026-08-24.md).
