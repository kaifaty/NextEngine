# Physical sound V18 O0 — intrinsic coverage result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT / O0_CAPABILITY_REJECT` |
| Protocol | [V18 P0b](physical-sound-v18-p0b-hybrid-truth-protocol-2026-09-01.md) with unchanged [V17 P0a O0](physical-sound-v17-p0a-disjoint-factorized-truth-protocol-2026-09-01.md) |
| Implementation | Git `d268756f`; two deterministic Python modules and 10 focused tests |
| Allowed claim | Graph-geodesic distance safely detects disconnection and ambient shortcuts, but nearest-context distance alone does not certify context thinning at the frozen rate |
| Product effect | None; F0/I0 remain unopened and authored clips remain authoritative |

## Execution

The implementation and development-only successful controls were committed
before any test-role O0 decision was evaluated. Two complete official runs then
wrote only to independent empty external directories:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v18-o0-intrinsic-coverage-run-a`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v18-o0-intrinsic-coverage-run-b`.

Each directory contains `8` files, `5,547,882` file bytes and `16,126` complete
per-query score records. Every corresponding file is byte-identical. The
ordered complete-file-map digest is
`955f875b4e58c56d3b776fa75d66fa9b600bd0b9676ec09bd46ba6dcd1ea08fe`;
manifest SHA-256 is
`7080d2990c77afb9e1c30bc4583a1866a2f6544335f58fe65e7a789091d20387`
and report SHA-256 is
`a7da01dffab6a04220306cbba4853230f99627894e6c70f2662e8bbdce35a7e9`.

The pinned environment was CPython `3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`,
float64 CPU and one numerical-library thread. Integration-row generation, F0
parameter loading, B0 external-artifact reads, real/source/protected/network
access and all prior-artifact access counters are exactly zero.

## Frozen identity

- P0b protocol SHA-256:
  `216acd012108725cbcbcf5454f1db707a6ac688e182bfd9fa92d45082280475a`.
- Parent P0a protocol SHA-256:
  `39c1a1e94436a4ddc8b2f3755bab841c4e2b1e541cf2963134fc6445ba6bc77e`.
- B0 result dependency SHA-256:
  `c275dc49664e91064f01a9123727a8f1cc5c61b8a2b47a95f1b942f56680ce75`.
- Corpus SHA-256:
  `e83d30ccc3c6cea7c2e62dbfb388edc477c0e73c949e633d01ef2f681918c775`.
- Geometry-record SHA-256:
  `9375802435d04accfa98982528f9407f9fe2ef3558314e6c8a28453b3ce9f80d`.
- Score SHA-256:
  `f64e1535eb1e6cc91aeb537cf23cfaa8eb49074e80a774747e19bf98ca2f7c9f`.

Implementation hashes embedded in both runs are:

| File | SHA-256 |
| --- | --- |
| `physical_sound_v18_o0_common.py` | `baa201197a9790c7c827e36d043ccdbf3362b7458f207b4cd7a3d5f41cecccad` |
| `physical_sound_v18_o0_oracle.py` | `ce3bcfe86ce5459a3180b46f339e71cfeb80bfacf13880ac3ff630b3717bcb70` |

## Result

The decision is `O0_CAPABILITY_REJECT`: `8/10` single-run gates pass and the
complete repeat gate passes. Thresholds frozen only from development are
`0.1450103452` intrinsic and `0.2192065460` Euclidean.

| Endpoint | Intrinsic | Euclidean control | Frozen requirement |
| --- | ---: | ---: | ---: |
| Valid test rejected | `0.0000` | `0.0000` | intrinsic `<=0.10` per topology and at least 11/12 objects |
| All mutation queries rejected | `0.95061` | `0.72063` | diagnostic aggregate |
| Rejection-minus-false-rejection utility | `0.95061` | `0.72063` | intrinsic strictly better |
| RolledSheet ambient shortcut rejected | `1.0000` | lower than intrinsic | intrinsic `>=0.95` |
| Component isolation | `1.0000` every topology | finite ambient distances | intrinsic `>=0.95` |

The two failing gates are:

1. `mutation_class_topology_95_percent`: thinning rejects only
   `0.650206` of Plate and `0.537255` of RolledSheet queries. Every other
   mutation/topology cell passes, including intrinsic-cap Plate `0.955479`,
   intrinsic-cap RolledSheet `0.973856` and every component-isolation cell
   `1.0`.
2. `mutation_objects_11_of_12`: only `7/12` test objects reach the per-object
   `>=0.95` aggregate. The lowest are Glass Plate `0.857451`, Glass RolledSheet
   `0.858796`, Wood RolledSheet `0.873206` and Wood Plate `0.930493`.

This is not a false-OOD or topology-shortcut failure. All twelve valid objects
and every valid topology have zero rejects. The rejected hypothesis is
narrower: absolute distance to the nearest remaining context point does not
encode how much the context set was thinned when the surviving farthest-point
samples still lie within the ordinary valid fill radius.

## Decision and remaining uncertainty

O0 is closed without threshold, percentile, count or mutation repair. F0 and
I0 are not run because P0b requires O0 pass first. The opened O0 test meshes and
their thinning outcomes may diagnose a successor but cannot select or validate
one.

A successor must use a new preregistered coverage family and fresh test
identities. The smallest falsifiable research question is whether a
deterministic context-density/fill certificate, separate from nearest-query
distance, can detect thinning while retaining graph-geodesic disconnection and
ambient-shortcut guarantees. It must compare against this exact raw-distance
baseline and preserve zero false OOD before any F0-like learned field resumes.
