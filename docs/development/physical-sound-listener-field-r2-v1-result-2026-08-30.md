# Physical sound R2 listener-field V1 result

| Field | Value |
|---|---|
| Date | 2026-08-30 |
| Result | `REJECT_LISTENER_FIELD_V1 / TWO_DETERMINISTIC_TRAINING_REPEATS / TWO_BYTE_IDENTICAL_EVALUATIONS / SHADOW_SEALED` |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Roadmap | [Neural physical sound](../plans/physical-sound-synthesis-roadmap.md), R2 |
| Runtime effect | None; external training/evaluation only |

## Question and answer

R2 V1 tested whether a small neural coordinate field can reconstruct seven
held-out Green Goblet listener responses from eight fixed-impact context
responses better than both frozen R1 controls.

The answer is no for the preregistered time-domain latent representation.
Training and evaluation are reproducible, but neither candidate is strictly
better on all five primary endpoints. No threshold changed and no candidate is
selected.

## Frozen protocol

The trainer receives only the eight even microphone rows and the seven odd
listener coordinates. It receives no query audio path. The query references
are introduced later by a separate evaluation manifest.

Frozen training profile:

- Python `3.12.13`, NumPy `2.5.2`, PyTorch `2.13.0+cu130`, MLflow `3.15.2`;
- MLflow is pinned by the isolated `lab/pyproject.toml` and `lab/uv.lock`, not
  added to any engine/runtime dependency surface;
- deterministic single-thread CPU `float64`, seed `2608300101`;
- two-layer width-32 `tanh` coordinate network, fixed `8,000` Adam steps;
- canonical-sign centered context Gram eigendecomposition;
- rank-4 modal latent and rank-7 full-centered modal-plus-residual candidates;
- non-finite or `abs(sample) >= 1` output rejects before PCM16 cooking;
- smallest passing rank would win, but only after beating both R1 controls on
  every frozen primary endpoint.

V1 runner `f09fea28…946` froze the numeric protocol but stopped after its first
candidate when MLflow `3.15.2` rejected the legacy file tracking backend. It
published no training or evaluation report and read zero query/shadow audio.
The partial external output is preserved as infrastructure-negative lineage.

V2 runner `b2d14f50…294c0` changes only MLflow storage to an external SQLite
database plus external artifact directory. Numeric candidates, seed, data and
selection rules are unchanged.

| Artifact | SHA-256 |
|---|---|
| V2 manifest | `31d9b97ed70fd316fa957ca048fd7919b98ca0da3513ab93c6b71b8bdbc3979b` |
| V2 preflight report | `0470e10386f1d6ef87fe61d57ace85e51e80c31269abcb39d2094c1bb535906b` |
| Deterministic training report | `1f82d2154d44505e96dc17a988062b5b2002effd4a3b36797772041109da9f7f` |
| MLflow lineage A | `f77bfb2201ffcd469a3137c513ff28be4c3da36d36b483554c76c6e87c8ae090` |
| Evaluation report | `379cfa4d7da56581eda6d3921eaf518f6726b9f407d8e978b1ddb898618d514e` |

Training runs A/B produce byte-identical deterministic reports, raw float64
checkpoint descriptors/weights and all 14 cooked prediction WAVs. Their
MLflow run IDs intentionally differ. Run A records:

- rank-4: run `ffc4239ea9b8485da67282df52489f01`, checkpoint
  `c90ff0e9…c20a`, train NRMSE `-18.4151 dB`;
- rank-7: run `bf430ba1d2344120bd6e3e111262d6c0`, checkpoint
  `cebf23e5…3915`, train NRMSE `-46.7294 dB`.

Both candidates are cookable; maximum pre-cook peaks are `0.5895` and
`0.5685`. Training reports zero query-audio and zero method-holdout/shadow
bytes read.

## Frozen evaluation

The evaluator binds training and MLflow lineage, the R1 baseline report, the
V2 projection, all seven development query references and the exact Rust
metric sources. It invokes the existing `physical-sound-eval` implementation
for level and multi-resolution spectrum metrics; sample-synchronous PCM NRMSE
uses the already frozen R1 equation. Two complete evaluation trees are
byte-identical.

| Candidate/control | Mean level | P95 level | Mean spectrum | P95 spectrum | Mean waveform NRMSE |
|---|---:|---:|---:|---:|---:|
| nearest control | `2.8510` | `8.5090` | `9.9771` | `12.6548` | `3.1073` |
| linear control | `2.5373` | `4.7094` | `9.1745` | `10.9250` | `2.4129` |
| neural rank-4 | `3.0836` | `6.6975` | `9.2962` | `11.1041` | `2.2844` |
| neural rank-7 | `2.5476` | `3.8883` | `9.1940` | `11.0573` | `2.6130` |

Rank-4 beats both controls only on mean waveform NRMSE. Rank-7 beats both on
P95 level, but misses linear control by `0.0103 dB` on mean level, `0.0195 dB`
on mean spectrum and `0.1324 dB` on P95 spectrum; it also regresses mean
waveform NRMSE. Near misses remain failures under the preregistered conjunctive
rule.

Decision:

`RejectListenerField`

## Interpretation and next hypothesis

The data show a representation trade-off, not an optimizer failure: rank-7
fits the eight context rows almost exactly while still failing held-out
listeners. Increasing rank, width, epochs or changing thresholds would be
post-hoc tuning and is rejected for this lineage.

The leading causal hypothesis is listener-dependent propagation phase. The
published impact/listener coordinates imply different path lengths, while V1
asks a smooth time-domain latent field to learn both geometric delay and object
radiation from only eight positions. This is an inference from the failure
pattern, not yet a supported physical claim.

One bounded successor may preregister exactly one representation change:
remove a fixed coordinate-derived time-of-flight phase using an explicit speed
of sound, fit the unchanged listener latent field in that aligned domain, then
restore query delay before the unchanged PCM cooker and R1 evaluation. It must
retain the same rows, endpoints, controls, seed policy and sealed roles. If
that successor also fails, pause implementation and run the repository's
persistent-problem research escalation instead of trying another nearby MLP.

## Allowed claim

Allowed:

`R2_V1_REPRESENTATION_REJECTED / TRAINING_AND_EVALUATION_REPRODUCIBLE`

Not allowed:

- neural quality, exact-object, material-wide or shared transfer credit;
- method-holdout or admission-shadow access;
- automatic validator, admission, public schema or runtime promotion.
