# NSR3-B4E2D7R19R62 -- restoration-exit transaction contract

Date: `2026-08-25`

Status: `FROZEN / IMPLEMENTATION AUTHORIZED / PRIVATE COPY-ON-WRITE ONLY`.

Parent: `fa7d48de`, R61 stdout SHA-256
`cdfe69bfccba8161708a92636a256313a3efbb27a8f6e7bb1bfa915a04047cb4`,
semantic `18794b59910842019f4d99ed468cb55ffa6b925bbb3c7c0b4ccfbe46c0377436`
and route `TOPOLOGY_OWNED_ROW_LOCAL_AUDIT_CANDIDATE`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r62-restoration-exit-transaction|v1|parent=fa7d48de:cdfe69bfccba8161708a92636a256313a3efbb27a8f6e7bb1bfa915a04047cb4:18794b59910842019f4d99ed468cb55ffa6b925bbb3c7c0b4ccfbe46c0377436:TOPOLOGY_OWNED_ROW_LOCAL_AUDIT_CANDIDATE|source=position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;witness-040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd;state-a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2;filter-source-entry-6dd5f24766c14efd390df5fe178b0ee32d6a7afa5cfc51b517762c48175ad550;filter-next-19c1311bf84aaa236a8ce4ec82359ffadb6db404a721ea804f66daabef472c19;r61-candidate-upper-78472fcb347565bdcc4be443a3cbf220cc5459fd5959a1bcc1c8bd1d428e55bf;active-82086595b4ed35abd2bf7f7cf6107359eaa9fe92be80df7aad1174dbf043bdb3|mapping=promoted-position-exact-r43;witness-dimensionless-cached-next-normal-only;spacing-times-witness-not-applied|fresh=passive-r61-capture;static-index1;workspace-at-r43;superset-anchored-r43;filtered-pairs-equal-workspace;coordinate-f-h-psi;source-to-r43-contact36000;normal-endpoint-contact-diagnostic36000|filter=gamma0.5;empty-old-filter;source-entry-inserted;candidate-acceptable-h-branch;canonical-one-entry-root;composite-al-merit-excluded|trust=next-radius0.0625;normal-radius0.03125;witness-global-l2-within-normal;all-r43-contact-intervals-contain-witness;diagnostic-normal-endpoint-contact-safe|certificate=reuse-r61-topology-owned-owner;same-r43-workspace-and-witness;one-directed-jvp;one-row-scan;candidate-positive0;candidate-active-root-exact;candidate-upper-root-exact;binary128-none|payload=copy-on-write;position-root;filter-root;radius-bits;witness-root;superset-root;workspace-topology-root;certificate-root;phase-ordinary-trqp-ready;publish-only-after-all-gates|controls=parent;source;capture;workspace;topology;coordinate;filter;contact;normal;certificate;payload;atomic-publication;work;rollback;route-precedence;valid-publish;corrupt-position-reject;corrupt-witness-reject;corrupt-filter-reject;corrupt-radius-reject;corrupt-topology-reject;corrupt-certificate-reject;gate-failure-no-publish;publish-once|routes=restoration-exit-parent-rejected;restoration-exit-source-rejected;restoration-exit-capture-rejected;restoration-exit-workspace-rejected;restoration-exit-topology-rejected;restoration-exit-coordinate-rejected;restoration-exit-filter-rejected;restoration-exit-contact-rejected;restoration-exit-normal-rejected;restoration-exit-certificate-rejected;restoration-exit-payload-rejected;restoration-exit-atomic-publication-rejected;restoration-exit-work-rejected;restoration-exit-transaction-candidate|precedence=parent,source,capture,workspace,topology,coordinate,filter,contact,normal,certificate,payload,atomic,work,candidate|work=parent-r61-replays1;new-static-index1;new-r43-superset1;new-r43-workspace1;workspace-release1;new-coordinate-evaluations1;new-contact-tests72000;new-directed-jvp1;new-row-scan1;transaction-controls9;new-binary128-0;new-density-sweeps0;new-vjp0;new-hvp0;new-model0;new-nonlinear-normal-workspace0;new-tangential-step0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58-r59-r60-r61=unchanged;r61-public-output=byte-exact;promotion=private-payload-only;r43-runtime-commit=none;witness-apply=none;cached-normal-consume=none;filter-runtime-commit=none;restoration-runtime-exit=none;switching=none;trust-runtime-update=none;multiplier-update=none;hessian-update=none;following-outer=none;tolerance=none;gamma-production-selection=none;capacity-change=none;timing=none;runtime=none;production=none|credit=one-private-restoration-exit-transaction-candidate-only
```

SHA-256: `aebba856f1088d41440d18d0948636941fa9e3fa3c21d93fc776a4e518776422`.

## Hard gates

1. Exact R61 public bytes/semantic/route. Add only a passive capture and require
   the unchanged public R61 command to remain byte-exact.
2. Exact immutable R30 source, R43 trial/contact-owner, R58 state/witness and
   R61 candidate certificate roots.
3. Rebuild one workspace and one fresh superset anchored at exact R43. Its
   filtered pairs must equal the workspace pairs; the following iteration must
   not inherit topology ownership only from the old R30 anchor.
4. Freshly evaluate R43 `f`, `h` and `psi`; reproduce R46 coordinates and the
   conservative `gamma=1/2` feasibility-branch admission against the canonical
   source entry. Composite AL merit is not a gate.
5. Repeat all 36,000 source-to-R43 contact tests. Derive every contact interval
   at R43, require the exact dimensionless witness inside it and require
   `||witness||_2 <= 0.03125 = 0.5*0.0625`. Materialize
   `R43 + SPACING*witness` only as an uncommitted contact diagnostic and repeat
   36,000 tests from R43 to that endpoint.
6. Invoke the unchanged R61 topology-owned audit on the same R43 workspace and
   exact witness. Require exact upper/active roots and zero candidate-positive
   rows; binary128 is forbidden.
7. Build a canonical payload for exact R43 position, one-entry filter, radius
   `0.0625`, cached dimensionless normal witness, fresh superset/workspace and
   R61 certificate identities, phase `ordinary-trqp-ready`. The physical
   witness endpoint is never part of committed position.
8. Copy-on-write publication occurs exactly once and only after all gates.
   Nine controls must accept the valid payload and reject corrupt position,
   witness, filter, radius, topology, certificate and pre-gate publication.
9. Exact work/lifecycle/rollback and route precedence. Any failure publishes no
   candidate and leaves every transitive root byte-exact.

Require two clean Release builds and byte-exact outputs. PASS is one private
restoration-exit transaction candidate. It does not mutate runtime state,
execute the cached normal, form a tangential step, update multipliers/Hessian,
run another outer, authorize timing or establish production readiness.

Rationale:
[R62 research](../../development/nonlocal-nsr3b4e2d7r19r62-restoration-promotion-research-2026-08-25.md).
