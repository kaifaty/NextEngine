# NSR3-B4EP10SIRDIREQ3 -- topology/incoming fusion candidate contract

Status: `CLOSED / FAIL / CPU_GATE / REVERTED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirdireq3-topology-incoming-fusion-candidate|v1|parent=1b84bc737e11cd1e3469b35c604ce50ecff32e0897bcc651267fb0c96073273f:52c5911327c05f647b65d93efd0b21e209abb446661db0d03b5455e80f631312:44d3279faf411ec8c0eab094cdf8a2b065f9c2327c4f2805f4a1915834f1e521|implementation=2671e0a86f06202f70868a1db3ed5974327b89b3|baseline=b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9:539f1ec507e439adc50b14cdf5da616024e041a3131aa56a2002c40a83e4e7e7|commands=exact:nominal-hydro-topology-incoming-fusion-candidate-8,cpu:nominal-hydro-topology-incoming-fusion-cpu-ab-8|ownership=topology-local-plan;neighborhood-publish;tape-consume;existing-retention;release-all-exits|construction=metadata-degree;row-fill-source+endpoints;canonical-pair-target-fill;no-sicd;no-fallback|work=plans226;pairs85716150;directed150845996;regions3411;partitions218304;consumer=evaluation226,hvp459|capacity=added<=16777216;checked|negatives=missing-endpoint,target-cursor;reject-before-evaluation|exact=two-fresh-candidate;old-sirdi-byte-exact;roots+counts+order|cpu=process-clock;one-warmup-each;pairs=AB,BA,AB;affinity0-7;candidate-wins3of3;median-speedup>=1.03;paired-speedup-range<=1.10|timing=cpu-work-only;wall-no-credit|failure=retain-sirdi|reference=closed|credit=nominal-fused-candidate-reprofile-only
```

Identity SHA-256:
`e2381ed56b0c6be4ff178d49f8e2f02706648c2be2359471c037c3acfb5b6d69`.

## Commands

Add only:

```text
--nominal-hydro-topology-incoming-fusion-candidate-8
--nominal-hydro-topology-incoming-fusion-cpu-ab-8
```

The exact command runs the SIRDI nominal transaction with split incoming and
directed scratch reuse, replacing only SICD construction with the fused plan.
The CPU command compares the unchanged SIRDI control and exact candidate in
one process. Neither command changes default/runtime behavior.

## Implementation boundary

Add a candidate flag separate from the Q2 audit flag. In candidate mode,
`b4ep10d_owner_filter_superset` builds the plan locally and publishes it via
the existing `JointNeighborhood` incoming fields. `b4ep3_cached_topology`
must skip `b4ep10sicd_build_incoming_plan`; the existing evaluation path must
perform the same validation and move into
`current_topology_gather_plan`.

Do not use `JointTopologyIncomingFusionAuditTrace::pending_plan` as the real
owner. Do not change floating formulas, pair/directed/target order, target
folds, evaluation/HVP arithmetic, retention or directed scratch. Candidate
failure is fail-closed with no SICD fallback. SIRDI with the candidate flag
off must not allocate or execute fused-candidate storage.

## Exact stage

Require two fresh candidate processes, both exit zero with byte-identical
stdout and empty stderr. Require:

- identity exact and the same frame, aggregate, trajectory, legacy-ledger and
  policy-ledger roots as SIRDI/Q2;
- 226 topology plan builds/publications/consumptions, 226 evaluation calls and
  459 HVP calls;
- 85,716,150 current-pair visits and 150,845,996 degree, source, endpoint,
  incoming, endpoint-read and target-write entries;
- 3,411 parallel regions, 218,304 logical partitions and unchanged team,
  physical, query, coefficient, fusion, retention and directed-scratch counts;
- strictly increasing target rows, zero order/coverage/ownership/fallback
  failures and checked added payload at most 16 MiB;
- missing-endpoint and target-cursor injections both reject before evaluation,
  publication or fallback;
- a separate old SIRDI process retains stdout SHA-256 `539f1ec5...e7e7`.

Any failure stops before CPU measurement and retains SIRDI.

## CPU-work stage

Use `CLOCK_PROCESS_CPUTIME_ID` only after clock resolution is at most 1 us.
Run one unmeasured warmup of each path, then three balanced serialized pairs
in order `AB, BA, AB` with external affinity `0-7`. Exclude JSON duration
fields from semantic hashes. Require every measured baseline and candidate to
pass its exact control, all three candidate CPU samples to win, median paired
speedup at least `1.03` and maximum/minimum paired speedup at most `1.10`.

Failure or an inconclusive CPU gate retains SIRDI and closes this candidate
without rerun. PASS selects the candidate only for the nominal research path
and authorizes one candidate-specific CPU residual profile.

Process CPU is work evidence, not wall latency. Do not record a wall-speed,
FPS, real-time or production claim from this stage.

## Authority

No result authorizes default/runtime/GPU/schema integration, B4E2, broad
corpus, multi-macro, 50k-particle or production use. A separately qualified
wall-throughput window and later production-roadmap gates remain required.

## Closure

Exact stage passes twice, but the one admitted CPU experiment wins `1/3` with
median paired speedup `0.983844x` and paired range ratio `1.565388`. The
candidate and CPU harness are reverted; SIRDI remains selected. See the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdireq3-topology-incoming-fusion-candidate-evidence-2026-08-22.md).
