# 01 — CPU DFSPH water reference

## Outcome

Build an offline/runtime-independent Rust laboratory that produces a bounded,
repeatable free-surface water trajectory corpus. It is the numerical oracle for
later boundaries, GPU mirrors and regressions; it is not a production physics
backend and adds no `crates/contracts` type.

## Required design

- Dedicated lab crate behind an explicit tool command; Rust 1.97.1, safe Rust.
- Material-specific SoA: stable `SampleId`, position/velocity `f64`, mass,
  density, pressure/divergence accumulators and active flags.
- Fixed smoothing length, particle mass, time step and substep count per exact
  profile; no adaptivity, variable time step, viscosity model zoo or GPU path.
- Compact-support kernel and gradient selected once in the profile.
- Neighbor structure is rebuilt cache. Cell and pair traversal sort by stable
  cell key then `SampleId`; no hash-map iteration affects a reduction.
- DFSPH divergence solve followed by density solve, each with explicit
  tolerance, minimum/maximum iterations and stable residual reduction.
- Nonfinite state, capacity excess or non-convergence rejects the whole step
  and retains the previous accepted frame.
- Corpus artifacts live outside Git; checked-in fixtures contain only small
  canonical summaries/golden hashes when implementation begins.

## Initial scenarios

| Scenario | Primary measure |
|---|---|
| Hydrostatic column | density error, pressure stability, center-of-mass drift |
| Free-fall block | absence of artificial pressure before impact |
| 3D dam break | front position, mass, kinetic/potential energy trend |
| Still tank | long-horizon velocity/density drift |
| Drain/orifice | mass flux and no boundary leakage |

Use at least one published/independently generated reference curve where
available; visual plausibility alone is not a pass condition.

## Evidence and exit

`CONTINUUM-WATER-REF-P1` records profile hash, tool commit, scenario inputs,
per-step convergence, conservation metrics and trajectory hash. Repeat runs,
insertion-order permutations and worker counts 1/2/4 must agree exactly on the
declared CPU profile. Exit requires all scenarios to remain finite and within
predeclared error bounds at a chosen particle budget. Performance is reported,
not gated, in this package.

## Explicit non-goals

Rendering, PhysX coupling, save formats, streaming, GPU execution, adaptive
particles, thermal/chemical effects and gameplay queries.
