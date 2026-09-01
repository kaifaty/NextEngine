# Physical sound V24 M0 — exact-object neural student protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / MODEL_VALUES_UNOPENED / ONE_CPU_SEED / BLUE_BOWL_DISCLOSED_DEVELOPMENT_ONLY` |
| Roadmap package | V24 `M0` |
| Input authority | [X0 result](physical-sound-v24-x0-blue-bowl-pilot-result-2026-09-01.md), combined V3 SHA-256 `c43ba8eac68e32a8ef8fbe37d3ffa3d21db1d59e0443766cee1d98f51cad70bb` |
| Product effect | None; external feasibility evidence only, never validator, material admission or runtime authority |

## Question and bounded claim

Can one fixed compact contact-conditioned neural field learn T0's causal modal
representation and improve prediction of the already disclosed Blue Bowl query
over frozen classical controls, without consuming an absent X0 axis or asking a
human to select a sound?

M0 is an exact-object feasibility test, not Metal admission, real holdout,
cross-object generalization or a cooker candidate. It may establish that a
neural representation is worth carrying into M1/V0. It cannot establish that
Glass sounds correct in general because every X0 value is disclosed
development evidence and its support, excitation, composition and impact
normal remain unknown.

## Frozen inputs and access order

The implementation receives the exact external T0 root, X0 root and combined
V3 manifest. It verifies the T0/X0/result/manifest hashes before parsing and
reconstructs T0's complete artifact list to verify the frozen teacher
artifact-root SHA-256.

| Evidence | Use in M0 | Forbidden use |
| --- | --- | --- |
| T0 `train` context | parameter training | role-specific threshold selection |
| T0 `train` query | fixed training diagnostic only | optimizer/checkpoint selection |
| T0 `development` | candidate/control comparison and fixed final-step diagnostic | architecture, seed or capacity retry |
| T0 `calibration` | uncertainty temperature and fixed OOD calibration only | model-gradient updates |
| T0 `method_holdout` | one-shot after model/weights hash is frozen | checkpoint or threshold changes |
| T0 `admission_shadow` | sealed; commitment only | any materialization in M0 |
| X0 transfer contacts `0..2` | phase-tolerant real adaptation context | raw-force, support or composition supervision |
| X0 transfer contact `3` | disclosed final development query | gradient updates or retry selection |
| ObjectFolder `000/020` | material/envelope adaptation context | contact/listener supervision |
| ObjectFolder `039` | disclosed distributional development query | paired contact loss or gradient updates |
| REALIMPACT row `2407` | sealed commitment only | hash, samples, metrics or materialized row |

The full execution order is contract fixture, deterministic preprocessing,
control fit, neural A, neural B, byte comparison, synthetic development,
candidate hash freeze, one-shot synthetic method holdout, disclosed X0 query
and final report. A failure stops before later roles. Admission shadow is never
opened.

## Deterministic preprocessing

Every transform is hash-cached outside Git and recomputed A/B:

1. Parse `NEMESH01`, `NEMODT01` and `NEGAIN01` exactly and verify mesh binding,
   sorted ten-mode targets and finite values.
2. Compute a 24-value object descriptor: vertex/triangle counts, bounding-box
   lengths, centroid, covariance eigenvalues, surface-area/volume proxies and
   eight radial quantiles. Open surfaces set a declared volume-mask bit rather
   than receiving a fabricated volume.
3. Compute a 24-value contact descriptor from centered/scaled XYZ, radial
   coordinate, distances to eight farthest-point landmarks and twelve fixed
   sinusoidal XYZ features. Farthest-point ties use lowest vertex index.
4. Material and support are fixed one-hot-plus-known-mask features. X0 unknown
   support/composition use zero values with mask `0`; no imputation is allowed.
5. T0 targets are log frequency, log decay and signed normalized gain. The
   canonical renderer stays float64/48 kHz and uses no per-contact peak scale.
6. REALIMPACT preprocessing uses its already frozen absolute-peak-to-sample-512
   alignment and 144,000-sample window. A single closed-form log-gain nuisance
   is fit per observed row; no frequency warp, denoise, crop search or response
   equalizer is allowed.
