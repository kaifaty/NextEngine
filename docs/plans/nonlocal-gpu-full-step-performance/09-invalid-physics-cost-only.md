# NCGP11 invalid-physics cost-only benchmark

Status: **FROZEN BEFORE IMPLEMENTATION**
Profile: `nonlocal-water-50k-invalid-physics-cost-only-v1`
Date: 2026-08-31

## Purpose and claim ceiling

NCGP10 established that the compensated pressure-f64 CUDA route and its
independent CPU route agree closely, but the shared penalty-only physical model
fragments during the hydrostatic corpus. The user explicitly authorized a
separate benchmark to obtain implementation-cost numbers before that model is
repaired.

Every NCGP11 result is therefore labelled
`INVALID_PHYSICS_COST_ONLY / NO_WATER_QUALITY_CLAIM`. It may answer how much GPU
time the current full-step implementation consumes. It must not establish
correct water, a passing physics corpus, game readiness, integrated frame time
or roadmap completion. NCGP10 remains `PHYSICS_REFUTED_BOUNDED`; this benchmark
does not replace or weaken that result. CPU DFSPH remains the product fallback.

## Frozen implementation route

- Use `nonlocal_water_corrected_profile()` without coefficient or tolerance
  changes: `dt=1/240 s`, `spacing=0.05 m`, `horizon=0.15 m`, mass `0.125 kg`.
- Use the reviewed compensated binary32 state and
  `CompensatedScalePressureF64` arithmetic variant.
- Use the unpreconditioned Steihaug--Toint solver with exactly 128 total HVP.
- Use the existing dynamic two-pass canonical CSR graph, three ghost layers,
  analytical swept boundary projection and full state finalization.
- Allocate one `NonlocalGpuWorkspace` before timing. No allocation or workspace
  resizing is allowed inside a measured step.
- Do not modify CUDA kernels, solver rules, physics, graph ordering or work
  limits to improve the result of this benchmark.

## Frozen workloads

All samples and ghosts are canonicalized through the existing binary32 input
route. The three profiles are:

| ID | Lattice | Dynamic samples |
| --- | --- | ---: |
| `cost-4k` | `20 x 20 x 10` | 4,000 |
| `cost-16k` | `40 x 20 x 20` | 16,000 |
| `cost-50k` | `50 x 40 x 25` | 50,000 |

The 50k profile is the requested exact workload. The smaller profiles expose
scaling and give useful numbers if 50k fails capacity or work admission.

Before any timing for a profile, run one measured capacity/work probe. It must
report input root, sample/ghost counts, directed pairs, maximum degree,
allocated device bytes, HVP used, outer trials and the exact typed failure. A
capacity, nonfinite, CUDA, neighbor overflow or work-budget failure stops that
profile and is a valid benchmark result; it must never be truncated or retried
to green.

## Reset and measurement window

Each invocation starts from exactly the same immutable initial state bytes:

1. re-upload the canonical state and ghosts outside the primary timing window;
2. invoke exactly one full GPU step with CUDA-event measurement enabled;
3. synchronize the stop event and retain its work/failure receipt;
4. discard the resulting trajectory state before the next invocation.

The untimed reset makes this a repeatable single-step cost benchmark. It is not
a 240-step trajectory benchmark. Reset/upload wall time is measured separately
as integration tax and is never added to or hidden inside the primary result.

The primary full-step window includes:

- current/reference graph rebuilds used by the step;
- density, active-set, energy and gradient evaluation;
- every HVP and solver reduction;
- host scalar trust-region decisions and their synchronization;
- swept boundary/contact handling, state integration and final publication.

The current instrumentation publishes four disjoint diagnostic buckets:
`graph_ms`, `density_energy_gradient_ms`, `hvp_ms`, and
`step_remainder_ms = total_ms - the first three`. The remainder includes solver
control, scalar transfers, boundary and final integration; it must not be
mislabelled as boundary-only time.

Excluded: process startup, workspace allocation, reset upload, CPU oracle,
trajectory/visible-surface observers, JSON serialization, renderer, PhysX and
future runtime integration. These exclusions must be printed in the result.

## Sampling and statistics

Run, in order, for each admitted profile:

- 16 conditioning invocations;
- 8 warmup invocations;
- 32 measured invocations;
- a second fresh process with the identical protocol.

Every invocation is synchronized. No sample is removed. There is no retry to
green. Sort the 32 measured values and publish nearest-rank p50/p95/p99 using
indices `15/30/31` respectively, plus minimum, maximum and arithmetic mean.
Publish the same distribution for total, graph, density/energy/gradient, HVP,
remainder and untimed reset/upload wall time.

For the first bounded implementation pass it is permissible to run a single
probe process before committing to the full two-process window. The final
claim must distinguish `PROBE_ONLY` from `TWO_PROCESS_COMPLETE`.

## Classification

The original standalone compute budget remains diagnostic, not a correctness
gate:

- `INVALID_PHYSICS_COST_WITHIN_ORIGINAL_BUDGET` only if both fresh processes
  complete and each has `p95 <= 4 ms` and `p99 <= 6 ms` for total GPU step;
- `INVALID_PHYSICS_COST_ABOVE_ORIGINAL_BUDGET` if both complete but either
  process exceeds a bound;
- `INVALID_PHYSICS_CAPACITY_OR_WORK_REFUTED` for typed admission/work failure;
- `INVALID_PHYSICS_COST_INCONCLUSIVE` for apparatus, identity or incomplete
  sampling failure.

No classification above changes R8, SPEC-38, ADR-076/081 or the product
fallback. A later corrected pressure-state model must pass its own correctness
corpus and be benchmarked again.

## Identity and evidence

The versioned JSON report must include contract/source/commit/tree/binary,
profile and input roots, executable command, GPU/CUDA environment, compiler
flags, all sample counts, work/capacity fields, per-stage distributions,
integration-tax distribution and the explicit claim ceiling. A result root
binds those fields and the raw timing samples.

Before a final NCGP11 handoff run two clean Release builds. If 50k falls within
the original timing budget, run `compute-sanitizer` memcheck/initcheck/synccheck
before using the result as optimization input. This cost-only experiment does
not update `docs/roadmap.md`.
