# Source-derived salience-selector synthetic control result — PS-2 — 2026-08-28

## Outcome

Two executions of the frozen synthetic control emit byte-identical report
`fa94071059c3e3f6dd3e71eed187affe08c0ca36c120cd483b8c2e8da9560b3e`
and decision `SalienceSelectorSyntheticControlSupported`. All 14 conjunctive
gates pass.

This supports the relative spatial-dominance plus tail-persistence selector on
one declared known-truth counterexample. It does not validate REALIMPACT,
Ceramic Cup, a material identity, mechanics, perceptual quality, admission,
runtime use or `Pass`. The two runs make zero network requests, read zero real
or Planter payload bytes and execute no physics.

## Repeated result

| Measure | Source-derived candidate | Existing top-16 comparator |
| --- | ---: | ---: |
| Selected modes | 16 | 16 |
| Truth matches | 16 | 4 |
| False positives | 0 | 12 |
| Truth recall | 1.0 | 0.25 |
| Truth precision | 1.0 | 0.25 |
| Median matched frequency error | 0.001941 cents | 0.011253 cents |

The onset spectrum has 35 local candidates. Fractional dominance reduces that
to 17: all 16 persistent truth modes plus one strongest member of the dense
transient cluster. The `900 ms` persistence stage rejects that last transient,
leaving 16 truth modes and no transient match. Candidate recall therefore
improves by `0.75` over the fixed top-16 selector.

All three complete-signal scale variants (`0.125`, `1`, `8`) select the exact
same 16 FFT bin indices and signature
`c2c0e2e5467eee3ba87b54e9a7f158f5eb208953c27dc368a4d8dee5e7695f2f`.
This closes the synthetic scale-invariance gate introduced because the public
REALIMPACT object archive does not expose the full-object normalization needed
to reproduce the notebook's absolute threshold.

The already supported adaptive estimator composes with the persistent set:

- valid adaptive fits: `16/16`;
- median decay error: `0.395200 dB/s`; and
- median fit `R²`: `0.990873`.

The deterministic 46,080,000-byte fixture hashes to
`0ba06a4f3f149c1bed14d55daa4d557dba88cf777e416494081ab163a6227c68`.

## Interpretation

**Supported:** the previous top-16 ranking can be causally displaced by dense,
short-lived onset components; fractional spectral dominance followed by a
separate tail-persistence test recovers known persistent modes in this case and
is invariant to scalar gain.

**Not supported:** an exact copy of the REALIMPACT absolute `5 dB` selector,
general recovery on recordings, or an explanation of the opened Ceramic
failure. Ceramic stays closed to method development.

## Decision and next action

Status is
`SALIENCE_SELECTOR_SYNTHETIC_CONTROL_SUPPORTED / SCALE_INVARIANCE_SUPPORTED /
CURRENT_TOP16_COUNTEREXAMPLE_CONFIRMED / INDEPENDENT_REAL_OBJECT_REQUIRED /
MECHANICS_BLOCKED / PLANTER_SEALED`.

Select one previously unopened development object from the frozen official
roster using a deterministic metadata-only rule and freeze its archive-entry
discovery before any member payload access. Apply the selector unchanged on
that object. Any failure rejects real transfer; it does not authorize tuning,
Ceramic reuse or physics.
