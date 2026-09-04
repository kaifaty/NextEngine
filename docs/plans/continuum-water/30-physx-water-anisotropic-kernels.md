# PhysX water lane — anisotropic kernels from the fluid (ADR-106 step 8)

| Field | Value |
| --- | --- |
| Research ID | `PHYSX-WATER-PRESENT-R7` (research report, not a product check) |
| Status | `RUN / G1-G5 PASS at revision 2` (2026-09-04; revision 1 failed G3) |
| Parent | ADR-106 (Accepted 1.0); plans 25-29; plan 24 item 3 (kernels); ADR-102 revision 4 of the particle pass (the `kernels` field, empty so far from the lane) |
| Purpose | the lane fills the particle pass's per-particle kernel from the fluid's own neighbourhood (Yu and Turk 2010), so a sheet of particles renders as a sheet and a bead as a bead, instead of every particle as a sphere at the profile radius |

## Frozen scope

- **Kernel (game, `apps/game/src/physx_water.rs`).** The neighbour sweep
  of plan 27 (linked cells, forward half, radius `h = 2 × spacing =
  0.1 m`) also accumulates, per particle, the weighted sums of the
  neighbour offsets with the weight `w = 1 − (d/h)³` (self included with
  weight 1). The weighted covariance `C = Σw·δδᵀ/Σw − mmᵀ` (with `m` the
  weighted mean offset) is diagonalised by cyclic Jacobi rotations; the
  semi-axes are `r_i = KS · √λ_i` with `KS = 1` (a uniform ball of radius
  `h` gives `√(h²/5) ≈ 45 mm`, the profile radius), the smallest axis is
  raised to `r_max / KR` with `KR = 4`, and every axis is clamped to
  `[0.25, 2] × profile radius`. The kernel sent is `G = R·diag(1/r_i)·Rᵀ`
  packed `xx xy xz yy yz zz` (inverse metres, the pass's convention). A
  particle with fewer than `KERNEL_MIN_NEIGHBOURS = 8` neighbours sends
  the zero kernel (the pass's isotropic fallback); spray particles are
  discarded by the pass anyway.
- **Statistics.** The stderr line gains `kernels_last_permille` (particles
  with a non-zero kernel). The run report of plan 28 is unchanged.
- **Not in scope.** Kernel smoothing over frames, Laplacian smoothing of
  the centres (Yu and Turk's second step), a GPU implementation.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `host-check`, `play`, `persistence-replay`, `water-present` PASS |
| G2 unit | the Jacobi solver returns the eigenvalues of a diagonal matrix and of a rotated one within `1e-5`; an isolated particle sends the zero kernel; a flat sheet (grid at spacing 0.05 m, one layer) yields the vertical semi-axis as the smallest with `r_max / r_min ≤ 4`; a filled ball yields all three axes within 10 % of each other |
| G3 cost | the pour demo (`--physx-water --physx-water-pour --maximum-frames 120`, alone on the host): `analysis_mean_us ≤ 2500` at peak ≥ 5,000 particles |
| G4 look | capture of the pour demo at frame 100 read by a human against the plan 29 capture of the same frame: the block and the sheet over the level read as a continuous surface, not beads; recorded as an observation either way |
| G5 clean run | `--physx-water --maximum-frames 120`: `active: true`, `peak_particles > 0`, `kernels_last_permille > 0` |

## Result (2026-09-04)

### Revision 1 (the frozen sweep, cyclic Jacobi)

| Gate | Reading | Verdict |
| --- | --- | --- |
| G2 unit | solver, isolated particle, flat sheet (vertical axis smallest, ratio ≤ 4), filled ball (axes within 10 %) all pass | pass |
| G3 cost | pour demo: `analysis_mean_us 3697` at peak 5,270 particles (`kernels_last_permille 803`) | **fail** |

The timing probe (`bench_neighbourhood`, 5,000 particles on a jittered
0.05 m lattice, 25 neighbours each) read 4.1 ms: the Jacobi solver alone
1.5 ms (300 ns per particle), the pair sweep 2.1 ms, the covariance and
scatter 0.5 ms.

### Revision 2 (apparatus, same kernel definition)

Recorded corrections, none of them touching the frozen kernel (weights,
`KS`, `KR`, the clamp band, the neighbour floor):

1. The closed trigonometric form for the eigenvalues and cross products
   of `A − λI` for the vectors, re-orthogonalised for near-repeated values;
   the Jacobi solver stays as the test oracle (2,000 random and degenerate
   matrices agree within `1e-3` relative, vectors orthonormal). 80 ns per
   particle.
2. Particles counting-sorted by cell into structure-of-arrays positions;
   the candidate distances of a particle against a cell run computed in a
   branch-free block. Alone this changed little: the sweep's cost is the
   hit work, not the loads.
3. The sweep split across up to four scoped threads by contiguous ranges
   of sorted particles over the full 27-cell neighbourhood, each writing
   only its own particle's count, moments and kernel; the `a < b` pairs are
   edges for the sequential union-find of the clusters. The probe reads
   2.0 ms.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `host-check`, `play` (root `5f0c8bcd…`), `persistence-replay` (root `03901d76…`), `water-present` PASS | pass |
| G2 unit | as revision 1, plus the oracle comparison; the plan 27 brute-force parity test of counts and clusters still passes on the new sweep | pass |
| G3 cost | pour demo alone on the host: `analysis_mean_us 2176`, `analysis_max_us 4172`, peak 5,270, `kernels_last_permille 746` | pass |
| G4 look | frame 100 of the pour demo against the plan 29 capture of the same frame (the same seed, the same camera): the splash reads as streaks and sheets where revision 0 read as beads; the bulk over the level is a continuous mass with fewer visible spheres. From the water start the camera is far enough that the change is modest; a closer read needs a capture from inside the basin (not available without input) | pass, observation |
| G5 clean run | `active: true`, `peak_particles 608`, `kernels_last_permille 1000`, `analysis_mean_us 330` | pass |

Observation: the lane's own step cost (`cost_mean_us`, the fluid step,
read and set) read 1.1 ms in both sessions of revision 2 against 0.7-0.8
ms before; the sweep's threads share the cores with the CUDA driver's
thread and the game's worker. Recorded, not tuned.
