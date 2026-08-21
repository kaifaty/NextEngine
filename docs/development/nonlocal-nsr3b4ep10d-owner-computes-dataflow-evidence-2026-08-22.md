# NSR3-B4EP10D owner-computes dataflow evidence -- 2026-08-22

Status: `PASS / BIT_EXACT_OWNER_COMPUTES_DATAFLOW_CANDIDATE`

## Result

The serial owner-computes alternative reproduces the unchanged returned
oracle at every admitted call:

- 226/226 cached topology/filter/CSR alternatives are exact;
- 226/226 fused evaluation alternatives are exact;
- 459/459 pressure-tape HVP alternatives are exact;
- order, coverage, topology, evaluation and HVP mismatches are all zero;
- fallback count is zero.

Two fresh Release processes exit zero with empty stderr and produce identical
6,989-byte stdout at SHA-256
`22ed1368916e01cb10e701123c7185c2f1702a4a485ccbcfede9109d0fd1155c`.
Their semantic result is
`e3223da62d96f54c3ef125dc5511e71fe9fd0103822c117d2f81fe02d14248a2`.

This proves that the scatter-to-owner transformation can preserve the
existing binary64 result on the exact nominal transaction. No thread ran and
no speedup was measured.

## Exact work

| Audit | Calls | Input/output work |
|---|---:|---:|
| topology pair flags | 226 | 91,595,540 |
| topology compacted pairs | 226 | 85,716,150 |
| evaluation density gathers | 226 | 150,845,996 |
| evaluation directed values | 226 | 131,987,230 |
| evaluation target gathers | 226 | 263,974,460 |
| transpose-plan validations | 226 | exact source/order/two-target coverage |
| HVP directed values | 459 | 242,957,856 |
| HVP target gathers | 459 | 485,915,712 |

The topology alternative uses flags, checked exclusive prefix, canonical pair
compaction and centre-owned CSR rows. Density rows preserve the restriction of
global pair order. Evaluation/HVP transpose rows preserve global source
centre/slot order for each target. Those order properties are validated, not
assumed.

## Memory boundary

- maximum topology plan plus topology scratch: 4,919,594 bytes;
- maximum owner plan plus evaluation/HVP scratch: 26,712,560 bytes;
- maximum simultaneously admitted added payload: 29,557,700 bytes
  (28.19 MiB).

This is below the frozen 67,108,864-byte nominal cap. It is one-workspace
research evidence, not a 50k-particle production capacity claim.

## Build and regression

- implementation commit:
  `66796758224d08aaace4b19e8a2a2b3ad7654807`;
- compiler: GCC 15.2.0, Release `-O3 -DNDEBUG -ffp-contract=off
  -fno-fast-math`;
- `compile_commands.json`: 15,205 bytes,
  `b3735ce688cc5d7abb43d5abc10102fd50b09056b9c5fcad0633ae6f5c32fd0a`;
- executable: 3,892,288 bytes,
  `bd4119e26eae6c257721884a9996d1c4f9f4ceb1ed37cf8847e6b19076556434`;
- Build ID: `904a191781f69a0d1e184fb35d40a1c78d161c4e`.

Source hashes:

- `boundary_reference.cpp`:
  `48a281efa663b653793266b42a6dcf090c4f9cff15cd8c56b8d2812a93fa79c9`;
- `boundary_reference.hpp`:
  `690297a5558f528659c177ee16f514dcb4cc5f9617970a4d04bafb4b8481d95d`;
- `formula_reclosure_main.cpp`:
  `25442b91f420d645089602aaf54542a339dc4d77b5fbaaae95bc0fe7d2a3d038`.

The final binary retains exact B4EP7I stdout
`8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095`
and B4EP9 semantic result
`44e93e6e4ee24dcc623fe36d4c99ad4456f482fe1b47d18410ffb08204f9cd72`.

External artifacts remain outside Git:

- build: `/home/kaifaty/.cache/nextengine/external/build-nonlocal-b4ep10d.cPbQTS`;
- positive runs: `/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10d.4VRZvb`;
- regressions: `/home/kaifaty/.cache/nextengine/external/regress-nonlocal-b4ep10d.EvztjO`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10d-evidence|v1|identity=db02821e280df90a285fbbebeea8cc2ec87630891cef105e0d2a4b9f6c3bdc88|implementation=66796758224d08aaace4b19e8a2a2b3ad7654807|result=e3223da62d96f54c3ef125dc5511e71fe9fd0103822c117d2f81fe02d14248a2|stdout=22ed1368916e01cb10e701123c7185c2f1702a4a485ccbcfede9109d0fd1155c|binary=bd4119e26eae6c257721884a9996d1c4f9f4ceb1ed37cf8847e6b19076556434|build-id=904a191781f69a0d1e184fb35d40a1c78d161c4e|compile=b3735ce688cc5d7abb43d5abc10102fd50b09056b9c5fcad0633ae6f5c32fd0a|audits=226,91595540,85716150;226,150845996,131987230,263974460,226;459,242957856,485915712|payloads=4919594,26712560,29557700|mismatches=0,0,0,0,0,0|regressions=8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095,44e93e6e4ee24dcc623fe36d4c99ad4456f482fe1b47d18410ffb08204f9cd72|decision=b4ep10i-contract-research
```

SHA-256:
`431ba5d5596cd2742d042294255ea02c6b67e5499e8d9fe81f1bdf527a5acb44`.

## Decision

Retain `BIT_EXACT_OWNER_COMPUTES_DATAFLOW_CANDIDATE`. B4EP10I may now freeze
an opt-in OpenMP implementation and controlled `1/2/4/8/16`-worker
correspondence contract. It may not infer a speedup from this audit.
