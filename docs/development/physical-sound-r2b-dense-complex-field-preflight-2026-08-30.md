# Physical sound R2B dense complex-field preflight

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Scope | One Green Goblet fixed-impact 600-position listener field; external research only |
| Decision | `DENSE_COMPLEX_FIELD_DATA_READY / R2C_COMPLEX_TRAINING_AUTHORIZED` |
| Public/runtime authority | None; SPEC-45 remains `Proposed`, no model has trained, and authored clips remain the mandatory fallback |

## Question and answer

R2B asked whether the published REALIMPACT field provides enough hash-closed
spatial coverage for a phase-preserving experiment without leaking query audio
into preprocessing or opening method holdout/admission shadow.

Yes for the next isolated optimizer experiment. Two independent acquisitions
and two full representation/control preflights are byte-identical. The frozen
complex STFT round-trips within the numeric gates, and all three classical
controls are measured before optimization. The exact decision is
`ReadyForComplexFieldTraining`; it is not a trained-model, quality, admission,
runtime or product decision.

## Hash-closed internet acquisition

Profile `green-goblet-dense-listener-block-v3` validates the official
REALIMPACT archive identity and ZIP metadata before bounded range acquisition.
It extracts only the first fixed Green Goblet impact block and validates all
published arrays, row hashes, coordinates, vertex and impact identity.

| Artifact | SHA-256 or exact value |
| --- | --- |
| Acquisition manifest A/B | `487423a2973348968d87072e015bb895f4f3703ae4e1e45bcaf9a9d3f5e6b7b3` |
| Acquisition report A/B | `5a4a41f5486a90fa7761100a19b80a02b44ddf03a18cd79bfb7b261fbd9e413e` |
| Compressed 512 MiB prefix | `f8a48f8c71d2979a054a9cf116afa4e76195aee79688d480fe81f84ba1a02794` |
| Raw 600-row f32 block | `381e958c53bf73b2aa62bf8dae874cf0167945a916c62514a9d2ab81e3c5188e` |
| Raw block size | `499,975,200` bytes |
| Official archive / downloaded ranges | `2,311,697,935` / `537,434,728` bytes (`23.248%`) |
| Rows / gantry columns | `600` / `40` |
| Context / query rows | `420` / `180` |
| Optimizer and sealed-role reads | `0` steps / `0` bytes |

The source grid has ten azimuth planes from `0°` through `180°`, four
published distance offsets and 15 microphone heights per gantry column. Query
uses every column at `40°`, `100°` and `160°`; context uses every remaining
azimuth plane. Splitting complete columns and complete angle planes prevents
the old interleaved-neighbour leakage pattern.

The acquisition command performs one sequential request at a time and fails
closed on response/header/source drift. Raw audio, cached ranges and the
499 MB block remain outside Git. No user recording or local object capture is
part of this boundary.

## Frozen representation and isolation

Representation `complex-stft-sqrt-periodic-hann-2048-hop512-v1` uses mono
float32 transfer responses, complex64 features, a 2,048-sample square-root
periodic Hann window, 512-sample hop, 408 frames and 1,025 non-negative
frequency bins. Context-only global peak normalization maps the context peak
to `0.92` PCM16 full scale and applies that scale unchanged to query.

Isolation counters are explicit:

- query rows used for normalization: `0`;
- query rows cached for candidate fit: `0`;
- method-holdout or admission-shadow bytes read: `0`;
- query rows read for final preflight/control evaluation: `180`.

The forward/inverse gates pass over all 600 rows:

| Gate | Frozen limit | Observed worst case |
| --- | ---: | ---: |
| Float round-trip NRMSE | `<= -140 dB` | `-153.3479625163743 dB` |
| Maximum absolute float error | `<= 1e-6` | `9.030764136497282e-9` |
| Maximum PCM16 difference | `<= 1 LSB` | `1 LSB` |
| PCM16 mismatch samples | report-only | `16` over all 600 rows |

The 1-LSB gate is deliberate. An earlier draft required byte-identical PCM,
but a float-control round trip showed that 16 boundary samples can differ by
one quantization unit while the float error remains below `1e-8`. Another
draft normalized over all 600 rows and was rejected before optimization
because query peaks would influence fit preprocessing. V3 fixes both issues;
the superseded drafts carry no readiness credit.

## Frozen classical controls

Every control predicts the same 180 grouped query rows and is evaluated by the
existing Rust physical-sound evaluator. Lower is better.

| Control | Mean level | P95 level | Mean spectrum | P95 spectrum | Mean waveform NRMSE |
| --- | ---: | ---: | ---: | ---: | ---: |
| Nearest context azimuth, same distance/microphone | `1.8869 dB` | `5.8350 dB` | `9.1139 dB` | `11.9913 dB` | `2.8994 dB` |
| Linear bracketing azimuth, same distance/microphone | `3.8099 dB` | `7.3829 dB` | `8.9453 dB` | `13.0820 dB` | `1.9557 dB` |
| Log-magnitude plus shortest-arc phase complex STFT interpolation | `1.8584 dB` | `5.2755 dB` | `9.2948 dB` | `11.8003 dB` | `2.5267 dB` |

No control dominates all five endpoints. A learned candidate may pass R2 only
by being strictly lower than every frozen control on every endpoint; pooled or
four-of-five improvement remains a rejection.

## Reproducibility

The Python preflight source hashes to
`95aaec57abd779d77e51d0c0d0515f173aaa72e7d5683f68bc5a0fd809003afc`.
Runs A and B use the two independent acquisition manifests, recompute all 600
source/feature hashes and round trips, render 180 references plus 540 control
WAVs, and invoke the Rust evaluator for every control. Their complete 288 MB
output trees compare without differences. Both final reports hash to
`e3db03ac75efc2d7153b8c3f02172a47eb5ad384cb85ff489810f2f239d31696`.

## R2C protocol frozen by the preflight

The next experiment compares exactly two candidates with a shared
coordinate/time/frequency MLP that emits real and imaginary pressure:

1. `dense_complex_field_data_only_v1`, complex-STFT L1 plus log-magnitude L1,
   Helmholtz weight `0`;
2. `dense_complex_field_helmholtz_v1`, the same model/data loss and deterministic
   midpoint collocation, Helmholtz weight `0.0001`.

Both use listener and published impact coordinates, frame time and frequency.
The physics term is frozen as
`laplacian(p) + (2*pi*f/343)^2*p = 0` over `93.75–12,000 Hz`; query audio is
forbidden in loss, normalization, checkpoint selection and early stopping.
Model shape, initialization, seed, steps and environment must be frozen in the
training manifest before optimizer step one. This is one physics/no-physics
ablation, not a hyperparameter grid.

## Decision and next action

Allowed now:

`DENSE_COMPLEX_FIELD_DATA_AND_REPRESENTATION_READY_ONLY`

Not allowed:

- learned-quality, material-identity or exact-object impact-axis credit;
- method-holdout/admission-shadow access;
- validator, cooker admission, public schema or runtime promotion;
- changing the grouped query planes, preprocessing, controls or five-endpoint
  conjunctive rule after training begins.

The next commit boundary is R2C: freeze one hash-closed training manifest,
train the two preregistered candidates with the same architecture, seed and
budget, repeat each training, freeze checkpoints without query feedback, then
evaluate each once on the 180 query rows. A failure closes this candidate
family or triggers a new evidence-backed representation revision; it does not
authorize a nearby tuning grid.
