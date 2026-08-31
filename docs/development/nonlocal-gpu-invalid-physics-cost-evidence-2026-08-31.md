# NCGP11 invalid-physics GPU cost evidence

Date: 2026-08-31  
Result: **INVALID_PHYSICS_COST_ABOVE_ORIGINAL_BUDGET**  
Claim ceiling: **COST ONLY / NO WATER-QUALITY OR GAME-READINESS CLAIM**

## Why this measurement exists

NCGP10 proved close CPU/GPU correspondence for the compensated pressure-f64
route, but both routes fragmented in the frozen hydrostatic physical corpus.
The user explicitly authorized measuring the cost of that implementation before
repairing the common pressure model. NCGP11 therefore preserves the NCGP10
physical refutation and measures identical reset single steps only.

This result does not mean that the simulated state is correct water. It does
not include rendering, PhysX or engine integration, and it does not change the
CPU DFSPH product fallback.

## Frozen identity and host

- Contract:
  `docs/plans/nonlocal-gpu-full-step-performance/09-invalid-physics-cost-only.md`
- Source commit: `982c923577ab8870392256dd8263917de20ff8e0`
- Source tree: `88eace9d7e5000b1dfe5b80eb4fbf78c028ad409`
- Contract root:
  `85cf45bfb0eb2f5bc44dd68bbe2d7c1c69d6482444cb842dd3a6e0e500476f09`
- Source root:
  `628211fefdbcecc6d56ef5c4e2131bcf49470978dd1ea068f8c18423d57b59d3`
- Two clean Release binaries were byte-identical:
  `9b45d158e22c35bf6b5722d79b7b65d52b3c40349140d08b5dcb97cee2e2ebfe`
- GPU: NVIDIA GeForce RTX 3080, SM 8.6.
- CUDA runtime/driver reported by the harness: `13030/13030`.
- Compiler: GCC 15.2.0 and CUDA 13.3.73; fmad, FTZ and fast-math disabled by
  the frozen flags.

Clean build directories:

- `/tmp/nextengine-ncgp11-build-vr9wG3`
- `/tmp/nextengine-ncgp11-build-b-zMN18l`

Both clean binaries ran the three-profile probe successfully and reproduced
identical input, work and step roots. Timing values are intentionally not
expected to be byte-identical.

## Measurement protocol

For each profile and each of two fresh processes, the harness performed 16
conditioning, 8 warmup and 32 measured invocations. Before every invocation it
restored the same canonical input with an untimed upload, then measured one
complete pressure-f64 GPU solver step with CUDA events. No outliers were
removed and nearest-rank p50/p95/p99 used indices `15/30/31`.

The primary window contains dynamic graph rebuilds, density/active-set,
energy/gradient, all HVP and reductions, trust-region decisions, boundaries,
integration and publication. Reset upload, CPU oracle, observers, JSON,
renderer and PhysX are excluded. Reset/upload wall time is reported separately.

## Full-step distributions

All values are milliseconds.

| Samples | Process | p50 | p95 | p99 | Result root |
| ---: | :---: | ---: | ---: | ---: | --- |
| 4,000 | A | 226.624 | 240.768 | 242.584 | `4d2275f74e8abab0bcd9543505de2b22fc630a234d6c466538e0f5df35462750` |
| 4,000 | B | 241.272 | 244.775 | 246.240 | `602b0486e6a331e5ee1c5c215d0b5761ba5263d87b13a1b4636259cbb9fb0a6c` |
| 16,000 | A | 356.440 | 371.452 | 382.950 | `4f670413735c1214e9caa8f7e6764c404de4898f25702f8dc3ac5103b00b918f` |
| 16,000 | B | 359.182 | 368.446 | 375.914 | `7d203fb03635be3164c15f5b16511c43b795011e966b8c79c97ced585a6e9baa` |
| 50,000 | A | 1110.827 | 1144.928 | 1175.501 | `c6298eb7b036f90fd3f677b6167f6e23d02cec5a6528d4a814ee63a85d66a2d2` |
| 50,000 | B | 1116.431 | 1164.745 | 1168.384 | `f731a92083cca474b91d3d6688f4c95d625ce5451aa14aebb4c9ee5579d5bfe8` |

The 50k result is about `0.86--0.90` solver steps per second. Against the old
standalone diagnostic budget, p95 is `286--291x` above 4 ms and p99 is
`195--196x` above 6 ms. The classification is therefore
`INVALID_PHYSICS_COST_ABOVE_ORIGINAL_BUDGET` in both processes.

## 50k stage breakdown

| Stage | Process A p95 | Process B p95 | Typical share |
| --- | ---: | ---: | ---: |
| Repeated dynamic graph builds | 528.817 ms | 533.902 ms | ~46% |
| Density + active set + energy + gradient | 243.043 ms | 246.294 ms | ~21% |
| All HVP | 348.801 ms | 353.503 ms | ~31% |
| Solver control + transfers + boundary/integration remainder | 24.701 ms | 30.363 ms | ~2% |
| Reset/upload integration tax, excluded | 11.579 ms | 11.439 ms | n/a |

The work receipt is exact across every measured invocation:

- 50,000 dynamic and 43,056 ghost samples;
- 5,711,868 directed pairs, maximum degree 123 of the admitted 256;
- 54 graph builds and 46 HVP applications;
- 13 outer trials, all 13 accepted;
- 137,251,397 allocated device bytes;
- work root
  `8ce569d45b4da924e53f761822452a6143e2e6b6bd327cb7213f4afa59f4dbc0`;
- step root
  `6111deb26795de3cfc84513ca6264f62204d559ba8cab99e8564ff5686a343a0`.

For comparison, 4k uses 54 graph builds/51 HVP and 16k uses 50 graph
builds/45 HVP. The current bottleneck is therefore architectural repetition,
not capacity or a single slow boundary kernel. A single full step repeatedly
reconstructs/evaluates the same graph-class work inside the nonlinear solve.

## Verification and remaining risk

Passed:

- two clean Release builds;
- byte-identical binaries;
- two fresh complete timing processes per workload;
- exact repeated work and step roots within and across processes;
- 50k capacity, maximum-neighbor and 128-HVP admission.

Not run:

- compute-sanitizer, because the frozen NCGP11 contract requires it before
  optimization use only when 50k is within the old timing budget;
- CPU oracle and physical corpus by design of this cost-only route;
- engine/runtime integration, rendering and PhysX.

The smallest evidence-backed performance successor is to eliminate redundant
graph rebuilds across evaluations whose accepted state has not changed, then
fuse compatible owner-row density/energy/gradient work and remeasure the same
frozen cost workload. That successor remains invalid-physics until the pressure
state model is repaired and independently passes the correctness corpus.
