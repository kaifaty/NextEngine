# Multi-output decay estimator control preflight — PS-2 — 2026-08-28

## Outcome

The first synthetic counterfactual after the Ceramic Cup shared-decay result is
hash-closed before execution. External manifest
`cd8ee8565d391a5543d39901c35b223c710f71968308db1e52ba7e50694b2078`
binds runner `f0483c397843994d23793a4f3392dd49d9ce5d90ec218caa1b10f2a846a2068e`,
the deterministic 15-output fixture, spatial-modal-power estimator,
node-contaminated single-output comparator, exact source hashes and all success
gates. Two clean preflights emit byte-identical report
`88b017a1b35b1ff1bf350fc52fb30e62b38acbfc5e796913fe0980e4c83b1153`
with decision `MultiOutputDecayControlFrozen`.

This is synthetic-only evidence. Preflight performs no real-data access,
network request, physics or Planter access and grants no validator, admission,
quality or runtime credit.

## Frozen fixture

- 15 outputs, 48 kHz, 192000 samples and onset sample 240;
- 16 exact modal frequencies from `311` through `7013 Hz`;
- amplitude decay exponents `1.15 + 0.09i s⁻¹`, with exact expected dB/s;
- deterministic mode/channel participation and phase;
- channel 7 direct gain `0.01`, representing a near-node;
- channel 7 delayed local component gain `0.30` at `0.35 s`; and
- channel-major `<f8` bytes hashed in the execution report but not stored.

The delayed component is deliberately confined to the near-node channel. It is
not a proposed room model; it is a counterexample intended to make the current
single-output early-decay statistic fail while leaving the shared object modes
identifiable from the other outputs.

## Candidate estimator

`spatial-modal-power-15-v1` retains all V2 frequency bounds, FFT sizes, peak
separation, tail matching, time windows, three-bin tracking, regressions and
summary metrics. It changes one operation only: per-frame/per-frequency power
is summed over 15 outputs before peak selection and decay fitting. For one
input, this is the squared Euclidean norm of the multi-output response vector;
no cross-channel phase or spatial parameter is fit.

The frozen comparator is the unchanged V2 extractor on channel 7.

## Frozen gates

- exactly 16 selected modes;
- persistent recall at least `0.75`;
- median frequency error at most `40 cents`;
- decaying-mode fraction at least `0.75`;
- median tail-prediction RMSE at most `24 dB`;
- median error against known decay at most `3 dB/s`;
- channel-7 decaying fraction at most `0.49`; and
- spatial-minus-single decaying fraction improvement at least `0.25`.

The bound Rust fixture still repeats through the Python parity implementation
with maximum absolute difference `3.0233593406592263e-12`.

## Next action

Commit and transfer this checkpoint, then execute the deterministic control
twice. On any failure reject this estimator revision without weakening a gate
or reusing real rows. Only a byte-identical full pass may authorize a separately
frozen read-only Ceramic Cup multi-output counterfactual.
