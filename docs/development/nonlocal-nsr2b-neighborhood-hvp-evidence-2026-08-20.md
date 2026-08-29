# Nonlocal NSR2-B neighborhood/HVP evidence -- 2026-08-20

Status: `PASS / CANONICAL_CELL_NEIGHBORHOOD_V1 / NSR2C_AUTHORIZED`

## Outcome

The deterministic signed-cell pair structure reproduces the all-pairs oracle
bit-for-bit on all seven frozen controls. Pair membership, every objective
component, every gradient/HVP component, density extrema and momentum residual
are exact binary64 matches.

| Fixture | Particles | Unique current/reference pairs | Result |
|---|---:|---:|---|
| compressed pair | 2 | `1 / 1` | exact |
| combined tetrahedron | 4 | `6 / 6` | exact |
| repulsive/attractive surface pairs | 2 each | `1 / 1` | exact |
| 2x2x2 lattice | 8 | `28 / 28` | exact |
| 3x3x3 lattice | 27 | `351 / 351` | exact |
| 4x4x4 lattice | 64 | `1880 / 1880` | exact |

The pressure HVP executes its `J^T J` term through sorted per-center adjacency
without scanning zero entries. This retains all-pairs accumulation order while
changing the operation shape from dense particle scans to active neighbor
visits.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| raw report, run 1 | `cf8fa9a8a6ba64e33eb13c2006502f12327086bd7edc8b675862836cf0deee6d` |
| raw report, run 2 | `cf8fa9a8a6ba64e33eb13c2006502f12327086bd7edc8b675862836cf0deee6d` |
| semantic result | `cd00dd238ef233e41d67bb2524917e5c659dffe801deec333cebbea0ce72163a` |

NSR0/NSR1 and both NSR2-A failure reports remain byte-identical after the
neighborhood implementation.

## Decision

Select `canonical-cell-neighborhood-v1`. Authorize NSR2-C integration into the
unpreconditioned trust solver and bounded 125/512/1000-particle operation-count
scaling. This still makes no wall-time, GPU, physical-corpus or production
claim.

