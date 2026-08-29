# Nonlocal corrected GPU neighborhood audit — author evidence

| Field | Value |
| --- | --- |
| Research ID | `NCGA1` revision 1 |
| Contract | `docs/plans/nonlocal-corrected-gpu-neighborhood-audit/00-neighborhood-index-correspondence-contract.md` |
| Contract SHA-256 | `fb8e9235cf0a1bc4c78b2bab245a46d467d906f4fa101285cd14c6f37c5e5b65` |
| Author result | `EXACT_NEIGHBORHOOD_INDEX_CORRESPONDENCE_SUPPORTED_BOUNDED` |
| Review status | `PENDING` |
| Product status | `REPORT_ONLY`; SPEC-38 and ADR-076 remain `Proposed` |
| Host | Linux x86-64, NVIDIA GeForce RTX 3080, compute capability 8.6, driver `610.43.02`, CUDA compiler/runtime `13.3.73` / `13030` |
| Frozen build | `/tmp/nextengine-ncga1-release-fzhBvU` (outside Git) |

## Outcome

The isolated CUDA candidate exactly reproduced the independent host graph on
all eight frozen fixtures. The compared payload includes the stable
`(packed_cell_key, SampleId)` cache, canonical owner IDs, CSR offsets and
neighbor `SampleId`s. The fixed scrambled cloud and its reversed input have the
same graph and work roots.

All four deliberately wrong CUDA identities were rejected by the common
comparator. This confirms that the apparatus observes radius inclusivity,
signed floor-cell mapping, stable identity and adjacent-cell traversal rather
than accepting aggregate counts alone.

This result does not retrofit the historical float full-solver path. That path
still indexes its CSR by array position and is therefore not itself evidence
for SPEC-38 stable `SampleId` cache identity. NCGA1 supplies a separate
canonical-integer candidate; moving it into a complete solver is outside this
contract.

## Frozen candidate identity

| Identity | SHA-256 / Git object |
| --- | --- |
| Candidate snapshot | `083e11644d1a4639f1fc9a884d9b2dd24808f7b4` |
| Candidate tree | `68a99b8437bdf1e41fb86712afe949bbfc5ad188` |
| Immediate parent | `2781c2a314a23ce2000a901ca3ed0445b1caa03c` |
| Immediate parent-to-snapshot binary diff | `8b1bd6cd124b0449c3cc18198ccad596bbe05b8d7a8b60d22154a42564e4088d` |
| Frozen-contract base | `8855fbc7b713a883a7fdc281c87a7bd829388ce1` |
| Contract-base-to-snapshot binary diff | `a00268b93baa41d046349cf6c4c9d184d871f01efbb5fa350181c0baf2417552` |
| CMake source | `f9ff06c41a505b2d251244e44ec32577f8318925182f22ac8691711771cc90be` |
| Header/DTO | `9cc74ac2b82ff0d1465e06bfb077211b4b32630406d7f6d4d560c7cf3b27ccbf` |
| Independent host oracle | `3c99dd30768e766c630a81c44afa36130c8a9b68f8692ca74d4ab623376cb8ae` |
| CUDA cell candidate | `5a346513e97fbe37138dd6681e197e243033f806396c53c54bc8a4365c7d0ccc` |
| Harness/fixtures/comparator | `f56eea323588b1d6498bd8e31382519304807a230f686460efdc20ba54e8d11e` |
| Release executable | `51c81e2ef874bfafe04bd9ada288198a34e77c8c998a7206c2224924721833e1` |
| Process stdout | `0a89d92ceff561f6e8f128299ead356abaa7820cd05025fd2c1f6b3d85906b51` |

Two independently configured Release directories produced the exact executable
hash above. The target strips non-loadable symbol tables in Release because
NVCC otherwise embeds a random temporary-file suffix there; executable loaded
content and output were already identical before that packaging normalization.

## Exact result roots

