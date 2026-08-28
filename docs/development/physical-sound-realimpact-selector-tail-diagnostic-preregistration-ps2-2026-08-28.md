# PS-2 Iron Skillet selector/tail diagnostic preregistration — 2026-08-28

## Outcome

Freeze one read-only diagnostic on the already opened `17_IronSkillet`
impact-zero block. The experiment separates two explanations for the rejected
real transfer without tuning either method:

1. the source-derived ±10% dominance rule selects a composition whose adaptive
   decays are less identifiable than the unchanged top-16 comparator; and
2. the fixed 900 ms persistence observation is later than the useful modal
   interval.

This is a causal diagnostic, not a candidate-selection run. It cannot promote
an earlier tail, weaken a threshold, authorize another object, run mechanics or
grant quality/runtime admission.

## Exact evidence and competing hypotheses

Repeated parent report `a9c4ae36ae04bf9a9772cb8a1a6c55e2862ed918eaff3ae6d9a2043acb07206d`
rejects transfer because source-derived persistence is `6/17 = 0.352941…`
against `0.50`, and only `3/6 = 0.50` persistent candidates have a valid
adaptive fit against `0.75`. Its unchanged top-16 diagnostic has valid and
decaying fractions `0.9375/0.9375`.

The serious hypotheses are:

- **H-selector:** ±10% dominance sparsifies or changes the real onset set in a
  way that loses identifiable modes, despite passing the synthetic fixture.
- **H-tail:** the 900 ms tail observation loses real modes that remain visible
  earlier.
- **H-both:** both mechanisms contribute.
- **H-neither:** neither frozen ablation closes its unchanged threshold, so the
  current FFT peak/persistence framing is insufficient.

## Primary-source research

- Commit-pinned audio_dspy `find_freqs` first finds local FFT peaks and then
  rejects one whenever a stronger bin exists within ±10%. Retrieved source
  SHA-256 is `a1e5b991…cc494`.
  [Pinned implementation](https://raw.githubusercontent.com/jatinchowdhury18/audio_dspy/2ad0b05f81b014c27612f6f91087265ee52e9238/audio_dspy/modal_tools.py)
- The commit-pinned REALIMPACT notebook applies that modal workflow to measured
  impact data. Retrieved notebook SHA-256 is `2a8ce3c6…fc7b4`.
  [Pinned notebook](https://raw.githubusercontent.com/samuel-clarke/RealImpact/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/modes_dsp_sweep.ipynb)
- Sirdey et al. model impact recordings as sums of exponentially damped
  sinusoids. On a real struck metal plate, full-band analysis is inadequate;
  their Gabor-subband ESPRIT route extends the observation horizon, estimates
  per-band order with ESTER and removes insignificant/duplicate components
  after estimation. Retrieved PDF SHA-256 is `d52c9a3f…9bdd0`.
  [DAFx-11 paper](https://www.dafx.de/paper-archive/2011/Papers/61_e.pdf)

The bounded inference is that another threshold adjustment is not justified.
If the existing-block diagnostic supports a selector-composition problem or is
inconclusive, the next candidate should be proved first on a synthetic
multichannel damped-sinusoid control, with subband estimation preceding
perceptual/energy pruning.

## Frozen execution

| Item | Frozen value |
| --- | --- |
| Object and rows | `17_IronSkillet`, impact 0, rows `0..14` only |
| Decoded block | `e26d1df1…7eeb`, `553,317,600` bytes total |
| Onset | unchanged 15-output spatial-norm rule; expected sample `29` |
| Candidate | source-derived ±10% dominance selector |
| Comparator | existing top-16 fixed-separation selector, diagnostic only |
| Adaptive estimator | unchanged spatial adaptive RMS envelope |
| Tail starts | `100`, `200`, `400`, `900` ms |
| FFT | unchanged `65,536` samples; every tail view spans `1365.333…` ms |
| Matching | unchanged injective assignment within `40` cents |
| Selector threshold | adaptive-valid fraction `0.75` |
| Tail threshold | persistence recall `0.50` |

The three earlier tails are a preregistered logarithmic diagnostic grid; 900 ms
is the rejected baseline. The report publishes all four and MUST NOT choose the
best. Because each FFT covers about 1.365 seconds, a tail start is explicitly a
long support window, not an instantaneous measurement.

## Frozen classification

- `SourceSelectorCompositionMismatchSupported`: source onset adaptive-valid is
  below `0.75` while comparator onset adaptive-valid meets `0.75`, and no tail
  timing criterion is met.
- `FixedTailTimingMismatchSupported`: baseline source persistence is below
  `0.50`, at least one earlier preregistered tail meets `0.50`, and no selector
  composition criterion is met.
- `CombinedSelectorAndTailMismatchSupported`: both conditions hold.
- `ExistingBlockAblationInconclusive`: neither condition holds.

The manifest, runner and a repeated zero-network preflight MUST be hash-closed
and committed before analysis. Analysis then runs twice from the same block;
reports must be byte-identical. No network request, additional payload, object
substitution, threshold/tail-grid edit, denoising, physics solver or Planter
access is permitted.

## Hash-closed preflight

The frozen runner SHA-256 is `8e815598…6b9bf`; external manifest SHA-256 is
`00d05074…0281e`. Preflights A/B are byte-identical at
`4fc075c4…5ff2c` and decide `IronSkilletSelectorTailDiagnosticFrozen`.
They bind both repeated rejected parent reports, the decode report, the complete
decoded-block identity, all six source files, the four tail starts, unchanged
thresholds and the three primary research artifacts. Each reports zero network
requests, zero additional payload, zero physics runs, zero Planter bytes and no
quality/runtime credit. Numeric analysis has not run at this commit boundary.
