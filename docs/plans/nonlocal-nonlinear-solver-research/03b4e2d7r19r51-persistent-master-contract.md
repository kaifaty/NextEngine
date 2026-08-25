# NSR3-B4E2D7R19R51 -- persistent-master contract

Date: `2026-08-25`

Status: `FROZEN V3 / IDENTITY ENCODING RECLOSURE / IMPLEMENTATION NEXT / ROLLBACK ONLY`.

Parent: `717ffe6c`, R50 stdout SHA-256
`1ee396be61be90534573b8c476256d80b46ee803584617a6ab34900b8727b56d`,
semantic `a97b34d19b19b6d39725de8fdaf9ce38db924b7d39bfc823e07ea9bee86c7d8d`.

## Invalid v2 identity retained

```text
nextengine.nonlocal.nsr3b4e2d7r19r51-persistent-active-face-master|v2|parent=717ffe6c:1ee396be61be90534573b8c476256d80b46ee803584617a6ab34900b8727b56d:a97b34d19b19b6d39725de8fdaf9ce38db924b7d39bfc823e07ea9bee86c7d8d:ACTIVE_FACE_HILDRETH_CLOSURE_CANDIDATE|source=r50-witness-e2ccfb4281b2b598b0a9864a8c9bb92f5c2512e018501147ca82b39b14b67592;cache-root-31a6713866e951e716b8fd77e40ae532c3d9d281d3b40b0d16cd8039c2c5112e;outer-root-04a91d0379f71f9c6dfb711bf385c8771837bffde1f176c995ebe7d814c5b20b;cache492;terminal-positive488;maximum-upper1.0297972392379541e-12|master=stable-ascending-union-of-r50-cache-rows-and-terminal-directed-positive-rows;include-currently-negative-cached-rows;capacity512;overflow-before-partial-add;no-row-drop|basis=reuse-r50-captured-gradient-and-gram-column-bit-exact;missing-row-one-vjp-plus-one-jvp;fixed-operator|solver=one-outer;hildreth-zero-dual;exact-eight-sweeps;stable-master-order;checkpoints1,2,4,8;inactive-release-via-lambda-zero;no-tolerance-stop|globalization=target=current+correction;unchanged-r48-ball-box-projection;feasible-chord;dyadic-alpha1-through2^-8-largest-first;fresh-directed-audit;accept-first-certified-or-strict-psi-h-maximum-upper-decrease|certificate=unchanged-r48-all-row-directed-jvp;zero-positive-only;outside-master-terminal-positive-count-exact;no-infeasibility-claim|selection=certificate-first;else-persistent-expansion-required-if-strict-progress-and-outside-master-positive>0;else-persistent-master-closure-candidate-if-strict-progress;else-r50-reference-retained|controls=r50-parent-bytes;cache-root;stable-union;capacity;basis-reuse;missing-basis;gram;hildreth;projection;dyadic-globalization;directed-certificate;outside-master;work;rollback;route-precedence|routes=persistent-parent-rejected;persistent-source-rejected;persistent-workspace-rejected;persistent-union-rejected;persistent-capacity-rejected;persistent-basis-rejected;persistent-gram-rejected;persistent-hildreth-rejected;persistent-projection-rejected;persistent-globalization-rejected;persistent-primal-audit-rejected;persistent-work-rejected;restoration-next-trqp-compatible-candidate;persistent-constraint-generation-expansion-required;persistent-master-closure-candidate;persistent-r50-reference-retained|precedence=parent,source,workspace,union,capacity,basis,gram,hildreth,projection,globalization,primal-audit,work,certificate,outside-master,selection|work=parent-r50-replays1;moved-workspace1-new;workspace-release1;captured-cache-reused;missing-rows<=20;row-vjp<=20;gram-jvp<=20;candidate-directed-jvp<=9;pairpasses<=49;sweeps8;projection-calls1;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;capacity-change=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-persistent-master-classification-only
```

Invalid Markdown-line SHA-256 (includes the terminal newline):
`6146af2c43ced8a9a5348147388c53add99ef41a7821434884efef6fd12428c9`.

The first executable replay rejected this mismatch before the persistent master
was formed. It has no solver credit.

## Frozen v3 identity

The exact v3 runtime literal is the v2 code block above with its first `|v2|`
token replaced by `|v3|`; every other byte is unchanged. No terminal newline is
part of the runtime identity.

SHA-256: `9785a2168fdcb436cc06ff7c20277284c9e8fd5d27288374469f936f95ef5fcf`.

## Hard gates

1. Reproduce exact R50 bytes, semantic, route, witness/cache/outer roots,
   source operator, contact/trust geometry and rollback.
2. Form the exact stable ascending union of cached and terminal-positive rows.
   Keep currently negative cached rows; do not truncate or drop.
3. Reject before mutation if the union exceeds 512. Reuse captured basis/Gram
   bytes and build only missing rows, at most 20.
4. Require the unchanged positive-diagonal and `1e-12` relative Gram symmetry
   audits over the full master.
5. Run exactly one zero-dual eight-sweep Hildreth solve in stable master order.
   No early sweep stop, warm start or fitted tolerance is admitted.
6. Project through the unchanged R48 ball-box operator and audit
   `alpha=1,1/2,...,2^-8` largest first. Accept only a fresh certificate or strict
   simultaneous `psi`/`h`/maximum-upper decrease.
7. Count terminal directed-positive rows outside the persistent master exactly;
   this count owns the expansion-versus-closure research classification.
8. Compatibility still requires zero positive all-row directed uppers. Master
   feasibility and dual stationarity are insufficient.
9. Build and release exactly one new moved workspace. Exact operator work is at
   most 20 new row VJPs, 20 Gram JVPs, nine directed audits and 49 pair passes.
   Parent R50 work is not new R51 work.
10. Exact rollback. No witness/state/filter commit, restoration exit, trust
    update, following outer or timing.

Require two clean Release builds and byte-exact outputs. Any PASS is one private
persistent-master classification only, not a completed restoration transaction,
runtime permission or production evidence.

Rationale:
[R51 research](../../development/nonlocal-nsr3b4e2d7r19r51-persistent-master-research-2026-08-25.md).
