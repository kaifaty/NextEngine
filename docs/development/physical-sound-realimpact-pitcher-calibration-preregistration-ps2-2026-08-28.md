# REALIMPACT Pitcher calibration preregistration — PS-2 — 2026-08-28

## Outcome

The Pitcher-only calibration protocol is frozen before acoustic payload access.
External manifest
`c60621ccf30442ba8fe4c25533325782ef1807c1be98101eae78fb04d18ad4a7`
binds the supported exact-weld Pitcher geometry block, one exact compressed
audio prefix, decoder and row identities, the 16-to-64 frequency mapping,
Bempp/cooker inputs, controls, conjunctive gates and stop-before-Planter
fallback. Two clean validator runs emit byte-identical report
`2ae1bc0b1f77f70c18d4a61a965e1dd95799579a6e56d91f57be84a5a9799c72`.

Decision: `PitcherCalibrationProtocolFrozen`. The validator reads the bounded
external geometry block and prerequisite reports, performs zero network
requests and reads zero reserved audio payload bytes. This is preregistration
only. It gives no measured spatial-transfer, material, naturalness, corpus
admission, runtime or ProductCheck credit.

## Frozen calibration input

- object: `65_PitcherCeramic`, calibration only;
- geometry block: `bcd54087…9acc`, `3718246` bytes, `NEPSGEO1`;
- geometry state: `1024/2048` BEM vertices/faces, 64 scalar modes, maximum
  relative eigen residual `3.14076e-13`;
- archive entry:
  `65_PitcherCeramic/preprocessed/deconvolved_0db.npy`, CRC32 `53f05f2d`;
- request budget: one exact `536870912`-byte compressed prefix;
- decoded shape: NPY `<f4`, C order, `[3000, 230470]`, 128-byte header;
- selected rows: impact ordinal 0, rows `0..599`, exactly `553128000` raw row
  bytes; and
- failure semantics: if that fixed prefix cannot finish row 599, publish an
  immutable rejection. Do not request another byte or change the decoder.

Planter audio remains prohibited in this phase. Even a Pitcher calibration pass
can open it only after a new one-shot holdout manifest binds the immutable
calibration result.

## Frozen analysis

Metadata must prove the `10 × 4 × 15 = 600` angle/distance/microphone product
and the angle-major, distance, microphone row order. Row 7
(`angle=0,distance=0,micID=7`) supplies normalization and the only 16 measured
frequencies allowed to fit the scalar geometry-to-frequency scale.

The extractor is `injective-modal-16-fft65536-v2`. Its Rust entry/DSP and the
spatial projection DSP are bound by source hashes. A future calibration runner
must first prove selected-byte identity and numeric parity on a committed
synthetic fixture; a Python port cannot silently redefine the extractor.

The scalar model is `f=alpha*lambda`. Calibration enumerates every `16 × 64`
candidate scale, selects the minimum squared-log2-error strictly increasing
16-of-64 assignment by dynamic programming, applies the frozen tie breaks,
refits `alpha` once by geometric mean and reruns the assignment once. Median
and p90 frequency errors must be at most `0.20/0.35` octaves.

For each admitted mode, the frozen candidate projects the geometry field
through Bempp-cl `0.4.2` and the already validated full-angular cooker. It may
fit no spatial audio value. The controls retain the fixed 3D coordinate RBF
(`sigma=0.52 m`, ridge `0.001`) on 90 anchors and the normalization-listener
constant. All 510 remaining listeners are held across height, angle and
distance.

At least 12 modes must remain after eigensolver, mapping, reference-magnitude,
GMRES and cooker checks. Every held stratum must satisfy the absolute gates;
the candidate must also beat both constant and RBF comparisons. A per-mode
candidate failure selects RBF for that mode; any object-level or conjunctive
failure selects the existing authored clip.

## Validator boundary

The repository command
`physical-sound-registry realimpact-transfer-preregister` recognizes the exact
calibration manifest hash. It validates five prerequisite JSON artifacts, the
Pitcher geometry bytes and four local implementation hashes. It cannot fetch
the network or accept an alternate manifest. The report records the exact
prefix, decoded dimensions, split, extractor, mapping and claim boundary.

The manifest, geometry/audio payload, reports and future decoded blocks remain
external. No dataset, recording, generated audio or cache enters the
repository.

## Reproduction

```text
cargo run -p xtask -- physical-sound-registry \
  realimpact-transfer-preregister \
  --manifest <external-pitcher-calibration-manifest.json> \
  --output <external-empty-run-directory>
cmp <run-a>/report.json <run-b>/report.json
```

## Smallest next action

Implement the bounded calibration runner and its synthetic Rust-parity fixture.
Only after that local check passes may the runner make the one frozen 512 MiB
Pitcher range request. Repeat the analysis from the cached immutable prefix.
On rejection stop before Planter; on calibration support freeze a separate
Planter holdout manifest before its payload is touched.
