# Physical sound PS-2 — BEM panel-quadrature discriminator

Date: `2026-08-28`

Status:
`SEVEN_POINT_PANEL_QUADRATURE_HYPOTHESIS_REJECTED / ABSOLUTE_GATES_PASS / REFINEMENT_AND_CONTROL_COMPARISON_FAIL / BYTE_IDENTICAL_REPEAT / V1_REPORT_PRESERVED / NO_FRESH_REALIMPACT_DATA_OPENED`

## Why this experiment exists

The first pulsating-sphere boundary solver passed four absolute field gates but
failed its preregistered 80-to-320-panel refinement gate. Two competing
hypotheses remained:

1. three-point panel quadrature is too weak near adjacent panels, so a fixed
   higher-order rule will restore convergence;
2. planar geometry, singular treatment or the indirect collocation boundary
   formulation dominates, so more regular panel samples will not help.

Manifest
`physical-sound/ps2-surface-bem-v1/quadrature-manifest.json`, SHA-256
`b9ff02c9097893f9a568f25bc8bd9c4a32e6593e3a104ab114e888521c5dff47`,
was frozen before running the candidate. It binds the original manifest/report
hashes, repeats the same analytical fixture and meshes, and changes only the
symmetric triangle rule from three to seven points. The candidate must pass
the original absolute gates, reach fine/coarse median error ratio `<=0.8` and
reduce fine median error to `<=0.75x` the three-point control.

## Result

The implementation regenerates the complete historical V1 report and requires
its exact SHA-256 before evaluating the candidate. `quadrature-a` and
`quadrature-b` publish byte-identical reports:

| Artifact | SHA-256 |
| --- | --- |
| quadrature manifest | `b9ff02c9097893f9a568f25bc8bd9c4a32e6593e3a104ab114e888521c5dff47` |
| historical V1 report | `6f74a30988ff349c270d6cb0ba37cfbf905e1dc1b078868660d01fb1c622a689` |
| quadrature report A/B | `7460750eb82508a69df52caa65b4d2a5ea41636cb948fffecd1662eb70d3b48d` |

| Metric | Three-point control | Seven-point candidate | Gate |
| --- | ---: | ---: | ---: |
| Fine median relative complex error | `0.0203677` | `0.0212777` | candidate/control `<=0.75` |
| Fine maximum relative complex error | `0.0212507` | `0.0222686` | `<=0.15`, pass |
| Fine maximum magnitude error | `0.182647 dB` | `0.191300 dB` | `<=0.75 dB`, pass |
| Fine maximum phase error | `0.299201°` | `0.323329°` | `<=10°`, pass |
| Fine direction span | `0.000196 dB` | `0.000159 dB` | `<=0.25 dB`, pass |
| Fine/coarse median ratio | `2.858899` | `2.610851` | `<=0.8`, fail |
| Candidate/control fine median ratio | — | `1.044675` | `<=0.75`, fail |

Decision: `SevenPointPanelQuadratureHypothesisRejected`.

Seven-point regular quadrature slightly improves directional symmetry and the
refinement ratio, but it does not restore convergence and makes fine median
error `4.47%` worse. Do not try another ordinary fixed regular-panel rule on
this collocation formulation.

## Research escalation and next action

This is the second coherent remediation cycle with the same convergence
blocker, so further local quadrature tuning stops. Primary-source review found:

- Bempp's Helmholtz operators are Galerkin double integrals and explicitly use
  special singular quadrature where source and test points coincide;
- the frozen NeuralSound reference computes Neumann-to-Dirichlet data with
  hypersingular and adjoint-double-layer operators, rather than the indirect
  single-layer collocation equation used by this harness;
- current Bempp-cl is `0.4.2`, repository revision
  `a1eaaef9f96b9dd3d7c56b076740e06852a6e1c0`, and provides a CPU/Numba path.

The next bounded experiment is an independent Bempp-cl pulsating-sphere
control in an external Python 3.12 environment, with exact package/revision and
output hashes. It must pass analytical field and mesh-refinement controls
before non-spherical surface modes or fresh REALIMPACT data are opened. This
package grants no converged BEM/FFAT, material, perceptual, admission or runtime
credit.

Primary references:

- [Bempp Helmholtz boundary operators and singular quadrature](https://bempp.com/handbook/api/boundary_operators.html#boundary-operators-for-the-helmholtz-equation)
- [Bempp-cl combined exterior Helmholtz example at the frozen revision](https://github.com/bempp/bempp-cl/blob/a1eaaef9f96b9dd3d7c56b076740e06852a6e1c0/examples/helmholtz/helmholtz_combined_exterior.py)
- [Bempp installation and CPU/Numba option](https://bempp.com/installation.html)
- [NeuralSound classical Neumann-to-Dirichlet implementation](https://github.com/hellojxt/NeuralSound/blob/b18e81b1e3dba7e963b09a7d9a45b584707d805f/src/classic/bem/bemModel.py)
