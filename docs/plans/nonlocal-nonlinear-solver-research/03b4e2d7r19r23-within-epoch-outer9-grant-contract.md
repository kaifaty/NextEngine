# NSR3-B4E2D7R19R23 -- within-epoch outer-9 grant contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r23-within-epoch-outer9-grant|v1|parent=32d633fb:32d98ee5549b5321201d9280ddb700967a02de6a83bd8abc283bd2a7d4b18790:9a5380ef56951213b544b14573e7983d84f1b768e9b16b5aa41a33084fb16296|source-state=dc983c93075040fc2f23080072807f8a1b691adade4f3d7a950be3c9f44b36d9;execution-receipt=96975d4dda71d7490e91ccccfa95c51de480c1bcd3191367f527d5b001c2fa8e;history=fa9e2db82287c5f3faac6d8c8031c55bab78230fd708479a569329152c74be7c;epoch1;next-outer9;slice182;cumulative705;primal0x3e55dc122c000000;stationarity0x3d15a9f60fbefbe2;admissible0|policy=d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e|limits=16,16,34,512,288,64,8704|used=9,3,32,1,182,705,46,28,679,26,2,28,0|encoding=source-owner-NEALROW1-1-44-60;grant-NEALOGT1-1-276-292;receipt-NEALOGR1-1-116-132;active-owner-NEALOOW1-1-108-124;state-NEALOGS1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=owner-before-1d7364cd688d8be1cfc0cc3f12fa58975cd8fe318c702da1201c81cbfbb9d7c0;owner-after-f4bb27171627bffbbfdd47551ec84eb2b842d17b29e0f2ff1464c34c01febd3b;grant-4dd8744547f991b40e5871a4c1823c3e073952dc7b51d6c28acabd92cebf5d18;receipt-16b216ea8c7748ee80a5551f3baf5ee8f3afedea13fe5b88c6283cf07f7de285;active-owner-bd3b2356eb10bf422ae07c16ba9433a63a63da1ccd7e567dcdd3dd3bc4fcee81;state-b9b8a2d3f0827d9a66a54aeb24f70c674015b8d968d8aaae16189e4adc72ef82|success=source-consumed1;active-unconsumed;epoch1;slice182;cumulative705;all-ledger-exact;next-outer9;physics-immutable|controls=valid;source;owner-root;duplicate;stale;epoch-change;slice-reset;cumulative-change;next-outer;grant-corrupt;receipt-corrupt;active-owner-corrupt;abort-before-commit;duplicate-replay|routes=outer-grant-source-rejected;outer-grant-owner-rejected;outer-grant-duplicate-rejected;outer-grant-stale-epoch-rejected;outer-grant-ledger-rejected;outer-grant-context-rejected;outer-grant-grant-rejected;outer-grant-receipt-rejected;outer-grant-active-owner-rejected;outer-grant-aborted;outer9-grant-candidate|precedence=source,owner,duplicate,stale,ledger,context,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|outer9=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-private-within-epoch-outer9-grant-candidate-only
```

Identity SHA-256:
`84c510befbdb80777c6aa00cb3650ecf3212cc9d68a0541d0b603768208e0996`.

## Required command

Add `--nonlocal-al-within-epoch-outer9-grant`. Reproduce exact R22 parent,
source/receipt/history and every frozen root; preserve complete ledger; run
the fixed rollback/idempotence corpus; execute zero new solver work; reproduce
output from two clean Release builds.

## Hard failures and authority

Any identity/parent/source/root/ledger/route/rollback/zero-work/reproducibility
mismatch is hard FAIL. This grants one private outer-9 owner only: no outer 9,
epoch reset, publication, timing, runtime integration or production use.

Research basis:
[D7R19R23 research](../../development/nonlocal-nsr3b4e2d7r19r23-within-epoch-outer9-grant-research-2026-08-24.md).
