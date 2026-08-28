# REALIMPACT Pitcher execution preflight — PS-2 — 2026-08-28

## Outcome

The final Pitcher calibration execution revision is hash-closed before any
reserved audio access. Two local runs produced byte-identical report SHA-256
`94d5e1e6eeed3db8ec35aef8d1a41ed0b12c3ec8a64e8df50c164d686c239b32`
and decision `PitcherCalibrationExecutionPreflightSupported`.

This checkpoint authorizes exactly the already preregistered one-request
Pitcher calibration. It does not establish a calibration result, measured real
3D transfer, material identity, perceptual quality, domain admission or runtime
credit. Pitcher and Planter payload bytes read remain zero; Planter stays
sealed.

## Frozen execution identity

The external execution manifest is
`pitcher-execution-manifest.json`, 8,025 bytes, SHA-256
`8e791327595f43c672361e486ed50aa8512f1c8068de5efdbe6212a81df8ba45`.
It binds:

- execution script SHA-256
  `c5900a9c01e1d882d671d43d53041470b5dbf56f9474017f6a60fc2930ecbbfa`;
- Python `3.12.13`, Bempp-cl `0.4.2`, NumPy `2.5.2`, SciPy `1.18.1`,
  Numba `0.67.0` and the `numba` device interface;
- the exact Rust extractor and spatial projector source hashes, including
  extractor DSP `131bbf42…9ca4` and spatial DSP `f3de4189…f834`;
- the exact-weld original Pitcher bbox centre
  `[-0.010383875, 0.00011222500000000052, 0.074711685]`, diagonal
  `0.25945066250909654 m`, the 56 frozen unit directions and radii
  `2L/4L/10L`;
- Bempp DP0/P1 spaces, `default_nonlocal` boundary and `dense` potential
  assemblers, double precision, GMRES relative tolerance `1e-8`, restart `200`
  and maximum `1000` iterations;
- the full-real outgoing cooker degrees `2/4/6`, ridge `1e-12`, smallest-pass
  selection rule and all frozen held-field gates;
- one exact HTTPS range request for compressed bytes
  `4900621..541771532`, raw-DEFLATE NPY decoding and exactly 600 rows / 553,128,000
  decoded bytes;
- the preregistered frequency, per-mode, held-stratum, RBF-comparison and stop
  gates without weakening them.

The runner derives the expansion centre and length again from the frozen
`welded_points` array and rejects manifest drift. It also requires all parent,
geometry, runner-parity and cooker artifacts by exact hash, schema, status and
decision before it can authorize acquisition.

## Local controls

Both runs used the exact Rust fixture report `2e3db2d3…5572`. The Python/Rust
extractor maximum absolute error remained `3.0233593406592263e-12`. A synthetic
degree-4 full-angular field was fit on all 56 directions at `2L` and evaluated
without refitting at `4L/10L`:

| Metric | Result |
| --- | ---: |
| Fit max peak-normalized complex error | `7.78036e-14` |
| Held max peak-normalized complex error | `1.94377e-13` |
| Held max active relative complex error | `2.94879e-13` |
| Held max active magnitude error | `2.38575e-12 dB` |
| Held max active phase error | `1.14246e-11°` |
| Held minimum directional complex correlation | `1.0` |

The 56-direction f64 block hash is
`4c21d4318b396be07e831f8625ef29865952b60f5ea6d48fa80f492bceffa945`.
The control proves only that the pinned local basis/fit/evaluation path can
recover an exactly representable outgoing field.

## Access accounting

For both runs:

- network requests: `0`;
- reserved audio payload bytes read: `0`;
- Planter audio payload bytes read: `0`.

The Bempp import warning about the optional Gmsh interactive module is
non-gating; no geometry shape helper or plotting path is used.

## Reproduction

Use the pinned Bempp virtual environment and the external immutable fixture:

```text
<bempp-venv>/bin/python \
  lab/scripts/physical_sound_realimpact_pitcher_execute.py \
  --stage preflight \
  --execution-manifest <experiment>/pitcher-execution-manifest.json \
  --rust-fixture <experiment>/extractor-parity-fixture-a \
  --output <fresh-external-output>
```

The two reports were compared with `cmp` after independent fresh output
publication.

## Next action

Commit and transfer this hash-closed runner checkpoint. Then invoke `acquire`
once into an immutable external cache. A failed or incomplete request is a
published rejection: no retry, prefix growth, decoder change or Planter access
is allowed. If acquisition succeeds, decode rows `0..599` once and execute the
same analysis twice from that cache without further network access.
