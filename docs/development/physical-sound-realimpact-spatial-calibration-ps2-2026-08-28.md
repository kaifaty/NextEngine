# PS-2 REALIMPACT vertical spectral-participation calibration — 2026-08-28

## Decision

`RelativeVerticalSpectralParticipationSupported` for one narrow measurement
scope: relative selected-mode magnitude along the published 15-microphone
vertical line at azimuth `0` and distance offset `0`.

This is not a three-dimensional radiation field, absolute amplitude,
arbitrary-angle/distance interpolation, physical mode-shape identity, glass
identity, naturalness, corpus admission, validator `Pass` or runtime-content
authority. The ordinary clip path remains the production fallback.

## Why this experiment exists

Transfer calibration V2 established that the frozen injective extractor can
recover relative modal-frequency/damping structure on an object-disjoint
holdout. It had only one listener row per object. The subsequent Green Goblet
pilot proved the exact row-to-microphone identity for one 15-listener impact
block but deliberately fitted no spatial model.

The open question was therefore falsifiable: can one candidate selected on a
second object predict held listener heights better than a constant-response
baseline on a third object, without changing the model after holdout access?

## Frozen protocol before new listener access

The external preregistration manifest SHA-256 is
`077b9a468aefb4b55a6f06585dc5085dc2b0b04a3fe9468a20ed5d1383d3f2cb`.
It was written before Shell Plate or Skull Cup listener rows `1..14` were
downloaded. It declares prior exposure of Green rows `0..14`, Shell row `0`
and Skull row `0`; the last two came from the earlier modal/damping study.

The object split is fixed:

- development: `93_GreenGoblet`;
- calibration/selection: `51_ShellPlate`;
- one-shot holdout: `60_SkullCup`.

Each object uses microphones `0..14`. Microphone `7` is the relative-level
reference. Anchors are `0,2,4,6,7,8,10,12,14`; held listeners are
`1,3,5,9,11,13`. Per-listener onset is the first sample crossing `2%` of its
own peak. A `65,536`-sample Hann window is projected at that object's 16 frozen
transfer-V2 selected frequencies. Magnitudes are converted to dB and centered
on microphone `7`.

Four candidates were frozen:

1. piecewise vertical linear interpolation;
2. degree-2 polynomial with ridge `0.001`;
3. degree-3 polynomial with ridge `0.001`;
4. vertical Gaussian RBF with sigma `0.52 m` and ridge `0.001`.

Every candidate had to pass the Green development gates. Shell then selected
the lowest deterministic calibration loss among candidates passing both Green
and Shell. The selected candidate identity and loss were serialized and hashed
before the Skull block was opened. Skull received no fitting or reselection.

The conjunctive gates were frozen as:

- at least 12 components;
- median absolute held-listener error at most `7 dB`;
- nearest-rank p90 absolute error at most `18 dB`;
- persistent-component median error at most `8 dB`;
- at least `0.5` of components improved over the microphone-7 constant
  baseline;
- aggregate median-error ratio to that baseline at most `0.9`.

The calibration loss is `median + 0.25*p90 +
2*(1-improved_fraction) + 0.25*persistent_median`.

## Result

All four candidates passed Green development. On Shell, only linear and RBF
passed all gates. RBF had the smaller calibration loss and was selected as
`vertical-rbf-sigma052-ridge001-v1`.

| Partition | Median error | p90 error | Persistent median | Ratio to constant | Improved components | Loss |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Green development | `4.3375 dB` | `13.2101 dB` | `4.9071 dB` | `0.6600` | `0.8125` | `9.2418` |
| Shell calibration | `3.3409 dB` | `13.0449 dB` | `6.1933 dB` | `0.6695` | `0.6250` | `8.9005` |
| Skull holdout | `4.6881 dB` | `13.0040 dB` | `4.0082 dB` | `0.7623` | `0.5000` | `9.9412` |

The Skull result passes all six frozen gates. Its improved-component fraction
is exactly the `0.5` threshold, so the positive result has no margin on that
gate. It supports the declared narrow interpolation claim but is not strong
enough to propose a general spatial representation.

## Reproducibility and bounded acquisition

Two complete command runs are byte-identical:

- report: `abc13a9cbdcaac6118197267f55264a03088130c362758e1212960dbce57389d`;
- selection snapshot: `0aea2b6238c75aff71b781192692954c181ecefbcdae304fb2e8acf49a483320`;
- Shell compressed prefix: `40bab3cd3507a4743bb805c27f73067e001dbcd59601d0eac1625d6605068861`;
- Shell 15-row block: `c1bd696b52b463ae5de5370e3c6e7677021565ef40ffb058491a146fe4155eab`;
- Skull compressed prefix: `96d85e02ad4f5ccd2cf629e1a7bd4845a9e179b26bbe2ecfc8fb35ec3be857cc`;
- Skull 15-row block: `ca2260c333ed20a8cc0cffe1aa360ccf0eb2d0a1b97085c984eb375c9d18607d`.

Each object acquisition validates EOCD, exact central-directory hash, all
small metadata members, row `0` against its previously frozen transfer hash,
and the row-to-impact/angle/distance/microphone/listener identity for rows
`0..14`. Shell transfers `17,429,633` range bytes (`0.74399%` of the archive);
Skull transfers `17,399,455` (`0.74720%`). Raw blocks and reports remain in
the external experiment store and are not repository content.

## Primary-source lineage

- [REALIMPACT project page](https://samuelpclarke.com/realimpact/)
- [Frozen REALIMPACT source repository](https://github.com/samuel-clarke/RealImpact)
- [REALIMPACT CVPR paper](https://jiajunwu.com/papers/realimpact_cvpr.pdf)
- [Shell Plate archive](https://downloads.cs.stanford.edu/viscam/RealImpact/51_ShellPlate.zip)
- [Skull Cup archive](https://downloads.cs.stanford.edu/viscam/RealImpact/60_SkullCup.zip)

The frozen repository revision remains
`fca2bd6cbb7e9f96ac61328d2a0d51594bf01987`; prerequisite transfer report
SHA-256 remains `01346767…c444`; Green manifest and block remain
`fcf44d41…50de` and `8bcffd0a…75ca`.

## Consequences and next discriminator

The spatial-model blocker is narrowed, not closed globally. The next smallest
experiment keeps the selected RBF candidate frozen and evaluates at least two
additional object-disjoint impact blocks. It must add explicit angle and
distance partitions before any 3D/radiation-field claim. If the exact-boundary
improvement gate fails on either new holdout, retain this result as a one-line
pilot and reject promotion of the representation.

All eight exact-domain claims, calibrated domain/OOD/shadow false-pass risk,
material identity, perceptual quality, source-model search, contact projection,
whole-mixer cost and P1 production integration remain open.
