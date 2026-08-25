# NSR3-B4E2D7R19R49 -- restoration Hager--Zhang contract

Date: `2026-08-25`

Status: `FROZEN V1 / IMPLEMENTATION NEXT / ROLLBACK ONLY`.

Parent: `b2d160d7`, R48 stdout SHA-256
`70a66443d0a278697e3e4d8d4456f80f5f1ccb1269596d3e5ce569e87d9b90b6`,
semantic `7d83b855aeda6c32202e197c65c94f88235e6e997563f7847ddb5972d88a7e91`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r49-restoration-hz-primal-accelerator|v1|parent=b2d160d7:70a66443d0a278697e3e4d8d4456f80f5f1ccb1269596d3e5ce569e87d9b90b6:7d83b855aeda6c32202e197c65c94f88235e6e997563f7847ddb5972d88a7e91:RESTORATION_CERTIFICATE_UNRESOLVED|source=r43-position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01;r48-witness-29c5da31f537a3bbecc8826c42266cb94b25ff2ed22bb8691333d825f30a7f6d|problem=unchanged-r48;y=r43-projected-trial;c=fresh-density-inequality-at-y;A=spacing*Jc-at-y;phi(d)=0.5*norm2(max(c+A*d,0));all-rows;fixed-stable-superset|geometry=unchanged-r48;next-radius0.0625;normal-radius0.03125;global-l2;current-contact-box;zero-in-every-interval;source-half-skin-implied|generator=guarded-raw-hager-zhang;d0=0;g0=At-max-c-positive;first-steepest;history=previous-gradient-and-accepted-feasible-chord;beta=r38-long-double-hz;raw-normalize;target=current-plus-unit-raw;project-target-with-r48-euclidean-ball-box;chord=target-current;restart-invalid-beta-or-nonfinite-or-raw-nondescent-or-empty-chord-or-projected-nondescent;restart-history-becomes-accepted-steepest|max-accepted16;checkpoints1,2,4,8,16;exact-projected-stationarity-only|line=alpha-in-0,1;exact-r32-piecewise-all-row-hinge-on-feasible-chord;binary128-event-order;long-double-fold;direct-kkt;strict-objective-decrease|certificate=after-every-accepted-step;fresh-r48-directed-jvp-row-upper;zero-certified-positive-only;unchanged-rounding;early-stop-only-on-certificate;parent-r48-dual-unchanged;no-new-infeasibility-claim|selection=primal-certificate-first;exact-stationary-unresolved-second;otherwise-accelerator-candidate-only-if-terminal-strict-psi-h-maximum-upper-dominance-over-r48-and-pairpasses<=48-and-memory-streak>=2;else-r48-reference-retained|controls=r48-parent-bytes;r39-dense-hz-root;dense-ball-box-chord;dense-active-switch-restart;dense-exact-line;dense-primal-rounding;source;workspace;geometry;projection;generator;line;certificate;comparison;work;rollback;route-precedence|routes=restoration-hz-parent-rejected;restoration-hz-source-rejected;restoration-hz-workspace-rejected;restoration-hz-geometry-rejected;restoration-hz-projection-rejected;restoration-hz-generator-rejected;restoration-hz-line-rejected;restoration-hz-primal-audit-rejected;restoration-hz-comparison-rejected;restoration-hz-work-rejected;restoration-next-trqp-compatible-candidate;restoration-hz-stationary-unresolved;restoration-hz-primal-accelerator-candidate;restoration-hz-reference-retained|precedence=parent,source,workspace,geometry,projection,generator,line,primal-audit,comparison,work,certificate,stationarity,selection|work=parent-r48-replays1;static-index1;superset-build1;moved-workspace1;releases1;accepted<=16;line-jvp<=16;directed-certificate-jvp<=16;vjp<=16;pairpasses<=48;projection-calls<=32;projection-scans-bounded;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-primal-accelerator-classification-only
```

SHA-256: `e61ba2c8ca22f43c09ca7ad6a8949af6437b91f13b1f45b5b5c17926dbae62b8`.

## Hard gates

1. Reproduce exact R48 bytes, semantic, route, source/operator/contact roots and
   unresolved parent result. The parent dual remains unchanged and cannot be
   relabelled.
2. Rebuild exactly one moved workspace and the exact R48 contact-box/normal-ball
   geometry. Reuse the audited R48 projector without metric changes.
3. Start at zero. Use the frozen raw Hager--Zhang coefficient only as a memory
   proposal. Guard strict raw descent and strict projected-chord descent; any
   invalid/nonfinite proposal restarts to projected steepest.
4. Both endpoints of every line belong to the exact convex feasible set. Audit
   every projection and chord; a projection failure is not an unresolved solver
   outcome.
5. Use only the exact piecewise all-row hinge minimum on `alpha in [0,1]`.
   Binary128 event order, long-double folds, direct KKT and strict decrease are
   mandatory.
6. Run a fresh directed JVP after every accepted step. Compatibility requires
   zero positive row upper bounds under the unchanged R48 forward enclosure.
   Maintained residuals and exact-line stationarity cannot authorize it.
7. Stop at the first primal certificate or at 16 accepted steps. Exact projected
   stationarity is separately unresolved; no fitted tolerance or retry exists.
8. Without a primal certificate, accelerator selection requires strict terminal
   dominance over R48 in `psi`, `h` and maximum row upper, no more than 48 pair
   passes and a memory streak of at least two. This selects research only.
9. Dense HZ, active-switch/restart, ball-box chord, exact-line, rounding and all
   route-precedence controls must pass.
10. Exact work, release and rollback. No witness/state/filter commit,
    restoration exit, trust update, following outer or timing.

Require two clean Release builds and byte-exact outputs. A primal PASS is one
private next-TRQP compatibility candidate only, not a completed restoration
transaction, runtime permission or production evidence.

Rationale:
[R49 research](../../development/nonlocal-nsr3b4e2d7r19r49-restoration-hz-research-2026-08-25.md).
