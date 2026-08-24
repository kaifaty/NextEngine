# NSR3-B4E2D7R19R47 -- filter-compatibility contract

Date: `2026-08-25`

Status: `PASS / RESTORATION_COMPATIBILITY_REQUIRED / CLOSED /
ROLLBACK ONLY`.

Parent: `3f42e2d2`, R46 stdout SHA-256
`ecbdc13b1cf1ca889fed1f0ce98f4f4613c06eb222d2e132271cd23cdddaf4e3`,
semantic `8b3390babbb896cef73a487bbf2512abc8f2031ff6ad7299d506928ac0d59cf0`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r47-filter-compatibility|v2|parent=3f42e2d2:ecbdc13b1cf1ca889fed1f0ce98f4f4613c06eb222d2e132271cd23cdddaf4e3:8b3390babbb896cef73a487bbf2512abc8f2031ff6ad7299d506928ac0d59cf0:FILTER_FEASIBILITY_STEP_CANDIDATE|source=r43-position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;endpoint-30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960;trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187|linear=direct-captured-linear-psi-and-h-authoritative;source-minus-predicted-diagnostic;absolute-error<=gamma4*(abs-source+abs-predicted+abs-direct);direct-h=sqrt(2*direct-psi)-binary64-exact;finite-nonnegative;exact-zero-required-for-strict-linearized-inequality-compatibility;no-fitted-tolerance|geometry=endpoint-global-l2<=0.25;source-active-contact-tangent;no-new-contact-evaluation|filter=r46-admitted-exact;admission-necessary-not-sufficient|restoration=ordinary-theta-step-requires-compatible-normal;filter-acceptable-incompatible-step-selects-restoration;restoration-exit-requires-filter-acceptable-next-trqp-compatible;next-state-compatibility-not-claimed|controls=two-subtraction-forward-bound;zero-linear-compatible;positive-linear-restoration;filter-pass-compatibility-fail;route-precedence;parent;source;linear;trust;contact;filter;work;rollback|routes=filter-compatibility-parent-rejected;filter-compatibility-source-rejected;filter-linear-metric-rejected;filter-trust-rejected;filter-contact-rejected;filter-admission-rejected;filter-compatibility-work-rejected;restoration-compatibility-required;filter-normal-step-compatible-candidate|precedence=parent,source,linear,trust,contact,filter,work,compatibility|work=parent-r46-replays1;new-workspaces0;new-jvp0;new-vjp0;new-hvp0;new-model0;new-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46=unchanged;correction=none;candidate-commit=none;filter-runtime-commit=none;restoration-execution=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-filter-compatibility-classification-only
```

SHA-256: `09b1ce5efc452cbbd4220f6bb09f720d23fdb777b65de7d796e9c64d0e41799f`.

## V1 linear-metric reclosure

V1 stopped at `FILTER_LINEAR_METRIC_REJECTED` because it required
`fl(source-fl(source-linear))` to reproduce `linear` bit-for-bit. Binary64
subtraction is not invertible. V2 keeps the directly evaluated R43 linear
metric authoritative and bounds the reconstructed diagnostic by
`gamma(4)*(|source|+|predicted|+|direct|)`. Direct
`h=sqrt(2*direct_psi)` remains bit-exact. No physical, filter, compatibility,
geometry, route or authority rule changed. The v1 attempt receives no
scientific credit; see the
[v1 diagnostic](../../development/nonlocal-nsr3b4e2d7r19r47-filter-compatibility-v1-diagnostic-2026-08-25.md).

## Hard gates

1. Exact R46 parent bytes, semantic and
   `FILTER_FEASIBILITY_STEP_CANDIDATE`, plus exact immutable R43 source,
   endpoint, trial and contact-owner roots.
2. Capture the exact R43 source, linear and trial hinge metrics. The directly
   evaluated linear metric is authoritative. Reconstruct
   `source_psi-predicted_reduction` diagnostically and require its absolute
   difference to fit the frozen `gamma(4)` two-subtraction forward bound.
   Require direct `h=sqrt(2*direct_psi)` bit-exactly.
3. Strict linearized inequality compatibility requires finite exact
   `linear_psi==0` and `linear_h==0`. A positive value selects restoration;
   no observed tolerance may be introduced.
4. Recheck the inherited global-L2 endpoint trust bound and every
   source-active contact-tangent component from immutable captured state. No
   new workspace, contact traversal or derivative pass is admitted.
5. R46 filter admission is necessary but cannot override failed compatibility.
   A restoration exit is not claimed without a compatible next-state TRQP.
6. Dense controls must distinguish exact-zero compatibility, positive-residual
   restoration and filter-pass/compatibility-fail routing.
7. Exact zero new scientific work and rollback. No state/filter commit,
   restoration execution, switching/trust update, next outer or timing.

Require two clean Release builds and byte-exact outputs. A result is one
private compatibility classification only; it is not a restoration solver,
accepted transaction, runtime permission or production evidence.

Rationale:
[R47 research](../../development/nonlocal-nsr3b4e2d7r19r47-filter-compatibility-research-2026-08-25.md).
