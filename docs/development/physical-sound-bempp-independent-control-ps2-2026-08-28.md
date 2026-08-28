# Physical sound PS-2 — independent Bempp analytical control

Date: `2026-08-28`

Status:
`INDEPENDENT_BEMPP_ANALYTICAL_CONTROL_SUPPORTED / ALL_SEVEN_GATES_PASS / BYTE_IDENTICAL_REPEAT / ANALYTICAL_SPHERE_ONLY / NO_FRESH_REALIMPACT_DATA_OPENED`

## Purpose and protocol

Two repository collocation variants produced small absolute sphere-field error
but failed mesh refinement. The regular seven-point rule did not repair it, so
this experiment changes the adjacent layer instead of tuning that solver again:
an independent Galerkin implementation with singular quadrature and a direct
Neumann-to-Dirichlet formulation.

The external Python 3.12 environment pins:

| Component | Revision |
| --- | --- |
| Python | `3.12.13` |
| Bempp-cl | `0.4.2`, Git `a1eaaef9f96b9dd3d7c56b076740e06852a6e1c0` |
| NumPy / SciPy | `2.5.2` / `1.18.1` |
| Numba / llvmlite | `0.67.0` / `0.49.0` |
| meshio | `5.3.5` |
| device interface | CPU `numba` |

Manifest SHA-256
`33d30a3579ee3abfb2e9b1081a08ff89271f621ca3199c3ac2dd8e4fe9e13873`
freezes the same radius, three `ka` values, three listener radii and ten
directions as the repository control. Bempp regular-sphere levels `2/3` provide
`128/512` panels. The direct formulation is:

`W p = (-0.5 I - K′) q`, followed by `field = -S q + D p`.

`q` is the prescribed unit pressure normal derivative. Boundary assembly uses
`default_nonlocal`, potentials use `dense`, precision is double and GMRES is
frozen at relative tolerance `1e-10`, restart `50`, maximum `500` iterations.

The first manifest revision tried the boundary assembler identifier for
potential operators and stopped before any field result. V2 separated the two
documented Bempp APIs before metrics were opened; physical fixture and gates
did not change.

## Implementation and result

[`physical_sound_bempp_control.py`](../../lab/scripts/physical_sound_bempp_control.py)
validates the manifest and exact runtime versions, rejects repository-local
outputs, solves the six mesh/frequency systems, compares 180 complex field
conditions with the analytical pulsating-sphere solution and atomically
publishes canonical JSON.

Runs `run-a` and `run-b` are byte-identical:

| Artifact | SHA-256 |
| --- | --- |
| manifest A/B | `33d30a3579ee3abfb2e9b1081a08ff89271f621ca3199c3ac2dd8e4fe9e13873` |
| report A/B | `ba638a21942d3cb518a4a1bb58606090eaac5338d143a9135aebcef9cc4f01f1` |

| Metric | 128 panels | 512 panels | Frozen fine gate |
| --- | ---: | ---: | ---: |
| Median relative complex error | `0.0422383` | `0.0110873` | — |
| Maximum relative complex error | `0.0485931` | `0.0127100` | `<=0.05`, pass |
| Maximum magnitude error | `0.432612 dB` | `0.111091 dB` | `<=0.25 dB`, pass |
| Maximum phase error | `1.88170°` | `0.484326°` | `<=2°`, pass |
| Maximum direction span | `0.025118 dB` | `0.007352 dB` | `<=0.01 dB`, pass |
| Fine/coarse median ratio | — | `0.262493` | `<=0.7`, pass |

All six GMRES solves return `info = 0`; the worst recorded final residual is
`7.016e-11` against `1e-8`. Decision:
`IndependentBemppAnalyticalControlSupported`.

## Claim boundary and next step

This closes one narrow prerequisite: a pinned independent classical solver can
produce a convergent complex acoustic field for an analytical sphere. It does
not yet prove a non-spherical surface mode, FEM eigenmode coupling, real-object
transfer, material identity, perceptual quality, admission or runtime cost.

Keep REALIMPACT and both ceramic holdouts sealed. Next freeze one synthetic
non-spherical prescribed surface mode with a Bempp field as immutable target,
then test whether the repository cooker can reproduce its near/far directional
transfer. Only that success may authorize a fresh object-disjoint real-data
calibration.

Primary references:

- [Bempp Helmholtz boundary operators](https://bempp.com/handbook/api/boundary_operators.html#boundary-operators-for-the-helmholtz-equation)
- [Bempp-cl source at the frozen revision](https://github.com/bempp/bempp-cl/tree/a1eaaef9f96b9dd3d7c56b076740e06852a6e1c0)
- [NeuralSound classical BEM formulation](https://github.com/hellojxt/NeuralSound/blob/b18e81b1e3dba7e963b09a7d9a45b584707d805f/src/classic/bem/bemModel.py)