| Artifact | SHA-256 |
| --- | --- |
| Ordered fixture corpus | `9b7f2278b0af6f90f00867578a5f3f0229ae982e77aa5fc31827366256f18269` |
| Independent reference graph set | `c8b905f78a161fbb1fa40a4bd05c7458e9d47be98343e934f3c1e12c11ca01e8` |
| CUDA candidate graph set | `c8b905f78a161fbb1fa40a4bd05c7458e9d47be98343e934f3c1e12c11ca01e8` |
| CUDA work receipt set | `b01eb736e6b0f6afcbb127826e9a87dd68a42cff11bbddbf0bb30d19d3783f63` |
| Wrong-identity control set | `282e69554503ad3f724927bb58f40f6e600382bb221ad801ec0f23b32f71d190` |

Per-fixture results:

| Fixture | Samples | Directed pairs | Maximum degree | Result |
| --- | ---: | ---: | ---: | --- |
| `isolated_nonzero_id` | 1 | 1 | 1 | exact |
| `support_edge` | 4 | 8 | 3 | exact |
| `negative_cell_floor` | 6 | 24 | 5 | exact |
| `duplicate_positions_distinct_ids` | 4 | 12 | 4 | exact |
| `adjacent_and_diagonal_cells` | 7 | 31 | 6 | exact |
| `scrambled_cloud` | 12 | 60 | 8 | exact |
| `scrambled_cloud_reversed` | 12 | 60 | 8 | exact, same root/work as prior row |
| `water_lattice_4x4x4_scrambled` | 64 | 1,192 | 30 | exact |

Every row contains self exactly once, is strictly ordered, is symmetric and
uses an ascending stable owner ID. Ten cold allocations/executions per fixture
produced byte-identical graph payloads and work receipts.

## Negative and admission controls

| Control | Required discriminator | Result |
| --- | --- | --- |
| `strict_radius` | exact support edge | rejected |
| `truncate_signed_cell` | negative cell key/cache order | rejected |
| `array_index_identity` | scrambled stable IDs | rejected |
| `same_cell_only` | adjacent/diagonal cells | rejected |

Empty input, duplicate `SampleId`, out-of-range coordinate and 257-sample
capacity input all returned their exact pre-result failure class in both host
and CUDA entry paths.

## Commands and checks

Fresh configure/build:

```text
cmake -S crates/continuum-water/tools/nonlocal-feasibility \
  -B /tmp/nextengine-ncga1-release-fzhBvU -DCMAKE_BUILD_TYPE=Release
cmake --build /tmp/nextengine-ncga1-release-fzhBvU \
  --target nonlocal-corrected-cuda-neighbors \
           nonlocal-corrected-cuda-terms nonlocal-feasibility -j2
```

Primary command, executed twice with exact stdout:

```text
/tmp/nextengine-ncga1-release-fzhBvU/nonlocal-corrected-cuda-neighbors --self-test
```

CUDA Compute Sanitizer `memcheck`, `initcheck` and `synccheck` each exited zero
with `ERROR SUMMARY: 0 errors`.

Retained non-regressions:

- NCGA0 corrected full-pair term stdout remains exact at
  `5342fb400c07d429645f66ebb1f596d3e1a2fcd69e16dc5f8d6640c700cfbcd7`;
- historical gather/pointer-swap/specialized CUDA tiny corpus remains `11/11`
  PASS, all `neighbors_passed=true` and reused-instance outputs exact.

Repository state was clean after the candidate snapshot. No historical CUDA,
NCGA0, architecture, roadmap, runtime or public-contract source was changed.

## Claim ceiling and next action

Author evidence supports only the exact finite NCGA1 neighborhood/cache/index
claim on the frozen GPU/toolchain. It gives no local matrix, local solve,
trajectory, stability, visual, performance, runtime or canonical-GPU result.

The candidate now requires the single fresh independent review. A positive
review would permit a new separately frozen local energy/source/matrix assembly
audit; it would not permit a full solver or performance claim.
