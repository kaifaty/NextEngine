# Physical sound V20 M0b — confound-resistant metric research

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `BOUNDED_RESEARCH_COMPLETE / SUCCESSOR_SELECTED / I1_UNOPENED` |
| Trigger | [M0 repeat-exact reject](physical-sound-v20-m0-phase-consistent-metric-result-2026-09-01.md) |
| Scope | Opened development `1401…1412` only; no B0/F0 refit and no fresh or real value |

## Falsifiable question

Can an amplitude-residualized spectral-shape metric and a normalized decay-slope
metric preserve every successful M0 control while creating the frozen `0.90x`
gap for the four rejected cells?

Three explanations were considered:

1. **H1 — amplitude confound:** absolute DE measures modal excitation strength
   as well as damping, so an accepted F0 gain field can dominate a harmful
   damping perturbation.
2. **H2 — log compression/confound:** raw MRLM is monotonic but mixes broadband
   level and modal-gain variation with peak displacement, leaving too little
   margin between 20 and 40 cents.
3. **H3 — entire acoustic family is unusable:** no deterministic waveform
   statistic can separate the accepted component envelopes from the declared
   perturbations, so integration would have to remain blocked.

M0 supports H1 and H2 and contradicts H3: MRSC, transient energy, physical
owners and every monotonicity gate already pass.

## External evidence and limits

- [Parallel WaveGAN](https://arxiv.org/html/1910.11480v2#S3.SS2) defines
  spectral convergence and log-STFT magnitude as complementary training losses.
  It supports retaining both views, but supplies no validation threshold and
  does not prove that raw log magnitude is independent of amplitude.
- [Neural Acoustic Context Field](https://arxiv.org/abs/2309.15977) computes a
  backward cumulative squared-magnitude curve and compares its log decay at
  multiple resolutions. It supports measuring decay trend rather than isolated
  absolute cells; its room-acoustic thresholds do not transfer to impacts.
- [Schroeder's original method](https://labrosa.ee.columbia.edu/~dpwe/papers/Schro65-reverb.pdf)
  derives a backward-integrated squared impulse response to smooth phase/modal
  beating in decay measurement. It supports cumulative energy, not a reusable
  numeric gate for this corpus.
- [Modal log-decrement identification](https://doi.org/10.1016/j.jsv.2011.04.013)
  exploits exponential free-response slope to estimate damping. It supports
  making slope, rather than initial amplitude, the owner of damping error.

These sources motivate operators only. All margins below come solely from the
frozen development controls and must be revalidated on fresh I1 after P0c.

## Bounded counterfactual comparison

The exact M0 corpus and ladders were reevaluated without changing B0/F0. Each
candidate uses corpus p95 and the same base acceptable controls.

### Frequency owner

| Candidate | Uniform ratio | Alternating ratio | Decision |
| --- | ---: | ---: | --- |
| raw MRLM | `0.923528202x` | `0.905518841x` | reject |
| MRSC | `0.890030102x` | `0.892955907x` | retain blocking |
| frame-centered log-magnitude L1 | `0.852403401x` | `0.807065192x` | select |
| frame-centered log-magnitude RMS | `0.807166x` | `0.788544x` | redundant, reject |
| per-frame unit-L2 magnitude | about `0.880x` | about `0.886x` | passes but hides scale by a less interpretable normalization |
| frequency derivative of log magnitude | about `0.664x` | about `0.677x` | passes but is more sensitive to bin noise |

Frame-centered log magnitude subtracts each frame's mean log magnitude before
the L1 comparison. The signed gain/gradient physical layer still owns absolute
and per-mode amplitude, so this residualization removes a duplicate confound
without removing amplitude validation.

### Damping owner

For each of three frequency bands, the candidate computes frame energy,
Schroeder-style backward cumulative energy, normalization by the first
cumulative value, log with a fixed floor, and adjacent-frame log slope.

| Candidate | Positive ratio | Negative ratio | Decision |
| --- | ---: | ---: | --- |
| absolute time/frequency DE | `1.603109918x` | `1.577244048x` | reject |
| normalized cumulative EDC level | `1.027622082x` | `1.061913362x` | reject; modal mixture still shifts level |
| normalized EDC log-slope RMS | `0.773503069x` | `0.831442570x` | select |

The selected slope metric also stays strictly monotonic for both damping
ladders. It compares attenuation rate while band energy scale cancels exactly.

## Decision

Freeze M0b with only two semantic changes:

1. retain MRSC as blocking; replace raw MRLM's blocking role with
   frame-centered log-magnitude L1, while reporting raw MRLM as diagnostic;
2. replace absolute DE's blocking role with normalized backward-EDC log-slope
   RMS, while reporting absolute DE as diagnostic.

Identity, polarity, transient, physical/hard, control ladders, `0.90x` margin,
Spearman `0.90`, role isolation, resource ceilings and legacy attribution stay
unchanged. No threshold is selected from I1.

Rejected alternatives are lowering the margin to `0.95`, squaring MRLM merely
to reshape its ratio, dropping acoustic damping in favor of parameter equality,
or tuning B0/F0 against the metric. Each would hide rather than remove the
observed confound.
