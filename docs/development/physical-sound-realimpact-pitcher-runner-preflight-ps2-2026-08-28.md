# REALIMPACT Pitcher calibration runner preflight — PS-2 — 2026-08-28

## Outcome

The local calibration runner preflight passes twice without network or reserved
audio access. The repository now generates a deterministic synthetic signal
through the exact frozen Rust `injective-modal-16-fft65536-v2` implementation.
Fixture report `2e3db2d3f231d69b67b2a3a094f83fbc7ed10144a02f4c424d75bbbb2f85572f`
and sample block `c8316f81da79b9823cbeaf1c0c49ee1708d2112fca8b4905a792546fd3f40477`
repeat byte-identically.

The independent Python runner reads those exact f64 samples and reproduces all
16 Rust modes. Its maximum absolute numeric difference is `3.02336e-12`, below
the frozen `2e-7` absolute tolerance. It also parses and validates the exact
Pitcher geometry block, exercises the one-refit 16-of-64 dynamic program and
reconstructs the complete `90/510` listener split. Two preflight reports repeat
at `6e60d71faba4808ee6cfd5990e3fc16fc14e7f9e0ca78b272c0756984bfcfd2d`.

Decision: `PitcherCalibrationRunnerPreflightSupported`. This is a local
implementation/parity checkpoint only. Audio execution remains explicitly
disabled in this runner revision. It grants no real transfer, material,
quality, admission, runtime or ProductCheck credit.

## Frozen controls

The Rust fixture generator binds source
`131bbf42d01e633b6cac0c4e0340178f3f5a0a83324bb64630d8423da8179ca4`
and emits four seconds of exact `f64le` samples at 48 kHz. Sixteen damped modes
from 311 to 7013 Hz exercise onset, peak selection, injective tail matching and
damping tracks. The report records zero network requests and zero REALIMPACT
payload bytes.

The Python preflight revision
`cf93b1fc436d1e04965142e7c846d665529565025b6421f0ea6164c47fd06772`
validates:

- calibration manifest `c60621cc…4a7`;
- geometry block `bcd54087…9acc`, its `NEPSGEO1` layout and required arrays;
- exact Rust fixture/report hashes and complete analysis parity;
- the strictly increasing dynamic-program assignment and one geometric-mean
  scale refit; and
- the angle-major `600` listener coordinates, normalization row 7, 90 anchors
  and 510 held conditions.

The synthetic mapping control selects proxy indices
`[2,3,5,6,8,11,12,16,20,23,29,36,39,48,56,63]`, with median/p90 errors
`0.03823/0.10811` octaves. These numbers validate the algorithm only; the
fixture frequencies are not a Pitcher calibration result.

## Exact spatial projection helper

`realimpact-transfer-project` is implemented for the future decoded 600-row
block. It includes the same source file bound by the manifest rather than
approximating its direct Hann-windowed complex projection in Python. The helper
requires exactly 600 rows × 230470 float32 samples, exactly 16 ordered measured
frequencies and normalization row 7. It emits a mode-major normalized complex
block, per-row hashes and a deterministic report while reading no network or
Planter data.

Only dimension/source tests have run because Pitcher audio is still sealed.
The helper cannot accept another object shape or arbitrary row count under this
command.

## Remaining execution choice

Before the single 512 MiB request, a final execution manifest must bind the
new immutable preflight/helper source hashes, the external Bempp environment,
the 56-direction source artifact, spherical expansion origin, staged
acquisition/decode identities and exact cooker/RBF evaluation implementation.
This closes choices that the higher-level calibration manifest intentionally
described semantically but did not encode as executable bytes.

## Reproduction

```text
cargo run -p xtask -- physical-sound-registry \
  realimpact-transfer-fixture --output <external-fixture-run>

lab/.venv/bin/python \
  lab/scripts/physical_sound_realimpact_pitcher_calibration.py \
  --manifest <external-pitcher-calibration-manifest.json> \
  --rust-fixture <external-fixture-run> \
  --output <external-preflight-run> \
  --preflight-only
```

## Smallest next action

Implement and hash-close the separate execution runner/manifest. Run its local
Bempp/cooker controls without REALIMPACT audio. Only then acquire the exact
Pitcher prefix once, decode rows `0..599`, execute calibration twice from the
immutable cache and stop before Planter on any failure.
