# Physical sound PS-2 — elastic FEM eigenmode to Bempp and cooker

Date: `2026-08-28`

Status:
`ELASTIC_FEM_MODE_TO_BEMPP_SUPPORTED / FULL_ANGULAR_NEAR_TO_FAR_COOKER_SUPPORTED / COARSE_PROTOCOL_REJECTED / BYTE_IDENTICAL_REPEATS / NO_FRESH_REALIMPACT_DATA_OPENED`

## Purpose

The preceding triaxial checkpoint prescribed its boundary velocity directly.
This package closes the next synthetic coupling question: can a displacement
eigenvector produced by a three-dimensional linear-elastic FEM solve be
projected onto a closed acoustic boundary, propagated by the independent
Bempp solver and compressed by the repository full-angular cooker without
changing the frozen spatial error gates?

The fixture is intentionally synthetic. It is a solid triaxial ellipsoid with
semiaxes `0.08 / 0.10 / 0.13 m`, density `1000 kg/m^3`, Young's modulus
`100 MPa`, Poisson ratio `0.25` and a fixed concentric core of normalized
radius `0.25`. These values define a numerical control, not glass, a real
support fixture or an acoustic material profile.

## Frozen FEM and coupling method

The repository script is
`lab/scripts/physical_sound_bempp_fem_mode.py`. It reuses the clean-room
linear tetrahedral elasticity assembly from the controlled glass-corpus lab
and solves the generalized symmetric eigenproblem

```text
K phi = lambda M phi
```

with SciPy `eigsh`. It avoids unconstrained Delaunay remeshing: each registered
level uses deterministic concentric regular-sphere shells, affine triaxial
deformation and a fixed prism-to-tetrahedron split. The protocol:

- solves the lowest 16 modes with the core degrees of freedom removed;
- selects the lowest fine mode whose surface-normal energy is at least `0.2`
  and whose degree-four axisymmetric residual is at least `0.25`;
- matches the same mode on coarser levels using correlation of 56 normalized
  surface-normal samples, with a frozen minimum of `0.9`;
- converts the selected displacement to one constant normal-velocity value per
  surface triangle and normalizes that profile before Bempp;
- evaluates the same 56 directions at `2L / 4L / 10L` using the pinned direct
  Neumann-to-Dirichlet Bempp formulation.

All meshes, eigenvalues, eigenvectors, reports and generated fields remain
external. The repository contains the recipe and the hash-registered consumer,
not generated numerical artifacts.

## Coarse registered protocol: immutable negative

Manifest SHA-256:
`eaff559de9840a852100b14717f652555d54d465974b5946b0813fd8a03a79d4`.
It uses angular levels `1/2/3` and radial divisions `4/8/16`. Runs `run-a/b`
repeat at report SHA-256
`a58f28b9cdf13820f7622787bd63720751fb05143133a0b1db39138b81cf90ef`.

The selected mode remains index 3, but the frequencies
`497.202 / 388.960 / 361.199 Hz` and surface profiles have not entered their
frozen convergence envelope. Fine-versus-medium eigenfrequency difference is
`7.6859% > 5%`, surface-profile difference is `22.9973% > 12%`, and the BEM
field reaches `12.1585% > 8%` maximum peak difference, `14.0654% > 12%`
active relative difference, `1.31644 > 1 dB` magnitude difference and
`0.787862 > 0.75` refinement ratio. The decision is
`ElasticFemEigenmodeToBemppCouplingRejected`; no threshold was changed after
inspection.

## Refined registered protocol: positive coupling control

The separately frozen refined manifest SHA-256 is
`1adfe3d68ee5c9d806fe2161311a322cb6df2dd6161acbe340369fc9ee5aabf7`.
It changes only the declared mesh schedule to angular levels `2/3/4` and
radial divisions `8/12/16`, giving:

| Level | Nodes | Tetrahedra | Fixed nodes | Surface panels | Selected frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Coarse | `529` | `2,816` | `133` | `128` | `388.960 Hz` |
| Medium | `3,097` | `17,408` | `775` | `512` | `363.091 Hz` |
| Fine | `16,417` | `94,208` | `4,105` | `2,048` | `355.651 Hz` |

Runs `refined-run-a/b` repeat byte-identically at report SHA-256
`a71515fa9edf71663986f1a0c8e37627fff499da5eed5b84aff41a69b23da667`.
All 16 frozen gates pass.

