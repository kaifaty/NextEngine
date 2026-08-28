# PS-2 Iron Mortar broad-band holdout execution preregistration — 2026-08-28

## Outcome

Implement the frozen four-stage execution for the object-disjoint
`43_IronMortar` holdout: zero-access preflight, one four-request acquisition,
offline decode of one synchronized 15-microphone group, then two identical V2
analyses.

The execution implements the already frozen protocol without changing any
range, threshold, region policy, estimator, fit, pruning or control.

## Frozen stages

1. `preflight` binds protocol runner/manifest/report and every imported source;
2. `acquire` makes the exact three metadata requests and one 32 MiB audio-prefix
   request once;
3. `decode` proves all 600 impact-zero metadata rows and writes exactly 15 full
   f32 waveform rows;
4. `analyze` detects the 5% spatial onset, uses exactly 60,000 samples and runs
   complete rank-7 partial-SVD discovery, fit, spatial split, damped ablation and
   predictive windows.

Acquisition failure is a published rejection and permits no retry. Decode or
analysis failure stops with authored-clip fallback. Analysis runs twice from the
same immutable decode and must be byte-identical.

## Claim boundary

A passing analysis may support object-disjoint real method transfer only. It
does not establish material identity, sound quality, exact-domain admission,
physical parameter recovery or production runtime fitness. Every report records
zero physics and Planter access; post-acquisition stages record zero new network
or audio-payload access.

## Frozen evidence identity

Runner, execution manifest and repeated zero-access preflight hashes are filled
after implementation is committed. They MUST be committed before acquisition.
External artifacts remain under
`~/.codex/experiments/nextengine/physical-sound/ps2-realimpact-iron-mortar-broadband-holdout-v1/`.
