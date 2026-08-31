# Physical sound R3A V8 — object-91 fit rejection and V9 residual research

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Scope | Frozen fit-only evaluation of ObjectFolder Real object `91` and bounded successor research |
| Result | `REJECT_V8_REAL_FIT_REPRESENTATION` |
| Next revision | `V9_EXPLICIT_MODES_PLUS_TIME_VARYING_LEARNED_NOISE_BANDS` |
| Product authority | None; authored clips remain mandatory |

## Executive result

The frozen V8 runner was executed twice on fit contacts `18/12/4`. Every
artifact is byte-identical between runs. Both capacities satisfy the shared
and per-contact byte budgets, but both fail the spectrum and modal-frequency
endpoints on all three contacts. No capacity is selected and contact `20`
(development) is not decoded. Contact `27` (sealed), method holdout and
admission shadow also remain untouched.

This rejects the V8 **real residual representation**, not explicit modes or ML
as a whole. A fit-only counterfactual shows that a dense phase-preserving
in-band residual can recover the missing spectrum, but costs `214,564` bytes
before modal/contact overhead. The frozen `64 KiB` record can store only a
sparse global residual, which destroys low-energy bins and deep notches that
the log-spectrum and modal endpoints intentionally expose.

The next falsifiable representation therefore keeps the explicit modal path
and replaces the global sparse FFT tail with a compact **time-varying
filterbank/noise-band residual predicted by an offline neural decoder**. It is
a new, source-disjoint V9 revision; object `91` may be used only as negative
diagnostic evidence and never for V9 selection or threshold tuning.

## Exact run identity

Both runs used commit `d72ba9fefb18022ac273e20ac506122d5f0a3b88`, the
pinned Python `3.11.15` environment, NumPy `1.26.4`, SciPy `1.11.4`, and one
thread for OpenBLAS, OMP and MKL.

| Artifact | SHA-256 in run A and run B |
| --- | --- |
| `manifest.json` | `6077250c0c099cbae2fd31ce54a8a19a7d12093366e998872f34ec8895381f6c` |
| `model.json` | `cf9c09cd912b5950ddc4f8c76939194e594a3979f7c83b6c2c8ae4f2d1658954` |
| `report.json` | `9c49abc82f0dc3fa9b4f32ecd496f7e42971882dbb8adef368f4f0f98366d438` |

External run roots:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/r3a-v8-object91-fit-run-a`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/r3a-v8-object91-fit-run-b`.

The source prefix remains
`5ef9789ad023233377aa60d58f66100f0bb62dd5df52556e051e757d13797313`.
The runner decoded `1,728,000` source sample values and used `864,000`
aligned fit-analysis values. Development and sealed decoded counters are both
exactly zero.

## Frozen gates

| Endpoint | Maximum |
| --- | ---: |
| absolute RMS level error | `0.5 dB` |
| gain-matched multiresolution log-spectrum RMSE | `4.0 dB` |
| normalized envelope RMSE | `0.20` |
| median modal-frequency error | `100 cents` |
| T60 relative error | `0.35` |
| shared decoder | `4,194,304 bytes` |
| contact record | `65,536 bytes` |

Capacity selection was frozen as global damping if all contacts pass, else
per-contact damping if all contacts pass, else rejection. No third capacity or
post-fit repair was allowed.

## Exact fit result

The compact columns below are level dB / spectrum dB / envelope / modal cents /
decay-relative-error. A value at or below the frozen gate passes.

| Capacity | Contact | Metrics | Record bytes |
| --- | --- | --- | ---: |
| global damping | `18` | `5.6025 / 16.3915 / 0.08995 / 2962.06 / 2.0561` | `63,392` |
| global damping | `12` | `2.1919 / 16.5188 / 0.04023 / 1226.72 / 0.0957` | `63,392` |
| global damping | `4` | `0.7046 / 15.9338 / 0.01969 / 6043.99 / 0.3709` | `63,392` |
| per-contact damping | `18` | `5.6033 / 16.2984 / 0.09001 / 2258.08 / 1.9748` | `63,520` |
| per-contact damping | `12` | `2.1850 / 16.6276 / 0.04016 / 1080.78 / 0.0390` | `63,520` |
| per-contact damping | `4` | `0.6991 / 15.7147 / 0.01958 / 6044.04 / 0.2538` | `63,520` |

Both capacities use `58,512` shared bytes. Envelope passes everywhere, while
spectrum and modal frequency fail everywhere. Per-contact damping changes
decay on contacts `12` and `4`, but does not materially change the common
failure. Damping capacity is therefore not the leading explanation.

## Bounded fit-only diagnostic

This diagnostic is exploratory and receives no candidate or quality credit.
It reads only already-opened fit contacts and keeps the evaluator unchanged.
The purpose is to discriminate the representation failure before another
implementation cycle.

