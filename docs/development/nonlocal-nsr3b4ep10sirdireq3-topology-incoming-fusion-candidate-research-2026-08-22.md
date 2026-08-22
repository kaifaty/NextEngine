# NSR3-B4EP10SIRDIREQ3 topology/incoming fusion candidate research -- 2026-08-22

Status: `COMPLETE / DIRECT_NEIGHBORHOOD_PUBLICATION_SELECTED / IMPLEMENTATION_NEXT`

## Question

What is the smallest opt-in implementation that consumes the exact Q2 fused
plan, removes the redundant SICD builder and preserves the selected SIRDI
physics and ownership boundary?

Q2 proves all 226 fused plans byte-exact and reduces standalone construction
from 665,142,896 entry visits and 678 OpenMP regions to one 85,716,150-pair
pass and zero regions. It does not prove the fast consumer or speed.

## Alternatives

1. Move the audit trace's pending plan into evaluation. Reject: diagnostic
   state would become the real data channel and couple correctness to audit
   lifetime.
2. Publish the fused plan directly from topology through `JointNeighborhood`.
   Select: these incoming fields already carry the accepted SICD plan into the
   unchanged evaluation/tape consumer.
3. Rebuild the plan inside evaluation. Reject: it preserves the redundant
   topology traversal and changes the established consumer boundary.
4. Cache the plan across topology queries. Reject: active-pair identity still
   lacks a horizon-margin certificate; Q2 proves construction, not reuse.
5. Add a partitioned parallel target fill. Defer: deterministic partition
   offsets and extra regions are a separate hypothesis. The canonical serial
   pair fill is already exact and bounded.

## Selected ownership

The candidate remains opt-in and uses this path:

```text
owner-filter topology
  -> local degree/endpoints/fused plan
  -> JointNeighborhood incoming fields
  -> existing split-incoming validation/move
  -> pressure tape
  -> existing evaluation/HVP/retention lifetime
```

Topology writes degree during existing metadata and source/endpoints during
existing row fill. One canonical current-pair traversal fills targets while
checking endpoint bounds, row cursor capacity and increasing slot order. A
successful plan is published directly into the neighborhood. The subsequent
SICD builder is skipped only in candidate mode.

The implementation must not fall back. Any missing endpoint, target cursor,
payload or ownership error fails before evaluation. Ordinary SIRDI never
allocates candidate storage and remains the rollback.

The successful nominal candidate should remove exactly 678 regions and 43,392
logical partitions, leaving 3,411 regions and 218,304 partitions. It retains
226 plan publications, 226 evaluation consumers, 459 HVP consumers and all
physics/query/retention counts. Added plan plus construction scratch is capped
at 16 MiB; Q2 observed less than 9 MiB without shadow comparison storage.

## Measurement route

The shared host remains unqualified for short-margin wall A/B. Split evidence
into two stages:

1. Two fresh duration-free candidate processes must be byte-identical, retain
   all physics roots and work/ownership invariants, and reject missing-endpoint
   and target-cursor injections before evaluation.
2. One serialized in-process CPU experiment uses
   `CLOCK_PROCESS_CPUTIME_ID`, one warmup per path and balanced pairs
   `AB, BA, AB` under affinity `0-7`. Require three candidate wins, median
   baseline/candidate CPU speedup at least `1.03` and paired-speedup range at
   most `1.10`.

The CPU stage may select the candidate only for nominal research and another
candidate-specific profile. It is not wall latency, FPS, real-time or
production evidence. If exactness or CPU gates fail, retain SIRDI without
rerunning or weakening gates.

## Decision

Freeze B4EP10SIRDIREQ3 as an opt-in direct-publication implementation plus a
separate CPU-work A/B command. Do not enable the path by default, reuse the Q2
shadow as real storage, change the target fold, run B4E2/broad corpus or claim
wall speed.
