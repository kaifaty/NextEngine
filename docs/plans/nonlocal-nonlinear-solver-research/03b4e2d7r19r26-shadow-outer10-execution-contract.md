# NSR3-B4E2D7R19R26 -- shadow outer-10 execution contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r26-shadow-outer10-execution|v1|parent=610dc8bb:b20fffc6ac40ca1f16feb91e412bf7ed7a5be1430502c8921827662ec206cd69:fc8e0182409785c966c4f1c7f391c910e6a30d3d942335a6eb53b1b3e6fc18a9|source=r24-state=749f0805dc2c4b80e2207ea456071271d9c16521515f0554a13dd7203da32d98;r24-receipt=9c618bd10b6aeaed9ae9e5fddd9f7287e4df6d7adf3e76e467543c70e8674e28;r25-state=b82cd881e7b50794de029e6829e8575b86630d50b0f4caaf74a7526a3bb66153;grant=f825ede7b3fa52c93b47806f92e40f34eefb93daca54d0b2d3092bf0870eede7;grant-receipt=cc39098c863c2fa614086e3702029b0c04182bdfd501ffbf94753d9bd410a8fa;active-owner=2f6aea4168bdc513f21099f363a9aa28cc9171620f4a7c558b2cc340d6a6f6c6;history=d39b98c23d63217b0145229b593154fa3b49e2ea2b701ba708c51f5f8eb0b28d;position=b59fdcd410ab2612246324deb5417aa4cec908c644e7ba0d970c348ca088861a;dual=69c062bb1ce89250d978dce462b8f733bc88bfd48c1fe880e57290be0e07550a;epoch1;outer10;slice231;cumulative754;source-primal0x3e54e2d2e4000000;source-stationarity0x3dda59ffa2498bd8|owner-consume=flags1-to3;consumed-root=3e956008c7c633d5ab74e249af21f1a4503ae9f5709603cdc763b0047d45e28d|candidate=slice-budget512;total231;available281;oracle=unsliced-budget8704;total754;available7950;both-one-outer10;same-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=10,2,24,1,231,754,50,30,726,28,2,30,0|bounds=outer1;inner<=16;new-hvp<=281;new-workspaces<=238;new-precision<=34;cumulative-new-hvp<=7950|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model;stationarity-trend-report-only|result=roots-derived-after-independent-execution;receipt-NEALOER1-schema1-body404-total420;state-NEALOES1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-outer10-parent-rejected;shadow-outer10-source-rejected;shadow-outer10-grant-rejected;shadow-outer10-owner-rejected;shadow-outer10-duplicate-rejected;shadow-outer10-stale-epoch-rejected;shadow-outer10-resource-rejected;shadow-outer10-candidate-rejected;shadow-outer10-oracle-rejected;shadow-outer10-oracle-mismatch;shadow-outer10-aborted;shadow-outer10-execution-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer10-execution-candidate-only
```

Identity SHA-256:
`bef2ffec766fd82708ea62ce4090093bce5b33d2389c76d58dd0473ca2e8a2a5`.

## Required command

Add `--nonlocal-al-shadow-outer10-execution`. Reproduce exact R25 parent and
source chain; execute one slice candidate and one independent unsliced oracle;
require complete bit/work equivalence before owner consume and canonical
receipt/state commit. Candidate starts at `231/512`, oracle at `754/8704`.

## Hard failures

Identity/parent/source, owner/resource, candidate/oracle, equivalence,
canonical roundtrip, ledger, rollback/idempotence, scope or two-build/process
mismatch is hard FAIL. Oracle capacity/state cannot rescue candidate failure.
Cap 34 remains part of the experiment; stationarity trend is report-only.

## Authority boundary

One private candidate outer 10 plus one comparison oracle only. No subsequent
outer, public/world commit, substep, macro, trajectory, timing, runtime or
production authority.

Research basis:
[D7R19R26 research](../../development/nonlocal-nsr3b4e2d7r19r26-shadow-outer10-execution-research-2026-08-24.md).
