# NSR3-B4E2D7R19R19 -- within-epoch outer-7 grant contract

Status: `PASS / OUTER7_GRANT_CANDIDATE / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r19-within-epoch-outer7-grant|v1|parent=ad6daf67:3047e85dc42f22afe287b47b0bc3aaa2e9de3bb767b01b6d4c516497c51841e8:baadf44a6ae0a2d83865dc41d5e9de1fef82e64eb52f36fc0b3a78a9c213e14d|source-state=5ad2f99d356c9f5f7553d84189e1e279dcd360bdb9a588370a874b2c5fe707bc;resume-receipt=5a029f24a82b60117492f1929a2d63489d2321b63286a9260282101b74ef8a64;history=a36fa9c02821f1eac2785aa0ec35632f088810b3be625303271d2b9750b8a5b1;epoch1;next-outer7;slice50;cumulative573;primal0x3e5afe281c000000;stationarity0x3d5eabe1f7af94f6;admissible0|policy=d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e|limits=16,16,34,512,288,64,8704|used=7,2,25,1,50,573,37,23,552,21,2,23,0|encoding=source-owner-NEALROW1-1-44-60;grant-NEALOGT1-1-276-292;receipt-NEALOGR1-1-116-132;active-owner-NEALOOW1-1-108-124;state-NEALOGS1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=owner-before-6ead6c878eb810ab6ed7046014e02afd8ea6aeba1d8c1bd57b2f2e33f8bc37a1;owner-after-ea9f947b8778e3147f3ba83f3fbf6a2f04f703cb1b3a45def70739b2c71ca0e4;grant-cb1b13d53754e7b1408d66eeb0a00f7dc02a697ccd338d8d77834a6f8b1a6776;receipt-28b5b27673dc9ed247957c159f7c4ac071a074902c918a273327a0d0a40b513a;active-owner-f88ea265a2e2544b76ae335770bb548da6aef76c7a3883ff7ea87e62e6f92ac4;state-37d1556efa98ceb68db566a6f553d82295fc6b233fcbfb0aa6d393a96a4ef6b5|success=source-consumed1;active-unconsumed;epoch1;slice50;cumulative573;all-ledger-exact;next-outer7;physics-immutable|controls=valid;source;owner-root;duplicate;stale;epoch-change;slice-reset;cumulative-change;next-outer;grant-corrupt;receipt-corrupt;active-owner-corrupt;abort-before-commit;duplicate-replay|routes=outer-grant-source-rejected;outer-grant-owner-rejected;outer-grant-duplicate-rejected;outer-grant-stale-epoch-rejected;outer-grant-ledger-rejected;outer-grant-context-rejected;outer-grant-grant-rejected;outer-grant-receipt-rejected;outer-grant-active-owner-rejected;outer-grant-aborted;outer7-grant-candidate|precedence=source,owner,duplicate,stale,ledger,context,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|outer7=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-private-within-epoch-outer7-grant-candidate-only
```

Identity SHA-256:
`1b16ea2def4561ff4f2c92879c40b1868c8894c0f53d900eafbc9778c7b50ec0`.

## Required command

Add `--nonlocal-al-within-epoch-outer7-grant`. It must reproduce exact R18
stdout/semantic bytes, reconstruct and validate the R18 state/receipt/history,
reproduce every frozen canonical root, preserve epoch/slice/cumulative and all
other ledger fields, run every fixed route with rollback/idempotence, execute
zero solver work, and reproduce byte-exact output from two clean Release
builds.

## Hard failures

Identity/parent bytes, R18 reconstruction, any canonical target, any ledger or
context change, route precedence, rollback/idempotence, zero-work scope or
build/process mismatch is hard FAIL.

## Authority boundary

This contract grants one private within-epoch outer-7 grant candidate only.
It does not authorize outer 7, epoch reset, public/world commit, timing,
runtime integration or production use.

Research basis:
[D7R19R19 grant research](../../development/nonlocal-nsr3b4e2d7r19r19-within-epoch-outer7-grant-research-2026-08-24.md).

Result:
[D7R19R19 evidence](../../development/nonlocal-nsr3b4e2d7r19r19-within-epoch-outer7-grant-evidence-2026-08-24.md).
