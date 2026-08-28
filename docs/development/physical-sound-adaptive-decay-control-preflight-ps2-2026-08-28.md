# Adaptive modal-decay synthetic control preflight — PS-2 — 2026-08-28

## Outcome

The source-faithful adaptive-decay hypothesis is hash-closed before execution.
External manifest
`926921e26e948f7d1eff5bce5eda1c8eb60ae79b25aa72179f56e5dd6a3ccedf`
binds runner
`fe59b3d6c8845ed9433bab6f225625d53485f4245e7dac5968c4952eb8459c20`,
the prior synthetic pass and Ceramic rejection, deterministic known-truth
fixture, adaptive estimator, fixed-window comparator, dependency versions and
all gates. Two clean preflights emit byte-identical report
`f51e513a784126c91e132cc834dc2cbdce98406d83f76b3efa815dbe973f7b49`
with decision `AdaptiveDecaySyntheticControlFrozen`.

This is synthetic-only preflight. It reads no real payload, performs no network
request or physics run, and accesses no Planter payload. It grants no
observation, quality, admission or runtime credit.

## Source-derived candidate

The candidate follows the pinned REALIMPACT analysis structure rather than
copying V2's universal interval:

1. retain the existing spatial-power selection of 16 frequencies;
2. for every frequency and output, apply causal fourth-order Butterworth
   low-pass then high-pass filters over a `20 Hz` band;
3. sum filtered power over 15 outputs and form a causal RMS envelope with
   `eta = 0.01 s`;
4. locate the per-mode envelope maximum;
5. define noise floor as the maximum envelope dB over the final `10%`;
6. fit between `90%` and `10%` of the peak-to-floor dynamic range; and
7. require at least `20 dB` dynamic range and `50 ms` fit length.

Primary provenance is the REALIMPACT notebook at commit
`fca2bd6c…1987` and `audio_dspy` modal tools at commit
`2ad0b05f…9238`. The implementation is an independent bounded control; no
downloaded source is executed.

## Frozen fixture

- 15 outputs, 48 kHz and 384000 samples;
- the same 16 frequencies and known decay exponents used in the preceding
  successful multi-output control;
- weak direct modal response at sample `240`;
- stronger repeat excitation delayed by `0.55..0.90 s` per mode, preserving
  the same post-excitation decay truth;
- deterministic spatial participation and phase; and
- Gaussian floor from NumPy `PCG64`, seed `20260828`, standard deviation
  `2e-6`, with NumPy `2.5.2` and SciPy `1.18.0` frozen.

The repeat excitation is a stress counterexample for a universal early window,
not a claim about Ceramic physics.

## Frozen gates

- exactly 16 selected modes;
- valid adaptive fits for at least `0.75` of modes;
- median frequency error at most `40 cents`;
- median adaptive decay error at most `3 dB/s`;
- median fit `R²` at least `0.95`;
- complete fixed-window truth association;
- fixed-window median decay error at least `6 dB/s`; and
- adaptive error reduction at least `5 dB/s`.

Any failure rejects this revision. A full pass proves only that the adaptive
statistic can recover known decay in the declared delayed/noisy counterexample.

## Preflight correction

An initial draft report described the causal filter cascade as high-pass then
low-pass while both the official helper and implementation execute low-pass
then high-pass. The description, runner hash and manifest were corrected before
any control execution. Only final preflights `c/d` and hash `f51e513a…7b49`
authorize the run; draft `a/b` authorize nothing.

## Next action

Commit and transfer this checkpoint, then execute the deterministic control
twice. On failure reject it without weakening a gate. Only a byte-identical
full pass may authorize a separately frozen read-only Ceramic adaptive
counterfactual. Real payload, fetching, mechanics and Planter remain blocked.

