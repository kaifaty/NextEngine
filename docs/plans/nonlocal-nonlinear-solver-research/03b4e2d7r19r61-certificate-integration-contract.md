# NSR3-B4E2D7R19R61 -- topology-owned certificate-integration contract

Date: `2026-08-25`

Status: `CLOSED / PASS / TOPOLOGY_OWNED_ROW_LOCAL_AUDIT_CANDIDATE /
ROLLBACK ONLY`.

Parent: `0c77720d`, R60 stdout SHA-256
`6c6c62b6efb196d97dc4210ad7bcb56732f0c2d107a2df6aa10e9de84005c7f7`,
semantic `d93dbdc95eb66cf45f074541ae44b621755f45f011d2f1589af834a45238aec9`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r61-certificate-integration|v1|parent=0c77720d:6c6c62b6efb196d97dc4210ad7bcb56732f0c2d107a2df6aa10e9de84005c7f7:d93dbdc95eb66cf45f074541ae44b621755f45f011d2f1589af834a45238aec9:ROW_LOCAL_CERTIFICATE_VALIDATION_CANDIDATE|source=r58-state-a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2;witness-040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd;legacy-upper-9e8ad22488a6a49f66f7d2734c9573692a807cbfdc6c3d09874b24a590151ff8;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;r59-histogram-3d4a798f7d9ea0fc71816091e786778f2368d8c0ff944b45200b004fa09dabdd;r59-comparison-5253b1c5a41fbb798cbe25df4a9c16afe663e4167b332aa9e0114c99b6de82de;r60-full-30ed1a37eb96f0967f10a36c9f014459ab4ffe4c5b26609f74a653a1798f3a50;r60-comparison-f0f18b90c88931ec6dc417ccf257109bfb094e541b17b0f69c06154dfab52ff8|owner=one-ALNormalizedWorkspace;one-witness;topology-and-certificate-same-lifecycle;no-caller-degree-cache|topology=offsets-size=rows+1;offset0=0;monotone;offset-last=directed-slot-count;all-directed-pair-indices-valid;derived-maximum=neighborhood-maximum;degree=offset-difference-including-radius-skips|operator=one-fresh-al-r29-directed-jvp;binary64;stable-row-slot-order|shared=raw=c+image.value;scale=abs(c)+image.absolute-sum;one-row-scan|shadow=gamma(16*maximum-degree+66)*scale;upper=raw+bound;must-equal-r58-upper-vector-active-root-count234-maximum1.3678206846699582e-24-row2522|candidate=gamma(16*row-degree+66)*scale;upper=raw+bound;must-equal-r59-local-active-root-count0-maximum-1.9354174334860891e-23-row4930;histogram-and-comparison|relations=candidate-upper<=shadow-upper-all;candidate-active-subset-shadow;shared-raw-image-root;shadow-certified-implies-candidate-certified|controls=r60-parent-bytes;source;workspace;topology;master;witness;owner;degree;shadow;candidate;relations;dense-controls;work;rollback;route-precedence|dense=retain-r60-degree-controls5;dual-path-valid-lower-and-max-degree;forced-offset-last-reject;cases6|routes=integration-parent-rejected;integration-source-rejected;integration-workspace-rejected;integration-topology-rejected;integration-master-rejected;integration-witness-rejected;integration-owner-rejected;integration-degree-controls-rejected;integration-shadow-rejected;integration-candidate-rejected;integration-relation-rejected;integration-controls-rejected;integration-work-rejected;topology-owned-row-local-audit-candidate;integration-reference-retained|precedence=parent,source,workspace,topology,master,witness,owner,degree,shadow,candidate,relation,controls,work,candidate,reference|work=parent-r60-replays1;moved-workspace1;workspace-release1;fresh-directed-jvp1;owner-row-scans1;dense-controls6;new-pairpasses1;new-quad-row-passes0;new-density-sweeps0;new-box-blocks0;new-vjp0;new-basis0;new-gram0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58-r59-r60=unchanged;legacy-replacement=none;witness-apply=none;restoration-promotion=none;restoration-exit=none;binary128=none;gamma-definition=unchanged;operator-arithmetic=unchanged;r43-restoration-commit=none;filter-runtime-commit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-topology-owned-certificate-integration-candidate-only
```

SHA-256 (exact runtime literal, no terminal newline):
`ab51a986d1931551526becef8d38abd23a352b4d4ade994badb49c8b9533f504`.

## Hard gates

1. Reproduce exact R60 stdout, semantic, route and R58--R60 source roots.
2. Build/release one exact moved workspace and verify topology, master and
   witness before calling the new owner.
3. The owner owns topology, derives all row degrees and executes exactly one
   fresh directed JVP plus one row scan. Caller-supplied degree is forbidden.
4. Reproduce the R58 global shadow vector/count/root/maximum and the R59 local
   candidate count/root/maximum/histogram/comparison.
5. Require candidate upper no greater than shadow and candidate active subset
   on every row, with shared raw/image identity.
6. Run five retained R60 degree controls and one dual-path/topology control.
7. Apply frozen route precedence, exact work and rollback. No legacy
   replacement, witness application, restoration promotion/exit or timing.

Require two clean Release builds and byte-exact outputs. PASS creates one
private reusable integration candidate only.

Rationale:
[R61 research](../../development/nonlocal-nsr3b4e2d7r19r61-certificate-integration-research-2026-08-25.md).

Closure evidence:
[R61 evidence](../../development/nonlocal-nsr3b4e2d7r19r61-certificate-integration-evidence-2026-08-25.md).
