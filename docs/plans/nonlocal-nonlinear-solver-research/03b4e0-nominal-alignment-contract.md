# NSR3-B4E0 -- nominal alignment preflight contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / NO_TRAJECTORY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e0-nominal-alignment|v1|candidate=66e318cb69e0b0c0a3a40a2beafa2099ebf151287581b191a242e82dac6d6f3c|formula=nuv-variational-fcr2+split-static-boundary-r0|solver=nuv-newton-krylov-r0+outer-state-hessian-tape-v1|publication=e713a61649fc230b189fca9eda3628b69f9369c706df35f0a2b080a4bd189a70|reference-attestation=9cf5fc571fee7bc0be5585d9b467f90cd8a27f40d9cf39d6be999b0c466cbccc|reference-profile=ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746|scenarios=hydro,dam;orifice-excluded|alignment=ids,coordinates,boundary-set,dt,mass,density,gravity,schedule,canonical-q99|controls=id-swap,boundary-um1,schedule-step|trajectory=none|credit=b4e1-design-only
```

Identity SHA-256:
`c53a112830cb94c4139da75c2044d28f61673cb1aa05c116098967aee41f7bbe`.

## Authority and parent closure

R1E PASS selects the new external DFSPH reference candidate and B4C4C1 selects
the packaged Nonlocal implementation. B4E0 may add only a manifest/geometry/
neighborhood preflight to `nonlocal-formula-reclosure`:

```text
nonlocal-formula-reclosure --nominal-alignment-preflight
```

It must not open an external payload, execute a spectral estimate, construct a
KKT solve, advance time, publish a nonzero frame or authorize B4E comparison
execution. It creates no runtime/public/persistence schema and no CUDA or
production authority.

## Frozen shared profile

Both cases require exactly:

```text
samples=6000
spacing=0.05 m
radius=0.025 m
horizon=0.15 m (Nonlocal candidate only)
mass=0.125 kg
rest_density=1000 kg/m^3
macro_dt=1/240 s
gravity=(0,-9.81,0) m/s^2
initial_velocity=(0,0,0)
nonlocal_terms=pressure-only;lambda=mu=gamma=0
```

Fluid positions and candidate sample IDs use loop order `iy,iz,ix` with `ix`
fastest and coordinate `0.025 + 0.05 * index`. This is not the existing tiny
fixture helper's `iz,iy,ix` order.

| Scenario | Box cells | Fluid cells | Support count | Fluid root | Boundary root | Schedule | R1D scenario root |
|---|---:|---:|---:|---|---|---|---|
| Hydro | `20x20x20` | `20x15x20` | 5,824 | `7d4e661d08de08b18d43a76342329b51f6ae98bca9baee3d850e0f403eae5606` | `25de85b5eeec041c12bbb5de10e00b8374457b4cf09cb61d99dfc4d5511d8d62` | `0..1200/every24` | `c430b679dfeec33a6ac12c51df75ddee7e7bc48484c6c05188219f0a727909a0` |
| Dam | `80x20x20` | `20x15x20` | 16,384 | `9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76` | `1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d` | `0..720/every4` | `8d0a0a85adba50d4784245d460c83757bcce92841d804f4739b81faf27fd5f09` |

Support is the two-layer complement over cell indices `[-2,n+1]` on each
axis. Candidate input order may differ, but the canonical coordinate set must
equal the exact reference set with no duplicate or interior support sample.
Contact clearance is `[0.025, box_extent-0.025]` on every axis.

## Initial canonical controls

Without advancing time, construct one candidate canonical frame at step zero
and decode it. For both cases require:

```text
sample_count=6000
mass=750 kg
COM=(0.5,0.375,0.5) m exactly in canonical micrometres
q99_x=0.975 m
q99_y=0.725 m
velocity=(0,0,0) for every stable ID
all positions inside exact centre clearance
```

The frame/root and independently recomputed aggregate root must agree. Swap
stable IDs 0 and 1 and require the fluid projection/root to change.

## Neighborhood and capacity controls

Build one immutable static-support index and one candidate one-pass flat
neighborhood at the initial state. Require:

- one static index, no nested adjacency rows and one flat offset array;
- exact 6,000/5,824 or 6,000/16,384 participant counts;
- at most 160 neighbors per fluid sample and at most 960,000 stored pairs;
- finite initial evaluation and no all-pairs candidate evaluation/HVP;
- stable repeat of pair, static-index and flat-layout roots.

Change the first boundary x coordinate by exactly one micrometre in a private
fixture and require the boundary/set/static-index roots to change. Change the
final schedule step by one and require the scenario projection/root to change.
Neither mutation may reach a solver object.

## Report and exit

The canonical JSON report has no timing or host paths. Two fresh processes
must be byte-identical and bind the B4C4C1, publication, R1E and R1D identities,
both scenario projections, analytic initial aggregates, neighborhood work and
three mutations.

PASS selects only `NOMINAL_ALIGNMENT_CANDIDATE` and authorizes B4E1 one-macro
resource-probe research/contract design. It keeps
`trajectory_started=false`, `reference_curve_decoded=false`,
`b4e_comparison_execution_authorized=false`, `runtime_authority=false` and
`production_authority=false`. Any mismatch stops before B4E1.
