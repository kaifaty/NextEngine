# Physical sound PS-2 — Bempp quadrupole surface mode and Rust cooker

Date: `2026-08-28`

Status:
`QUADRUPOLE_SURFACE_MODE_AND_OUTGOING_MULTIPOLE_COOKER_SUPPORTED / ALL_FROZEN_GATES_PASS / BYTE_IDENTICAL_REPEATS / NO_FRESH_REALIMPACT_DATA_OPENED`

## Purpose

The independent Bempp sphere control established convergence only for a
direction-independent monopole. This package tests the adjacent missing
capability without opening real data: a prescribed axisymmetric quadrupole
normal-derivative profile

`q(theta) = P2(cos(theta)) = (3 cos(theta)^2 - 1) / 2`

on the same sphere. It has directional lobes and exact zeros, while its
outgoing field remains analytically checkable:

`p(r, theta) = h2^(1)(k r) P2(cos(theta)) / (k h2^(1)'(k a))`.

The independent solver uses the pinned Python/Bempp environment from the
[sphere control](physical-sound-bempp-independent-control-ps2-2026-08-28.md).
The repository successor then fits an axisymmetric complex outgoing-multipole
basis only at `1.5a` and predicts the full field at `3a` and `10a`.

## Frozen protocol and setup lineage

The final source manifest SHA-256 is
`3a67f4add95dfc08fe7f55a08d9f6372fa478c1712a5c375910e23c1e2650e39`.
It freezes:

- Bempp-cl `0.4.2` revision
  `a1eaaef9f96b9dd3d7c56b076740e06852a6e1c0`, Python `3.12.13` and CPU/Numba;
- direct `W p = (-0.5 I - K′) q`, `field = -S q + D p`;
- regular-sphere levels `2/3`, or `128/512` panels;
- `ka = 0.25, 0.75, 1.5`, listener radii `1.5a, 3a, 10a` and 22 directions;
- polar, equatorial, intermediate `|P2| = 0.25` and eight exact nodal directions;
- radial pullback `x / ||x||`, an area-weighted mean-flux gate and explicit
  `1e-12` profile-classification tolerance;
- no fresh REALIMPACT access, network training, quality, admission or runtime credit.

Two pre-result protocol defects remain immutable negative lineage:

| Revision | Observation | Decision |
| --- | --- | --- |
| V1 manifest `935632ae…e745a`, report `81a394a4…08fa` | Evaluating `P2(x_z/a)` on planar triangles produced mean Neumann flux `−0.029605`/`−0.007791` on 128/512 panels, nodal leakage `0.8246` and correlation `0.5144`. | Setup reject. V2 changed only the pullback to `P2(x_z/||x||)` and added a zero-mean gate. |
| V2 manifest `6cd3b16c…0d3a`, report `1074d9be…0fd9` | The field passed, but floating `0.2499999999999999 >= 0.25` excluded 72 intended intermediate-lobe rows from active-only metrics. | Insufficient metric coverage. V3 added only an explicit `1e-12` classification tolerance; field samples, thresholds and solver were unchanged. |

V1/V2 outputs are retained externally and receive no final checkpoint credit.

## Independent Bempp result

Runs `run-a` and `run-b` are byte-identical:

| Artifact | SHA-256 |
| --- | --- |
| source manifest A/B | `3a67f4add95dfc08fe7f55a08d9f6372fa478c1712a5c375910e23c1e2650e39` |
| Bempp report A/B | `e8e1d4d594c0062979dafe6bcf3b29e72f3125f4e272e29dd41b7fb3d8816437` |

