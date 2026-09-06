# V28 R0 protocol — M0c prefix-bounded resource gate

| Field | Frozen value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / VALUE_INDEPENDENT_ONLY / OFFICIAL_VALUES_SEALED` |
| Roadmap | [V28](../plans/physical-sound-synthesis-roadmap-v28.md) R0–R2 |
| Research predecessor | [V28 H0](physical-sound-v28-prefix-bounded-training-research-2026-09-02.md) |
| Scientific question | Can the unchanged compact modal student complete reproducibly when synthetic autograd renders exactly the prefix consumed by its frozen loss? |
| Allowed claim | Execution/resource feasibility and, only after a fresh official A/B, the inherited feasibility decision |
| Forbidden claim | Naturalness, material admission, arbitrary-force transfer, runtime authority or recovery of interrupted M0b values |

## Decision fixed before code

M0b is spent. M0c is a fresh execution-equivalent successor with one allowed
delta:

```text
M0b synthetic render frames = target_waveform.shape[-1] = 144,000 official
M0c synthetic render frames = max(frozen spectral windows) = 4,096
```

The target signal remains aligned and stored at its inherited length. Only the
prediction passed to the synthetic spectral loss is prefix-bounded. Every
metric, evaluator or future cooker must request the length it actually owns;
M0c must not globally shorten `render_prediction`.

No seed, capacity, feature, target, optimizer, batch, step count, variant, loss
weight, spectral window, threshold, split or access rule may change.

## Frozen inheritance

### Normative evidence and preprocessing

- P0a protocol SHA-256:
  `54522c26eb3db62ad316ea6001f9380651f760329f0db8556b441ad286b649e8`.
- M0b owning implementation root:
  `d013ec35f6499cfc17f5beeec3db8d245f2a296b88a104cc80047b2ece04f456`.
- M0b preprocessing behavior, barycentric queries, padded peak alignment,
  official context shape and surface report are unchanged.
- Official combined manifest:
  `c43ba8eac68e32a8ef8fbe37d3ffa3d21db1d59e0443766cee1d98f51cad70bb`.
- T0 evidence:
  `884da56ff9005e9dd63e11ec74bafa8796b187b15ed6099fbca99dda7617e7ed`.
- X0 lineage:
  `e4f6bb117a09f77b2bf8602ec895c95ae63262bc9d42db1233d972d73242534f`.
- `ffmpeg`:
  `bdf6aabffdba7411edff8d36c389d695257fcdf823d196020176e117612862f6`.
- `ffprobe`:
  `a6dac1e9e8631e04075d06854cc4069308aec279bb04683b4a0bca5e4f54b2fe`.

### Inherited source hashes

| File | SHA-256 |
| --- | --- |
| `physical_sound_v25_m0a_common.py` | `592b38137eb088e4ffc8087c3c68c0c19b0a8ceed8d049e7e9b2f6a002573e89` |
| `physical_sound_v25_m0a_evaluate.py` | `e5c48a74f5df89d60433d2ef1605bce648250836809e04721a2aa1fefe3e552a` |
| `physical_sound_v25_m0a_model.py` | `6075b4121c5eb45aca2934308fb4e73c078c467669e458c0d36b65b066f4009b` |
| `physical_sound_v25_m0a_train.py` | `86ada58126ad901021cdf297cef2fc46059dc33763e960dbdc21ebf3972630b5` |
| `physical_sound_v26_m0b_common.py` | `5811796e75c23bde709027af77be97278c6773c2d0a71ecc13d2a2cbb7bd9624` |
| `physical_sound_v26_m0b_surface.py` | `c12799014ac8d7f735668bf465103dce7f90ba9cf4592eb27e5545bdd37b1705` |
| `physical_sound_v26_m0b_alignment.py` | `1452848dc0ba8c678df0041b36cde4ad0ef9daad4a4431ee758a0d15dc8c8314` |
| `physical_sound_v26_m0b_preprocess.py` | `12bf572706b504b5e048a4ea9f85ef58c1a336c49f1baa802cd663fa53bb5ced` |
| `physical_sound_v26_m0b_train.py` | `0c4492b7dace6d9dafe7729b7a1975c40d9b6c9cb2a2d6d28ff23f2fc037d40c` |

The implementation must reject before preprocessing when any inherited hash,
this protocol hash or its own implementation root differs.

## Frozen model and optimization

M0c inherits without modification:

- CPU float32, seed `3101`, one intra-op and one inter-op thread,
  deterministic algorithms, no AMP/TF32/compile;
- `ContactModalField`, `23,142` trainable parameters and limit `40,000`;
- 10 ordered modes, 8 residual atoms and the same bounded transforms;
- AdamW, synthetic LR `1e-3`, real LR `2e-4`, weight decay `1e-5`,
  gradient clip `1.0`;
- official profile: batch `16`, `1,500` synthetic steps and `500` real steps;
- five complete variants in order: candidate A, candidate B, no-geometry,
  no-contact and no-residual;
- synthetic loss weights `1.0 frequency + 0.5 decay + 1.0 gain +
  0.5 spectrum + 0.1 residual`, plus `0.25` remesh consistency;
- real adaptation, synthetic replay, ridge control, hard causal checks,
  development/method-holdout/disclosed-real gates and access order.

M0c declares:

```text
TRAINING_SPECTRUM_WINDOWS = (256, 1024, 4096)
TRAINING_RENDER_FRAMES = 4096
```

Synthetic targets shorter than `4,096` are a conformance error. Targets may be
longer, but no sample at index `>=4096` may influence the current synthetic
loss or gradient.

## Implementation surface

Create exactly these new owners:

- `lab/scripts/physical_sound_v28_m0c_common.py` — protocol, manifest and
  implementation-root boundary;
- `lab/scripts/physical_sound_v28_m0c_model.py` — explicit aliases to the
  frozen M0a model plus the sole prefix-bounded `synthetic_loss` owner;
- `lab/scripts/physical_sound_v28_m0c_resource.py` — value-independent
  equivalence and official-shape training-surface resource oracle;
- `lab/scripts/physical_sound_v28_m0c_train.py` — M0b full-entry owner with M0c
  identity and the M0c model injected into inherited training/evaluation paths;
- `lab/tests/test_physical_sound_v28_m0c_train.py` — focused conformance,
  mutation and full-entry tests.

Do not modify any inherited M0a/M0b source. M0c schemas are new and cannot be
accepted by M0b entry points:

- manifest `nextengine.experimental-physical-sound-v28-m0c.manifest.v1`;
- report `nextengine.experimental-physical-sound-v28-m0c.report.v1`;
- resource report
  `nextengine.experimental-physical-sound-v28-m0c.resource.v1`;
- experiment `m0c-prefix-bounded-neural-student-v1`.

`MODEL_ID` remains the inherited compact representation identity; experiment,
protocol and implementation roots distinguish M0c execution.

## Gate E — exact prefix equivalence

Run before any official manifest is accepted.

### Random cases

Use seed offsets `0..4` from `3101`, batch `3`, frozen model initialization,
random finite object/contact inputs and random legal modal targets. For each
case compare a full `144,000` render with a `4,096` render using identical model
state and targets.

Require `torch.equal` for:

1. `render_prediction(full)[..., :4096]` and `render_prediction(prefix)`;
2. total synthetic loss;
3. each named component;
4. every parameter gradient.

Maximum absolute difference must be exactly `0.0`.

### Bound and mutation cases

- finite all-negative, all-zero and all-positive feature rows;
- legal target frequencies spanning `20..18,000 Hz`, positive decay and gains
  at `-1`, `0`, `1`;
- mutate target samples only at indices `>=4096`: loss/gradients remain exact;
- mutate at least one sample inside the first `4096`: the rendered spectral
  component must change;
- target length `4095`, NaN/Inf prediction or non-positive frame request must
  reject through the public M0c boundary.

Any mismatch or accepted invalid case is `IMPLEMENTATION_CONFORMANCE_REJECT`.

## Gate C — complete owner conformance

The inherited contract fixture runs through the complete M0c entry twice in
fresh external directories. It must:

- exercise preprocessing, five variants, freeze, method holdout,
  disclosed-real query, report and local MLflow paths;
- publish the same nine canonical artifacts as M0b with M0c schema/identity;
- reproduce every canonical output byte recursively across A/B;
- preserve M0b surface/alignment reports and mutation rejections;
- keep official-quality flags false under the fixture and label it
  `contract_fixture_has_no_quality_claim`;
- expose no admission shadow, sealed RealImpact row, network or runtime
  authority.

Any missing inherited callback/API, partial publish, schema cross-acceptance or
byte mismatch is `IMPLEMENTATION_CONFORMANCE_REJECT`.

## Gate R — official-shape resource oracle

The oracle uses fixed random fixture values only. It does not read the combined
manifest, T0, X0, official audio, protected roles or interrupted R1 staging.

Frozen workload per isolated run:

| Dimension | Value |
| --- | ---: |
| Synthetic examples | `32` |
| Stored target frames | `144,000` |
| Transfer contexts | `3 × 4,096` |
| Recording contexts | `2 × 4,096` |
| Variants | `5` in inherited order |
| Synthetic steps | `5 × 1,500 = 7,500` |
| Real/replay steps | `5 × 500 = 2,500` |
| Total optimizer steps | `10,000` |

Candidate A/B equality is checked internally and reduced to a boolean. The
resource report must not expose weights, predictions, loss values, gradients,
audio or their hashes. It may contain only identity, workload counts, wall
seconds, peak RSS, boolean conformance/resource gates and stable reason codes.

Run the oracle twice as fresh processes with:

- `MemoryMax=4G`, `MemorySwapMax=0`, one thread and network denied;
- wall time `<=600 s` per run;
- peak RSS `<3.5 GiB` (`3,758,096,384` bytes);
- process exit `0`, all `10,000` steps complete and internal candidate A/B
  exact;
- normalized reports equal after removing wall/RSS diagnostics.

Wall/RSS are measured outcomes, not deterministic payloads or model-selection
features. Timeout, OOM, incomplete count, non-exact candidate A/B or report
leakage is `RESOURCE_REJECT`; do not open official values.

## Fresh official R2 access order

R2 is authorized only after E/C/R pass, the implementation root is committed
and a new external M0c manifest binds this protocol and root.

1. Create a fresh output/MLflow root; never reuse M0b staging or manifest.
2. Run official A under one thread, network denied, `MemoryMax=4G`, no swap and
   outer wall `900 s`.
3. If A publishes no canonical terminal report, record the terminal resource or
   conformance result and do not start B.
4. If A publishes a canonical terminal result, run B once from the same exact
   input bytes into another fresh root under the same envelope.
5. Compare canonical artifacts recursively. Volatile MLflow run IDs and wall/RSS
   diagnostics are excluded by schema, not deleted after comparison.
6. Publish one exact terminal result: `PASS`, `REPRESENTATION_REJECT`,
   `METHOD_HOLDOUT_REJECT`, `DOMAIN_GAP_REJECT`,
   `IMPLEMENTATION_CONFORMANCE_REJECT`, `RESOURCE_REJECT` or
   `REPEAT_EXACT_REJECT`.

No checkpoint, capacity, seed, horizon, threshold or candidate is selected
after A. A quality reject closes M0c. A resource/conformance reject says nothing
about quality and cannot select a waveform/codec successor.

## Access and artifact policy

- No network during preprocessing, oracle, training or evaluation.
- All inputs/outputs/MLflow roots are absolute external paths.
- Datasets, weights, predictions, PCM and generated reports remain outside Git.
- Admission shadow and sealed RealImpact row `2407` remain inaccessible.
- The interrupted R1 surface report remains evidence but is not an M0c input.
- Local MLflow is diagnostic only; it has no registry, serving or authority.
- Atomic publication is mandatory; a failure leaves no canonical output.

## Verification before R2

1. Python compile for new modules/tests.
2. Ruff check and format check for new modules/tests.
3. Focused M0a + M0b + M0c unit and complete-entry tests.
4. Gate E including every negative mutation.
5. Gate C twice with recursive canonical byte comparison.
6. Gate R twice under the frozen cgroup/network envelope.
7. Exact inherited-hash/protocol/root mutation rejection.
8. Boundary scan; known unrelated findings remain reported, not hidden.

The protocol commit, implementation commit and conformance/resource result
commit are separate. Only the result commit may change Roadmap V28 R1 from
blocked to complete and authorize R2.
