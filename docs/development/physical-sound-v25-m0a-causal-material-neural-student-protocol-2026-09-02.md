# Physical sound V25 M0a — causal-material exact-object neural student protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-02` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION_COMPLETION / MODEL_VALUES_UNOPENED / ONE_CPU_SEED / BLUE_BOWL_DISCLOSED_DEVELOPMENT_ONLY` |
| Roadmap package | V25 `M0-I` and `M0-E` |
| Supersedes | [V24 M0](physical-sound-v24-m0-exact-object-neural-student-protocol-2026-09-01.md) before official execution because its isolated `E`/density gates were not observable from one-hot inputs |
| Input authority | [X0 result](physical-sound-v24-x0-blue-bowl-pilot-result-2026-09-01.md), combined V3 SHA-256 `c43ba8eac68e32a8ef8fbe37d3ffa3d21db1d59e0443766cee1d98f51cad70bb` |
| Product effect | None; external feasibility evidence only, never validator, material admission or runtime authority |

## Question and bounded claim

Can one fixed compact contact-conditioned neural field learn T0's causal modal
representation and improve prediction of the already disclosed Blue Bowl query
over frozen classical controls, without consuming an absent X0 axis or asking a
human to select a sound?

M0a is an exact-object feasibility test, not Metal admission, real holdout,
cross-object generalization or a cooker candidate. It may establish that a
neural representation is worth carrying into M1/V0. It cannot establish that
Glass sounds correct in general because every X0 value is disclosed development
evidence and its support, excitation, composition, material constants and
impact normal remain unknown.

## Frozen inputs and access order

The implementation receives the exact external T0 root, X0 root and combined
V3 manifest. A trusted evidence owner verifies their hashes and reconstructs
T0's complete artifact list to compare the frozen aggregate teacher artifact
root. Protected bytes and hashes used by that aggregate check are not returned
to candidate code, reports or MLflow.

| Evidence | Use in M0a | Forbidden use |
| --- | --- | --- |
| T0 `train` context | parameter training | role-specific threshold selection |
| T0 `train` query | fixed training diagnostic only | optimizer/checkpoint selection |
| T0 `development` | candidate/control comparison and fixed final-step diagnostic | architecture, seed or capacity retry |
| T0 `calibration` | uncertainty temperature and fixed OOD calibration only | model-gradient updates |
| T0 `method_holdout` | one-shot after model/weights hash is frozen | checkpoint or threshold changes |
| T0 `admission_shadow` | sealed; aggregate root verification only | row/value/hash materialization outside the trusted owner |
| X0 transfer contacts `0..2` | phase-tolerant real adaptation context | raw-force, support, composition or material-constant supervision |
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
2. Compute a 24-value geometry descriptor: log vertex/triangle counts,
   normalized bounding-box lengths, bounding diagonal, normalized centroid,
   normalized covariance eigenvalues, log surface-area/volume proxies, closed
   volume mask, mean normalized radius and eight radial quantiles. Open
   surfaces receive volume `0` and mask `0`, never fabricated volume.
3. Compute a 24-value contact descriptor from centered/scaled XYZ, radial
   coordinate, distances to eight deterministic farthest-point landmarks and
   twelve fixed sinusoidal XYZ features. The first landmark is farthest from
   the centroid; every tie uses lowest vertex index.
4. Append fixed semantic material features: one-hot
   `[elastic-a, elastic-b, elastic-c, glass]` plus `material_label_known`.
5. Append physical-material values
   `[log(E/1e9 Pa), log(rho/1000 kg m^-3), poisson, log(damping_base),
   1000*damping_slope]` plus one all-or-none
   `physical_parameters_known` mask. T0 values are the exact frozen teacher
   table below. X0 values are all zero with mask `0`; the Glass semantic label
   remains known and does not authorize numeric inference.
6. Append support one-hot `[cantilever-clamped-root,
   simply-supported-all-edges]` plus `support_known`. X0 support uses zeros and
   mask `0`.
7. T0 targets are log frequency, log decay and signed normalized gain. The
   canonical renderer stays float64/48 kHz and uses no per-contact peak scale.
