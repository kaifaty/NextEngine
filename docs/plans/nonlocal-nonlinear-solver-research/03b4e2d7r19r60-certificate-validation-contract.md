# NSR3-B4E2D7R19R60 -- independent certificate-validation contract

Date: `2026-08-25`

Status: `FROZEN / IMPLEMENTATION AUTHORIZED / ROLLBACK ONLY`.

Parent: `e2553f04`, R59 stdout SHA-256
`495bbd213784c4e4f7500c33e2be32c05d41b859145282770364a8fc3a143a4b`,
semantic `e97d68233ce067e7fb948d6cd38bf79ecafb403e16961fadb9a3e9bcb0ce071d`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r60-certificate-validation|v1|parent=e2553f04:495bbd213784c4e4f7500c33e2be32c05d41b859145282770364a8fc3a143a4b:e97d68233ce067e7fb948d6cd38bf79ecafb403e16961fadb9a3e9bcb0ce071d:ROW_LOCAL_ENCLOSURE_CERTIFICATE_CANDIDATE|source=r59-r58-state-a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2;witness-040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd;upper-9e8ad22488a6a49f66f7d2734c9573692a807cbfdc6c3d09874b24a590151ff8;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;histogram-3d4a798f7d9ea0fc71816091e786778f2368d8c0ff944b45200b004fa09dabdd;comparison-5253b1c5a41fbb798cbe25df4a9c16afe663e4167b332aa9e0114c99b6de82de;degree44,113,53,5992;current234;local0;strict5992;closed234;current-max1.3678206846699582e-24-row2522-degree102;local-max-1.9354174334860891e-23-row4930-degree111|structural=claimed-degree-vector-equals-flat-offset-difference;derived-maximum-equals-neighborhood-maximum;nominal-pass;dense-pass;forced-one-row-undercount-reject;overcount-reject;decreasing-offset-reject;maximum-mismatch-reject|domain=stable-directed-row-slot-order;binary64-runtime-expression-replay;nonzero-subnormal-count;nonfinite-count;displacement,relative,jacobian,component-product,dot-partial,term,row-accumulator,absolute-sum,raw,bound,upper|oracle=fresh-full-binary128-directed-formula;binary64-owned-position,witness,radius,weight;compensated-row-fold;row-local-gamma128(16*degree+66);full-upper=full-raw+full-bound|dominance=binary128(local-binary64-upper)>=full-upper-all-rows;publish-failures;minimum-margin;worst-row-degree;roots|controls=r59-parent-bytes;source;workspace;topology;master;witness;r59-reproduction;dense-degree;binary128;comparison;work;rollback;route-precedence|routes=validation-parent-rejected;validation-source-rejected;validation-workspace-rejected;validation-topology-rejected;validation-master-rejected;validation-witness-rejected;validation-r59-reproduction-rejected;validation-degree-controls-rejected;validation-binary128-rejected;validation-comparison-rejected;validation-work-rejected;validation-arithmetic-domain-stronger-bound-required;validation-dominance-rejected;row-local-certificate-validation-candidate;validation-reference-retained|precedence=parent,source,workspace,topology,master,witness,r59,dense,binary128,comparison,work,domain,dominance,validation,reference|work=parent-r59-replays1;moved-workspace1;workspace-release1;row-local-scans1;dense-degree-controls5;fresh-binary128-row-traversals1;new-pairpasses0;new-quad-row-passes1;new-density-sweeps0;new-box-blocks0;new-jvp0;new-vjp0;new-basis0;new-gram0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58-r59=unchanged;integration=none;certificate-authority=private-validation-only;binary128=offline-only;gamma-definition=unchanged;operator-arithmetic=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-row-local-certificate-validation-only
```

SHA-256 (exact runtime literal, no terminal newline):
`15313184ff0f552c9731da249d135d1075ead0334988bf9834a0bf72e3ce42ca`.

## Hard gates

1. Reproduce exact R59 stdout, semantic, route, source roots, row counts,
   degree aggregates, maxima and comparison roots before new work.
2. Build/release one exact moved workspace and verify topology, 494-row master
   and exact selected witness.
3. Recompute the complete R59 scan and require exact roots/counts/maxima.
4. Require exact nominal-workspace degree validation, then run five frozen
   dense cases: one valid case passes; undercount, overcount, decreasing-offset
   and maximum-mismatch mutations fail.
5. Execute exactly one fresh binary128 row traversal and arithmetic-domain
   audit. Preserve stable row/slot order and row-local operation counts.
6. Publish every-row containment, minimum margin, worst row/degree, subnormal
   and nonfinite counts and exact roots. Do not fit a failed row.
7. Apply frozen route precedence, exact work and rollback. No integration,
   witness application, restoration exit, solver work or timing.

Require two clean Release builds and byte-exact outputs. PASS authorizes only
research and freezing of a separate integration contract.

Rationale:
[R60 research](../../development/nonlocal-nsr3b4e2d7r19r60-certificate-validation-research-2026-08-25.md).