| Metric | 128 panels | 512 panels | Frozen fine gate |
| --- | ---: | ---: | ---: |
| Active / nodal conditions | `126 / 72` | `126 / 72` | exact intended coverage |
| Median peak-normalized complex error | `0.041954` | `0.011383` | — |
| Maximum peak-normalized complex error | `0.167902` | `0.045546` | `<=0.08`, pass |
| Maximum active relative complex error | `0.176203` | `0.048013` | `<=0.10`, pass |
| Maximum active magnitude error | `1.68360 dB` | `0.427383 dB` | `<=0.9 dB`, pass |
| Maximum active phase error | `0.784109°` | `0.204596°` | `<=5°`, pass |
| Maximum nodal leakage / peak | `7.43e-6` | `1.29e-6` | `<=0.03`, pass |
| Minimum directional complex correlation | `0.9999669` | `0.9999978` | `>=0.995`, pass |
| Absolute area-weighted Neumann mean | `1.09e-17` | `6.12e-18` | `<=1e-12`, pass |
| Fine/coarse median ratio | — | `0.271311` | `<=0.7`, pass |

All six GMRES solves return `info = 0`; maximum final residual is
`7.933e-11` against `1e-8`. Decision:
`IndependentBemppQuadrupoleSurfaceModeSupported`.

## Repository near-to-far cooker

Cooker manifest SHA-256
`e76d82cb9194cb0c3c8d8b8489825e29c03ab5b7e5038b8b68d68b9e16d398fc`
hash-binds the final Bempp manifest/report before fitting. For each of the
three frequencies, seven directions at `1.5a` fit complex coefficients for
`h_l^(1)(kr) P_l(cos(theta))`, `l = 0..3`. All 22 directions at both `3a`
and `10a` are held out. The implementation uses first-kind outgoing Hankel
functions, column scaling, complex ridge normal equations and deterministic
partial pivoting.

Runs `cooker-run-a` and `cooker-run-b` repeat at report SHA-256
`054901ee4281c140bfeafab67d3fa5f39079ffb3ec6ecb5f7faae4c3b371f871`.

| Held metric | Observed | Frozen gate |
| --- | ---: | ---: |
| Conditions / active conditions | `132 / 84` | exact intended coverage |
| Minimum degree-2 coefficient energy fraction | `0.999906812` | `>=0.98`, pass |
| Median peak-normalized complex error | `0.00123888` | — |
| Maximum peak-normalized complex error | `0.00785624` | `<=0.03`, pass |
| Maximum active relative complex error | `0.0299615` | `<=0.05`, pass |
| Maximum active magnitude error | `0.0518391 dB` | `<=0.45 dB`, pass |
| Maximum active phase error | `1.68978°` | `<=2.5°`, pass |
| Minimum directional complex correlation | `0.999837649` | `>=0.999`, pass |

Decision: `SurfaceModeOutgoingMultipoleCookerSupported`.

## Conclusion, claim boundary and next step

The pinned independent solver recovers a convergent directional surface mode,
including its nodal structure. A repository-owned complex outgoing-multipole
cooker fitted only in the near field then reproduces the held far field. This
supports the classical surface-mode-to-radiation representation on one exact
axisymmetric synthetic class.

It does **not** establish non-spherical geometry, FEM eigenmode coupling,
arbitrary 3D radiation, real-object transfer, material identity, perceptual
quality, corpus admission, runtime cost or a ProductCheck. The earlier
REALIMPACT order-three rejection remains valid: basis sufficiency on its own
native mode does not prove that a compact axisymmetric basis explains real
objects.

Keep both ceramic holdouts and all fresh REALIMPACT payload sealed. Next freeze
one genuinely non-spherical closed mesh with a prescribed surface mode, require
coarse/fine Bempp agreement and cross-check its cooked field. Only that success
may authorize a fresh object-disjoint real-data spatial calibration.

Primary references:

- [Bempp Helmholtz boundary operators](https://bempp.com/handbook/api/boundary_operators.html#boundary-operators-for-the-helmholtz-equation)
- [Bempp-cl frozen source revision](https://github.com/bempp/bempp-cl/tree/a1eaaef9f96b9dd3d7c56b076740e06852a6e1c0)
- [SciPy spherical Bessel functions](https://docs.scipy.org/doc/scipy/reference/generated/scipy.special.spherical_jn.html)
