# NSR3-B4E2D7R19R27 -- within-epoch outer-11 grant contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r27-within-epoch-outer11-grant|v1|parent=a38bbe72:67a4734dc549f19e57812813ec89e21ce23a90d5e1bb8415a41b7fcd1de78295:6f5d971e6c0dfe3e4f21c48130e3e82f861355a42691b836f5adcc4b78c97f22|source-state=317f63c5a3f02df3cbb3e905fed9b0b1740ebcb363799c22d4febb3a837fe1fd;execution-receipt=3b5c6dd3b8824eddc3aad566a8b663d344e1702f7958dced701c61710d688776;history=7aaf5e5077f937f03db754e407243e8c6b61b0f813931cc59d83f7f7dd866785;epoch1;next-outer11;slice282;cumulative805;primal0x3e54ad2eec000000;stationarity0x3d1770a9a48ed9c8;admissible0|policy=d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e|limits=16,16,34,512,288,64,8704|used=11,2,26,1,282,805,54,32,775,30,2,32,0|encoding=source-owner-NEALROW1-1-44-60;grant-NEALOGT1-1-276-292;receipt-NEALOGR1-1-116-132;active-owner-NEALOOW1-1-108-124;state-NEALOGS1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=owner-before-526ac852b7e52a48e2692b611f2e4ea8e8356fd52019fbeeb9c4935db404f281;owner-after-94e0619fd8babb7398035bdfb06e17a569b18cd15957f09b1a32680502089eba;grant-3ded4fc34e99c45657fad87a0aeb4853e1d948817efad60b4106f659c4adefaa;receipt-53a938176504b9059504c511402b5b6f3e8b4dfb17bc6a2aa0f42204160cc330;active-owner-431466b59941f862c97c75c5614be7d2dd525199bfbe9ea5e175753bc4cd431e;state-369cab15f75ee90237aa5cbe334aa0da5f857e218bec9812773e43faa9682226|success=source-consumed1;active-unconsumed;epoch1;slice282;cumulative805;all-ledger-exact;next-outer11;physics-immutable|controls=valid;source;owner-root;duplicate;stale;epoch-change;slice-reset;cumulative-change;next-outer;grant-corrupt;receipt-corrupt;active-owner-corrupt;abort-before-commit;duplicate-replay|routes=outer-grant-source-rejected;outer-grant-owner-rejected;outer-grant-duplicate-rejected;outer-grant-stale-epoch-rejected;outer-grant-ledger-rejected;outer-grant-context-rejected;outer-grant-grant-rejected;outer-grant-receipt-rejected;outer-grant-active-owner-rejected;outer-grant-aborted;outer11-grant-candidate|precedence=source,owner,duplicate,stale,ledger,context,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|outer11=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-private-within-epoch-outer11-grant-candidate-only
```

Identity SHA-256:
`98a135b8600c66d628e41827803f4ea2573330f2ba198f509f740d53087209e5`.

Add `--nonlocal-al-within-epoch-outer11-grant`; reproduce exact R26 parent and
all frozen roots/ledger; run the fixed rollback/idempotence corpus; execute
zero new solver work and reproduce two clean Release outputs.

Any mismatch is hard FAIL. This grants one private outer-11 owner only: no
outer 11, cap/epoch/policy change, publication, timing or production use.

Research basis:
[D7R19R27 research](../../development/nonlocal-nsr3b4e2d7r19r27-within-epoch-outer11-grant-research-2026-08-24.md).
