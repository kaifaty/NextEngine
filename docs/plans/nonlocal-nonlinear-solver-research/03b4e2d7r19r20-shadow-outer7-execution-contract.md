# NSR3-B4E2D7R19R20 -- shadow outer-7 execution contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r20-shadow-outer7-execution|v1|parent=3a4601fb:0b002ee449562c08b3be84bc94eff219c72d3c2afdf41490d7b65a1fddaffb24:e49c1fa1a3f8621c289bb75c7bc470a72338d3433ab94f49750f87d15d421b97|source=r18-state=5ad2f99d356c9f5f7553d84189e1e279dcd360bdb9a588370a874b2c5fe707bc;r18-receipt=5a029f24a82b60117492f1929a2d63489d2321b63286a9260282101b74ef8a64;r19-state=37d1556efa98ceb68db566a6f553d82295fc6b233fcbfb0aa6d393a96a4ef6b5;grant=cb1b13d53754e7b1408d66eeb0a00f7dc02a697ccd338d8d77834a6f8b1a6776;grant-receipt=28b5b27673dc9ed247957c159f7c4ac071a074902c918a273327a0d0a40b513a;active-owner=f88ea265a2e2544b76ae335770bb548da6aef76c7a3883ff7ea87e62e6f92ac4;history=a36fa9c02821f1eac2785aa0ec35632f088810b3be625303271d2b9750b8a5b1;position=494dc8d29030f06b79b7047a82ce83eb994e863541becc37f60d3343f08ff29f;dual=9ec049a088270e211435a5cc16b4fea23826c261145209cb28748b8dec92c778;epoch1;outer7;slice50;cumulative573|owner-consume=flags1-to3;consumed-root=94df43022a9b8fd28e00f926db066e065bd8fa17ff65598b01dfe48e11e47f8a|candidate=slice-budget512;total50;available462;oracle=unsliced-budget8704;total573;available8131;both-one-outer7;same-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=7,2,25,1,50,573,37,23,552,21,2,23,0|bounds=outer1;inner<=16;new-hvp<=462;new-workspaces<=251;new-precision<=41;cumulative-new-hvp<=8131|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model|result=roots-derived-after-independent-execution;receipt-NEALOER1-schema1-body404-total420;state-NEALOES1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-outer7-parent-rejected;shadow-outer7-source-rejected;shadow-outer7-grant-rejected;shadow-outer7-owner-rejected;shadow-outer7-duplicate-rejected;shadow-outer7-stale-epoch-rejected;shadow-outer7-resource-rejected;shadow-outer7-candidate-rejected;shadow-outer7-oracle-rejected;shadow-outer7-oracle-mismatch;shadow-outer7-aborted;shadow-outer7-execution-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer7-execution-candidate-only
```

Identity SHA-256:
`a9a172f787195595c948259a73d658bb23894b99a7b1ac4ceefae83afb39e43e`.

## Required command

Add `--nonlocal-al-shadow-outer7-execution`. It must reproduce exact R19
stdout/semantic bytes, reconstruct the exact R18/R19 source chain, execute one
outer-7 slice candidate and one independently initialized unsliced oracle,
and require complete bit/work equivalence before consuming the active owner
and emitting canonical receipt/state objects.

The candidate starts at total HVP `50` with maximum `512`; the oracle starts at
`573` with maximum `8704`. Their deltas, not absolute totals, must match.

## Hard failures

Identity/parent/source mismatch, stale or duplicate ownership, invalid
prework ledger, candidate or oracle failure, candidate/oracle mismatch,
canonical roundtrip failure, ledger drift, rollback/idempotence failure,
scope overrun or two-build/process mismatch is hard FAIL. Candidate slice
exhaustion cannot borrow capacity or state from the oracle.

## Authority boundary

This contract grants one private shadow outer-7 candidate plus one comparison
oracle only. It does not grant public/world commit, another outer update,
another substep, macro, trajectory, timing, runtime integration or production
use.

Research basis:
[D7R19R20 research](../../development/nonlocal-nsr3b4e2d7r19r20-shadow-outer7-execution-research-2026-08-24.md).
