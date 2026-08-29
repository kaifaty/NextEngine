# Nonlocal continuum next-performance research — 2026-08-20

Status: `REPORT_ONLY / ROADMAP_INPUT / NO_W2_CREDIT`

## Question

Which algorithms and data layouts can credibly move the retained Nonlocal GPU
prototype from one coherent 48k feasibility fixture to an exact 50k,
dynamically representative `4/6 ms` p95/p99 reclosure, and which ideas should
remain later research rather than enter the immediate implementation queue?

## Local evidence that controls the answer

The closed NR4 identity records water-48k p95 `4.019520 ms`, but that number is
not the production stop target:

- it is `48,000`, not the hard `50,000` scale;
- it has no decision p99;
- it measures one boundary-free regular lattice generated in lexicographic
  `z/y/x` order;
- every execution begins from the same state and freezes neighbors from that
  initial position;
- it does not measure a multi-substep advected trajectory or a basin boundary;
- it remains report-only GPU work with no canonical/runtime authority.

The final profiler is nevertheless decisive about priority. Density,
incompressibility and viscosity pair kernels own about `84.0%` of water-48k
and `94.3%` of viscous-16k GPU kernel time. Neighbor visitors add `11.8%` and
`3.1%` respectively. Clear, handoff and launch work do not justify another
generic launch-cleanup phase.

O4 also provides a useful negative result: spatial reordering is slower when
the input is already a coherent lattice and exact owner-neighbor association
must be preserved. It does not answer whether sorting helps after stable sample
IDs become spatially disordered over many substeps.

## Primary-source findings

