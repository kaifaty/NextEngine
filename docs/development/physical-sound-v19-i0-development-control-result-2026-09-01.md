# Physical sound V19 I0 — development control result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `DEVELOPMENT_CONTROL_REJECT / I0_NOT_RUN / INTEGRATION_VALUES_UNOPENED / V19_CLOSED` |
| Protocol under test | [V19 P0b](physical-sound-v19-p0b-field-integration-protocol-2026-09-01.md) |
| Inputs | Exact passing [V18 B0](physical-sound-v18-b0-deterministic-global-baseline-result-2026-09-01.md) and [V19 F0](physical-sound-v19-f0-residual-harmonic-field-result-2026-09-01.md) external artifact trees |
| Allowed claim | The frozen I0 waveform contract is inconsistent with the accepted component-error contract on already-opened development identities |
| Product effect | None; authored clips remain the complete fallback and no real, protected, cooker, demo or runtime role opens |

## Why the control ran before I0

P0b fixed both component tolerances and complete-render gates before I0. After
F0 passed, the first non-training composition on the already-opened development
role showed that two frozen gates can reject a composition whose individual
physical parameters remain inside their accepted bounds. Opening fresh I0
identities would not resolve that contradiction; it would only spend one-shot
evidence on an invalid measuring instrument.

The control therefore loaded only the exact B0/F0 artifacts and regenerated the
already-opened V19 development objects. It generated zero `1601…1612` mesh,
truth, prediction, metric or waveform values. No real waveform, force sample,
external dataset, protected calibration, method holdout or admission shadow was
read.

## Exact observation

The development composition passes B0 parameter gates, every F0 field gate,
coverage, fallback completeness, all four hard mutations, modal peak matching,
the frozen multiresolution spectrum gate and zero-access checks. It rejects only
the phase-sensitive early-waveform and Hilbert-envelope gates.

| Endpoint | Frozen gate | Observed | Result |
| --- | ---: | ---: | --- |
| Frequency median / p95 | `<=20 / 60 cents` | `6.805895657 / 14.054472399` | pass |
| Frequency maximum | `<=60 cents` | `21.341365011` | pass |
| Damping median / p95 | `<=0.08 / 0.20` | `0.001691263 / 0.004196576` | pass |
| First 1,024-sample waveform NRMSE | `<=0.35` | `0.444680134` | reject |
| Hilbert-envelope NRMSE p95 | `<=0.15` | `0.477920524` | reject |
| Multiresolution spectrum RMSE median | `<=2.5 dB` | `2.324623020 dB` | pass |
| Full waveform NRMSE | diagnostic | `0.647685550` | diagnostic |

All twelve primary objects reject negative damping, unordered frequencies,
missing fallback and dependency-hash mutations after the missing-fallback probe
was corrected to inject a guaranteed absent key.

## Causal counterfactual

The same development corpus was rendered three ways without training or
selection:

| Composition | Early waveform NRMSE | Envelope p95 | Full waveform NRMSE | Spectrum median |
| --- | ---: | ---: | ---: | ---: |
| F0 gains + exact global modes (`field-only`) | `0.180933743` | `0.237417301` | `0.176892066` | `1.388123046 dB` |
| Exact gains + B0 global modes (`global-only`) | `0.413946723` | `0.466979329` | `0.632584459` | `1.729930845 dB` |
| Frozen B0 + frozen F0 (`combined`) | `0.444680134` | `0.477920524` | `0.647685550` | `2.324623020 dB` |

This falsifies the hypothesis that the early-waveform rejection principally
comes from the learned field. The global-only counterfactual nearly reproduces
the combined failure despite excellent modal frequency/damping accuracy.
Field-only passes the waveform threshold but also proves a secondary fact: the
`0.15` envelope threshold is already tighter than the accepted F0 amplitude
field capability.

## Why raw samples disagree with the accepted physics

For two modes `sin(2*pi*f*t)` and `sin(2*pi*(f+delta_f)*t)`, relative phase
grows as `2*pi*delta_f*t`. A fixed cents error therefore creates increasing
sample error even when pitch remains perceptually and physically close. At
`1 kHz`, the observed `14.05 cents` p95 corresponds to about `8.15 Hz`; after
`64 ms` its relative phase is already about `3.28 rad`. Summed modal signals
also change constructive/destructive interference, so the Hilbert envelope of
the mixture is not invariant to the same allowed pitch error.

Raw waveform NRMSE is meaningful when the reference and candidate share exact
frequencies, onset and phase, or after a separately justified alignment. It is
not a coherent blocking metric alongside a nonzero cents tolerance over a long
modal render.

## Bounded external research

Primary work supports a multi-view contract rather than substituting one raw
sample distance for all sound quality:

- [DDSP (Engel et al., ICLR 2020)](https://arxiv.org/abs/2001.04643) combines
  interpretable signal-processing structure with learning and explicitly
  separates controllable pitch and loudness. It supports retaining modal
  parameters as first-class evidence instead of learning a waveform black box.
- [Parallel WaveGAN (Yamamoto et al., ICASSP 2020)](https://arxiv.org/abs/1910.11480)
  defines multiresolution STFT loss from spectral convergence and log-magnitude
  terms at several analysis windows. Its ablation reports better perceptual
  quality than a single STFT. This supports a phase-tolerant time-frequency
  endpoint, but its speech thresholds do not transfer to impact sound.
- [DiffSound (Jin et al., SIGGRAPH 2024)](https://arxiv.org/abs/2409.13486)
  couples modal physics, a differentiable synthesizer and a hybrid audio loss;
  its loss ablation uses optimal transport to escape large frequency
  displacement and a local multiscale term for refinement. This supports
  separating coarse modal-frequency mismatch from local reconstruction.
- [NeuralSound (Jin et al., ACM TOG 2022)](https://arxiv.org/abs/2108.07425)
  reports numerical frequency/transfer accuracy and a separate 106-subject
  perceptual comparison across material/object examples. It does not collapse
  physical correctness and perceived similarity into one waveform scalar.
- [Differentiable Modal Resonators (Diaz et al., 2022)](https://arxiv.org/abs/2210.15306)
  trains an explicitly modal filter bank with an audio-domain objective and
  evaluates log-frequency/log-magnitude structure while separately identifying
  high-frequency decay error. This supports keeping decay and spectral
  evidence distinct.

These sources motivate V20's metric family. They do not prove a Next Engine
threshold, real-material validity or product readiness; those remain fresh,
preregistered experiments.

## Decision

V19 I0 is closed before integration execution. The uncommitted runner is kept
only as a reproducible development-control and fail-closed guard; its official
`run` path and integration-object generator reject unconditionally. The
reserved `1601…1612` metadata remains sealed from values and is retired.

V20 must first calibrate a phase-consistent integration contract on opened
development identities. It will keep exact modal frequency/damping, signed
gain/gradient, coverage, corruption, fallback and determinism gates; replace
raw-sample and mixed Hilbert-envelope blockers with preregistered
multiresolution magnitude/log-magnitude and decay/energy endpoints; and reserve
fresh identities before any new one-shot integration.

No threshold is loosened after seeing I0, no F0/B0 artifact is refit, and no
test or protected role is opened.
