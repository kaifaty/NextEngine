# NSR3-B4E2D7R19R22 -- shadow outer-8 execution contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r22-shadow-outer8-execution|v1|parent=0346640b:88253c46b9c1763ee4855be5909d1f9b58cf02828efa582b86a1eab61e1cb1a0:99d0a756044173aebb640e6d2bc301ac09c993b4ecb92d95bad886b6b4437719|source=r20-state=1da2e0f54f4f15409ee29f8eb1be10798940635212c64448d12cc7e92982b84f;r20-receipt=ec7a61ea0123865de888304c77ba2b90df17337dc8196e208745c5ae970c6bfd;r21-state=64f90f969b76efcddb331705a653c8d31858e6d502555615d6eea2071c78093f;grant=6df9d41926767b1cca9df69f576170253e2eb6682435850372bbf4420b7d0b43;grant-receipt=0675f2ab94286aee2b85afe6cf496bcef8fd615efc514143ecf978c51e2bc0f3;active-owner=1987820aa99c22ac2d49abbb894994370a2f85231e9b4792bfc518e3d0eb4c54;history=7916f59936efde08993e6d94733e5e7254afa8d6d5b7ab0b26bc723d91978b9d;position=bda0ac4fa2be5c29484a17c520a968046849a49aa892a7c60b04f808a6199afd;dual=dc5bcc6bb918c3336ebe4bd99e03d7b8e1c92a61ca0db73418303ed0a59484eb;epoch1;outer8;slice102;cumulative625|owner-consume=flags1-to3;consumed-root=08001aecf61cc43ee130187f626264aa093420830244e06293ee7824486c9721|candidate=slice-budget512;total102;available410;oracle=unsliced-budget8704;total625;available8079;both-one-outer8;same-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=8,2,27,1,102,625,41,25,602,23,2,25,0|bounds=outer1;inner<=16;new-hvp<=410;new-workspaces<=247;new-precision<=39;cumulative-new-hvp<=8079|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model|result=roots-derived-after-independent-execution;receipt-NEALOER1-schema1-body404-total420;state-NEALOES1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-outer8-parent-rejected;shadow-outer8-source-rejected;shadow-outer8-grant-rejected;shadow-outer8-owner-rejected;shadow-outer8-duplicate-rejected;shadow-outer8-stale-epoch-rejected;shadow-outer8-resource-rejected;shadow-outer8-candidate-rejected;shadow-outer8-oracle-rejected;shadow-outer8-oracle-mismatch;shadow-outer8-aborted;shadow-outer8-execution-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer8-execution-candidate-only
```

Identity SHA-256:
`cbf62bb2861f3fdc7f9cccd3d34348b3d735d356ea093d00407754ac027d6081`.

## Required command

Add `--nonlocal-al-shadow-outer8-execution`. Reproduce exact R21 parent and
source chain; execute one slice candidate and one independent unsliced oracle;
require complete bit/work equivalence before owner consume and canonical
receipt/state commit. Candidate starts at `102/512`, oracle at `625/8704`.

## Hard failures

Identity/parent/source, owner/resource, candidate/oracle, equivalence,
canonical roundtrip, ledger, rollback/idempotence, scope or two-build/process
mismatch is hard FAIL. Oracle capacity/state cannot rescue candidate failure.

## Authority boundary

One private candidate outer 8 plus one comparison oracle only. No subsequent
outer, public/world commit, substep, macro, trajectory, timing, runtime or
production authority.

Research basis:
[D7R19R22 research](../../development/nonlocal-nsr3b4e2d7r19r22-shadow-outer8-execution-research-2026-08-24.md).
