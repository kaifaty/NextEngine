# PS-2 broad-band common-pole control preregistration — 2026-08-28

## Outcome

Freeze the first synthetic end-to-end control that does not receive truth band
centres. It must discover energetic Gabor regions across 500–12,000 Hz, run the
previously supported common-pole core in neighboring bins, merge duplicate
estimates, fit per-output complex amplitudes, prune only after estimation and
reconstruct the modal signal.

Success still grants no real-transfer or sound-quality claim. It authorizes
only a separately hash-closed read-only counterfactual on the already opened
Iron rows.

## Calibration/holdout separation

Discarded exploratory seed `20260830` was used to choose the fixed discovery,
order-support, clustering, energy and reconstruction thresholds. It is not an
evidence run and its values are not stored as an artifact. The runner can
generate only the previously unseen PCG64 holdout seed `20260831`; the manifest
forbids calibration-seed execution and all post-preflight tuning.

## Frozen fixture

- 15 outputs, 48 kHz, 60,000 samples, onset 240;
- seven strong truth modes at `1373, 1401, 3479, 3512, 6237, 9488, 11231 Hz`;
- amplitude decays `1.8, 4.5, 2.7, 8.5, 5.5, 14, 9 /s`;
- one weak nuisance resonance at `7313 Hz`, decay `3/s`, amplitude `0.04`;
- one shared broadband burst decaying at `70/s` plus independent `1e-8` noise;
- deterministic output-dependent gains/phases, holdout seed `20260831`.

The 28 Hz and 33 Hz close pairs remain below the `46.875 Hz` Gabor-bin
spacing. Truth frequencies are used only by the final synthetic comparison,
never by discovery, pole estimation, clustering, fitting or pruning.

## Frozen candidate

1. Compute the first 256 Blackman-Harris 1024/32 Gabor frames for every
   positive-frequency bin.
2. Sum complex energy over frames and outputs. In 500–12,000 Hz, select local
   maxima at or above `-30 dB` relative to the largest bin.
3. Analyze each selected bin and its ±1-bin neighbors with the frozen
   24-lag/15-output common-pole core. Retain an analysis only when its selected
   order-score margin is at least `50×`.
4. Single-link pole estimates within `1 Hz`. Choose the member with greatest
   order-score margin, then bin energy, then lower bin.
5. Jointly fit cosine/sine coefficients for every cluster and every output over
   all post-onset samples. This is equivalent to one complex amplitude per
   mode/output.
6. Compute reconstructed mode energy and prune clusters below `-25 dB`
   relative to the strongest only after estimation/fitting.
7. Resynthesize retained modes. Measure strong modal-truth NRMSE, full observed
   NRMSE and observed NRMSE after the declared burst has decayed by 60 dB.

Scale variants `0.125/1/8` test only the input-driven region/bin signature;
exact numeric pole-scale invariance was already closed by the parent control.

## Frozen gates

- exactly 6 regions and 18 neighborhood bins are selected;
- at least 10 duplicate estimates are removed;
- 8 pre-prune clusters become 7 retained and 1 pruned cluster;
- all 7 strong truth modes match, with zero retained false positives;
- the 7313 Hz weak nuisance is the pruned cluster;
- maximum frequency/decay error is `0.25 Hz/0.75/s`;
- strong modal-truth NRMSE is at most `0.08`;
- full observed NRMSE is at most `0.20`;
- post-transient observed NRMSE is at most `0.10`;
- region/bin discovery is exact under all three scales.

Runner, manifest and two identical zero-real preflights MUST be committed before
the holdout run. Runs A/B must then be byte-identical. Failure rejects the
revision without changing fixture, thresholds or seeds. No network, real
payload, Iron timing/parameter reuse, physics, Planter, quality admission or
runtime credit is allowed.

## Frozen evidence identity

- runner SHA-256:
  `317fa3a2e4f05f6c212ccdbee0575554d3610753573db359a68f3e756a73ec8d`;
- external manifest SHA-256:
  `a8f59bdea71862c5d670448bc058f8838bfa41b292d8fea26dead90fb96d040b`;
- preflight A/B report SHA-256:
  `eab320847fc1c2b3a18443ec49701e97256ac5f50fa263f32fbff5e3a53ce8bc`;
- the preflights are byte-identical and each records zero real-payload bytes,
  network requests, physics runs and Planter payload bytes.

The manifest and preflight reports live outside Git under
`~/.codex/experiments/nextengine/physical-sound/ps2-broadband-common-pole-control-v1/`.
This document plus the committed runner bind them before either numeric holdout
run is permitted.
