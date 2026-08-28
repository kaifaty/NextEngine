# Source-derived salience-selector synthetic control preflight — PS-2 — 2026-08-28

## Outcome

The scale-invariant modal-salience hypothesis is hash-closed before execution.
External manifest
`e11ffd56dea9af8d906b7029292ca264c38f332d327b358a577a67f1285724f1`
binds runner
`62043142a91e5bd4fe815a0b11c76678935a867d57dae90262336c59b5004ed2`,
the successful adaptive synthetic control, the rejected Ceramic adaptive
counterfactual, deterministic known truth, the existing top-16 comparator,
dependency versions and all gates. Two clean preflights emit byte-identical
report
`75b333113c0a0ce3b71c298848ba0a3ed539fa6fdcb28fd4f8fae4509bcab5f0`
with decision `SalienceSelectorSyntheticControlFrozen`.

This is synthetic-only preflight. It reads no real payload, performs no network
request or physics run, and accesses no Planter payload. It grants no
observation, quality, admission or runtime credit.

## Source boundary and normalization decision

Pinned primary sources establish two separate facts:

1. `audio_dspy.find_freqs` rejects a candidate when a strictly stronger FFT
   bin exists within `±10%` of its frequency;
2. the REALIMPACT notebook calls that helper with `above = 20 Hz`,
   `thresh = 5 dB` and `frac_off = 0.1` on `deconvolved.npy`.

The pinned preprocessing source normalizes the complete deconvolved object by
one full-object global maximum before also writing `deconvolved_0db.npy`. The
public per-object archive available to this study exposes only the latter, and
the current observation protocol reads a bounded 600-row block. Therefore the
notebook's absolute `5 dB` threshold cannot be reconstructed without an
unavailable full-object scale factor.

The frozen candidate deliberately preserves the source's fractional-dominance
rule but replaces the non-transferable absolute threshold with a per-window
relative `-55 dB` floor. Its allowed claim is
`source-derived fractional dominance`; it is not an exact reproduction of the
notebook threshold.

## Frozen candidate and counterexample

- sum Hann-windowed FFT power over 15 outputs;
- keep local peaks from `250..12000 Hz` above the relative floor;
- reject each peak if any strictly stronger FFT bin lies within `±10%` of its
  bin index;
- bound the salient set to 64 candidates;
- repeat at a `900 ms` tail window and keep injective matches within
  `40 cents`; and
- pass persistent candidates to the already supported adaptive RMS-envelope
  decay estimator.

The 48 kHz, 384000-sample fixture contains 16 widely separated persistent
modes from `830..11350 Hz` with known decay exponents. Nineteen dense modes from
`270..699 Hz` start much louder but decay at `30 s⁻¹`. They are intentionally
constructed to occupy the fixed top-16 selector while leaving one
fractionally-dominant transient for the persistence stage to reject. NumPy
`PCG64` noise uses seed `20260828` and standard deviation `2e-6`.

Scale factors `0.125`, `1` and `8` multiply the complete signal. The persistent
FFT bin identity must remain exactly equal for all three variants.

## Frozen gates

- exactly 16 persistent modes, all 16 matched to truth and no false positive;
- truth recall and precision both `1.0`;
- median frequency error at most `40 cents`;
- at least one salient onset transient rejected and no transient left in the
  persistent set;
- existing top-16 comparator truth recall at most `0.50`;
- candidate recall advantage at least `0.50`;
- exact scale-invariant persistent bin selection;
- valid adaptive fits for at least `0.75` of modes;
- median adaptive decay error at most `3 dB/s`; and
- median adaptive fit `R²` at least `0.95`.

Any failure rejects this revision without gate weakening or real-data access.
A full pass proves only discrimination and recovery on the declared synthetic
counterexample.

## Next action

Commit and transfer this checkpoint, then execute the deterministic control
twice. Only byte-identical full passes may authorize metadata-only
preregistration of one previously unopened non-Planter object. Real payload,
mechanics and Planter remain blocked.
