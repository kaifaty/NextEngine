# NSR3-B4EP10SIC incoming-plan construction research -- 2026-08-22

Status: `COMPLETE / PAIR_ENDPOINT_BUILDER_AUDIT_SELECTED`

## Input

B4EP10SID proves the split fold and bounds its work, but its audit derives the
incoming view by first building a full target plan. A useful implementation
must emit one participant entry per current directed slot without sorting,
atomics or a pressure-active transpose.

## Topological identity

The current source CSR already visits every incident pair in participant-ID
order. Record, for each current pair, the directed slot owned by each fluid
endpoint:

```text
pair_source_slot[pair]
pair_participant_slot[pair]  // fluid-fluid only
```

For a fluid target, its own adjacency row lists fluid neighbours in ascending
ID. Selecting the other endpoint's slot therefore emits incoming slots in
ascending source order, which is also global directed-slot order.

For a support target, current pairs remain globally sorted by fluid source and
then participant. A support-target pair CSR built from that order emits source
slots in the same canonical order.

No floating state or compression is involved.

## Selected audit builder

For each current topology:

1. parallel source-row scan writes `source_by_slot` and unique pair endpoint
   slots;
2. build support pair CSR in preserved current-pair order; this serial audit
   work is explicitly counted and is a future topology-metadata fusion target;
3. parallel target count derives fluid incoming degree from fluid neighbours
   and support degree from the support pair CSR;
4. serial checked prefix creates incoming target offsets;
5. parallel target-owned fill emits and validates incoming slots.

The isolated audit adds three parallel regions per topology, 678 regions and
43,392 logical partitions across 226 plans. These counts are construction
evidence, not the final integration architecture: source mapping can later
fuse with topology row fill, and support CSR with existing pair metadata. One
target fill region is expected to remain.

## Oracle and alternatives

The oracle is B4EP10SID's incoming view derived from the validated full current
plan. Candidate `source_by_slot`, target offsets and target slots must match it
byte-for-byte.

- Dense per-partition target histograms are rejected after B4EP10PCI's CPU
  increase.
- Stable radix sort is rejected because pair endpoint dual indexing already
  gives target order without key/value passes.
- Atomic cursors remain rejected because arrival order is observable.
- A support all-pairs target scan is rejected as quadratic.

## Decision

Freeze B4EP10SICD as a side-by-side construction audit. PASS may authorize
only a separately frozen floating integration/A-B contract. B4E2, broad
corpus, runtime, GPU, schema and production remain blocked.
