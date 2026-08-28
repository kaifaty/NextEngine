# PS-2 REALIMPACT modal-radiation representation diagnostic — 2026-08-28

## Decision

`ComplexMultipoleRepresentationDevelopmentRejected / NoFreshDataOpened /
ClassicalSurfaceModeBemFeasibilityNext`.

After fixed, object-conditioned and frequency-conditioned RBF failures, this
diagnostic changes the representation rather than tuning another bandwidth. It
preserves the complex response of every selected mode and fits a bounded
axisymmetric outgoing multipole basis. The candidate passes the absolute
condition gate for all fourteen already-open objects, but is materially worse
than the unchanged magnitude-RBF control on every aggregate comparison metric.

The two ceramic holdouts remain unopened. No fresh impact, angle or distance
payload is used. This development-only rejection grants no spatial field,
material identity, quality, admission, validator `Pass` or runtime authority.

## Frozen protocol

Manifest
`686c1d423bc46de81ba66104fbbca90fe8ae8ff865462f5f99cfae850f47a0ff`
was frozen before complex per-mode responses were inspected. It binds:

- the frequency-calibration source manifest `92ebe6a7…42f8`;
- frequency development/calibration reports `48a150d7…19b` and
  `42b6605d…983`;
- fourteen already-open selected blocks and their exact hashes/sample counts;
- 224 injective V2 modes, nine anchor and six held listeners;
- the coordinate-only magnitude RBF `sigma=0.52` control;
- candidate `axisymmetric-complex-multipole-order3-ridge001-v1`;
- the prior absolute condition gate and a comparison rule frozen before the
  complex response was read.

For every mode and listener, the candidate uses the frozen per-row onset and
65,536-sample Hann DFT. The complex response is divided by listener 7. About
the mesh bounding-box center and world z axis, its four complex basis functions
are

```text
h_l^(2)(k*r) * P_l(cos(theta)), l = 0..3
```

where `k=2*pi*f/343`. Nine anchors fit eight real coefficient components using
column-RMS normalization and ridge `0.001`; prediction is renormalized at
listener 7. Evaluation uses held-listener absolute error in relative magnitude
dB. The preregistered representation gate requires every absolute condition
gate, median object ratio `<=0.9`, every object ratio `<=1.1`, maximum p90
regression `<=2 dB` and aggregate improved fraction `>=0.6`.

## Result

Two complete runs are byte-identical at report SHA-256
`7cbf7c59613aad9fce256912355f7f4e5a8e7ce693491b7a5d5797931878cf25`.
The aggregate is:

| Metric | Observed | Required | Result |
| --- | ---: | ---: | --- |
| candidate condition gates | `14/14` | `14/14` | pass |
| median object candidate/control ratio | `1.25063` | `<=0.90` | fail |
| maximum object candidate/control ratio | `2.08612` | `<=1.10` | fail |
| maximum object p90 regression | `+7.25132 dB` | `<=2 dB` | fail |
| aggregate improved-mode fraction | `0.34821` | `>=0.60` | fail |

Only `32_WoodChalice` improves median error (`0.9482x`); thirteen of fourteen
objects are worse. The largest median loss is `67_IronPlate` at `2.0861x`.
The result is far from every comparative threshold, so it is not sensitive to
rounding or a marginal gate choice.

## Interpretation

Preserving phase is insufficient when the spatial basis discards the actual
surface mode shape. An axisymmetric `m=0`, order-three expansion around a
bounding box cannot encode object-specific nodal lobes, non-axisymmetric
directivity, diffraction and interference. Increasing order or adding
empirical harmonics on these opened rows would be another post-hoc basis tune
and is prohibited.

Primary modal-sound systems instead compute surface vibration modes first,
solve acoustic transfer from those boundary conditions and then compress each
mode's field as equivalent multipoles or a far-field acoustic-transfer map.
That makes the mode shape, not object identity or a fitted listener-coordinate
kernel, the source of radiation structure.

## Next bounded package

Before more REALIMPACT access, establish whether a reproducible classical
surface-mode-to-radiation toolchain is executable on one tiny synthetic
fixture:

1. freeze external source revision and environment; keep code, caches, meshes
   and outputs outside the repository;
2. run modal analysis and BEM acoustic-transfer generation on an analytical or
   independently checkable primitive;
3. require finite eigen residuals, frequency ordering, boundary/mesh identity,
   repeatable complex field/FFAT hashes and an analytical symmetry/control;
4. do not train or import a neural radiation model before the classical target
   and error budget exist;
5. only after the classical control passes, preregister one REALIMPACT
   simulation-to-real comparison with untouched impact/angle/distance data.

The official NeuralSound repository at commit
`b18e81b1e3dba7e963b09a7d9a45b584707d805f` exposes classical
`modalAnalysis.py` and `acousticTransfer.py` generation scripts backed by
SciPy/LOBPCG and Bempp-cl. This is the smallest concrete feasibility route.
OpenPBSO at commit `17ff89702d7818ae73387ab81fa6e38a6bef9f9c` can consume surface
modes and FFAT maps at runtime, but explicitly omits KleinPAT preprocessing, so
it is a later format/runtime reference rather than the next solver.

Clip/per-object fallback remains mandatory. Exact-domain admission, calibrated
OOD/shadow risk, contact projection, whole-mixer cost, authoring and P1 remain
open.

## Primary sources

- [REALIMPACT project](https://samuelpclarke.com/realimpact/)
- [REALIMPACT paper](https://jiajunwu.com/papers/realimpact_cvpr.pdf)
- [Precomputed Acoustic Transfer](https://graphics.stanford.edu/~djames/publication/precomputed-acoustic-transfer-output-sensitive-accurate-sound-generation-for-geometrically-complex-vibration-sources/)
- [KleinPAT](https://graphics.stanford.edu/projects/kleinpat/)
- [NeuralSound paper](https://arxiv.org/abs/2108.07425)
- [NeuralSound official implementation](https://github.com/hellojxt/NeuralSound/tree/b18e81b1e3dba7e963b09a7d9a45b584707d805f)
- [OpenPBSO](https://github.com/jhwang7628/openpbso/tree/17ff89702d7818ae73387ab81fa6e38a6bef9f9c)
