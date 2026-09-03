# Physical sound V41 B0 grouped baseline result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT / GLOBAL_PROTOTYPE_FLOOR / MATERIAL_ONLY_SIGNAL_INSUFFICIENT / M0_REBASELINE_REQUIRED`

Roadmap authority at execution: [Roadmap V41](../plans/physical-sound-synthesis-roadmap-v41.md)

## Outcome

B0 freezes the first honest comparison surface for the disclosed corpus. Six
deterministic methods consume only the C0 `generator_train` and
`generator_development` feature projections. They never read PCM, validator
records, protected data, a candidate model or the network.

The smallest method wins: one global train-parent prototype has the lowest
grouped development median RMSE. Coarse material prototypes, retrieval,
nearest-medoid, ridge and the small pointwise MLP all lose. This does not prove
one cause, but it rules out proceeding directly from coarse material one-hot
conditioning to `StructuredRecipeNet-v1`. The remaining discriminators are an
input-information bottleneck, project/recording-domain shift and a target
representation that still mixes object sound with capture conditions.

## Frozen implementation

- owner: `lab/scripts/physical_sound_v41_b0_grouped_baselines_v1.py`,
  `41,270 bytes`,
  `sha256=47f5dbb8b2648480a1a911db42dcfc3201184b664006ed5af671d960e0a9c539`;
- profile: `lab/profiles/physical-sound-v41-b0-grouped-baselines.v1.json`,
  `5,540 bytes`,
  `sha256=f0dcb617bb6c2e1fd22ff1ce418ed5df959966b979be2cf9f1ffbe67e9e13fdc`;
- tests: `lab/tests/test_physical_sound_v41_b0_grouped_baselines_v1.py`,
  `13,407 bytes`,
  `sha256=71ddeee5b8026f34bc3252817e36430ba44ccb0d03bf125a95b4657d668c5fbc`.

The profile binds SPEC-45, the immutable C0 result/profile/owner, the B0 owner,
the exact C0 train/development projections and NumPy `2.5.2`. It deliberately
does not bind a living roadmap document, so later planning updates cannot
invalidate this exact result.

## Evaluation contract

The target is a 105-dimensional masked vector:

- 24 spectral-band dB values;
- 48 transient-envelope dB values;
- 24 weighted modal-histogram bins;
- 9 global temporal/spectral values;
- decay is masked when the extractor did not observe it.

Rows are averaged inside their physical parent and every parent has equal
weight. Train statistics standardize the target. Development can rank only the
already frozen baselines; it cannot select a candidate or hyperparameters.

The six methods are:

1. global masked train-parent mean;
2. coarse-material masked prototype;
3. coarse-material nearest medoid;
4. deterministic retrieval copy;
5. closed-form multi-output ridge;
6. full-batch float64 one-hidden-layer MLP, width `32`, seed `4100`, `500`
   frozen Adam steps.

Train contains 44 records and 26 parents. Development contains 70 records and
30 parents: 21 have a coarse material represented in train, six have an exact
material label represented in train and nine are OOD. OOD rows are diagnostic
only and return `FallbackOutOfDomain`.

## Grouped development result

| Rank | Baseline | Mean parent RMSE | Median | P90 | Maximum |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1 | `global_prototype` | `1.355038717` | `1.337593650` | `1.655507658` | `1.805980052` |
| 2 | `ridge` | `1.423747418` | `1.443077073` | `1.710622090` | `1.808281658` |
| 3 | `pointwise_mlp` | `1.510257892` | `1.484006315` | `1.732485923` | `1.961708930` |
| 4 | `material_modal_prototype` | `1.512704334` | `1.484536856` | `1.733230305` | `1.965963600` |
| 5 | `retrieval_copy` | `1.574489300` | `1.534928793` | `1.811737430` | `1.990846413` |
| 6 | `nearest_local_medoid` | `1.632929157` | `1.608074705` | `1.835168030` | `1.990846413` |

The frozen reference floor is therefore the global prototype. Its category
mean RMSE is `1.136292121` global, `0.992714142` modal, `1.454961196` spectral
and `1.446386519` transient. The exact-label-supported subset contains only six
Glass/Wood parents and has median `1.410939677`; it is too small to establish
general exact-material learning.

Steel is not present in generator train. Its three development parents are
therefore only a coarse `metallic` transfer from train label `Metal`: mean
`1.276592179`, median `1.379583280`. B0 supplies no Steel-specialization claim.

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v41-b0-2026-09-03`

`run-a` consumes C0 `run-a` and `run-b` consumes C0 `run-b`. Recursive
comparison is empty. Each tree contains seven files and `635,355` bytes. The
SHA-256 of the canonical sorted `path / bytes / sha256` inventory is:

`7e12babaeeb1f54d28d3bfdb05c1c62fd76da648c9fefee961290b1b97a5bdc9`

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access_ledger.json` | 954 | `6f9c819f9586edfc646638acbfa7bbebfcd068405096e981db164b368cfe2978` |
| `metrics.json` | 22,262 | `ae2e5b3d34e8f9ae70286ef35743a91f5f025cf4ffc82a89781cc25067929d28` |
| `models.json` | 116,137 | `7bfc4f01a2fe36cb0f99d0861806acdb0293e1c25f3767c632f01b9bd73071d2` |
| `predictions.json` | 481,444 | `09b793328f18d9f7147954632b8dd1d293b5eaae2af37d6afb9db155a31f8f9d` |
| `profile.json` | 5,540 | `f0dcb617bb6c2e1fd22ff1ce418ed5df959966b979be2cf9f1ffbe67e9e13fdc` |
| `report.json` | 4,779 | `b884475708f3ee0be09fdd01c8f78797eba68939c60a20275434e5903b0da7ac` |
| `representation.json` | 4,239 | `a0276e87e731fba350452518fe59f49bce91b9afa08b02ebd3a9112a6b23d016` |

## Access, verification and authority

Both runs record zero validator, protected, PCM, candidate-model and network
access. All C0 feature hashes are verified; record, parent and component
identity is disjoint; all six models are finite; publication is atomic.

- focused B0 suite: `PASS`, `8/8` tests;
- combined D0/D1/C0/B0 suite: `PASS`, `33/33` tests;
- external CLI A/B and recursive byte comparison: `PASS`;
- repository boundary scan: expected pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- runtime ProductChecks: `NOT_RUN`, because B0 is external research tooling
  under SPEC-45 `Proposed` and changes no production consumer.

B0 returns `B0_GROUPED_BASELINE_SURFACE_FROZEN_M0_AUTHORIZED`, but its evidence
requires the M0 plan to be rebaselined before training. The next work must first
separate project-domain effects from missing object descriptors and prove that
a runtime-available descriptor surface contains signal beyond the global
prototype. Authored clips remain mandatory.