8. REALIMPACT uses the frozen absolute-peak-to-sample-512 alignment and
   144,000-sample window. One closed-form log-gain nuisance is fit per observed
   row; no frequency warp, denoise, crop search or response equalizer exists.
9. ObjectFolder MP4 audio is inspected by manifest-hashed `ffprobe`, decoded by
   manifest-hashed `ffmpeg` into float32 stereo 44.1 kHz, mixed by exact
   arithmetic mean and resampled once to 48 kHz with polyphase ratio `160/147`
   and Kaiser beta `5.0`. Only temporal/spectral distribution losses are legal.

The frozen T0 material table is:

| ID | density kg/m3 | Young's modulus Pa | Poisson | damping base | damping slope |
| --- | ---: | ---: | ---: | ---: | ---: |
| `elastic-a` | `2700` | `69000000000` | `0.33` | `8.0` | `0.0015` |
| `elastic-b` | `7850` | `200000000000` | `0.29` | `12.0` | `0.0010` |
| `elastic-c` | `8900` | `110000000000` | `0.34` | `18.0` | `0.0008` |

The implementation verifies this table against the pinned T0 implementation
hash before using it. Preprocessing emits source hash, transform ID, shape,
dtype, units and output hash. NaN, unexpected channel/rate/container, mesh
degeneracy, unavailable decoder or hash drift returns `FallbackOutOfDomain`
before training.

## Frozen model

The only candidate is `m0a-contact-modal-field-v1`, implemented in the pinned
Python 3.12/PyTorch environment and trained on CPU only:

- object input: 24 geometry + 5 semantic material + 6 physical material + 3
  support values, exactly 38 floats;
- contact input: exactly 24 floats;
- object trunk: `Linear(38,64) -> SiLU -> Linear(64,64) -> SiLU`;
- global head: ten strictly sorted frequencies via positive cumulative
  log-frequency gaps bounded to `20..18000 Hz`, plus ten positive decay rates
  via `softplus`;
- contact trunk: concatenated object/contact features through
  `Linear(88,64) -> SiLU -> Linear(64,64) -> SiLU -> Linear(64,64) -> SiLU`;
- contact head: ten signed modal gains through `tanh`;
- residual head: eight signed gains over eight fixed causal damped-cosine atoms
  log-spaced over `180..12000 Hz` and `18..240 s^-1`; T0 residual target is
  exactly zero;
- total trainable parameters must be `<= 40,000` and is recorded before
  optimization.

There is no object-ID embedding, lookup-table answer, waveform decoder,
stochastic noise, attention, graph network, pretrained generator, runtime
export or hidden material-default path.

## Training schedule and losses

Only seed `3101` is legal. Python, NumPy and PyTorch seeds, deterministic
algorithms, float32 parameters, CPU execution and one intra/inter-op thread are
mandatory. CUDA, TF32, AMP, compilation, data-loader workers and
nondeterministic kernels are forbidden.

1. Synthetic phase: `1,500` AdamW steps, batch `16`, learning rate `1e-3`,
   weight decay `1e-5`, gradient clip `1.0`; frozen seed-derived cyclic
   permutation over T0 train-context rows.
2. Real adaptation: `500` AdamW steps, learning rate `2e-4`; every step uses
   all three REALIMPACT contexts, ObjectFolder `000/020` and a fixed eight-row
   T0 replay batch. The object trunk stays trainable.
3. Final candidate: exactly step `2000`; no checkpoint search, early stop,
   scheduler, seed retry or resumed optimizer state.

| Loss | Weight | Legal evidence |
| --- | ---: | --- |
| log-frequency Huber | `1.0` | T0 teacher only |
| log-decay Huber | `0.5` | T0 teacher only |
| signed-gain Huber | `1.0` | T0 teacher only |
| contact-query rendered log spectrum | `0.5` | T0 teacher and REALIMPACT context |
| multi-resolution log spectrum (`256/1024/4096`) | `0.5` | real context only |
| decay-envelope slope | `0.25` | real context/ObjectFolder only |
| modal peak set | `0.25` | real transfer context only |
| T0 residual-zero penalty | `0.1` | T0 teacher only |
| remesh common-vertex consistency | `0.25` | T0 coarse/refined twins only |

