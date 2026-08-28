# Physical sound PS-2 — triaxial Bempp mode and full-angular cooker

Date: `2026-08-28`

Status:
`TRIAXIAL_FULL_NEAR_SHELL_COOKER_SUPPORTED / SPARSE_NEAR_ANGULAR_INTERPOLATION_REJECTED / BYTE_IDENTICAL_REPEATS / NO_FRESH_REALIMPACT_DATA_OPENED`

## Purpose

The preceding control proved one axisymmetric spherical quadrupole. This
package removes both simplifying symmetries without opening real data:

- the closed surface is a triaxial ellipsoid with semiaxes
  `0.08 / 0.10 / 0.13 m`;
- the prescribed Neumann mode is `q = 2 u_x u_z` in ellipsoid radial-pullback
  coordinates, so the field is odd in `x` and `z`, even in `y`, and cannot be
  represented by an axisymmetric `m=0` basis;
- the repository cooker uses full real spherical angular functions multiplied
  by outgoing first-kind spherical Hankel functions.

This is still a synthetic prescribed surface mode. It does not yet connect an
elastic FEM eigenvector to the acoustic boundary solver.

## Frozen independent Bempp protocol

Source manifest SHA-256:
`74d8ebd1339faddf5727d0e02e1671a3a2a9295b83524f1f9336e1ec3cee267d`.
It pins the same Python `3.12.13`, Bempp-cl `0.4.2` revision
`a1eaaef9f96b9dd3d7c56b076740e06852a6e1c0` and direct
Neumann-to-Dirichlet formulation as the preceding independent controls. The
fixture covers:

- regular-sphere refinements `2/3/4`, affinely deformed into
  `128/512/2048`-panel triaxial meshes;
- `kL = 0.75, 1.5`, radii `2L, 4L, 10L` and 56 directions from seven polar
  cosines by eight azimuths;
- zero area-mean leakage, all-GMRES, residual, three-level refinement and
  fine-versus-medium complex-field gates;
- synthetic-only data, with no quality, material, admission, runtime or
  ProductCheck credit.

Runs `run-a` and `run-b` are byte-identical at report SHA-256
`49bee8c27202eb89edee9be06def177f883bd751272a1700be98454bf31376ef`.

| Metric | Observed | Frozen gate |
| --- | ---: | ---: |
| Fine conditions | `336` | exact grid |
| Maximum fine/medium peak-normalized difference | `0.0330466` | `<=0.05`, pass |
| Maximum active relative difference | `0.0332302` | `<=0.08`, pass |
| Maximum active magnitude difference | `0.293538 dB` | `<=0.75 dB`, pass |
| Maximum active phase difference | `0.153479°` | `<=5°`, pass |
| Minimum directional complex correlation | `0.999999618` | `>=0.998`, pass |
| Median refinement ratio | `0.266540` | `<=0.65`, pass |
| Maximum final GMRES residual | `9.626e-11` | `<=1e-8`, pass |
| Maximum absolute area-weighted mode mean | `3.564e-18` | `<=1e-12`, pass |

Decision: `IndependentBemppTriaxialSurfaceModeSupported`.

## Frozen sparse-angular cooker: negative control

Manifest SHA-256:
`e3bbd547dd888cafa0be345d6abe3cb4d3c1569a2f12f362306bcb68b8c3851e`.
It fits 40 directions at `2L`, holds out the 16 directions at polar cosines
`±0.3`, and holds out every direction at `4L/10L`. The candidates were frozen
as axisymmetric degree 4, full-angular degree 2 and full-angular degree 4.

Runs repeat at report SHA-256
`6d5f653af224c85a2f31eb97994398b84fd79238fd0cee56195e247c1e2fef55`.
The axisymmetric control fails every held gate, as required. Full degree 2 also
fails. Full degree 4 passes relative, magnitude, phase and correlation gates,
but its maximum peak-normalized error is `0.0603221` against `0.05`; all worst
rows are on the held near shell. Its far-shell maxima are only `0.0059764` at
`4L` and `0.0063017` at `10L`.