7. ObjectFolder MP4 audio is decoded by one manifest-hashed local `ffmpeg`
   executable into float32 stereo 44.1 kHz, mixed by exact arithmetic mean and
   resampled once to 48 kHz by a frozen polyphase kernel. Container bytes remain
   the source authority. Only temporal/spectral distribution losses are legal.

Preprocessing emits a canonical cache manifest with source hash, transform ID,
shape, dtype, units and output hash. NaN, unexpected channel/rate/container,
mesh degeneracy, unavailable decoder or hash drift returns
`FallbackOutOfDomain` before training.

## Frozen model

The only neural candidate is `m0-contact-modal-field-v1`, implemented in the
repository-pinned Python 3.12/PyTorch environment and trained on CPU only:

- object descriptor: 24 values plus material/support known-mask features;
- contact descriptor: 24 values;
- object trunk: `Linear -> 64 -> SiLU -> 64 -> SiLU`;
- global head: ten strictly sorted frequencies via positive cumulative
  log-frequency gaps and ten positive decay rates via `softplus`;
- contact trunk: concatenated object/contact features through
  `Linear -> 64 -> SiLU -> 64 -> SiLU -> 64 -> SiLU`;
- contact head: ten signed modal gains through `tanh`;
- bounded residual head: eight signed gains over eight fixed causal
  damped-cosine atoms. T0 residual target is exactly zero; atom frequencies and
  decays are frozen log-spaced constants, never learned or randomized;
- total trainable parameters must be `<= 40,000` and the report records the
  exact counted value before optimization.

Sorted/positive transforms are part of the model, not post-hoc repair. There
is no object-ID embedding, lookup-table answer, waveform decoder, stochastic
noise, attention, graph network, pretrained audio encoder or runtime export in
M0.

## Training schedule

Only seed `3101` is legal. Set Python/NumPy/PyTorch seeds, deterministic
algorithms, float32 parameters, CPU execution and one intra/inter-op thread.
CUDA, TF32, AMP, compilation, data-loader workers and nondeterministic kernels
are forbidden.

1. **Synthetic phase:** `1,500` AdamW steps, batch `16`, learning rate
   `1e-3`, weight decay `1e-5`, gradient norm clip `1.0`. The batch permutation
   is a frozen seed-derived cycle over T0 train-context rows.
2. **Real adaptation:** `500` AdamW steps, learning rate `2e-4`. Each step uses
   all three REALIMPACT context contacts plus both ObjectFolder context
   recordings and a fixed eight-row T0 replay batch. The object trunk is
   trainable; T0 replay prevents unconstrained domain drift.
3. **Final candidate:** exactly step `2000`; no best-checkpoint search, early
   stop, scheduler, seed retry or resumed optimizer state.

Loss weights are fixed:

| Loss | Weight | Legal evidence |
| --- | ---: | --- |
| log-frequency Huber | `1.0` | T0 teacher only |
| log-decay Huber | `0.5` | T0 teacher only |
| signed gain Huber | `1.0` | T0 teacher only |
| contact-query rendered log spectrum | `0.5` | T0 teacher and REALIMPACT context |
| multi-resolution log spectrum (`256/1024/4096`) | `0.5` | real context only |
| decay-envelope slope | `0.25` | real context/ObjectFolder only |
| modal peak set | `0.25` | real context only |
| T0 residual-zero penalty | `0.1` | T0 teacher only |
| remesh common-vertex consistency | `0.25` | T0 coarse/refined twins only |

ObjectFolder contributes no pointwise waveform, contact, listener or modal
target. Unknown real excitation scale is eliminated only by the declared
closed-form nuisance gain. Candidate query values never enter a loss.

## Frozen controls and ablations

All controls share identical inputs, preprocessing and allowed roles:

1. `nearest-context-v1`: nearest intrinsic/XYZ context response, the existing
   X0 query baseline;
2. `fixed-feature-ridge-v1`: one closed-form ridge (`lambda=1e-3`) from the
   exact object/contact descriptors to the same modal/gain representation;
3. `neural-full-v1`: the frozen candidate above;
4. ablation `neural-no-geometry-v1`: zero object/geometry descriptor;
5. ablation `neural-no-contact-v1`: zero contact descriptor;
6. ablation `neural-no-residual-v1`: residual head fixed to zero.

