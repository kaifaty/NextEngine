# NSR3-B4EP10PC partitioned active-plan research -- 2026-08-22

Status: `COMPLETE / STABLE_PARALLEL_COUNTING_SORT_SELECTED`

## Input

B4EP10PI shows that eliminating 226 plan builds is useful but that scanning a
fixed superset plan increases CPU work enough to miss the 5% gate. The next
design must retain the compact active target rows and remove only their serial
construction bottleneck.

The problem is a stable transpose of active directed records:

```text
source-major directed slots
  -> two target records per pressure-active slot
  -> target-major CSR
  -> preserve source/slot order inside every target row
```

This is a stable counting-sort/CSR-transpose problem, not a floating-point
reduction.

## Selected algorithm

Use the existing 64 fixed logical partitions over contiguous source-centre
ranges. They are independent of the requested physical worker count.

1. **Count/source:** each logical partition writes its unique
   `source_by_slot` range and a private target-degree row.
2. **Target totals:** target-owned parallel rows sum the 64 integer counts.
3. **Offsets:** one serial checked prefix over 11,824 targets constructs the
   exact target offsets.
4. **Partition bases:** target-owned parallel work converts each target's 64
   counts into disjoint per-partition cursors in ordinal order.
5. **Stable fill:** every source partition scans centre then slot order and
   writes into its reserved target ranges.
6. **Validation:** target-owned parallel rows verify strictly increasing slots,
   source/participant coverage and exact row ends.

For a target, partition ordinal order is also global contiguous source-centre
order. Stable local scanning therefore reproduces the serial builder's exact
slot sequence. Every write has a unique owner; no atomic or floating reduction
is needed.

## Capacity

The partition-major `uint32_t` count/cursor matrix is
`64 * 11,824 * 4 = 3,026,944` bytes. Target totals add about 47 KiB. The
matrix is reused between count and fill rather than duplicated. This should
remain below the existing 64 MiB nominal owner boundary.

The algorithm adds five parallel regions per plan: count, target total,
partition base, fill and validation. Across 226 plans this is 1,130 regions
and 72,320 logical-partition executions. B4EP10R1 measured region
orchestration at only 1.11%, but this is a design input, not speed evidence.

## Alternatives

- **Atomics into target degrees/cursors:** rejected because fill order depends
  on arrival order.
- **One histogram per physical worker:** rejected because output would depend
  on worker count and scheduling.
- **Sort `(target,slot)` records:** stable radix/counting sort is possible but
  needs another full key/value buffer and more traffic than the bounded target
  domain requires.
- **Per-target scan of all directed slots:** exact but quadratic in targets.
- **Masked fixed plan:** preserved as B4EP10PI negative evidence; it raises
  CPU work and is not selected.

## Discriminator

B4EP10PCD must build both the current serial active plan and the partitioned
candidate for all 226 workspaces, then require byte-identical
`source_by_slot`, `target_offsets`, `target_slots` and payload size. It reports
the exact 150,845,996 directed, 131,987,230 active and 263,974,460 target
records, matrix capacity, executor additions and two corrupt count/base
negatives.

The audit does not use the candidate for evaluation or HVP and records no
timing. PASS may authorize only a separate B4EP10PCI implementation/A-B
contract.

## Decision

Freeze B4EP10PCD before implementation. B4E2, broad corpus, runtime, GPU,
schema and production remain blocked.
