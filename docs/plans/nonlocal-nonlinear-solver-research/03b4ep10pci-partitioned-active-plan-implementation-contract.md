# NSR3-B4EP10PCI -- partitioned active-plan implementation/A-B contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10pci-partitioned-active-plan-implementation|v1|parent=c6d14dd53d1669d2f370e057288567a284e7fde762a787f09d705ac0e45f5c06:34eecfa9116fc4de40098ecbec65077ef02ec95395da7c4eeb614139395efe6c:82ce97a52b8f4fa9ccccb8e4e2452dd09238c1a8e52ca4dc53139e2b6956d92a|implementation=74364cc514c7c21133b977d4d380a1d0e50a7c51|commands=baseline:nominal-hydro-owner-parallel-8,candidate:nominal-hydro-partitioned-active-plan-8|candidate=replace-serial-active-plan;builds226;partitions64|algorithm=stable-counting-sort;partition-major-u32-matrix;five-parallel-phases;serial-offset-prefix|order=partition-ordinal-then-source-center-then-slot|work=directed150845996;active131987230;target263974460;regions4541;logical-partitions290624|capacity=actual-peak<=67108864|semantics=b4ep10pcd-oracle;physics-roots-exact;old-commands-exact|timing=external-monotonic+gnu-time;one-warmup-each;three-pairs=AB,BA,AB;serialized;affinity=0-7|gates=candidate-exact-3of3;wins3of3;median-paired-speedup>=1.05;candidate-range-ratio<=1.10;rss-delta-kib<=16384|failure=retain-b4ep10i-serial-active-plan|reference=closed|credit=partitioned-plan-residual-timing-research-only
```

Identity SHA-256:
`5c96bd69d2a370a080482bb3284d44ee46a832aec3e551ddf0f6d517283a1943`.

## Implementation boundary

Add only:

```text
--nominal-hydro-partitioned-active-plan-8
```

Inside that command, replace serial active-plan construction with the exact
B4EP10PCD stable partitioned builder. Do not construct the serial plan, run
the B4EP10PCD audit or use the masked superset path. Old commands must not
allocate the partition matrix or enter its five regions.

The builder uses exactly 64 contiguous logical source partitions, a
partition-major `uint32_t` count/cursor matrix, parallel count/source, target
total, partition-base, stable-fill and validation phases, and one serial
checked target-offset prefix. No atomic, floating reduction, altered energy
fold, runtime option or public schema is allowed.

## Exactness, work and capacity

Require unchanged frame, aggregate, trajectory and both ledger roots; frozen
query/work receipts; exact energy, KKT and adaptive-level gates; 226
evaluation and 459 HVP calls; zero candidate failure or fallback.

Require 226 candidate builds over exactly 150,845,996 directed,
131,987,230 active directed and 263,974,460 target records. The complete
candidate transaction must report 4,541 executor regions and 290,624 logical
partitions. Maximum candidate-owned plan/matrix/owner scratch must not exceed
67,108,864 bytes.

The three measured candidate stdout streams must be byte-identical. The final
binary must preserve exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10PCD stdout
`a389e6491d3d13361f8ddf1a907eb0f8336623e991f14f8beebfc01676cfed42`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

## External A/B

On the same final Release binary and otherwise idle host:

1. pin both commands to physical CPUs `0..7` with `OMP_PLACES=threads`,
   `OMP_PROC_BIND=close` and dynamic teams off;
2. run one unmeasured warmup per command;
3. run three serialized pairs in order `AB`, `BA`, `AB`;
4. record monotonic wall nanoseconds, GNU Time user/system and maximum RSS;
5. require zero exit, empty program stderr and exact stdout before admitting
   duration.

PASS requires candidate exactness `3/3`, candidate wins `3/3`, median
same-pair baseline/candidate wall at least `1.05`, candidate max/min wall at
most `1.10`, and candidate median RSS no more than 16,384 KiB above baseline
median.

## Exit

PASS authorizes only separately frozen candidate residual-timing research.
Any functional failure rejects the implementation. A performance-gate failure
retains B4EP10PCD structural evidence but keeps B4EP10I's serial active plan
as selected execution. B4E2, broad corpus, runtime/GPU/schema and production
remain blocked.