| FEM metric | Observed | Frozen gate |
| --- | ---: | ---: |
| Coarse/medium eigenfrequency difference | `7.12461%` | `<=12%` |
| Medium/fine eigenfrequency difference | `2.09196%` | `<=5%` |
| Eigenfrequency refinement ratio | `0.293624` | `<=0.75` |
| Fine/medium surface-profile difference | `0.109647` | `<=0.12` |
| Fine/medium surface-profile correlation | `0.997404` | `>=0.97` |
| Surface-profile refinement ratio | `0.471678` | `<=0.75` |
| Minimum cross-level mode match | `0.976082` | `>=0.9` |
| Maximum solved relative eigen residual | `4.3861e-12` | `<=1e-8` |

The common acoustic condition is `355.651 Hz`, or `kL = 0.651493593`.

| Bempp metric | Observed | Frozen gate |
| --- | ---: | ---: |
| Fine/medium maximum peak difference | `0.0471238` | `<=0.08` |
| Maximum active relative difference | `0.0472961` | `<=0.12` |
| Maximum active magnitude difference | `0.401221 dB` | `<=1 dB` |
| Maximum active phase difference | `0.089116°` | `<=7°` |
| Minimum directional complex correlation | `0.9999984` | `>=0.995` |
| Median field refinement ratio | `0.350205` | `<=0.75` |

Decision: `ElasticFemEigenmodeToBemppCouplingSupported`.

## Frozen full-near-shell cooker

Cooker manifest SHA-256:
`d4daf0f3fa2e421330634789613f8c40130253bf69088077b140b94291fa4cf1`.
It binds the exact refined source report, fits all 56 directions at `2L`, holds
out all 112 conditions at `4L/10L`, keeps the prior thresholds and requires the
axisymmetric control to fail. Runs `cooker-run-a/b` repeat at report SHA-256
`c3101130b11c7f4b6118cf128b037f1ed16ff80d4b5ee2d0fda28551589ab476`.

| Candidate | Max peak | Max active relative | Max magnitude | Max phase | Min correlation | Decision |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Axisymmetric degree 4 | `1.000000` | `1.000001` | `141.599 dB` | `177.538°` | `3.12e-7` | reject control |
| Full degree 2 | `0.0309750` | `0.0625813` | `0.281925 dB` | `3.47005°` | `0.9996932` | pass, selected |
| Full degree 4 | `0.00377872` | `0.00613979` | `0.0445746 dB` | `0.275809°` | `0.9999962` | pass |
| Full degree 6 | `0.00326274` | `0.00388128` | `0.0198791 dB` | `0.201226°` | `0.9999973` | pass |

Decision: `FullAngularElasticFemModeNearToFarCookerSupported`. The frozen rule
selects degree 2 as the smallest full-angular model satisfying every gate.

## Implementation and claim boundary

The Python command computes the registered FEM/Bempp report:

```text
python lab/scripts/physical_sound_bempp_fem_mode.py \
  --manifest <external-json> --output <external-empty-directory>
```

The existing Rust command now accepts the registered FEM report as another
hash-closed source while retaining byte-identical outputs for the earlier
sparse and full-shell triaxial protocols:

```text
cargo run -p xtask -- physical-sound-registry bem-feasibility triaxial-cooker \
  --manifest <external-json> --output <external-empty-directory>
```

Historical report hashes remain exactly
`6d5f653a…ef55` and `a0e0d881…b0be`. The new code generalizes source levels,
direction sets, wave-number counts and correlation groups; it does not change
the associated-Legendre/Hankel basis or frozen candidate gates.

This package establishes one synthetic linear-elastic eigenmode-to-acoustic
coupling and one sufficient near-to-far representation. It does **not**
establish real-object transfer, material identity, support identity, absolute
impact amplitude, perceptual quality, corpus/domain admission, runtime cost,
production content or ProductCheck credit. REALIMPACT and the two ceramic
holdouts remained unopened; `Pass`, PS-3, AV-P0D and P1 remain disabled.

The smallest next action is a separately preregistered, fresh object-disjoint
real-data spatial-transfer calibration. Its exact split, payload identities,
candidate, controls and gates must be immutable before opening any reserved
REALIMPACT rows. Failure must retain per-object/clip fallback rather than tune
on the opened holdout.

Primary references:

- [SciPy generalized sparse eigensolver](https://docs.scipy.org/doc/scipy/reference/generated/scipy.sparse.linalg.eigsh.html)
- [SciPy Delaunay behavior considered and rejected for this deterministic mesh](https://docs.scipy.org/doc/scipy/reference/generated/scipy.spatial.Delaunay.html)
- [Bempp Helmholtz boundary operators](https://bempp.com/handbook/api/boundary_operators.html#boundary-operators-for-the-helmholtz-equation)
- [Bempp-cl frozen source revision](https://github.com/bempp/bempp-cl/tree/a1eaaef9f96b9dd3d7c56b076740e06852a6e1c0)
