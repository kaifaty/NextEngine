# Nonlocal NSR3-B0R kernel-normalization evidence -- 2026-08-20

Status: `PASS / FCR2_NORMALIZED_OBJECTIVE_CANDIDATE / REPORT_ONLY`

## Outcome

`lattice-normalized-cubic-v1` passes every frozen B0R gate under the new
`nuv-variational-fcr2` objective identity. The implementation adds one
explicitly selected immutable factor and applies it identically to `W`,
`dW/dr` and `d2W/dr2`. The old FCR1 path retains scale one.

The selected factor is `7.985668078772472`. It is computed from at most 343
integer reference-lattice offsets at `H/dx=3`, before particles move.

## Formula and curvature controls

| Control | Gradient FD error | HVP FD error | Symmetry error | Dense product error | Result |
|---|---:|---:|---:|---:|---|
| compressed pair | `1.05e-9` | `7.04e-10` | `0` | `1.83e-16` | PASS |
| normal viscosity pair | `1.85e-9` | `2.32e-12` | `0` | `1.69e-16` | PASS |
| tangential viscosity pair | `7.11e-12` | `1.82e-12` | `0` | `3.81e-17` | PASS |
| repulsive surface pair | `2.20e-9` | `4.82e-11` | `0` | `7.01e-17` | PASS |
| attractive surface pair | `2.39e-9` | `1.98e-11` | `0` | `2.95e-17` | PASS |
| combined tetrahedron | `5.13e-8` | `1.16e-10` | `5.59e-17` | `1.90e-16` | PASS |

All gradient errors are below `1e-7`; every HVP, symmetry and dense-oracle
gate passes. Pair momentum residual is exactly zero in five controls and
`7.19e-17` in the combined control. The compressed pair still contains a
negative Hessian mode; the normalization does not hide the curvature that
motivated Newton-CG.

## Physical-density discriminator

The independent `7x7x7` lattice has 343 particles and 12,111 admitted unique
pairs, below `80N`.

| Observable | Result |
|---|---:|
| uncompressed center density ratio | `0.9999999999999978` |
| center ratio after `0.99` isotropic spacing compression | `1.0308294231753403` |
| active compressed particles | `81` |
| normalized internal momentum residual | `3.03e-17` |

This closes the B0 defect without changing physical rest density or particle
mass. Free-surface particles remain below rest density and use the already
selected compression-only rule.

## Trust and tape controls

| Case | Outer / accepted / rejected | Objective eval / HVP | Minimum accepted ratio | Stop | Result |
|---|---:|---:|---:|---:|---|
| pressure pair | `8 / 4 / 4` | `9 / 16` | `0.841661` | scaled displacement | PASS |
| combined tetrahedron | `11 / 11 / 0` | `12 / 24` | `0.968370` | scaled displacement | PASS |

Every accepted step has positive model and actual reduction. Both cases
terminate under a previously declared stop, remain monotonic and conserve
internal momentum.

At 512 particles A1 and `outer-state-hessian-tape-v1` remain bit-exact across
12 outer states and 46 HVP calls. The tape performs 12 explicit HVP
correspondence checks, stays inside both pair and memory capacities, and ends
at state SHA-256
`06336c4386c1b421a5b8cf543bf83eac463d31c4ebbde8496736ee2b62084119`.
No performance number is reused or remeasured as a correctness claim.

## Repeatability and historical immutability

- B0R result SHA-256:
  `f93e789354be170ca680d12a2e473d148a7b6dca6af13a6ee5cf246bb97bedd4`;
- two raw B0R reports:
  `177fd9f5a53c594921a9c7751403c7ce6182045d6c828cead3da1e4390b0946b`;
- two raw B0 reports remain
  `57b3f0dc0e7c0152be4c8591c8adb079d12db6bc9d5180b65e124571dd11919a`;
- all eight frozen FCR1/NSR raw hashes remain unchanged, including expected
  negative preconditioner/scaling discriminators;
- A1 and A2 implementation choices remain report-only research components.

## Decision

Select `nuv-variational-fcr2` as `FCR2_NORMALIZED_OBJECTIVE_CANDIDATE` for
subsequent CPU research. FCR1 remains immutable synthetic-objective history.

Proceed to the separately frozen NSR3-B1 manufactured multi-step contract.
This pass proves normalized density and exact single-step optimization; it
does not yet prove temporal convergence, physical viscosity/surface response,
static boundaries, hydrostatics, CUDA correspondence or production readiness.

