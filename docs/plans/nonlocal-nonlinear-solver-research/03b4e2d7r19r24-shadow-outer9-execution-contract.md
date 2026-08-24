# NSR3-B4E2D7R19R24 -- shadow outer-9 execution contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r24-shadow-outer9-execution|v1|parent=1f177bb7:c1aca629743893dc973112f9ced8262728e641084cc172e09bfe8ce5bf467a01:27e37a5a05689e8483956038cbc63ce24ded6d0307c41b7e9e86a44c0c42bfa8|source=r22-state=dc983c93075040fc2f23080072807f8a1b691adade4f3d7a950be3c9f44b36d9;r22-receipt=96975d4dda71d7490e91ccccfa95c51de480c1bcd3191367f527d5b001c2fa8e;r23-state=b9b8a2d3f0827d9a66a54aeb24f70c674015b8d968d8aaae16189e4adc72ef82;grant=4dd8744547f991b40e5871a4c1823c3e073952dc7b51d6c28acabd92cebf5d18;grant-receipt=16b216ea8c7748ee80a5551f3baf5ee8f3afedea13fe5b88c6283cf07f7de285;active-owner=bd3b2356eb10bf422ae07c16ba9433a63a63da1ccd7e567dcdd3dd3bc4fcee81;history=fa9e2db82287c5f3faac6d8c8031c55bab78230fd708479a569329152c74be7c;position=68d0b821be6f1bdd71f70bf570035be7713cd578fe08774644cbd7c53c672b0f;dual=fbbe1973290706320cbb5f3f3280b6e62b493e3abad4d280ac2985951b7568f3;epoch1;outer9;slice182;cumulative705|owner-consume=flags1-to3;consumed-root=3f4b2fb5d84b3f27260e53e344d5141530b8105f577483aca7a435ec85372522|candidate=slice-budget512;total182;available330;oracle=unsliced-budget8704;total705;available7999;both-one-outer9;same-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=9,3,32,1,182,705,46,28,679,26,2,28,0|bounds=outer1;inner<=16;new-hvp<=330;new-workspaces<=242;new-precision<=36;cumulative-new-hvp<=7999|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model|result=roots-derived-after-independent-execution;receipt-NEALOER1-schema1-body404-total420;state-NEALOES1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-outer9-parent-rejected;shadow-outer9-source-rejected;shadow-outer9-grant-rejected;shadow-outer9-owner-rejected;shadow-outer9-duplicate-rejected;shadow-outer9-stale-epoch-rejected;shadow-outer9-resource-rejected;shadow-outer9-candidate-rejected;shadow-outer9-oracle-rejected;shadow-outer9-oracle-mismatch;shadow-outer9-aborted;shadow-outer9-execution-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer9-execution-candidate-only
```

Identity SHA-256:
`69a60aefa8ea6b080acf1235815f5fc8d38492644031a7bfa6f246f0768f72a1`.

## Required command

Add `--nonlocal-al-shadow-outer9-execution`. Reproduce exact R23 parent and
source chain; execute one slice candidate and one independent unsliced oracle;
require complete bit/work equivalence before owner consume and canonical
receipt/state commit. Candidate starts at `182/512`, oracle at `705/8704`.

## Hard failures

Identity/parent/source, owner/resource, candidate/oracle, equivalence,
canonical roundtrip, ledger, rollback/idempotence, scope or two-build/process
mismatch is hard FAIL. Oracle capacity/state cannot rescue candidate failure.
The unchanged per-trust cap 34 is part of the experiment.

## Authority boundary

One private candidate outer 9 plus one comparison oracle only. No subsequent
outer, public/world commit, substep, macro, trajectory, timing, runtime or
production authority.

Research basis:
[D7R19R24 research](../../development/nonlocal-nsr3b4e2d7r19r24-shadow-outer9-execution-research-2026-08-24.md).