ObjectFolder contributes no pointwise waveform, contact, listener or modal
target. Candidate query values never enter a loss.

## Frozen controls and ablations

1. `nearest-context-v1` — nearest contact-descriptor context response;
2. `fixed-feature-ridge-v1` — one closed-form ridge (`lambda=1e-3`) from the
   exact 38+24 inputs to modal/gain representation;
3. `neural-full-v1` — the frozen candidate;
4. `neural-no-geometry-v1` — zero the 24 geometry values only;
5. `neural-no-contact-v1` — zero the 24 contact values;
6. `neural-no-residual-v1` — residual head fixed to zero.

Optimized ablations use the same seed and step count. They are reports, never a
search surface.

## Gates

Two complete executions emit byte-identical canonical weights, predictions and
value-independent report fields. MLflow metadata is excluded from equality.

Hard and causal gates require:

- every source/cache/target hash resolves and protected roles stay private;
- finite strictly sorted `20..18000 Hz` frequencies, positive decay and render
  peak below `0.95` without per-query normalization;
- parameter count `<=40,000` and no forbidden device/operator/role access;
- exact force scaling at `0.5/1.0/2.0` after the declared global scale;
- for every T0 object, model counterfactual median frequency ratios for `4x E`,
  `4x density`, `2x thickness` and `2x length/planar scale` are respectively
  `2.0`, `0.5`, `2.0`, `0.25` within `10%` relative error. Physical mutations
  change only the physical vector; geometric mutations transform the declared
  T0 local coordinates and recompute the 24 geometry values;
- refined/coarse common-contact modal gains differ by at most `1e-5` absolute;
- contact gradients are finite and non-constant on every T0 object.

On T0 development query rows, full neural must achieve frequency log-RMSE,
decay log-RMSE, signed-gain normalized RMSE and rendered multi-resolution
log-spectrum distance each `<=0.90x` ridge. No-geometry and no-contact are each
at least `5%` worse on their corresponding target. No hard/remesh regression
is allowed.

After the candidate tensor hash freezes, one-shot T0 method holdout keeps every
neural/ridge ratio `<=0.95x`. Any miss closes M0a; admission shadow stays
unopened.

On REALIMPACT contact `3`, full neural composite distance is `<=0.95x` both
nearest and ridge, improves at least two of spectrum/envelope/peak-set and
passes hard/physical gates. ObjectFolder `039` does not increase the
context-to-query material-distribution distance over nearest. These are
disclosed development gates only.

Synthetic failure returns `REPRESENTATION_REJECT` without method holdout.
Synthetic pass plus real failure returns `DOMAIN_GAP_REJECT`; capacity does not
increase. A zero control denominator requires exact candidate zero, never an
epsilon ratio.

## Canonical artifacts, MLflow and resources

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

Weights and predictions use custom little-endian sorted-tensor containers with
fixed magic/version/name/dtype/shape/raw bytes. `torch.save`, pickle and
timestamped ZIP/NPZ are forbidden as authority.

MLflow `3.15.2` uses a local external file store only. Autologging, registry,
aliases, serving and remote tracking are disabled. Volatile run IDs, timestamps
and MLflow database bytes are diagnostic and excluded from equality/selection.

Each complete CPU run is limited to `1,800 s`, `4 GiB` peak RSS and `256 MiB`
canonical output. Two A/B executions are mandatory before a result claim.

Contract tests corrupt source/model/protocol hashes, material table and masks,
T0 binding, lane/role access, tensor order/dtype/shape/finiteness, contact
query, sorted-frequency transform, nuisance-gain policy, denominator, final
step, seed/device/thread settings, MLflow URI, sealed-row counter and output
occupancy. They also prove each causal mutation changes the intended input and
no other coordinate. Interrupted execution leaves no candidate freeze or
partial output.

One failed official execution spends M0a. Opened values cannot select a nearby
architecture, loss, threshold, seed, capacity, residual basis, physical
default or training length. Authored clips remain authority and runtime/public
promotion stays blocked regardless of result.
