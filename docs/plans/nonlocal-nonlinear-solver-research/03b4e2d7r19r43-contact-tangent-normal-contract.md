# NSR3-B4E2D7R19R43 -- contact-tangent normal-step contract

Date: `2026-08-24`

Status: `FROZEN / IMPLEMENTATION NEXT / ROLLBACK ONLY`.

Parent: `11c970e4`, R42 stdout SHA-256
`f7ae3a0c60efce0a78d3d1821e51b12c97f2fb8557724e5f3082bf493bb45106`,
semantic `fed11eefb017fce471b41fa25a20f5db9ea93e563b90fe5d02123380d52784d2`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r43-contact-tangent-normal-step|v1|parent=11c970e4:f7ae3a0c60efce0a78d3d1821e51b12c97f2fb8557724e5f3082bf493bb45106:fed11eefb017fce471b41fa25a20f5db9ea93e563b90fe5d02123380d52784d2|source=r42-route-stable-superset-relinearization-candidate;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;unprojected-trial-60e793aff919fada6b49c1b98be7ee5cc4a2c74a4254fa762cdece5700e1e804;endpoint-a73caf5baa6708ca89693077bad943dcdbf1f5069d50270ccc0f6b184a102e3a;physical-step-424c84b72409f5b6665874f17afc18cf2784ae84593e30be41eaeec0d25e514b;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01;contact-root-1df2083d4096522f329cdb1ff2a70c5da448ead0d40cda2cdb02e520c88f1ef7|projection=source-strict-positive-box-penetration-owns-tangent-halfspace;lower-active-dimensionless-component>=0;upper-active-component<=0;inadmissible-to-positive-zero;other-bits-retained;physical=spacing*projected-dimensionless;euclidean-norm-nonincreasing|contact=all-36000-source-projected-trial-face-tests;new0;no-face-penetration-increase;projection-owner-root;projected-ledger-root|coverage=one-source-anchored-r42-superset;exact-projected-r<=h-mask;certificate;canonical-correspondence;physical-h-unchanged|feasibility=source-linear-response-of-projected-step;predicted=source-psi-minus-linear-psi;actual=source-psi-minus-projected-trial-psi;both-positive;rho=actual/predicted;rho>=0.1|relinearization=fresh-projected-trial-radii-weight-first-constraint;zero-origin-mapping=AtrialT*max(cTrial,0);finite-root;no-stationarity-tolerance|merit=complete-normalized-inertia-plus-phr;two-precancelled-divided-evaluations;positive-reduction-required;always-long-double-binary64-owned;binary128-only-if-long-unresolved-or-sign-disagrees|controls=scalar-lower-upper-free-projection;multi-face;route-precedence;parent;source;projection;contact;superset;coverage;correspondence;feasibility;relinearization;divided-repeat;precision;work;rollback|routes=contact-tangent-parent-rejected;contact-tangent-source-rejected;contact-tangent-projection-rejected;contact-tangent-contact-rejected;contact-tangent-superset-rejected;contact-tangent-relinearization-rejected;contact-tangent-feasibility-rejected;contact-tangent-merit-precision-required;tangential-merit-step-required;contact-tangent-normal-step-candidate|precedence=parent,source,projection,contact,superset-and-relinearization,feasibility,precision,full-merit,candidate|work=parent-r42-replays1;static-index1;superset-build1;diagnostic-workspaces2;releases2;maximum-live2;filters2;source-jvp1;trial-vjp1;contact-tests36000;divided-repeats2;long-audits1;binary128-audits<=1;new-hvp0;new-model0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42=unchanged;unprojected-trial=unchanged;correction=none;candidate-commit=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-contact-tangent-normal-step-classification-only
```

SHA-256: `a6623368209ebc9cc08090ff40584fe0e16cc68d852ef2c7c1f1322f74b944fa`.

## Hard gates

1. Exact R42 parent bytes/semantic/route and exact R40 source, unprojected
   trial, endpoint, physical step, R42 superset and R41 contact roots.
2. Dense lower/upper/free and multi-face projection controls plus every route
   in frozen precedence.
3. Exact source-active face ownership; only inward endpoint components become
   positive zero. Projection is idempotent, finite and norm-nonincreasing;
   roots cover owner mask, projected endpoint/physical step/trial.
4. All 36,000 source/projected-trial faces have no new penetration and no
   face-wise increase. No contact epsilon or position clamp.
5. The unchanged source-anchored R42 superset passes its certificate, covers
   the projected active graph and reproduces an independent canonical
   projected workspace bit-for-bit. Physical `H` is unchanged.
6. Fresh source JVP gives projected linear hinge prediction; fresh nonlinear
   trial gives positive actual reduction and inherited `rho>=0.1`. Fresh trial
   VJP roots a finite zero-origin projected mapping without selecting a floor.
7. Two complete normalized precancelled divided evaluations are exact.
   Long double always audits the merit sign; binary128 is conditional only on
   unresolved/disagreeing long double. Positive resolved merit is required for
   the candidate; otherwise select the frozen tangential-merit route.
8. Exact frozen work and rollback; no HVP/model/outer, correction commit,
   state mutation or timing.

Require two clean Release builds and byte-exact outputs. Do not modify prior
evidence, accept/apply a step, tune coefficients/support/trust, run a following
outer or claim runtime/production authority.

Rationale:
[R43 research](../../development/nonlocal-nsr3b4e2d7r19r43-contact-tangent-normal-research-2026-08-24.md).
