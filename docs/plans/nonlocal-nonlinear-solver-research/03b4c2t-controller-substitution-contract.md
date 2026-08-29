# NSR3-B4C2T -- complete B4B2 joint pressure controller

Status: `PASS / JOINT_PRESSURE_B4B2_CONTROLLER_CANDIDATE / B4C3_DESIGN_AUTHORIZED`

Parent B4C2Q selects `JOINT_PRESSURE_KKT_QUERY_CANDIDATE`; semantic SHA-256
is `683f3ba1ee6f51814e75779e063e7e594a37b16812a97c855df7b2a3a096105e`
and JSON-without-final-LF SHA-256 must equal
`97ed02babe0a08972f7fb0f0d149903c2365480227d5d5266ac66fe2a76bb2c6`.

## Identity

```text
joint-pressure-b4b2-controller-r0
```

Execute both frozen B4B2 cases with contact forecast enabled. Every adaptive
controller interval and every fixed `48/96/192` reference interval must call
only the B4C2Q joint/tape backend with audit disabled.

## Exact trajectory gates

Against an independently executed legacy all-pairs case require bit-exact:

- pass/failure and failure string;
- all adaptive frames, spectrum source/value/calls, initial substeps,
  refinement depth, accepted/executed/discarded substeps and embedded gates;
- committed and discarded-level aggregate counters;
- complete final position/velocity, contact timing/features and all face
  populations, multiplier sums and impulses;
- support/contact/gravity reactions, KKT/ledger/penetration/energy/density/
  speed aggregates and precontact controls;
- every fixed `48/96/192` run, frame aggregate and convergence value;
- B4B2 frame comparison, physical and work gates;
- byte-identical `append_b4b_case`, forecast-work and KKT-work serialization.

Both candidate cases must pass the unchanged B4B2 gates. P1 must retain
`FORECAST_ACTIVE 21/42` on frame zero. P2 must retain inactive forecast through
frame 13, active forecast on frame 14 and start-active frame 15.

## Query trace gates

For each case report the incremental query-chain SHA-256, neighborhood/tape/
evaluation/HVP/trial counts, accepted promotions, rejected destructions,
maximum live workspaces, total pairs/directed records, total cell distance
tests, corresponding all-pairs candidate checks and maximum tape bytes.

Require:

- candidate all-pairs evaluation/HVP counters are zero;
- audit all-pairs evaluation/HVP counters are zero;
- neighborhood, tape and joint-evaluation counts are equal;
- every workspace satisfies strict cell-work reduction;
- at most two workspaces are live and zero remain on every exit;
- controller output remains exact despite discarded adaptive levels.

Query details are reduced into a chain digest; B4C2Q remains the normative
per-query record.

## Repeatability and exit

Two reports must be byte-identical. B4C2Q, B4C1, B4C0R and B4C0 raw reports
remain exact.

PASS selects `JOINT_PRESSURE_B4B2_CONTROLLER_CANDIDATE` and authorizes only
B4C3 canonical publish/decode continuation contract design. FAIL preserves
B4C2Q and blocks canonical work.

No canonical continuation, nominal run, CUDA, runtime, schema or production
authority is granted.

Execution passes all frozen gates; see the
[dated evidence](../../development/nonlocal-nsr3b4c2t-controller-substitution-evidence-2026-08-21.md).
