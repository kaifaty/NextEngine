# Physical Sound V13-M2c RealImpact five-impact control — exact result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `READY_FOR_M2D_CANONICAL_SOURCE_RECOVERY` |
| Control | RealImpact `93_GreenGoblet` |
| Claim | `publisher_derived_response_control`, relative only |
| Protocol | [M2c protocol](physical-sound-v13-m2c-realimpact-five-impact-control-protocol-2026-08-31.md) |
| Product authority | None; authored clips remain authoritative |

## Result

Two independent metadata-only executions are byte-identical. M2c binds the
official Green Goblet mesh, five distinct impact parents, the repeated 600-row
listener grid, one canonical listener per impact and five symmetric
leave-one-impact-out folds.

The large `deconvolved_0db.npy` entry was not requested or decoded. The prior
12.5-MiB listener block was also not read. RealImpact is now an exact
control-only lane for later modal-field experiments, not a fresh quality,
validator or admission source.

## Exact identity

- protocol SHA-256:
  `5ee0e0e5e4f4a41fed8902c9c53d908f3ea63b62aedfbc60c13b280166d8aebb`;
- runner SHA-256:
  `cb15f9abf7cca30798237a191189baa86c94cc17c76068079c9704a802311fa6`;
- manifest SHA-256:
  `c003ace2be576e999a13e084f29eac1057ee6609174bea9dc77c61165ff3b3a8`;
- report SHA-256:
  `afb9ae5c651cce3dc0c20d1295c533cc4f06b63dfb0549c4d1f54347d77ce8af`.

Repeated external outputs:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v13-m2c.gDKCdB/run-a`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v13-m2c.gDKCdB/run-b`.

## Frozen impact parents

| Parent | Mesh vertex | Position, metres | Canonical listener row |
| ---: | ---: | --- | ---: |
| `0` | `31676` | `[-0.020582,-0.039492,0.156666]` | `7` |
| `1` | `34221` | `[-0.004617,-0.032324,0.068698]` | `607` |
| `2` | `46` | `[-0.028124,-0.028790,0.126562]` | `1207` |
| `3` | `47577` | `[-0.009129,-0.013843,0.027653]` | `1807` |
| `4` | `21736` | `[-0.003571,-0.025430,-0.000987]` | `2407` |

The canonical listener is exactly `angle=0`, `distance=0`, `microphone_id=7`
at `[0.23,-0.04345,0.0] m`. Each impact is held exactly once; the other four
parents fit that fold. The remaining `2,995` listener rows are radiation
metadata and remain signal-closed.

## Geometry and metadata checks

| Check | Result |
| --- | --- |
| Archive central directory | exact `1,295` bytes and SHA-256 |
| Seven metadata/geometry entries | exact compressed/raw bytes, CRC32 and SHA-256 |
| Impact/listener arrays | exact shapes/dtypes, finite, `3,000` rows |
| Impact blocks | `5 x 600`, constant and distinct parent per block |
| Listener grid | 600 unique tuples, identical in all five blocks |
| Canonical rows | exact `7,607,1207,1807,2407` |
| Mesh | revision `mesh-96252fe02006-v1`, `48,174` vertices, `16,058` faces |

Mesh bounds are
`[-0.044627,-0.045314,-0.002039]…[0.047056,0.046114,0.163027] m`.

## Read accounting

| Counter | Exact value per run |
| --- | ---: |
| Network range responses | `8` |
| Compressed source bytes | `563,002` |
| Raw metadata/geometry bytes decoded | `3,769,734` |
| Metadata scalar values decoded | `30,000` |
| Audio compressed bytes requested | `0` |
| Audio sample values decoded | `0` |
| Prior listener-block payload bytes read | `0` |
| Source payloads emitted | `0` |

## Consequence and next step

M2c is complete. It supports only a future relative derived-response control;
there is no raw-force provenance, absolute-amplitude claim, exact support
identity or independent validation role.

M2d must now recover a viable fresh canonical source without reopening object
92. Candidates `59/82/93` lack the complete selected compact binding, so the
next step is a bounded source-feasibility research cycle: enumerate official
axes and acquisition costs, compare source alternatives, then preregister one
choice before waveform access. M3 real applicability remains blocked until
that decision, while the known-truth oracle may only be scheduled if the
roadmap explicitly separates its source-independent synthetic scope.
