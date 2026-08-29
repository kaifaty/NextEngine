# NSR3-B4E2D7R19R25 -- within-epoch outer-10 grant contract

Status: `PASS / OUTER10_GRANT_CANDIDATE / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r25-within-epoch-outer10-grant|v1|parent=b6fc1451:e401056652ba56aad8c9f7836ac963a57721ef4d3c498ae29cb1736e03562eed:5b623755c2fd1dc136e9e9281c32b387955b1daea4e435460e9f764892b88bbc|source-state=749f0805dc2c4b80e2207ea456071271d9c16521515f0554a13dd7203da32d98;execution-receipt=9c618bd10b6aeaed9ae9e5fddd9f7287e4df6d7adf3e76e467543c70e8674e28;history=d39b98c23d63217b0145229b593154fa3b49e2ea2b701ba708c51f5f8eb0b28d;epoch1;next-outer10;slice231;cumulative754;primal0x3e54e2d2e4000000;stationarity0x3dda59ffa2498bd8;admissible0|policy=d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e|limits=16,16,34,512,288,64,8704|used=10,2,24,1,231,754,50,30,726,28,2,30,0|encoding=source-owner-NEALROW1-1-44-60;grant-NEALOGT1-1-276-292;receipt-NEALOGR1-1-116-132;active-owner-NEALOOW1-1-108-124;state-NEALOGS1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=owner-before-3519d3b80578ac1dfc2b89b68340f1fe748775ab265a9535e2eb7bb5d75039bb;owner-after-11bf48f76ee1c521e2fa29d43e2b0a3f9cd1a03c56e4ee344497099e01e4991f;grant-f825ede7b3fa52c93b47806f92e40f34eefb93daca54d0b2d3092bf0870eede7;receipt-cc39098c863c2fa614086e3702029b0c04182bdfd501ffbf94753d9bd410a8fa;active-owner-2f6aea4168bdc513f21099f363a9aa28cc9171620f4a7c558b2cc340d6a6f6c6;state-b82cd881e7b50794de029e6829e8575b86630d50b0f4caaf74a7526a3bb66153|success=source-consumed1;active-unconsumed;epoch1;slice231;cumulative754;all-ledger-exact;next-outer10;physics-immutable|controls=valid;source;owner-root;duplicate;stale;epoch-change;slice-reset;cumulative-change;next-outer;grant-corrupt;receipt-corrupt;active-owner-corrupt;abort-before-commit;duplicate-replay|routes=outer-grant-source-rejected;outer-grant-owner-rejected;outer-grant-duplicate-rejected;outer-grant-stale-epoch-rejected;outer-grant-ledger-rejected;outer-grant-context-rejected;outer-grant-grant-rejected;outer-grant-receipt-rejected;outer-grant-active-owner-rejected;outer-grant-aborted;outer10-grant-candidate|precedence=source,owner,duplicate,stale,ledger,context,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|outer10=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-private-within-epoch-outer10-grant-candidate-only
```

Identity SHA-256:
`668841e76d3d6f1edfc6ba395dc43acdee37dffc690abf591081b25626a7ec90`.

## Required command

Add `--nonlocal-al-within-epoch-outer10-grant`. Reproduce exact R24 parent,
source/receipt/history and every frozen root; preserve complete ledger; run
the fixed rollback/idempotence corpus; execute zero new solver work; reproduce
output from two clean Release builds.

## Hard failures and authority

Any identity/parent/source/root/ledger/route/rollback/zero-work/reproducibility
mismatch is hard FAIL. This grants one private outer-10 owner only: no outer
10, cap change, epoch reset, publication, timing, runtime integration or
production use.

Research basis:
[D7R19R25 research](../../development/nonlocal-nsr3b4e2d7r19r25-within-epoch-outer10-grant-research-2026-08-24.md).

Result:
[D7R19R25 evidence](../../development/nonlocal-nsr3b4e2d7r19r25-within-epoch-outer10-grant-evidence-2026-08-24.md).
