# NCGP10 pressure-f64 4k corpus evidence — 2026-08-31

Status: `PHYSICS_REFUTED_BOUNDED / PERFORMANCE_NOT_RUN`

## Identity

- source commit: `d2d660d339b0706c576647916adae84834edded5`
- source tree: `4de8fc46193f8ff2c82c5b742aba2ba3a4d354d7`
- source root: `8292802346f354ae1737d7cab8e3358b73a64e43d46f14e8964a6a07d5873f1f`
- contract root: `c3cd3f963eeae7b7de96ab88fd8d804d58a81bb83682644a33372e884c394bed`
- clean Release binary: `c100933a9fd2b4d329fbdfc36654a6a8c05136194813be7f0d8b99f250544a74`
- repeated stdout SHA-256: `d1b7220033c5312f6a860fafdd533f11ed0279b807660fa0faa298d04fa3c188`
- host: RTX 3080, CUDA runtime/driver 13.3

Exact command:

```text
nonlocal-corrected-cuda-complete-4k --complete-4k-pressure-f64-corpus
```

## Pressure discriminator

The report first reproduces the exact NCGP9 primary-f32 failure, then passes
the predeclared narrow pressure-f64 route:

| Metric | primary f32 | f32 state + f64 pressure |
| --- | ---: | ---: |
| GPU / CPU active centres | `1362 / 978` | `978 / 978`, exact IDs |
| HVP relative L2 | `0.3275594955` | `5.4503722901e-7` |
| HVP cosine loss | `0.0473159273` | `1.4587528234e-13` |

This closes the GPU/CPU pressure-kink mismatch without changing state storage,
surface, viscosity, inertia, graph, solver or HVP ceiling.

## First physical failure

The pressure-f64 corpus completes 81 hydrostatic steps and stops on step 82 at
the frozen `visible_hydro_satellite` gate:

- corpus result root:
  `482e83749ee61ce1c6f04d730de4f267f22b642d3aa8831db51ed8d4ef39d10f`;
- scenario result root:
  `1b063d6fe3fe81280b1e59046320e823eda92d2179192211682faed38ee42b2e`;
- scenario receipt root:
  `bc6c2faf2c123364b6f5e393fd44e6253bbf71336fa0013e446f2e330eb23310`;
- CPU/GPU material components: `6 / 6`;
- CPU/GPU satellite area: `2.3592085% / 2.3597611%` against the frozen
  hydrostatic `<=1%` limit;
- CPU/GPU wet pixels: `17082 / 17078`;
- silhouette difference: `0.0468274%`;
- visible depth RMSE / p95 / p99: `1.6311 mm / 0.1496 mm / 0.4191 mm`;
- position RMSE / p99 / maximum through the failure: `0.1119 mm / 0.4456 mm /
  1.7069 mm`;
- density RMSE / maximum: `0.02688% / 0.27025%`;
- normalized momentum residual: `0.23521%`;
- positive energy excess and penetration: zero;
- GPU/permuted failure state and image roots: exact.

The separate GPU and CPU image roots differ as expected for independently
evolved trajectories, while their component count and satellite fraction are
nearly identical. This is not evidence of a CUDA correspondence defect. Both
routes produce the same macroscopic fragmentation, and the frozen absolute
hydrostatic topology gate correctly rejects it.

## Causal interpretation

Earlier local research already established that a uniform lattice under
gravity is a supported-column startup, not the discrete hydrostatic equilibrium
of the unilateral penalty. A finite penalty
`kappa/2 * max(rho/rho0 - 1, 0)^2` cannot carry nonzero hydrostatic pressure at
zero compression; water must first compress and launch acoustic motion. The
selected physical-water viscosity is intentionally tiny, so it does not hide
that startup with numerical damping.

The 2026 [Nonlocal paper](https://doi.org/10.1145/3799902.3811196) publishes the
unified position objective and selected examples, but not a resolution/SI
calibration or a hydrostatic-equilibrium theorem for this product profile. The
pinned [PeriDyno implementation](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/src/Dynamics/Cuda/ParticleSystem/SIUnifiedFluid/SemiImplicitUnifiedFluidSolver.cu)
uses fixed iterations and recomputes the unilateral density constraint; it
does not supply a persistent hydrostatic pressure state.

The existing CPU research selected a unilateral PHR augmented-Lagrangian
pressure state as the mathematical route able to represent positive pressure
at zero compression. That route has not reached a scalable, reviewed GPU
solver identity and cannot be silently substituted into NCGP10.

## Decision

NCGP10 closes `PHYSICS_REFUTED_BOUNDED`. Dam break, orifice, 16k/50k capacity,
the sealed 50k trajectory, sanitizers, independent performance review and all
full-step timing remain `NOT_RUN` by the frozen stop rule.

The next action needs a product decision:

1. fund a new pressure-state/augmented-Lagrangian solver program and later port
   the selected finite corpus to GPU; or
2. explicitly authorize timing the current penalty-only route as an invalid-
   physics cost diagnostic, without calling it water performance.

Changing the 1% topology gate or calling the uniform startup hydrostatic after
the result is rejected.