| Source | Finding used here | Boundary |
|---|---|---|
| [Nonlocal Unified Variational Framework paper](https://peridynamics.com/publications/2026-Liu-NUV.pdf) and [pinned PeriDyno implementation](https://github.com/peridyno/peridyno/tree/1aa892bb296fe766d2f9249c881b8605af23a69b/examples/Cuda/UnifiedFluid) | SISSM repeatedly performs density and local pair reductions; iteration count is a first-order cost | does not provide a realtime 50k game profile or unconditional global convergence |
| [CUDA C++ Best Practices Guide](https://docs.nvidia.com/cuda/cuda-c-best-practices-guide/) | coalesced global access, avoiding redundant loads, shared-memory reuse and measured register/occupancy trade-offs are the correct GPU questions | generic advice is not evidence that a particular kernel wins |
| [LAMMPS neighbor-list design](https://docs.lammps.org/Developer_par_neigh.html) | a Verlet-style cutoff plus skin can amortize list rebuilds; rebuild is triggered after sufficient displacement; spatial sorting is periodic rather than unconditional | transfer requires an exact Nonlocal membership/order proof |
| [HOOMD-blue GPU neighbor-list paper](https://doi.org/10.1016/j.cpc.2016.02.003) and [neighbor-list documentation](https://hoomd-blue.readthedocs.io/en/v2.9.0/nlist.html) | uniform cells are strong for dense, near-uniform cutoffs; stencils/LBVH become useful for asymmetric or sparse systems | current Nonlocal water has one uniform horizon, so LBVH is not the first candidate |
| [Anderson acceleration for geometry optimization and physics simulation](https://arxiv.org/abs/1805.05715) | guarded Anderson acceleration can reduce fixed-point iterations with an energy-based safeguard in related nonlinear physics solvers | applicability to SISSM must be demonstrated; it is an algorithm change |
| [Adaptive Particle Fission-Fusion for Dual-Particle SPH](https://doi.org/10.2312/pg.20251269) | the method reduces *virtual pressure samples* in a Dual-SPH projection and adds warm start | it is not adaptive material-particle resolution and cannot be transplanted into Nonlocal by analogy |
| [Mass Preserving Multi-Scale SPH](https://graphics.pixar.com/library/MultiScaleSPH/paper.pdf) | local high resolution can reduce particle count substantially when mass exchange is explicitly closed | the method is SPH-specific and does not prove a conservative Nonlocal split/merge |
| [Authors' current publication page](https://peridynamics.com/publications.html) | `Semi-Implicit Pairwise Descent for Nonlocal Continuum Mechanics` is still `Paper (to appear) / Code (to appear)` on 2026-08-20 | no formula or implementation may be inferred from the title |

## Ranked engineering conclusions

### 1. Freeze representative work before optimizing it

This is the immediate priority. A 0.49% p95 miss on the old 48k fixture is not
evidence that only a tiny optimization remains: exact 50k, p99 and dynamically
disordered storage may move the baseline in either direction. The next corpus
must distinguish coherent geometry, identical geometry with permuted stable
IDs, and an advected multi-substep state.

Neighbor membership must be rebuilt from the prior substep position and then
frozen only for the current nonlinear solve. The report must expose pair count,
degree distribution, storage distance and rebuild frequency for every state.

### 2. Fuse compatible pair traversals before exotic structures

Water and viscous profiles currently traverse the same directed CSR separately
for incompressibility and viscosity. A fused owner-gather kernel can compute
separate per-term accumulators during one adjacency walk and combine them in
the original term order after the loop. This can remove one full neighbor-data
walk without requiring atomics or unique-pair fragments.

The candidate starts with an arithmetic-association audit. If compiled code
cannot preserve the retained output, it becomes a separately rooted numeric
candidate and must hit the stiff-surface i2 gate before any timing. Density
cannot be fused across its global dependency merely to remove a launch.

### 3. Reduce bytes per directed edge

The hard first profile has at most 50k stable samples, so neighbor IDs fit in a
checked `u16` while CSR offsets remain `u32`. This is ordinary coordinate
compression applied to a bounded graph: it halves neighbor-index traffic and
roughly saves 10 MiB around the observed 5.2M directed-edge scale. Decode must
be exact, capacity must reject `> 65,535`, and the 100k stress profile retains
`u32`.

Aligned SoA/vector loads, launch-block variants and register-pressure reports
belong in the same memory-path tournament. Shared-memory tiling is admitted
only when a measured cell/owner tile reuses neighbor data; copying a value once
per consumer into shared memory is not reuse.

### 4. Re-test locality only on the workload O4 did not cover

Stable cell/radix sorting, packed cell ranges, prefix sums and CSR are the
useful competitive-programming transfers. They already exist in part. A new
layout candidate must be evaluated on permuted and advected states and pay for
its map/sort cost. The coherent-lattice O4 result is retained as a negative
control; it is not rerun in search of a different answer.

Morton/Hilbert order, LBVH, k-d trees and generic hash maps are not first-line
candidates. The horizon is uniform and the fluid is dense; more complex
structures become admissible only if a future sparse or variable-horizon
profile falsifies the uniform-cell assumption. Mo's algorithm, DSU and other
offline query techniques do not fit a mutating floating-point trajectory.

### 5. Amortize neighbor construction with a certified skin

A Verlet-style superset list can be reused while every sample stays within a
declared displacement bound. Every pair kernel still applies the exact current
horizon filter, and active neighbors are emitted in canonical stable-ID order.
The rebuild decision, maximum displacement reduction, skin size, extra-pair
ratio and amortized cost are part of the profile.

This is bounded upside, not the main rescue: neighbor work is about 12% of
water-48k kernel time and much less in the viscous profile. It follows the pair
fusion/memory path rather than preceding it.

### 6. Reduce iterations under a new solver identity

For five-iteration water, adaptive stopping has limited headroom. It is much
more promising for the 20–60 iteration viscous/surface regimes that motivate
Nonlocal. The order is:

1. derive a normalized fixed-point/KKT residual and stagnation/divergence
   classes;
2. create a high-iteration CPU `f64` stationary-result corpus;
3. measure adaptive checks every fixed cadence, including reduction cost;
4. only then test guarded Anderson acceleration;
5. audit Pairwise Descent only after a public paper or code release.

Position delta alone is not a residual. Warm start is deferred because hidden
continuation state would change restore/replay semantics; any later candidate
must be reconstructible from canonical input or explicitly specified as state.

### 7. Treat world-scale reduction as a separate representation program

Active high-fidelity domains, coarse far fields and true material-particle
split/merge can yield orders of magnitude, but they no longer execute the same
50k workload. They require mass, momentum, angular momentum, energy/error,
identity, handoff and repeated split/merge receipts. They cannot be used to
claim the standalone gate by simulating fewer samples.

The 2025 fission-fusion paper is useful evidence for adaptive auxiliary
pressure sampling, not for Nonlocal material adaptivity. A future Nonlocal
resolution lane therefore begins with conservation algebra and a tiny
split/merge oracle, not with porting that code.

Multi-GPU is later still. A bounded 50k local domain should first saturate one
device; halo exchange and per-iteration synchronization are unjustified until
a measured domain exceeds single-device capacity or time.

## Efficient use of the machine

The next roadmap avoids another long, low-information `run2`:

- one persistent process owns preallocated device buffers and batches all
  measured substeps;
- CPU fixture/oracle preparation and report hashing use worker threads while
  the GPU is not in a decision timing window;
- candidate builds may compile in parallel, but GPU timing is serialized to
  avoid contention;
- tiny and stiff-surface failure gates run before any percentile campaign;
- exploratory tournaments use `32` warm-ups and `96` alternating samples;
- only retained finalists receive `64` warm-ups, at least `512` measured
  samples and multi-substep p95/p99 evidence;
- a failed adjacent gate stops the family without a long corpus retry.

This increases resource use where parallelism is valid while preserving clean
GPU measurements.

## Recommended roadmap shape

```text
NP0 exact-50k + dynamic corpus
  -> NP1 exact-work pair/memory/topology tournament
      ├─> NP4 fixed-work performance decision
      ├─> NP2 residual + convergence acceleration -> NP4 algorithm decision
      └─> NP3 conditional active-domain/adaptive-resolution research
```

NP0 and NP1 are the immediate implementation queue. NP2 is a separate numeric
profile and is needed only for an algorithm-changing claim. NP3 is a side lane
and is not allowed to delay or relabel the exact 50k result. Pairwise Descent,
ML, multi-GPU and runtime integration remain watch/deferred lanes.
