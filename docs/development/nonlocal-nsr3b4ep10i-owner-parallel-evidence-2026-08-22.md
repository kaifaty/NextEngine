# NSR3-B4EP10I owner-parallel evidence -- 2026-08-22

Status: `PASS / BIT_EXACT_DETERMINISTIC_OPENMP_CANDIDATE`

## Result

The opt-in owner-computes backend passes at explicit worker counts
`1,2,4,8,16`. Every process:

- exits zero with empty stderr;
- reproduces the frozen B4EP7I physics, publication roots, ledgers and work
  chain;
- executes 3,411 OpenMP regions and exactly 218,304 fixed logical
  partitions;
- observes the requested team size in every region;
- reports zero team, coverage and worker failure;
- emits common correspondence SHA-256
  `917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca`.

Two fresh 16-worker processes produce byte-identical 7,900-byte reports at
SHA-256
`11aa81d704172a820eadcf941ce9b690567ab25c9a90601b98154c645e6c407f`.
This is correspondence evidence only. B4EP10I contains no duration and makes
no utilization, scaling or speedup claim.

## Deterministic execution boundary

OpenMP 4.5 is linked only into the research executable. The selected command
sets dynamic teams off and maximum active levels to one before any region.
Each nonempty operation is divided into 64 ranges by logical ordinal, and
OpenMP dispatches those ordinals with `schedule(static, 1)` and an explicit
team size.

All output and partition-status arrays are allocated before the region.
Workers own unique pair, centre or target slices. Prefix sums, topology
metadata, owner-plan construction and the canonical energy fold remain
serial. There are no atomics, OpenMP floating reductions or cross-worker
scatter writes.

The internal negative controls also pass:

- worker counts 0 and 17 reject as `OWNER_PARALLEL_CONFIGURATION` before an
  OpenMP region or candidate write;
- injected logical partition 3 rejects as `OWNER_PARALLEL_PARTITION_3` after
  exact once-only coverage validation and before a candidate is returned.

## Exact work

| Selected phase | Calls | Exact work |
|---|---:|---:|
| topology flags | 226 | 91,595,540 |
| topology compacted pairs | 226 | 85,716,150 |
| evaluation density gathers | 226 | 150,845,996 |
| evaluation directed values | 226 | 131,987,230 |
| evaluation target gathers | 226 | 263,974,460 |
| owner-plan builds | 226 | one per accepted tape |
| HVP directed values | 459 | 242,957,856 |
| HVP target gathers | 459 | 485,915,712 |

The maximum selected added payload is 26,712,560 bytes, below the frozen
67,108,864-byte nominal cap. This remains one-workspace nominal research
evidence, not a broad-corpus or production capacity result.

## Process receipts

| Workers | Report bytes | stdout SHA-256 | result SHA-256 |
|---:|---:|---|---|
| 1 | 7,896 | `bd3a55db1dad1cd2f9173b259ede5e32c5d337e3d0c13c032dd8318a37b1ff0f` | `d05d2faccf839dac83337b1eaf12d5c673581fefcbf884472945f9780656752c` |
| 2 | 7,896 | `8231d6099b1cfb730ffb37cd14f6a116821a4f00bd13053e3783933f69fa77bf` | `4c19ceacf690c95c9fe74dd078d47e82997dbc66e9286b4435d85ee2899bb036` |
| 4 | 7,896 | `f6a5ee62152971b0689b40b68102e2e1dfa983d40677004933474c66c1265255` | `d80d5294e3cf26d9ac5096a762cb890b45466988cd176db5edf9eabf18343c50` |
| 8 | 7,896 | `c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3` | `45937668e0f63a64756767259697a5303f84e35c61c3ba5319c93dd1f313b093` |
| 16 | 7,900 | `11aa81d704172a820eadcf941ce9b690567ab25c9a90601b98154c645e6c407f` | `4d09846b576043c7e2e81a827a27c190f30428c4614af074d047020f3dc2b764` |
| 16 repeat | 7,900 | `11aa81d704172a820eadcf941ce9b690567ab25c9a90601b98154c645e6c407f` | `4d09846b576043c7e2e81a827a27c190f30428c4614af074d047020f3dc2b764` |

## Build and regressions

- implementation commit:
  `abb7a06bdc16c3a906ce3b0e5d85f19a250dd9bd`;
