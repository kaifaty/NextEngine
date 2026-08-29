# NSR3-B4E2D7R19R28 -- shadow outer-11 execution contract

Status: `PASS / SHADOW_OUTER11_EXECUTION_CANDIDATE / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r28-shadow-outer11-execution|v1|parent=1ee7936f:3760daf964f16b64d6c7685b8cdde255f31812d5aacee55cc7c18d2a079d65c2:7f08a85aae890fcf48d207cbdd7be8424675b98eef25506aad68b604a16e47dd|source=r26-state=317f63c5a3f02df3cbb3e905fed9b0b1740ebcb363799c22d4febb3a837fe1fd;r26-receipt=3b5c6dd3b8824eddc3aad566a8b663d344e1702f7958dced701c61710d688776;r27-state=369cab15f75ee90237aa5cbe334aa0da5f857e218bec9812773e43faa9682226;grant=3ded4fc34e99c45657fad87a0aeb4853e1d948817efad60b4106f659c4adefaa;grant-receipt=53a938176504b9059504c511402b5b6f3e8b4dfb17bc6a2aa0f42204160cc330;active-owner=431466b59941f862c97c75c5614be7d2dd525199bfbe9ea5e175753bc4cd431e;history=7aaf5e5077f937f03db754e407243e8c6b61b0f813931cc59d83f7f7dd866785;position=174cad44a3edbc7eab514986e6be3d1bdd3c219aa68fc3c9913b164ce4058562;dual=63770158bc269b879c4e3b98ac12a176be334902f0c929d05d686a157ce40f87;epoch1;outer11;slice282;cumulative805;source-primal0x3e54ad2eec000000;source-stationarity0x3d1770a9a48ed9c8|owner-consume=flags1-to3;consumed-root=13feea1f5a87fb577ed5f9ccf89e0f46d6acdb03554ea1c48b5e2c81081c5264|candidate=slice-budget512;total282;available230;oracle=unsliced-budget8704;total805;available7899;both-one-outer11;same-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=11,2,26,1,282,805,54,32,775,30,2,32,0|bounds=outer1;inner<=16;new-hvp<=230;new-workspaces<=234;new-precision<=32;cumulative-new-hvp<=7899|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model;primal-progress-trend-report-only;stationarity-trend-report-only|result=roots-derived-after-independent-execution;receipt-NEALOER1-schema1-body404-total420;state-NEALOES1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-outer11-parent-rejected;shadow-outer11-source-rejected;shadow-outer11-grant-rejected;shadow-outer11-owner-rejected;shadow-outer11-duplicate-rejected;shadow-outer11-stale-epoch-rejected;shadow-outer11-resource-rejected;shadow-outer11-candidate-rejected;shadow-outer11-oracle-rejected;shadow-outer11-oracle-mismatch;shadow-outer11-aborted;shadow-outer11-execution-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer11-execution-candidate-only
```

Identity SHA-256:
`e7133e07d6104bb41da8ef95446459eccee331c44fea096867b3ae9230f940cd`.

## Required command

Add `--nonlocal-al-shadow-outer11-execution`. Reproduce exact R27 parent and
source chain; execute one slice candidate and one independent unsliced oracle;
require complete bit/work equivalence before owner consume and canonical
receipt/state commit. Candidate starts at `282/512`, oracle at `805/8704`.

## Hard failures

Identity/parent/source, owner/resource, candidate/oracle, equivalence,
canonical roundtrip, ledger, rollback/idempotence, scope or two-build/process
mismatch is hard FAIL. Oracle capacity/state cannot rescue candidate failure.
Cap 34 and policy remain exact; primal/stationarity trends are report-only.

## Authority boundary

One private candidate outer 11 plus one comparison oracle only. No subsequent
outer, public/world commit, substep, macro, trajectory, timing, runtime or
production authority.

Research basis:
[D7R19R28 research](../../development/nonlocal-nsr3b4e2d7r19r28-shadow-outer11-execution-research-2026-08-24.md).

Result:
[D7R19R28 evidence](../../development/nonlocal-nsr3b4e2d7r19r28-shadow-outer11-execution-evidence-2026-08-24.md).
