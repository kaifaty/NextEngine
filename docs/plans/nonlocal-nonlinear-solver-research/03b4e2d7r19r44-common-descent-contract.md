# NSR3-B4E2D7R19R44 -- contact-feasible common-descent contract

Date: `2026-08-24`

Status: `FROZEN / IMPLEMENTATION NEXT / ROLLBACK ONLY`.

Parent: `a47d7ea7`, R43 stdout SHA-256
`565c70d251612bac17c0b4d20b5ae4f284f9d64c2edd34871db640b1d58bc993`,
semantic `e1f019dae6e71e40633e56c79ee60e8e81dc3b146fdf55d75542298737f9eea6`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r44-contact-feasible-common-descent|v1|parent=a47d7ea7:565c70d251612bac17c0b4d20b5ae4f284f9d64c2edd34871db640b1d58bc993:e1f019dae6e71e40633e56c79ee60e8e81dc3b146fdf55d75542298737f9eea6|source=r43-route-tangential-merit-step-required;projected-endpoint-30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960;projected-trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01;trial-psi6.8540208964482797e-16;trial-active464;trial-mapping9.7690943252330519e-9|gradient=complete-normalized-inner-at-projected-trial;dimensionless=spacing*physical-gradient;raw-negative-gradient;source-contact-cone-projection;inward-to-positive-zero;other-bits-retained|operator=fresh-projected-trial-A=spacing*Jc;image=A*dcontact;positive-trial-rows|slopes=long-double-dot;merit=gT*dcontact;feasibility=max(cTrial,0)T*A*dcontact;strict-sign-no-fitted-tolerance;normalized-slopes;direction-and-image-roots|selection=both-negative-common-descent;merit-negative-feasibility-nonnegative-nullspace-required;merit-nonnegative-decomposition-required|controls=analytic-orthant-gradient;route-precedence;parent;source;workspace;gradient;projection;operator;slopes;work;rollback|routes=common-descent-parent-rejected;common-descent-source-rejected;common-descent-workspace-rejected;common-descent-gradient-rejected;common-descent-contact-rejected;common-descent-operator-rejected;common-descent-work-rejected;contact-feasible-common-descent-candidate;density-nullspace-projection-required;composite-decomposition-required|precedence=parent,source,workspace,gradient,contact,operator,work,merit-sign,feasibility-sign|work=parent-r43-replays1;static-index1;superset-build1;trial-workspace1;release1;trial-jvp1;long-double-dot2;new-vjp0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|line-search=none;moved-state=none;nullspace-solve=none;correction=none;candidate-commit=none;tolerance=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-contact-feasible-common-descent-classification-only
```

SHA-256: `7e10698ee75568845fe432f83d9a31e01f516a2f131840e83ee15ae8b48a0aba`.

## Hard gates

1. Exact R43 parent bytes/semantic/route and projected endpoint/trial, contact
   owner, R42 superset and trial residual facts.
2. Dense orthant-projected gradient and every frozen route control.
3. One exact projected-trial workspace reproduces stable-superset coverage,
   constraint/residual and complete normalized gradient. Map gradient through
   exact `SPACING` and root raw/contact-projected directions.
4. Contact direction satisfies every source-active tangent sign; projection
   is idempotent and norm-nonincreasing. Direction must be finite and nonzero.
5. One fresh pair-once JVP roots `A*d`. Long-double accumulations own complete
   merit and hinge directional signs; strict sign only, no fitted epsilon.
6. Exact frozen work and rollback. No line/null-space solve, moved state,
   precision audit, HVP/model/outer, correction commit or timing.

Require two clean Release builds and byte-exact outputs. Do not infer a finite
accepted step from first-order descent or claim runtime/production authority.

Rationale:
[R44 research](../../development/nonlocal-nsr3b4e2d7r19r44-common-descent-research-2026-08-24.md).