Three counterfactuals were compared with the per-contact-damping modal base:

1. `sparse_raw`: the frozen `14,500` shared complex bins before envelope,
   transient and output gain;
2. `dense_inband`: all `53,641` complex residual bins in `120–18,000 Hz` with
   original phase;
3. `dense_logmag_noise`: all in-band log magnitudes encoded as one byte and
   rendered with deterministic random phase, then the frozen envelope,
   transient and RMS correction.

| Contact | Modal-only spectrum/modal/decay | Sparse raw | Dense in-band | Dense logmag noise |
| --- | --- | --- | --- | --- |
| `18` | `13.303 / 1772.6 / 0.512` | `17.481 / 5439.5 / 16.06` | `0.604 / 121.7 / 0.002` | `14.038 / 6619.9 / 0.986` |
| `12` | `13.109 / 212.7 / 0.309` | `18.379 / 1080.3 / 5.38` | `0.303 / 94.4 / 0.942` | `13.735 / 7201.7 / 0.989` |
| `4` | `14.127 / 3889.6 / 0.486` | `17.155 / 6044.0 / 4.41` | `0.708 / 0.0 / 0.981` | `13.234 / 7155.9 / 0.996` |

`dense_inband` proves that the source, alignment and phase-preserving analysis
can recover spectrum and most modal identity. It is not admissible: complex
int16 storage alone costs `53,641 × 4 = 214,564` bytes, and level/decay still
need work. `dense_logmag_noise` costs `53,641` raw bytes but fails, showing that
a single stationary magnitude curve plus random phase is not the missing
representation. The needed degree of freedom is spectro-temporal, not another
global FFT capacity.

## Competing hypotheses

| Hypothesis | Evidence for | Evidence against | Decision |
| --- | --- | --- | --- |
| H1: too few damping parameters | Some decay errors improve with per-contact damping | Spectrum/modal failures remain almost unchanged on every contact | Reject as leading cause |
| H2: evaluator or source alignment is broken | Large V8 errors could be explained by a bad boundary | Dense phase-preserving residual reaches `0.30–0.71 dB` spectrum RMSE | Reject |
| H3: sparse global FFT selection removes perceptually important structure | Sparse raw is worse than modal-only; dense complex succeeds | Dense complex exceeds budget and is not a solution | Accept causal diagnosis |
| H4: stationary magnitude-shaped noise is sufficient | It fits the raw residual byte budget | It fails spectrum/modal/decay on all contacts | Reject |
| H5: explicit modes plus time-varying filterbank residual is worth a new test | It separates narrow resonances from nonstationary broadband energy | Not yet tested under NextEngine gates or a `64 KiB` contact record | Select smallest new experiment |

## Primary-source research

- Engel et al., [DDSP](https://openreview.net/pdf?id=B1x1ma4tDr), formalize a
  differentiable linear time-varying FIR noise synthesizer. The official
  [Magenta implementation](https://github.com/magenta/ddsp) was inspected at
  `cf5e62dfe5d5c80aa14761832233a2e68e840e53`.
- Barahona-Ríos and Collins,
  [NoiseBandNet](https://arxiv.org/abs/2307.08007), replace one time-varying
  FIR response with deterministic loopable noise bands and time-varying band
  amplitudes. They report better reconstruction than four DDSP filtered-noise
  variants in nine of ten sound-category/metric combinations, including
  knocking and metal effects. The official
  [implementation](https://github.com/adrianbarahona/noisebandnet) was
  inspected at `75d430b37cc0f4e7d3465e1eb317e00028a7e27f`; the downloaded paper SHA-256
  is `e7cedc9f5eaada9b347df6ef3c3eabcab7124c9e76de280a247155f9f129c634`.

NoiseBandNet is prior art, not a drop-in product choice. Its paper configuration
uses `2,048` bands, one amplitude every `32` samples, about `1.6–1.8 MB` of
weights and about `1 GB` of precomputed noise bands. V9 must exploit the hybrid
fact that explicit modes already carry narrow resonances: it may use a much
smaller residual bank, regenerate deterministic noise bases offline, and bake
ordinary clips. No upstream code, weights or audio are redistributed.

## Decision and scope guard

V8 real fit is closed. Do not:

- read development contact `20` or sealed contact `27`;
- change V8 bins, thresholds, capacities, damping, loss or postfilter;
- use object `91` to select V9 architecture or thresholds;
- claim real quality, R3B authorization, validator release or runtime neural
  inference.

V9 starts with a no-real-audio synthetic control for deterministic noise-band
rendering, compact time-varying amplitude decoding, exact repeat and neural
held-contact interpolation. A pass authorizes only a newly frozen,
source-disjoint real representation protocol.