- compiler: GCC 15.2.0, Release `-O3 -DNDEBUG -ffp-contract=off
  -fno-fast-math`, OpenMP CXX 4.5 (`_OPENMP=201511`);
- `compile_commands.json`:
  `89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00`;
- executable: 3,954,560 bytes,
  `b5bc2619f58c6eff8d219016b1a3d145add1ca63b4470e28e0b08be4dd325c46`;
- Build ID: `cd7f50a1c4eddf1e84b06e2ede21d215929896ac`.

Source hashes:

- `CMakeLists.txt`:
  `48301abd9ba4c4005c1b0d15714a2d4a2ecc07e20dffbc58707535d0bb99625c`;
- `boundary_reference.cpp`:
  `d8d787d8e1f525c2011f9c29bc30077dcb445e28ce077a65a699100508e2f064`;
- `boundary_reference.hpp`:
  `c73fdabc774335c13f32047a5f1a00bd78072e75003b7c6a874e983b39afe288`;
- `formula_reclosure_main.cpp`:
  `d7c0fc4a253a6cc5f6339ddc918edb643c66347e23b6cbc7c3172b78af6d974f`.

The final binary retains:

- B4EP7I exact stdout
  `8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095`;
- B4EP9 semantic result
  `44e93e6e4ee24dcc623fe36d4c99ad4456f482fe1b47d18410ffb08204f9cd72`;
- B4EP10D exact stdout
  `22ed1368916e01cb10e701123c7185c2f1702a4a485ccbcfede9109d0fd1155c`.

External artifacts remain outside Git:

- build: `/home/kaifaty/.cache/nextengine/external/build-nonlocal-b4ep10i.vgLN5u`;
- runs and regressions:
  `/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10i.L0yCs4`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10i-evidence|v1|identity=94a9d6e26959cfb5c72138457fddc9e72e09c7b4f29882654d0fbcb52759f26c|implementation=abb7a06bdc16c3a906ce3b0e5d85f19a250dd9bd|correspondence=917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca|results=1:d05d2faccf839dac83337b1eaf12d5c673581fefcbf884472945f9780656752c,2:4c19ceacf690c95c9fe74dd078d47e82997dbc66e9286b4435d85ee2899bb036,4:d80d5294e3cf26d9ac5096a762cb890b45466988cd176db5edf9eabf18343c50,8:45937668e0f63a64756767259697a5303f84e35c61c3ba5319c93dd1f313b093,16:4d09846b576043c7e2e81a827a27c190f30428c4614af074d047020f3dc2b764|stdout=1:bd3a55db1dad1cd2f9173b259ede5e32c5d337e3d0c13c032dd8318a37b1ff0f,2:8231d6099b1cfb730ffb37cd14f6a116821a4f00bd13053e3783933f69fa77bf,4:f6a5ee62152971b0689b40b68102e2e1dfa983d40677004933474c66c1265255,8:c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3,16:11aa81d704172a820eadcf941ce9b690567ab25c9a90601b98154c645e6c407f|repeat16=11aa81d704172a820eadcf941ce9b690567ab25c9a90601b98154c645e6c407f|binary=b5bc2619f58c6eff8d219016b1a3d145add1ca63b4470e28e0b08be4dd325c46|build-id=cd7f50a1c4eddf1e84b06e2ede21d215929896ac|compile=89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00|regions=3411;partitions=218304;teams=1,2,4,8,16|work=226,91595540,85716150;226,150845996,131987230,263974460,226;459,242957856,485915712|payload=26712560|negatives=0,17,3|regressions=8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095,44e93e6e4ee24dcc623fe36d4c99ad4456f482fe1b47d18410ffb08204f9cd72,22ed1368916e01cb10e701123c7185c2f1702a4a485ccbcfede9109d0fd1155c|decision=b4ep10s-contract-research
```

SHA-256:
`f8299c06b49dbdab866a98067db2809da58d41d21955a54df8e42634ab100cd2`.

## Decision

Retain `BIT_EXACT_DETERMINISTIC_OPENMP_CANDIDATE`. B4EP10S may now freeze
and execute a balanced serialized scaling experiment. Until that separate
evidence exists, the selected worker count, actual utilization, throughput
and speedup are unknown. B4E2, runtime/schema, GPU and production remain
blocked.
