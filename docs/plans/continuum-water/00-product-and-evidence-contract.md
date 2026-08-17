# W0 — Product and evidence contract

## Outcome

Freeze every input that can otherwise move after implementation: product
fixture, numerical profile, state classification, corpus, metrics, external
oracle, resource bounds, failure semantics and explicit non-goals. W0 is
documentation-only and proves no solver behavior.

## Water profile V1

| Field | Exact value |
|---|---:|
| Dimensions / units | 3D, SPEC-26 right-handed MKS, `+Y` up |
| Rest density | `1000 kg/m³` |
| Particle radius | `0.025 m` |
| Initial lattice spacing | `0.05 m` |
| Cubic-spline support radius | `0.1 m` |
| Uniform mass | `0.125 kg` |
| Time step | fixed `1/240 s` |
| Gravity | `[0, -9.81, 0] m/s²` |
| Density solver | minimum `2`, maximum `20`, mean error `<= 0.01%` |
| Divergence solver | minimum `1`, maximum `20`, mean error `<= 0.1%` |
| Canonical rounding | one checked `NearestTiesToEven` conversion per substep |
| Float execution | exact ADR-081 target/toolchain/FMA/rounding/subnormal/math/reduction/convergence profile |
| Hard capacity | `50,000` active samples for production; `100,000` only in stress report |

V1 has no warm start, surface tension, viscosity/vorticity model, variable
time step, emitter, sink, split/merge or moving region. Plane and box
analytical boundaries are mandatory. Sphere/capsule/moving boundaries begin
at W3 only when the rigid consumer needs them.

## Canonical and diagnostic classification

Exact canonical inputs/results:

- profile/scenario/tool revisions and SHA-256 roots;
- `SampleId`, sample count, tick/substep and failure code;
- published signed `i64` micrometre positions;
- published signed `i64` micrometre-per-second velocities;
- canonical sample order and state/report hashes.

Private `f64` density, residual, pressure/divergence factor, energy and
external-reference values are diagnostic inputs to declared metrics. They do
not survive a substep, choose identity/order or waive a canonical mismatch.
Uniform mass is profile-owned and sample mass is not duplicated in each state.

The float profile and adversarial Windows/Linux rounding/convergence fixtures
are frozen before W1 code. Two failed remediation cycles make the selected
authority research-only unless a separate fixed-point/soft-float decision lands.

## Product fixture

- Basin inner dimensions: `4 × 2 × 1 m`.
- Water depth: `0.75 m`.
- Nominal lattice: `80 × 40 × 15 = 48,000` samples.
- Production capacity: `50,000` samples.
- Dynamic crate: cube side `0.5 m`, exact authored mass `50 kg`, one stable
  `PhysicsBodyIdV1` at the future production boundary.
- Equilibrium displaced volume: `0.05 m³`; accepted immersion depth
  `0.20 ± 0.05 m`.
- Presentation: debug points/spheres and density/iteration/boundary overlays.
- Fallback: a separately authored dry basin selected only before activation.

The headless fixture stores one bounded production-command trace that pushes
the crate into the basin and interacts with it after float stabilization. Its
exact body ID, initial pose, command IDs, tick assignments, impulses and
expected checkpoints are frozen before W3 execution; direct test mutation is
forbidden.

## Reference corpus and metrics

| Scenario | Fixed measures | Pass threshold |
|---|---|---|
| Hydrostatic column | density distribution, centre of mass, boundary penetration, external-work balance | solver bounds; penetration `<= 2.5 mm`; normalized residual `<= 1%` |
| Free-fall block | pressure/divergence before impact | no artificial constraint failure or nonfinite value |
| 3D dam break | normalized front position and height curves | RMSE `<= 5%`; maximum error `<= 10%` |
| Still tank | long-horizon state root, density and energy trend | finite; solver bounds; no growing external-work residual above `1%` |
| Drain/orifice | mass flux curve and retained sample count | curve RMSE `<= 5%`; maximum error `<= 10%`; declared outlet accounting exact |
| Sealed boundary | sample count/mass and maximum centre penetration | count/mass exact; no leak; penetration `<= 2.5 mm` |

Energy and momentum metrics subtract declared gravity impulse, boundary work,
prescribed moving-body work and outlet flux. Naive conservation of a driven
or open scenario is not an admissible metric.

## Independent evidence

Use published DFSPH curves and an independently executed
[SPlisHSPlasH](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH)
checkout pinned to commit
`eccce86155776f6ac52d5080b1f720a52bf29450`. Compare aggregate curves,
convergence summaries and conservation values, not particle identity or exact
trajectory bytes. SPlisHSPlasH code is not copied or linked into Next Engine.
Its build, scene files, raw output and large comparisons live in an external
configured research store.

The primary algorithm source is the
[DFSPH paper](https://animation.rwth-aachen.de/media/papers/2015-SCA-DFSPH.pdf).
Any source/corpus change creates a new evidence profile; tolerances cannot be
widened after observing a failing run.

## Stop conditions

W1 cannot begin if any scenario geometry, horizon, output cadence, reference
curve source/hash, metric formula, hard capacity or expected failure code is
still implicit. A missing published curve is recorded as an explicit
`REFERENCE_NOT_AVAILABLE` case and replaced by the analytical invariant plus
independent-solver aggregate before implementation, not after a failure.

## Non-goals

Runtime integration, PhysX coupling, parallel speed, GPU, save schema,
streaming, surface rendering, surface tension, viscosity, adaptivity and
gameplay fluid queries.
