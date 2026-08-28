# REALIMPACT Pitcher geometry-spatial calibration — PS-2 — 2026-08-28

## Outcome

The preregistered Pitcher calibration rejects the
`cotangent-biharmonic-normal-mode-bempp-full-angular-v1` candidate. Two
independent offline analyses from the same immutable decoded block produced
byte-identical report SHA-256
`8bd5323cabdb4319f8465631c3c7a54438669ee10afa9cac1f6bb122537b1aea`
and decision `PitcherGeometrySpatialCalibrationRejected`.

The rejection is conjunctive and far from threshold: frequency mapping, every
held listener stratum and comparison to the frozen RBF control fail. Mode
coverage passes, and the numerical eigensolver/GMRES residuals remain well
inside their gates. This isolates the failure to the physical proxy and its
predictive field, not solver convergence.

Planter remains sealed. The authored clip baseline is retained. No real 3D
transfer, material identity, perceptual quality, domain admission, selective
`Pass` or runtime credit is granted.

## Immutable lineage

| Artifact | SHA-256 or exact fact |
| --- | --- |
| Original execution manifest | `8e791327595f43c672361e486ed50aa8512f1c8068de5efdbe6212a81df8ba45` |
| One compressed Pitcher prefix | `a0dd700645c89ab5f2d93779cb16177da8464bb0544d4501fd9c0287eda36cf5`, `536870912` bytes |
| Acquisition report | `899fbbe9b6f098f72438817d27d701d0eabc5b5c4a7a0cbbd45de1ad74a5c819`, exactly one request |
| Decoded Pitcher block | `182f2010410bf3061176830309b83c3dad482447c8dd2e5f40346f81e0591e0f`, `553128000` bytes |
| Decode report | `29496c6f8f8f5aebb0525d056b7bdf40575ea2b66fc04f1687e449e13e359eef` |
| Final repair manifest | `f51a60467e2b73245f0db397ad14e639d6fbe32bb447a8f31f93877fc5badb7e` |
| Final runner | `0bd52a0b958137705b51e8254ab87a8fe3303a4a48be0cb6a7150ee239f2a500` |
| Rust projection input | `2d7bf9ef445016e7bd0f684d76b601252a347dadadbf57ad0e99cc15fe6452b8` |
| Rust complex projection | `e47f5171043df98535bfc2eb48b15106c222e362db33eadab3947295e9964ee0`, `153600` bytes |
| Rust projection report | `572341d97e397bf2f292d6d55d9a386457dbf490e60a7e3d3a861c86ede1549b` |
| Calibration report A/B | `8bd5323cabdb4319f8465631c3c7a54438669ee10afa9cac1f6bb122537b1aea`, `104789` bytes |

`diff -qr` reports no difference between the complete A/B output directories,
including projection input, f64 projection and both reports. Analysis A and B
made zero network requests and read zero Planter payload bytes.

The failed JSON serializer and first lineage repair remain preserved in the
[repair evidence](physical-sound-realimpact-pitcher-serializer-repair-ps2-2026-08-28.md).
They change no numeric result.

## Gate results

### Frequency mapping

The frozen one-refit 16-of-64 dynamic program selected proxy indices
`[1,2,3,4,5,6,7,8,9,10,11,12,13,23,62,63]` with scale
`0.4550526273107732 m²/s`.

| Metric | Result | Gate | Pass |
| --- | ---: | ---: | --- |
| Median absolute frequency error | `0.566388` octave | `≤ 0.20` | No |
| p90 absolute frequency error | `1.626021` octave | `≤ 0.35` | No |

The measured modes contain thirteen closely spaced modes from `256.8` to
`524.5 Hz`, then `1485.3`, `11602.0` and `11988.2 Hz`. A single scalar scale on
the frozen biharmonic proxy cannot explain that spectrum within the declared
error envelope.

### Mode and solver admission

Thirteen of sixteen modes are admitted against the minimum of twelve; eleven
admitted modes have a matched tail. Maximum admitted eigen residual is
`2.85308e-13`, and maximum GMRES residual across all modes is `9.92682e-9`
against `5e-5`. Modes 13–15 have no degree `2/4/6` cooker satisfying the frozen
`4L/10L` gates; modes 0–12 admit at least one degree. Coverage passes, but this
cannot rescue the other conjunctive failures.

### Held 3D listener prediction

| Stratum | Median error | p90 error | Persistent median | Ratio to constant | Improved mode fraction | Pass |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| held-height | `19.8984 dB` | `48.7858 dB` | `17.3914 dB` | `2.5003` | `0.1538` | No |
| held-angle | `23.4366 dB` | `57.3911 dB` | `20.7533 dB` | `2.9508` | `0.1538` | No |
| held-distance | `22.9377 dB` | `45.2361 dB` | `20.9367 dB` | `3.0525` | `0.0` | No |

Every stratum misses all material absolute gates: median `≤7 dB`, p90
`≤18 dB`, persistent median `≤8 dB`, ratio to constant `≤0.9` and improved
component fraction `≥0.5`.

Across all 510 held listeners, the geometry candidate has median/p90 errors
`22.7300/48.9528 dB`; the frozen coordinate-only RBF has
`8.4065/21.3710 dB`. Candidate/RBF median ratio is `2.70386` against `≤0.95`,
p90 regression is `+27.5818 dB` against `≤2 dB`, and only `0.153846` of modes
improve against RBF versus the required `≥0.5`.

## Interpretation

Evidence supports two bounded conclusions:

1. A scalar cotangent-biharmonic surface spectrum with one global frequency
   scale is not an adequate elastic-shell proxy for this Pitcher.
2. Even where the synthetic Bempp/cooker calculation is numerically stable,
   its relative 3D field does not predict the measured deconvolved listener
   magnitudes better than the coordinate-only control.

The report does not identify which missing physical variable dominates. The
serious alternatives are wall/interior geometry and elastic shell mechanics;
support/impact coupling; and a source-specific mismatch between the published
transfer metadata and the assumed surface-normal excitation. Cooker degree,
RBF bandwidth, mapping thresholds and opened Pitcher values are not eligible
for retuning.

## Decision and next action

Status is
`PITCHER_GEOMETRY_SPATIAL_CALIBRATION_REJECTED / BYTE_IDENTICAL_REPEAT /
PLANTER_SEALED / AUTHORED_CLIP_FALLBACK`.

Do not create a Planter holdout manifest and do not reopen or extend the
Pitcher prefix. Before another implementation attempt, run a bounded research
cycle on unopened development sources that discriminates:

- `H-shell`: thickness/interior-aware elastic shell modes close the spectral
  and spatial error;
- `H-excitation`: support/impact coupling, rather than eigenmode family,
  dominates the mismatch;
- `H-metadata`: the measured transfer/reference coordinate interpretation is
  incompatible with the assumed radiation observation model.

The smallest safe next experiment must have a successful synthetic control and
an unopened development object. It must be frozen before access and must not
weaken the Pitcher gates or reuse Planter.
