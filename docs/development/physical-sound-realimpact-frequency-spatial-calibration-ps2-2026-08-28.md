# PS-2 REALIMPACT frequency-conditioned spatial calibration — 2026-08-28

## Decision

`FrequencyConditionedBandwidthCalibrationRejected / HoldoutUnopened /
RbfBandwidthFamilyRetired`.

The experiment tests the preregistered successor to the rejected object-level
bandwidth: predict a separate RBF bandwidth for every selected mode from
dimensionless acoustic scale `kL`, mesh aspect and normalized impact position.
The candidate preserves the absolute spatial condition gate on both fresh
calibration objects, but its central error is worse than the unchanged
coordinate-only `sigma=0.52` control. Two comparison gates fail, so the two
ceramic holdouts remain unopened.

This result concerns relative selected-mode magnitude on one `0 degree / 0 mm`
15-microphone vertical line. It grants no angle/distance interpolation, 3D
radiation, material identity, naturalness, corpus admission, validator `Pass`
or runtime authority.

## Frozen development protocol

Development manifest
`88cac5bcbc79870ac8c054288a5eb5b0f1300a9b697da71dd38c8108861c709f`
binds the previously verified selected blocks and rejected bbox lineage rather
than downloading development audio again. It freezes:

- twelve opened development objects: `49_PlasticBowl`, `67_IronPlate`,
  `31_WoodSlab`, `83_WoodVase`, `81_WoodPad`, `34_WoodMug`,
  `48_PlasticBowl`, `90_MetalLadle`, `9_BowlCeramic`, `68_WoodBoard`,
  `10_bowl` and `36_SmallMeasuringCup`;
- fresh calibration objects `100_Frisbee` and `32_WoodChalice`;
- preserved holdouts `65_PitcherCeramic` and `63_SmallPlanterCeramic`;
- exact archive, metadata, mesh and selected-block identities;
- the established injective V2 modes, `9 anchor / 6 held` listener split and
  coordinate-only `sigma=0.52` control;
- sigma grid `0.18, 0.26, 0.36, 0.52, 0.74, 1.05 m`;
- per-mode target selection by minimum held-listener median absolute error,
  with ties choosing the lower bandwidth;
- ridge `0.25`, equal total weight per object and the unchanged condition,
  calibration and holdout gates.

The five model features are intercept, `ln(kL)`, log minor/major and
middle/major mesh extents, and impact radius divided by bounding-box diagonal;
`k = 2*pi*f/343`. Object identity/name, material labels, fresh audio and held
listener samples at inference remain forbidden inputs.

## Development fit

The twelve objects yield 192 per-mode targets:

| Sigma | Modes |
| ---: | ---: |
| `0.18` | `23` |
| `0.26` | `25` |
| `0.36` | `23` |
| `0.52` | `27` |
| `0.74` | `32` |
| `1.05` | `62` |

The fitted log-bandwidth coefficients are:

| Feature | Coefficient |
| --- | ---: |
| intercept | `-0.6295915` |
| standardized `ln(kL)` | `0.0043444` |
| standardized log minor/major | `0.0204608` |
| standardized log middle/major | `0.0580888` |
| standardized impact radius | `0.0263449` |

The near-zero `kL` coefficient and narrow predictions show that the regression
mostly collapses back toward a constant bandwidth despite varied per-mode
development targets. Two complete development runs are byte-identical at
report SHA-256
`48a150d74e478ce55fb31047bc40f43727ab183d708ce1ae063c7a6aa2ad819b`.

## Fresh calibration result

Calibration manifest
`92ebe6a7fbb3207dd7d2082076af3209304a0972430d3e59fd24ed8fa99642f8`
binds the exact development report before either fresh payload is opened. The
model is not refit and no threshold changes.

| Object | Modes | Predicted sigma range | Candidate gate | Median ratio | p90 delta | Improved fraction |
| --- | ---: | ---: | --- | ---: | ---: | ---: |
| `100_Frisbee` | `16` | `0.5230–0.5248` | pass | `1.00265` | `-0.0105 dB` | `0.5000` |
| `32_WoodChalice` | `16` | `0.4879–0.4887` | pass | `1.02656` | `+0.2547 dB` | `0.5000` |

The aggregate candidate-condition gate and p90/fraction checks pass. Median
object ratio is `1.01460` against required `<=0.95`; maximum object ratio is
`1.02656` against `<=1.0`. These failures reject the candidate without
threshold ambiguity.

Two complete calibration runs, including both selected blocks, are
byte-identical. Their report SHA-256 is
`42b6605db6537e9c84f31494fd0cbc3acce41b807c00fede2b15d2f879b69983`.
No holdout manifest exists and no deconvolved payload byte from
`65_PitcherCeramic` or `63_SmallPlanterCeramic` was requested.

## Research escalation and next discriminator

Three coherent attempts now bound the same representation family: a fixed RBF
works only in the original narrow pilot, a generic multi-object extension
fails, and neither object-level nor per-mode bandwidth conditioning improves
fresh calibration. More bandwidth, ridge or descriptor tuning is prohibited.

Primary acoustic-transfer work explains the missing variable. Each structural
mode has its own surface vibration and wave-radiation field; practical systems
precompute that field with numerical acoustics and compress it as equivalent
multipoles or far-field acoustic-transfer maps. REALIMPACT likewise reports
that acoustic transfer varies drastically with listener location and publishes
600 field positions per impact specifically to calibrate this sim-to-real gap.
Bounding-box scale cannot represent modal nodal structure, diffraction or
interference.

The smallest next experiment changes representation rather than coefficients:

1. use only already-open development blocks to preserve complex per-mode
   response, not magnitude alone;
2. compare constant, current magnitude-RBF and one bounded low-order complex
   radiation basis on held listener positions;
3. treat this as representation-sufficiency development evidence only;
4. on success, freeze a fresh object/impact/angle/distance validation before
   implementing an offline surface-mode plus PAT/BEM-style cooker;
5. on failure, retain clip/per-object fallback and reject the compact basis.

The two ceramic holdouts remain sealed until a geometry- and mode-shape-driven
candidate, exact metrics and fallback are frozen. A neural radiation solver is
not the smallest next step: it would add model/data risk before the classical
representation itself has passed the bounded counterfactual.

## Primary sources

- [REALIMPACT project and recording geometry](https://samuelpclarke.com/realimpact/)
- [REALIMPACT CVPR paper](https://jiajunwu.com/papers/realimpact_cvpr.pdf)
- [Precomputed Acoustic Transfer](https://graphics.stanford.edu/~djames/publication/precomputed-acoustic-transfer-output-sensitive-accurate-sound-generation-for-geometrically-complex-vibration-sources/)
- [Interactive Acoustic Transfer Approximation](https://www.cs.columbia.edu/cg/transfer/)
- [KleinPAT](https://graphics.stanford.edu/projects/kleinpat/)
- [NeuralSound](https://arxiv.org/abs/2108.07425)
