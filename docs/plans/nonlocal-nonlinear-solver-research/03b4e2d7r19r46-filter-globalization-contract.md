# NSR3-B4E2D7R19R46 -- filter-globalization contract

Date: `2026-08-25`

Status: `FROZEN / IDENTITY SERIALIZATION RECLOSED / IMPLEMENTATION NEXT /
ROLLBACK ONLY`.

Parent: `c62bda39`, R45 stdout SHA-256
`2ab62170eb816eee7f71ee330e5303cdb6852a241cd5e7c54160fe0914cbabc7`,
semantic `55c04d4294290e212a48911ac717df10be34e39013751ed387b184df479a3773`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r46-filter-globalization|v1|parent=c62bda39:2ab62170eb816eee7f71ee330e5303cdb6852a241cd5e7c54160fe0914cbabc7:55c04d4294290e212a48911ac717df10be34e39013751ed387b184df479a3773:COMPOSITE_MERIT_RECOVERY_REQUIRED|source=r43-position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01|coordinates=f=0.5*norm2(y-yhat);h=norm2(max(c,0));psi=0.5*h*h;phr-excluded-from-f;f-lower-bound0;fresh-exact-support|filter=initial-empty;admission-against-initial-union-source;envelope=htrial<=(1-gamma)*hsource-or-objective-reduction>=gamma*hsource;gamma-k=2^-k;k1through24;strongest-first;all24-required;strongest-h-branch-required;resolution-slack>1024*epsilon*max(hsource,htrial,min-normal)|model=inherited-r43-actual-and-predicted-psi-reductions-positive;rho>=0.1|update=feasibility-step-adds-source-entry-to-hypothetical-next-filter;canonical-binary64-entry-order;candidate-not-added;no-state-commit|nonlinear=source-and-r43-trial-exact-workspaces;stable-superset-coverage;canonical-correspondence;all-36000-contact-faces|controls=coordinate-separation;pure-inertia-lower-bound;h-branch;f-branch;strongest-margin-reject;gamma-order;empty-current-ownership;filter-update-root;route-precedence;parent;source;workspace;model;topology;contact;work;rollback|routes=filter-parent-rejected;filter-source-rejected;filter-coordinate-rejected;filter-model-rejected;filter-topology-rejected;filter-contact-rejected;filter-restoration-required;filter-update-rejected;filter-work-rejected;filter-feasibility-step-candidate|precedence=parent,source,coordinate,model,topology,contact,envelope,update,work,candidate|work=parent-r45-replays1;static-index1;superset-build1;filters2;workspaces2;releases2;precancelled-divided-repeats2;contact-tests36000;gamma-tests24;new-jvp0;new-vjp0;new-hvp0;new-model0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45=unchanged;correction=none;candidate-commit=none;filter-runtime-commit=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-filter-feasibility-admission-classification-only
```

SHA-256: `afa98b317f26e6cc6a11d26253d2d27723d4e77f7f44b89ff3eabd9100214ee4`.

## Identity serialization reclosure

The first frozen annotation `316ee56d...d14` was computed by piping the
one-line identity through `sha256sum`; that operation included the line-feed
terminator. The executable identity projection, like every preceding probe,
is the 2,406-byte raw string without that terminator. Its correct SHA-256 is
`afa98b31...ee4`.

The identity text, parent, coordinates, margins, routes, work limits and every
hard gate are byte-for-byte unchanged. The first implementation attempt
therefore closed `FAIL / FILTER_COORDINATE_REJECTED` solely because the dense
identity control compared the raw-string hash with the newline-owned
annotation. It receives no filter-admission evidence credit. This reclosure
corrects only serialization ownership before a new run.

## Hard gates

1. Exact R45 parent bytes, semantic and
   `COMPOSITE_MERIT_RECOVERY_REQUIRED`, plus exact immutable R43 source,
   trial, owner and superset roots.
2. Fresh source/trial exact-support workspaces must match the unchanged R42
   superset masks and independently canonical pair membership.
3. Compute `f` only as normalized inertia and `h` only as the L2 norm of
   positive density inequalities. Require finite `f>=0`, finite `h>=0`, exact
   `psi=0.5*h*h` correspondence and two byte-exact pre-cancelled objective
   differences.
4. Test `gamma=2^-k`, `k=1..24`, strongest first, against the source entry.
   Every envelope must accept and `gamma=1/2` must accept specifically by
   feasibility. Its feasibility slack must exceed the frozen binary64
   resolution guard.
5. Preserve the exact R43 positive predicted/actual hinge reductions and
   model ratio `>=0.1`; a filter must not override a failed model.
6. Recheck exact superset coverage, canonical correspondence and all 36,000
   contact faces. No new or worsened penetration is allowed.
7. Root the initial empty filter, the source entry and the hypothetical next
   filter. A feasibility admission adds only the current source entry; it
   never commits the candidate or executes a following solve.
8. Exact bounded work and rollback. No new derivative/model/outer work,
   policy/tolerance/penalty change, state mutation or timing.

Require two clean Release builds and byte-exact outputs. A candidate is one
private filter-admission classification only; it is not a complete filter-SQP
algorithm, runtime permission or production evidence.

Rationale:
[R46 research](../../development/nonlocal-nsr3b4e2d7r19r46-filter-globalization-research-2026-08-25.md).
