# NSR3-B4EP10PCI partitioned active-plan evidence -- 2026-08-22

Status: `FAIL_PERFORMANCE / ACTIVE_PLAN_RETAINED`

## Result

The opt-in partitioned active-plan path is physically and structurally exact:

- all three measured candidate outputs are byte-identical at SHA-256
  `e4559fcd5762cbf53a8ed5cdf57899dd2df1c364d0287525bcf55bef3eac8062`;
- all physical roots and frozen work receipts match B4EP10I;
- 226 plans cover the exact 150,845,996 directed, 131,987,230 active
  directed and 263,974,460 target records;
- 4,541 executor regions and 290,624 logical partitions match the contract;
- zero plan, order, coverage or fallback mismatch;
- conservative candidate payload is 29,739,504 bytes, below 64 MiB.

The candidate wins all three balanced wall pairs and is highly stable, but
its median paired speedup is only `1.0336503543405948x`, below the frozen
`1.05x` gate. B4EP10PCI therefore fails performance and does not replace the
B4EP10I serial active-plan builder.

## External A/B

One warmup per command preceded serialized `AB`, `BA`, `AB` rounds on CPUs
`0..7`. All program stderr files are empty.

| Pair | B4EP10I wall ns | Partitioned wall ns | A/B |
|---:|---:|---:|---:|
| 1 | 5,816,659,647 | 5,614,327,242 | 1.0360385841933792 |
| 2 | 5,804,401,399 | 5,615,439,858 | 1.0336503543405948 |
| 3 | 5,751,757,846 | 5,607,662,500 | 1.0256961516496401 |

Candidate wall range ratio is `1.0013869162061733`. Median maximum RSS rises
from 96,384 to 98,188 KiB, a passing 1,804 KiB delta.

Median user CPU rises from 37.25 to 39.31 seconds and median system CPU from
0.77 to 1.40 seconds. The five parallel phases reduce wall by about 3.3%, but
their added integer scans, teams and barriers consume materially more total
CPU. This is a measured result for one nominal research macro, not throughput.

## Build and regressions

- implementation commit:
  `6d1652f60ca6955e4389f9b2955927b004541e93`;
- executable: 4,118,456 bytes,
  `2c86d951512315172a6c08e453e2dd0cf0fb2a59dd9a7454ad2e97c11ca9f2ec`;
- Build ID: `562be7d61eeefa6451ae26c6a981f9c4f921fbe7`;
- `compile_commands.json` SHA-256:
  `89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00`;
- raw A/B metrics SHA-256:
  `0754e0943002f2781450cdc1e4c6fd3a8d3055e59d66bf218aee71b638006c64`.

Source hashes:

- `CMakeLists.txt`:
  `48301abd9ba4c4005c1b0d15714a2d4a2ecc07e20dffbc58707535d0bb99625c`;
- `boundary_reference.cpp`:
  `aea08d39f1c2992e7446b50ff89a6d2c1cbfd75703ec764fe3997d3817cbfce8`;
- `boundary_reference.hpp`:
  `6cb23d5bfea89a476a13dfc58cd5d221a47b16213c4ecdcbe272e0f14899136b`;
- `formula_reclosure_main.cpp`:
  `8806f18aaf724262fc2dc528831ee0a0d2adf0e27a02b40782f57b544d6f6f1d`.

The final binary preserves exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10PCD stdout
`a389e6491d3d13361f8ddf1a907eb0f8336623e991f14f8beebfc01676cfed42`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

External artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10pci.mVzEEA`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10pci-evidence|v1|identity=5c96bd69d2a370a080482bb3284d44ee46a832aec3e551ddf0f6d517283a1943|implementation=6d1652f60ca6955e4389f9b2955927b004541e93|binary=2c86d951512315172a6c08e453e2dd0cf0fb2a59dd9a7454ad2e97c11ca9f2ec|build-id=562be7d61eeefa6451ae26c6a981f9c4f921fbe7|compile=89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00|metrics=0754e0943002f2781450cdc1e4c6fd3a8d3055e59d66bf218aee71b638006c64|stdout=A:c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3,B:e4559fcd5762cbf53a8ed5cdf57899dd2df1c364d0287525bcf55bef3eac8062|result=B:2616e1bcba850edd846c729a8de197f9a04fd08db38022dd6ebfd160f3dbccfe|correspondence=3774b48864dcd9a23c6dcd78b386606096ded9bebeb590a256a44916fe4f5a90|wall-ns=A:5816659647,5804401399,5751757846;B:5614327242,5615439858,5607662500|paired-speedups=1.0360385841933792,1.0336503543405948,1.0256961516496401;median=1.0336503543405948|medians-wall-ns=A:5804401399,B:5614327242|candidate-range-ratio=1.0013869162061733|rss-kib=A:96384,B:98188,delta:1804|user-s=A:37.25,B:39.31|system-s=A:0.77,B:1.4|gates=exact3of3;wins3of3;speed-fail;range-pass;rss-pass|work=builds226;directed150845996;active131987230;target263974460;regions4541;partitions290624;payload29739504|regressions=a389e6491d3d13361f8ddf1a907eb0f8336623e991f14f8beebfc01676cfed42,a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b|decision=reject-selected-fast-path;current-topology-plan-research
```

SHA-256:
`42794aa55ca69ea7431324fb0a1a17457a025c3040591f9abaeb3f7e3ce3410a`.

## Decision

Keep both partitioned and masked commands as opt-in negative evidence. Retain
B4EP10I as selected execution. The next research may examine a
compression-independent current-topology reverse plan built during topology
filtering; it must be audited before implementation. B4E2, broad corpus,
runtime, GPU, schema and production remain blocked.
