# PS-2 REALIMPACT shape-conditioned spatial calibration — 2026-08-28

## Decision

`MeshConditionedBandwidthCalibrationRejected / HoldoutUnopened`.

The experiment tests one deliberately small successor to the rejected fixed
vertical RBF: predict one RBF bandwidth per object from mesh bounding-box scale,
aspect and normalized impact position. The fitted candidate passes the original
quality gate on both calibration objects and improves p90 and most individual
mode medians, but it does not improve the aggregate median error over the frozen
coordinate-only `sigma=0.52` control. The preregistered calibration gate fails;
the two declared holdout objects remain unopened.

This result concerns selected-mode relative magnitude on one `0 degree / 0 mm`
15-microphone vertical line. It grants no angle/distance interpolation, 3D
radiation, material identity, naturalness, admission, validator `Pass` or
runtime authority.

## Object-disjoint preregistration

The official 50-object roster is pinned to REALIMPACT repository commit
`fca2bd6cbb7e9f96ac61328d2a0d51594bf01987`; `object_names.txt` hashes to
`3ee26ac9…ad5a`. Five previously opened objects are excluded. The remaining
UTF-8 object IDs are SHA-256 sorted without a newline; the first ten are
development, the next two calibration and the next two holdout. This chooses
roles without listening to or classifying their audio.

Before any selected deconvolved payload byte was opened, development manifest
`4e58eded790a2d00a667fd659827085f452e5a34094af62b7422da792a420b3d`
froze:

- ten development objects: `49_PlasticBowl`, `67_IronPlate`, `31_WoodSlab`,
  `83_WoodVase`, `81_WoodPad`, `34_WoodMug`, `48_PlasticBowl`,
  `90_MetalLadle`, `9_BowlCeramic`, `68_WoodBoard`;
- calibration objects `10_bowl` and `36_SmallMeasuringCup`;
- holdout objects `65_PitcherCeramic` and `63_SmallPlanterCeramic`;
- exact archive HTTP/ZIP identities, seven non-audio metadata members and
  mesh descriptors for all 14 objects;
- a 16 MiB compressed audio prefix, base rows `0..14`, the established
  injective 16-mode V2 extractor on anchor microphone 7 and the unchanged
  `9 anchor / 6 held` listener split;
- control `coordinate-only-rbf-sigma052-ridge001-v1`;
- one candidate, `mesh-bbox-conditioned-object-rbf-bandwidth-v1`;
- sigma grid `0.18, 0.26, 0.36, 0.52, 0.74, 1.05 m`, fixed feature
  standardization and ridge `0.25`;
- the original condition gate and calibration/holdout comparison rules.

The candidate's five inputs are intercept, log mesh bounding-box diagonal, log
minor/major and middle/major extent ratios, and impact radius divided by the
bounding-box diagonal. Object names, material labels, calibration/holdout
audio and held-listener samples at inference are forbidden inputs.

## Development fit

For each development object, the target is the grid bandwidth with minimum
frozen spatial `calibration_loss`; ties choose the lower bandwidth. Target
counts are:

| Sigma | Objects |
| ---: | ---: |
| `0.18` | `0` |
| `0.26` | `1` |
| `0.36` | `1` |
| `0.52` | `0` |
| `0.74` | `3` |
| `1.05` | `5` |

The spread proves that one universal `0.52` bandwidth is not the development
optimum, but it does not prove that the coarse descriptors predict bandwidth
out of sample. Two complete development runs, including all 150 selected rows,
mode extraction, grid evaluation and ridge fitting, are byte-identical. Their
report is
`3d18358b50442ee9b171bbacc154f7713e3240efa6f4d50f61f2598b7ac4962f`.
The runs transfer `173,659,678` bounded HTTP range bytes each and publish only
the selected external blocks and reports.

## Calibration result

Calibration manifest
`2f8ea9b3de11fe5353fcddac2370d1889a9841f9881d83adc63992dbefb1e578`
binds the exact development report before either calibration payload is
opened. The model is not refit and thresholds do not change.

| Object | Predicted sigma | Candidate gate | Median ratio to control | p90 delta | Improved modes |
| --- | ---: | --- | ---: | ---: | ---: |
| `10_bowl` | `0.8719` | pass | `1.0242` | `-0.1618 dB` | `0.5625` |
| `36_SmallMeasuringCup` | `0.6374` | pass | `1.0001` | `-0.4223 dB` | `0.7500` |

The aggregate improved-component fraction is `0.65625`, every candidate
condition gate passes and p90 improves. However, median object ratio is
`1.01215` against a required maximum `0.95`; maximum object ratio is `1.02421`
against `1.0`. These two independent failures reject the hypothesis without
threshold ambiguity.

Two calibration runs are byte-identical. The report is
`ffb176873e0fbb8cabf3083c9e9f507a31ecdc76cea83e61c1afa28a4a00aad6`;
each run transfers `34,789,615` bounded HTTP range bytes. No holdout manifest
was created and no deconvolved payload byte from `65_PitcherCeramic` or
`63_SmallPlanterCeramic` was requested.

## Interpretation and next discriminator

Bounding-box geometry can vary the fitted bandwidth and often reduce tail
error, but one bandwidth for all modes of an object is too coarse to improve
the central metric. Do not retune the descriptors, ridge or gates on the two
calibration objects.

The smallest next hypothesis is frequency-conditioned acoustic scale. Modal
directivity should depend on dimensionless `kL = 2*pi*f*L/c`, so a candidate may
predict a separate bounded bandwidth per mode from frequency, mesh scale,
aspect and impact descriptors. The next preregistration may use the now-opened
ten development plus two calibration objects as development, continue the
same roster ordering with fresh calibration objects `100_Frisbee` and
`32_WoodChalice`, and preserve the two current ceramic holdouts. Per-object or
clip fallback remains mandatory.

All eight exact-domain claims, calibrated domain/OOD/shadow false-pass risk,
material/perceptual quality, source-model search, contact projection,
whole-mixer cost and P1 integration remain open.

## Primary sources

- [REALIMPACT project page](https://samuelpclarke.com/realimpact/)
- [Frozen REALIMPACT repository](https://github.com/samuel-clarke/RealImpact/tree/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987)
- [Frozen official object roster](https://github.com/samuel-clarke/RealImpact/blob/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/dataset/object_names.txt)
- [REALIMPACT CVPR paper](https://jiajunwu.com/papers/realimpact_cvpr.pdf)