Controls/ablations use the same seed and step count where optimization exists.
They are reports, not a search surface: no winner may choose a new seed,
capacity, loss, contact subset, threshold or preprocessing revision.

## Gates

The two complete candidate executions must emit byte-identical canonical
weights, predictions and value-independent report fields. MLflow metadata is
not part of byte equality.

### Hard and causal gates

- every source/cache/target hash resolves and all protected roles stay private;
- frequencies are finite, strictly sorted and within `20..18000 Hz`; decay is
  finite/positive; render peak is below `0.95` without per-query normalization;
- parameter count `<= 40,000`; no forbidden device/operator/role access;
- force scaling at `0.5/1.0/2.0` is exact after the declared global scale;
- T0 isolated `E`, density, thickness and length/planar-scale counterfactuals
  have the analytic direction and exponent within `10%` relative error;
- refined/coarse common-contact modal gains differ by at most `1e-5` absolute;
- contact gradients are finite and non-constant on every T0 object.

### Synthetic quality gates

On T0 development query rows, `neural-full-v1` must achieve all of:

- frequency log-RMSE `<= 0.90x` fixed-feature ridge;
- decay log-RMSE `<= 0.90x` fixed-feature ridge;
- signed-gain normalized RMSE `<= 0.90x` fixed-feature ridge;
- rendered multi-resolution log-spectrum distance `<= 0.90x` ridge;
- no-geometry and no-contact ablations each worse than full by at least `5%`
  on their corresponding target;
- no hard, causal or remesh regression.

After the candidate tensor hash is frozen, the one-shot T0 method holdout must
keep every neural/ridge ratio `<= 0.95x`. Any miss closes M0; method-holdout
values cannot select a retry. Admission shadow remains unopened.

### Disclosed real development gates

On REALIMPACT contact `3`, full neural composite distance must be `<= 0.95x`
both nearest-context and fixed-feature ridge, improve at least two of spectrum,
envelope and peak-set metrics, and pass every hard/physical gate. ObjectFolder
recording `039` must not increase the context-to-query material-distribution
distance over nearest-context. These are disclosed development gates only and
never real holdout/admission credit.

If synthetic gates pass but real gates fail, report `DOMAIN_GAP_REJECT`; do not
increase capacity. If synthetic gates fail, report `REPRESENTATION_REJECT` and
do not open method holdout. If a control denominator is zero, require candidate
zero and report the ratio as exact equality rather than adding epsilon.

## Canonical artifacts and MLflow boundary

The owning CLI atomically emits outside Git:

```text
preprocess-manifest.json
control-report.json
candidate-weights.bin
candidate-predictions.bin
candidate-freeze.json
method-holdout-report.json       # only after freeze and prior gates
disclosed-real-report.json       # only after prior gates
report.json
```

`candidate-weights.bin` and `candidate-predictions.bin` use custom
little-endian sorted-tensor containers with fixed magic/version, tensor name,
dtype, shape and raw bytes. The weights are the candidate identity;
`torch.save`, pickle and timestamped ZIP/NPZ are forbidden as authority.

MLflow `3.15.2` uses a local external file-store only. Autologging, model
registry, aliases, serving and remote tracking are disabled. Each run logs the
frozen manifest/protocol/code hashes, exact parameters, step metrics and hashes
of canonical artifacts. Run IDs, wall timestamps and MLflow database bytes are
diagnostic and excluded from deterministic equality or selection.

## Resources, mutations and stop rules

Each complete CPU run is limited to `1,800 s`, `4 GiB` peak RSS and `256 MiB`
canonical output. Two A/B executions are mandatory before any result claim.

Contract tests corrupt source/model/protocol hashes, T0 target binding, lane or
role access, tensor order/dtype/shape/finiteness, feature masks, contact query,
sorted-frequency transform, nuisance-gain policy, control denominator,
checkpoint step, seed/device/thread settings, MLflow tracking URI, sealed-row
counter and output occupancy. Interrupted training/publishing leaves no
candidate freeze or partial output.

One failed official execution spends this M0 model family. Opened development
or method-holdout values cannot select a nearby architecture, loss weight,
threshold, seed, capacity, residual basis or training length. A successor
requires a new falsifiable hypothesis and protocol. Regardless of result,
authored clips remain authority and runtime/public promotion stays blocked.
