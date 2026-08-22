# NSR3-B4EP10SIRDIREQ2 topology/incoming fusion evidence -- 2026-08-22

Status: `PASS / CANDIDATE_CONTRACT_RESEARCH_AUTHORIZED / NO_SPEED_CREDIT`

## Result

Both fresh serialized audit processes exit zero, emit empty stderr and are
byte-identical at stdout SHA-256
`44d3279faf411ec8c0eab094cdf8a2b065f9c2327c4f2805f4a1915834f1e521`.
They retain identity `1b84bc73...273f`, SIRDI result `b4f847cb...77e9`, SII
result `f7b1542f...5fb2`, correspondence `1e4bedbb...a35d`, the five physics
roots and duration-free result `52c59113...1312`.

All 226 shadow plans build, compare and release at maximum live depth one.
Every `source_by_slot`, target offset, target slot and payload byte matches the
accepted SICD plan. Target rows are strictly increasing; the corrupted
endpoint and swapped target-order negatives both reject. Order, coverage,
lifetime and fallback failure counts are zero.

## Structural work

| Work | Accepted standalone builder | Fused candidate standalone work |
|---|---:|---:|
| aggregate entries | 665,142,896 | 85,716,150 |
| ratio | 1.0 | 0.12886877468807845 |
| added OpenMP regions | 678 | 0 |
| added logical partitions | existing builder work | 0 |

The shadow records exactly 150,845,996 incoming-degree increments, source
writes, endpoint writes, incoming entries, endpoint reads and target writes.
It visits 85,716,150 canonical current pairs. This proves that the separate
incoming construction can be represented by one current-pair fill while its
remaining work is attached to topology metadata and row fill. It is a work
reduction certificate, not a measured speedup.

Maximum shadow plan payload is 5,409,132 bytes, maximum shadow scratch is
3,808,701 bytes and conservative combined selected/shadow/negative payload is
21,758,020 bytes, below the frozen 64 MiB cap. Final live plans are zero.

## Reproducibility

Implementation commit is `b07d543e89242c3325a7c7d75617ac97c57b050a`.
The Release executable has SHA-256
`0b1899192fe22726ef23c192bd68d35b15797db9ea48c9910184e90affe2ff30`,
size 4,425,408 bytes and ELF Build ID
`07c0788d0c2f34695dc40c2a61ffae7ea5f6abcc`. Its
`compile_commands.json` SHA-256 remains
`79e135ca1ecbb1d76a862c944b574472ecce6fe4141684590f2da33488f03594`.

Source SHA-256 values are
`997894ddbb802357ccfe3945b828a5368d72ecc3f239d4c2f2418272777f2aab`
for `boundary_reference.cpp`,
`7148e864634909e03dfcc929d0fb5f71f828973c5dacecac295054ab858d4dd5`
for `boundary_reference.hpp` and
`a5282a1944a1da4a63c7391cf6c3b5538b3beb38cbcfb8d57678ac01af42808e`
for `formula_reclosure_main.cpp`. Raw reports remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sirdireq2.4O2CHR`.
The Release build passes `-Werror`.

A separate final regression process preserves old SIRDI stdout SHA-256
`539f1ec507e439adc50b14cdf5da616024e041a3131aa56a2002c40a83e4e7e7`
and empty stderr.

## Decision

Close B4EP10SIRDIREQ2 PASS. Research and freeze one separate opt-in candidate
implementation contract that consumes the fused plan instead of rebuilding
SICD. Preserve SIRDI as rollback and require exact plans, physics roots,
ownership and work accounting before any performance claim.

The shared host remains unqualified for short-margin wall A/B. This evidence
does not authorize the fast path directly, wall speed credit, B4E2, broad
corpus, runtime/GPU/schema integration or production use.
