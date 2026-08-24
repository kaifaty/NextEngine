# NSR3-B4E2D7R19R21 -- within-epoch outer-8 grant contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r21-within-epoch-outer8-grant|v1|parent=a696c3f7:df06ac4125082f715d200829b5afef203f0a9ab046f6e9018f97bd8066b1d113:a375f2e5aaabe89694ed2f2d4383f3cf1d51212c9a648d8094e5b17ec97d1412|source-state=1da2e0f54f4f15409ee29f8eb1be10798940635212c64448d12cc7e92982b84f;execution-receipt=ec7a61ea0123865de888304c77ba2b90df17337dc8196e208745c5ae970c6bfd;history=7916f59936efde08993e6d94733e5e7254afa8d6d5b7ab0b26bc723d91978b9d;epoch1;next-outer8;slice102;cumulative625;primal0x3e56fb819c000000;stationarity0x3d4514ef89e8a7f5;admissible0|policy=d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e|limits=16,16,34,512,288,64,8704|used=8,2,27,1,102,625,41,25,602,23,2,25,0|encoding=source-owner-NEALROW1-1-44-60;grant-NEALOGT1-1-276-292;receipt-NEALOGR1-1-116-132;active-owner-NEALOOW1-1-108-124;state-NEALOGS1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=owner-before-138f23969d4cc130e02ddd54cbe1fbbb76dd75b0beee0f53312de8f3116b0c7b;owner-after-ca301305414a08ca75cc8acda791d0c4a93152988d184f63316658ac278032d1;grant-6df9d41926767b1cca9df69f576170253e2eb6682435850372bbf4420b7d0b43;receipt-0675f2ab94286aee2b85afe6cf496bcef8fd615efc514143ecf978c51e2bc0f3;active-owner-1987820aa99c22ac2d49abbb894994370a2f85231e9b4792bfc518e3d0eb4c54;state-64f90f969b76efcddb331705a653c8d31858e6d502555615d6eea2071c78093f|success=source-consumed1;active-unconsumed;epoch1;slice102;cumulative625;all-ledger-exact;next-outer8;physics-immutable|controls=valid;source;owner-root;duplicate;stale;epoch-change;slice-reset;cumulative-change;next-outer;grant-corrupt;receipt-corrupt;active-owner-corrupt;abort-before-commit;duplicate-replay|routes=outer-grant-source-rejected;outer-grant-owner-rejected;outer-grant-duplicate-rejected;outer-grant-stale-epoch-rejected;outer-grant-ledger-rejected;outer-grant-context-rejected;outer-grant-grant-rejected;outer-grant-receipt-rejected;outer-grant-active-owner-rejected;outer-grant-aborted;outer8-grant-candidate|precedence=source,owner,duplicate,stale,ledger,context,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|outer8=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-private-within-epoch-outer8-grant-candidate-only
```

Identity SHA-256:
`acbe1ec9472e25af8402b625f539ff68f08c96dbfb097e7a09883521a7ef413a`.

## Required command

Add `--nonlocal-al-within-epoch-outer8-grant`. It must reproduce exact R20
stdout/semantic bytes, reconstruct the R20 state/receipt/history, reproduce
all frozen canonical roots, preserve epoch/slice/cumulative and every other
ledger field, run the fixed rollback/idempotence corpus, execute zero new
solver work and reproduce byte-exact output from two clean Release builds.

## Hard failures

Identity/parent bytes, source reconstruction, canonical target, ledger/context,
route precedence, rollback/idempotence, zero-work scope or build/process
mismatch is hard FAIL.

## Authority boundary

This contract grants one private within-epoch outer-8 grant candidate only.
It does not authorize outer 8, epoch reset, public/world commit, timing,
runtime integration or production use.

Research basis:
[D7R19R21 grant research](../../development/nonlocal-nsr3b4e2d7r19r21-within-epoch-outer8-grant-research-2026-08-24.md).
