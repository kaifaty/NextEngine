# NSR3-B4E2D7R19R18 -- shadow outer-6 resume contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r18-shadow-outer6-resume|v1|parent=a4fe1a74:d930b6d57b1bd11936949d7ef7ab2e7f2c607c048fd9b53db6fcfb94f41b2447:c0c6d0285a724dd3615f5cfab077e4b8bc86a73abede691b60c847889a15a173|source=069f8bdddfaeed4d09156f4577f9922d4f606f0c2aaee3e27ff5a66c2915b8f9;grant=266f8ac864d9a83e7bf71fa1df0b4dc33fe964d6064ced058287f04e61f4d475;active-owner=cc6b9e854ca39d940f826899f02827765f293b96bbf9429f7b5b7caceddae9b2;owner-state=d013821c2f5c862086bb56774b73b97a94f597aa5a6c05d87f9434d8f6b3ba54;epoch1;outer6;slice0;cumulative523|owner-consume=flags1-to3;consumed-root-abbef5a38a947f21945d9a96e50df2597a4ac4647a2d733c911ff55cb74eaa74|candidate=slice-budget512;total0;oracle=unsliced-budget8704;total523;both-one-outer6;same-position-dual-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=6,2,25,1,0,523,33,21,504,19,2,21,0|bounds=outer1;inner<=16;hvp<=512;workspaces<=255;precision<=43;cumulative<=8704|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model|result=roots-derived-after-independent-execution;receipt-NEALRSM1-schema1-body404-total420;state-NEALRST1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-resume-parent-rejected;shadow-resume-source-rejected;shadow-resume-grant-rejected;shadow-resume-owner-rejected;shadow-resume-duplicate-rejected;shadow-resume-stale-epoch-rejected;shadow-resume-resource-rejected;shadow-resume-candidate-rejected;shadow-resume-oracle-rejected;shadow-resume-oracle-mismatch;shadow-resume-aborted;shadow-outer6-continuation-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer6-continuation-candidate-only
```

Identity SHA-256:
`6f26279f6422ca0d9b4f76a9eb15ae10d8b961a2d807ba98ca6881055c80bcc0`.

## Required command

Add `--nonlocal-al-shadow-outer6-resume`. It must:

1. reproduce exact R17 stdout SHA
   `d930b6d57b1bd11936949d7ef7ab2e7f2c607c048fd9b53db6fcfb94f41b2447`
   and semantic result
   `c0c6d0285a724dd3615f5cfab077e4b8bc86a73abede691b60c847889a15a173`;
2. revalidate exact R16 source bytes and the R17 grant/receipt/active-owner
   chain before work;
3. independently execute one candidate slice-budget outer 6 and one unsliced
   cumulative-budget oracle outer 6 from exact cloned inputs;
4. require both outer updates to pass without precision/sign/oracle failure;
5. require exact outer/trial/payload/policy/work roots and identical resource
   deltas, allowing only the frozen budget-base difference;
6. enforce one-outer and all frozen structural/resource bounds;
7. consume the active owner and emit canonical resume receipt/state only after
   exact equivalence;
8. reproduce consumed-owner root
   `abbef5a38a947f21945d9a96e50df2597a4ac4647a2d733c911ff55cb74eaa74`;
9. run every fixed negative at its first route, with exact rollback and
   idempotent duplicate replay; after-work negatives reuse captured outputs;
10. execute no outer 7, second substep, macro, trajectory, public/world commit
    or timing;
11. run one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent bytes, chain validation, consumed-owner root, input cloning,
candidate/oracle pass, exact result/work/delta equivalence, bounds, receipt or
state roundtrip, negative route, rollback, duplicate idempotence, scope or
build/process repeat mismatch is hard FAIL.

## Route precedence

1. `SHADOW_RESUME_PARENT_REJECTED`;
2. `SHADOW_RESUME_SOURCE_REJECTED`;
3. `SHADOW_RESUME_GRANT_REJECTED`;
4. `SHADOW_RESUME_OWNER_REJECTED`;
5. `SHADOW_RESUME_DUPLICATE_REJECTED`;
6. `SHADOW_RESUME_STALE_EPOCH_REJECTED`;
7. `SHADOW_RESUME_RESOURCE_REJECTED`;
8. `SHADOW_RESUME_CANDIDATE_REJECTED`;
9. `SHADOW_RESUME_ORACLE_REJECTED`;
10. `SHADOW_RESUME_ORACLE_MISMATCH`;
11. `SHADOW_RESUME_ABORTED`;
12. `SHADOW_OUTER6_CONTINUATION_CANDIDATE`.

## Authority boundary

This contract grants one private candidate outer 6 plus one oracle outer 6
and at most one local shadow-state commit. It grants no outer 7, public/world
physics mutation, runtime schema, live budget change, timing or production
authority.

Research basis:
[D7R19R18 shadow-resume research](../../development/nonlocal-nsr3b4e2d7r19r18-shadow-outer6-resume-research-2026-08-24.md).
