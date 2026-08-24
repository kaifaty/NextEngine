# NSR3-B4E2D7R19R31 -- scalar-feasibility interval contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r31-scalar-feasibility-interval|v1|parent=b6e531a5:34cf7a56d5c449dca20bf1ab800e061f62bfa0e8145996a6c032e080519596ad:41c3e833a7c618281d94219e059bfbab85cc6631f2bc916478856484ece4808b|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;violated-bd06f7b7c5e1f1943e5c269bbfb192229504091381b8f20f0f4988eecb021c70;rhs-7cb60183e7b7a40d077e409282730bacbdf4ab92eb449a82ef87986e89d70c55;preimage-ebc17b986e68f823c2088aa3fd7719374bc5265b14cc3b751658bd65962c1430;response-fd75ab535dc05ed712a7eae7f4cfa1fc882d94e93c85c0a15f22b58e39137d7f;particles6000;rows1420;columns18000|problem=alpha-in-[0,1];selected:c+alpha*r<=0;inactive:c+alpha*r<=0;lower=max-selected-c/(-r);upper=min-inactive-(-c)/r|arithmetic=binary64-state;binary128-ratio-order;binary64-separate-multiply-add;nextafter-repair<=64|controls=dense-feasible;dense-inactive-blocker;dense-selected-impossible;dense-zero-margin;parent;source;partition;ratio-order;direct-predicates;work;rollback|observations=lower;upper;gap;blocker-roots;repair-ulps;safe-selected-progress;required-inactive-leakage;scaled-preimage|routes=scalar-interval-parent-rejected;scalar-interval-source-rejected;scalar-interval-dense-rejected;scalar-interval-partition-rejected;scalar-interval-order-rejected;scalar-interval-predicate-rejected;scalar-interval-work-rejected;scalar-feasibility-interval-candidate;inequality-active-set-reformulation-required|precedence=parent,source,dense,partition,order,predicate,work,classification|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-r30-once;new-pair-passes0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|correction=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-scalar-feasibility-interval-classification-only
```

Identity SHA-256:
`7cef89a34df72c81730c09243af64105602ccbffe4a73fb10f5935cfa595a60a`.

## Required command

Add `--nonlocal-al-scalar-feasibility-interval`. Reproduce exact R30 and its
captured constraint/preimage/full-response roots; pass all four dense controls;
derive the binary128-ordered lower/upper bounds; repair and directly verify
binary64 predicates; emit both possible classification routes and all frozen
observations.

## Hard failures

Identity/parent/source/partition, any dense control, nonfinite or inconsistent
ratio ordering, more than 64 nextafter repairs, failed direct predicates,
new pair/solver work, rollback or two-build/process mismatch is hard FAIL.
An empty interval is not hard failure when the proof closes; it selects
`INEQUALITY_ACTIVE_SET_REFORMULATION_REQUIRED`.

## Authority boundary

Read-only private diagnostic only. No correction, moved nonlinear evaluation,
floor classification, following outer, policy change, timing, state/public/
world mutation, runtime or production authority.

Research basis:
[D7R19R31 research](../../development/nonlocal-nsr3b4e2d7r19r31-scalar-feasibility-interval-research-2026-08-24.md).
