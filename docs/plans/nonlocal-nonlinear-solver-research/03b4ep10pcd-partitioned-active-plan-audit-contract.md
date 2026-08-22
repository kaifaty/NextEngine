# NSR3-B4EP10PCD -- partitioned active-plan audit contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10pcd-partitioned-active-plan-audit|v1|parent=45dcdee2ce3b5aee7b6f324abc68d9de392c6493a33c4a9d577d9013ca5e197b:15d3ceb70c6c33d6069bdec1cf1cf25e7d780e90450bf215844b9a13349eadaa:82be83e5131ae5eb3c49a687a764c851f79a81c107417922fd7285e35a986ce1:c9b664a3f57864d8916f6db40f233e2019e0c83c9ef80e089fa6212d14b078ce|implementation=b55b0dc7594057323d766b22b626ecde3e464e29|command=nominal-hydro-partitioned-active-plan-audit|baseline=serial-active-plan|algorithm=stable-counting-sort;source-partitions64;partition-major-u32-matrix|phases=parallel-count-source,parallel-target-total,serial-offset-prefix,parallel-partition-base,parallel-stable-fill,parallel-validate|order=partition-ordinal-then-source-center-then-slot|audit=plans226;source-by-slot,target-offsets,target-slots,payload-byte-exact|work=directed150845996;active131987230;target263974460|executor=additional-regions1130;additional-logical-partitions72320;workers8|capacity=matrix<=3026944;combined-added<=67108864|negatives=local-count;partition-base|runs=2;stdout-byte-exact;stderr-empty|timing=none;old-commands-exact|reference=closed|credit=b4ep10pci-implementation-contract-only
```

Identity SHA-256:
`c6d14dd53d1669d2f370e057288567a284e7fde762a787f09d705ac0e45f5c06`.

## Implementation boundary

Add only:

```text
--nominal-hydro-partitioned-active-plan-audit
```

The returned transaction remains the B4EP10I serial active-plan path. The
audit constructs a candidate plan beside it and compares exact arrays before
the candidate is discarded. Old commands must not allocate the histogram or
enter its five extra regions. No timing, atomics, floating reductions,
runtime option or public schema is allowed.

## Algorithm and exact oracle

Use exactly 64 contiguous logical source partitions and a partition-major
`uint32_t[64][target_count]` count/cursor matrix. Parallel phases use the
existing explicit 8-worker executor. The only serial phase is a checked target
offset prefix.

Require for every plan:

- candidate `source_by_slot`, `target_offsets`, `target_slots`, `payload_bytes`
  and success state equal the serial active plan exactly;
- each source slot written once, each active directed slot retained for its
  source and participant, and every target row strictly increasing;
- checked count sums, offsets, partition bases, cursor ends and payload;
- no fallback or partial successful plan.

Across 226 plans require 150,845,996 directed slots, 131,987,230 active slots,
263,974,460 target records, 1,130 added regions and 72,320 added logical
partitions. Matrix payload is at most 3,026,944 bytes; combined candidate plan,
matrix and existing owner scratch is at most 67,108,864 bytes.

Dedicated copies/injection must reject one wrong local target count and one
wrong partition base before an exact candidate is reported.

## Runs and regressions

Run two fresh Release processes pinned to CPUs `0..7` with
`OMP_PLACES=threads`, `OMP_PROC_BIND=close` and dynamic teams off. Require
zero exit, empty stderr and byte-identical stdout.

The final binary must retain exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10PD stdout
`ed205b78a818fbcef6a1b2edfef644af2451f422d809f37fa25696c1eb9f316e`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

## Exit

PASS authorizes only a separately frozen B4EP10PCI candidate implementation
and balanced A/B. Failure retains the serial active builder and B4EP10I.
B4EP10PI remains negative evidence. B4E2, broad corpus, runtime/GPU/schema and
production remain blocked.
