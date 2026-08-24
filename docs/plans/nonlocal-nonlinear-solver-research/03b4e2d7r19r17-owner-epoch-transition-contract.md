# NSR3-B4E2D7R19R17 -- atomic owner/epoch transition contract

Status: `FROZEN / PASS OWNER EPOCH TRANSITION CANDIDATE / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r17-owner-epoch-transition|v1|parent=c8a30bf7:d493c68da969911afe7228e26dbfcb32434ebdf3c028e4aaef91a9ca30af689f:e0ed3a9bf7f804eb2ee77ee65d9ebdfe10c71745e24a82bc341eec55047cbdcd|source=069f8bdddfaeed4d09156f4577f9922d4f606f0c2aaee3e27ff5a66c2915b8f9;history=9754a0bb714011fe4eb637060c6c1eee930e7eb4a1c7db991f664ae40011af7b;epoch0;next-outer6;slice523;cumulative523|policy=12f7a0956a2a7b911530f348834363db43d2ad0119fc5d6a19f9aa5145a06a6c|encoding=source-owner-NEALSOW1-1-44-60;grant-NEALEPG1-1-252-268;receipt-NEALORC1-1-116-132;active-owner-NEALAOW1-1-108-124;state-NEALOST1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=source-before-174c0609ed0dc2411fbf99fcda920c7c0c37d71ad51799e5facc397ebd2ca9db;source-after-51dc71281dc2c7b77e793e2fa040015764a71a941559195c68594bc61721bb58;grant-266f8ac864d9a83e7bf71fa1df0b4dc33fe964d6064ced058287f04e61f4d475;receipt-485b0cb29256f8ba2f0e6f8701925fa292cd409dc802b7189650009933431875;active-owner-cc6b9e854ca39d940f826899f02827765f293b96bbf9429f7b5b7caceddae9b2;state-before-c28e4b7d7ce948a2358d50626af5ac93c354584af350b4c5d3bf37290c8225d7;state-after-d013821c2f5c862086bb56774b73b97a94f597aa5a6c05d87f9434d8f6b3ba54|success=source-consumed1;active-unconsumed;epoch1;slice0;cumulative523;all-other-ledger-exact;envelope-immutable;payload-immutable|controls=valid;source-corrupt;owner-root;duplicate;stale;epoch-overflow;resource-exhausted;abort-before-prepare;abort-after-grant;abort-after-receipt;abort-before-commit;grant-corrupt;receipt-corrupt;active-owner-corrupt;duplicate-replay|routes=owner-transition-source-rejected;owner-transition-owner-rejected;owner-transition-duplicate-rejected;owner-transition-stale-epoch-rejected;owner-transition-epoch-overflow-rejected;owner-transition-resource-rejected;owner-transition-grant-rejected;owner-transition-receipt-rejected;owner-transition-active-owner-rejected;owner-transition-aborted;owner-epoch-transition-candidate|precedence=source,owner,duplicate,stale,overflow,resource,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-byte-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|resume=none;outer6=none;solve=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-atomic-owner-epoch-transition-candidate-only
```

Identity SHA-256:
`22d564118bbf04dd49865629bb4ad700df2f1bfadc91b9510de2acd7edf51e73`.

## Required command

Add `--nonlocal-al-owner-epoch-transition`. It must:

1. reproduce exact R16 stdout SHA
   `d493c68da969911afe7228e26dbfcb32434ebdf3c028e4aaef91a9ca30af689f`
   and semantic result
   `e0ed3a9bf7f804eb2ee77ee65d9ebdfe10c71745e24a82bc341eec55047cbdcd`;
2. retain the exact R16 envelope bytes/root, payload roots and all transitive
   parents;
3. reproduce all frozen canonical object sizes and target roots;
4. consume the source owner exactly once, increment epoch exactly once, reset
   only slice HVP and preserve cumulative/all other ledgers;
5. decode/re-encode and semantically validate every prepared object before a
   single copy-on-write state replacement;
6. run the complete fixed corpus and require the exact first route;
7. prove every precommit failure preserves exact composite-state bytes;
8. prove duplicate replay after success is idempotent and preserves exact
   after-state bytes;
9. prove zero new workspace/HVP/model/trial/precision/outer work and no
   physical mutation;
10. execute one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent bytes, transitive retention, source-envelope immutability,
canonical size/order/endianness, any target root, roundtrip, policy/ledger
transition, negative route, failure rollback, duplicate idempotence, zero-work
scope or build/process repeat mismatch is hard FAIL.

## Route precedence

1. `OWNER_TRANSITION_SOURCE_REJECTED`;
2. `OWNER_TRANSITION_OWNER_REJECTED`;
3. `OWNER_TRANSITION_DUPLICATE_REJECTED`;
4. `OWNER_TRANSITION_STALE_EPOCH_REJECTED`;
5. `OWNER_TRANSITION_EPOCH_OVERFLOW_REJECTED`;
6. `OWNER_TRANSITION_RESOURCE_REJECTED`;
7. `OWNER_TRANSITION_GRANT_REJECTED`;
8. `OWNER_TRANSITION_RECEIPT_REJECTED`;
9. `OWNER_TRANSITION_ACTIVE_OWNER_REJECTED`;
10. `OWNER_TRANSITION_ABORTED`;
11. `OWNER_EPOCH_TRANSITION_CANDIDATE`.

## Authority boundary

This contract grants one private, single-process copy-on-write owner/epoch
transition candidate. It is not concurrent or durable CAS and does not define
a public schema, authorize resume/outer 6, change live budget code, run timing
or grant runtime/production authority.

Research basis:
[D7R19R17 owner/epoch research](../../development/nonlocal-nsr3b4e2d7r19r17-owner-epoch-transition-research-2026-08-24.md).

Closed by the
[D7R19R17 evidence](../../development/nonlocal-nsr3b4e2d7r19r17-owner-epoch-transition-evidence-2026-08-24.md).