Decision: `FullAngularTriaxialNearToFarCookerRejected`. This result is retained
as evidence that degree 4 plus a 40-direction fit is not enough to interpolate
the omitted near-shell polar levels. The threshold was not changed.

## Frozen full-near-shell cooker: positive product-shaped control

An offline cooker can ask Bempp for the complete sampling shell. The follow-up
therefore changes the question rather than weakening the failed gate: fit all
56 directions at `2L`, then validate only the completely held `4L/10L` shells.
Manifest SHA-256:
`e69f09b20b4f3bef57c8008fe114ddc5e6ba7f3220baf3657fe8d1f624d1b896`.
It freezes the same thresholds, the negative axisymmetric control and full
degrees `2/4/6`; the smallest full-angular candidate passing every held gate is
selected.

Runs `full-shell-cooker-run-a/b` repeat at report SHA-256
`a0e0d881fbdc9e49f8cdca23668bb71197a9d505d1d35bb5048efa49fb23b0be`.

| Candidate | Max peak error | Max active relative | Max magnitude | Max phase | Min correlation | Decision |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Axisymmetric degree 4 | `1.000000` | `1.000000` | `306.586 dB` | `179.858°` | `9.78e-16` | reject control |
| Full degree 2 | `0.0460449` | `0.0728977` | `0.529088 dB` | `4.12315°` | `0.9990417` | pass, selected |
| Full degree 4 | `0.00348276` | `0.00382666` | `0.0323768 dB` | `0.194121°` | `0.9999988` | pass |
| Full degree 6 | `0.0189138` | `0.0305174` | `0.262458 dB` | `1.62375°` | `0.9999323` | pass, not selected |

The degree-6 regression relative to degree 4 is consistent with the bounded
7×8 sampling grid and is a reason not to select complexity merely because it
is available. The frozen rule correctly chooses degree 2 as the smallest model
that meets the declared error budget. Decision:
`FullAngularTriaxialNearToFarCookerSupported`.

## Implementation and claim boundary

The repository command is:

```text
cargo run -p xtask -- physical-sound-registry bem-feasibility triaxial-cooker \
  --manifest <external-json> --output <external-empty-directory>
```

It hash-validates both registered protocols and their Bempp inputs, implements
associated Legendre sine/cosine columns, outgoing Hankel radial functions,
column-scaled complex ridge fitting with deterministic partial pivoting, and
atomically publishes canonical JSON plus exact source copies. External
manifests, reports and generated fields remain outside the repository.

This package establishes a converged non-spherical, non-axisymmetric synthetic
surface field and a sufficient full-angular near-to-far representation when
the cooker samples the complete near shell. It does **not** establish FEM
eigenmode coupling, elastic material parameters, support conditions, real
object transfer, perceptual quality, domain admission, runtime cost or a
production content format. The REALIMPACT and ceramic holdouts remain sealed;
the clip fallback and disabled `Pass` state are unchanged.

The smallest next action is to compute one elastic synthetic FEM eigenmode on
a non-spherical closed mesh, project its surface-normal velocity into the same
Bempp boundary condition, and require mesh/eigenfrequency/field convergence
plus the full-shell cooker gates. This closes prescribed-mode-to-actual-mode
coupling before any fresh internet payload is opened.

Primary references:

- [Bempp Helmholtz boundary operators](https://bempp.com/handbook/api/boundary_operators.html#boundary-operators-for-the-helmholtz-equation)
- [Bempp-cl frozen source revision](https://github.com/bempp/bempp-cl/tree/a1eaaef9f96b9dd3d7c56b076740e06852a6e1c0)
- [SciPy spherical Bessel functions](https://docs.scipy.org/doc/scipy/reference/generated/scipy.special.spherical_jn.html)
