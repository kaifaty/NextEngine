# Physical sound V24 D0 — neural evidence-plane protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / CONTRACT_FIXTURES_ONLY / TEACHER_REAL_AND_MODEL_VALUES_UNOPENED` |
| Package | Roadmap V24 `D0`, compatible V3 extension of the existing `physical-sound-registry neural-data-plane` authority |
| Product effect | None; external research tooling only, model training and protected-role materialization remain disabled |
| Mandatory fallback | Existing authored clip remains required for every absent, invalid, rejected or OOD record |

## Question

Can the existing hash-closed neural data plane represent the three evidence
lanes required by Roadmap V24 without conflating their claims, fabricating
missing axes, exposing protected roles or creating a second source authority?

D0 answers only the record/projection question. It does not generate a teacher
corpus, decode a new real sample, train a model, calibrate a validator, bake a
clip or authorize runtime use.

## Immutable predecessor

| Dependency | SHA-256 |
| --- | --- |
| Current `neural_data_plane.rs` | `80436a344df3b52ba1c2a11c71abf48ce7986cde2720c717dc2d4b5eade57912` |
| Current `neural_data_plane/row_projection.rs` | `66e603b2d2513f0a531a75c2a590119ba12b44a5babc2ee2ef0df8ebe530d166` |
| Current `neural_data_plane/tests.rs` | `538c1529aa50f5101ab466183a73205584e6a58b56c65e6ef86ceef190ba4f2b` |
| PS-2N0 V2 evidence | `d9f574bf0a09fcc2accf5a45b0144530f518d1d15d7a7e28f8328c12d433bcf3` |
| V23 F2a result | `3e4ba2decac664ba80a80f88afdbb78e88756f69581f769067f59a93462f306c` |
| Roadmap V24 | `1a3e9fb568f10965f53a7da4390901f2240efaf2c82f284193c03bad1da47d58` |

The V2 schema, command name, output names, sealing behavior and existing tests
remain supported. D0 extends this owner in place; it does not add a second
registry command or Python contract.

## V3 schemas

```text
nextengine.experimental-physical-sound-neural-data-plane.manifest.v3
nextengine.experimental-physical-sound-neural-data-plane.projection.v3
nextengine.experimental-physical-sound-neural-data-plane.report.v2
```

`sealed-role-commitments.json` retains
`nextengine.experimental-physical-sound-neural-data-plane.sealed-roles.v1`.

Every V3 row adds required `evidence_lane`. The exact vocabulary is:

```text
synthetic_teacher
exact_real_transfer
identified_real_recording
```

`audio_semantics` adds exactly `synthetic_modal_render` to the existing
`force_deconvolved_transfer_response` and `recorded_impact_waveform` values.

## Lane bindings

| Evidence lane | Required audio semantics | Required claims | Forbidden claim |
| --- | --- | --- | --- |
| `synthetic_teacher` | `synthetic_modal_render` | material, geometry, support, impact, listener, excitation and `teacher_target` | none |
| `exact_real_transfer` | `force_deconvolved_transfer_response` | typed lineage plus audio/provenance; every available physical axis remains explicit | `teacher_target` |
| `identified_real_recording` | `recorded_impact_waveform` | typed lineage plus audio/provenance; material or other axes exist only with their own evidence | `teacher_target` |

V3 requires at least one row from every lane. A semantic mismatch, absent
synthetic axis, missing teacher target or real row carrying a teacher target
rejects before output creation. Real lanes may remain structurally incomplete;
their absent axes are counted and omitted, never inferred.

## Synthetic teacher target

`axes.teacher_target` is optional in the general record and required only for
`synthetic_teacher`. It has exactly:

```text
representation_id
mode_count
modal_parameters
contact_gain_field
evidence
```

- `representation_id` is frozen to `sorted-modal-contact-field-v1`;
- `mode_count` is an integer in `1..=256`;
- the other three fields are ordinary hash-checked `FileRef` values;
- D0 verifies bytes and hashes but does not interpret numeric target values;
- T0 will separately freeze the binary schema, units, ordering, physical
  controls and actual teacher corpus before generating them.

This keeps the causal teacher target distinct from the rendered waveform used
for deterministic format/cooker checks.

## Roles and leakage

Retain exact V2 roles and split rules:

```text
train  development  calibration  method_holdout  admission_shadow
```

Every role still requires target context and query rows. Object, source,
recording parent, condition, mutation parent and identical audio remain
disjoint across roles. Method-holdout and admission-shadow contents remain
absent from development-facing output; only their canonical commitments and
counts are emitted.

Evidence lane is not a partition key: the same lane may appear in several
roles, but no physical/source parent may cross roles. Family overlap remains a
reported few-shot property, not hidden leakage.

## Projection and report

The owning command still emits exactly:

```text
fit-projection.json
calibration-projection.json
sealed-role-commitments.json
report.json
```

V3 projected rows preserve `evidence_lane`, `audio_semantics` and the optional
verified teacher target. V3 report adds one optional `lane_capability_counts`
object with exact keys:

```text
synthetic_teacher
exact_real_transfer
identified_real_recording
teacher_target
lane_contract_complete
```

The field is absent for V2 so existing V2 output shape remains compatible.
`lane_contract_complete` counts rows satisfying their lane binding; it does not
grant model-training or quality authority. For every version:

```text
model_training_authorized = false
method_holdout_materialized = false
admission_shadow_materialized = false
```

## Complete owning-entry fixture

Before the implementation commit, focused tests must call the module's
`run_cli` entry point twice on one finite, temporary V3 fixture. The fixture:

- contains all five roles, context/query target rows and all three lanes;
- uses tiny deterministic bytes unrelated to future T0/X0/M0 values;
- traverses manifest parse, lineage validation, artifact resolution/hash,
  every lane/axis projection, leakage audit, sealed commitments and all four
  output writers;
- produces four byte-identical files in both runs;
- proves protected row IDs and hashes do not occur in fit/calibration output or
  commitments;
- checks the report counts and disabled authorities.

The same suite must execute actual failure paths for:

1. each of the three lane/semantics mismatches;
2. missing synthetic teacher target;
3. missing synthetic physical axis;
4. teacher target on either real lane;
5. invalid teacher representation, zero/oversized mode count and stale target
   artifact hash;
6. all six existing cross-role leakage classes;
7. an unknown field/schema and a non-empty output directory.

Existing V2 focused tests must remain green. Static symbol or callback lists do
not substitute for the twice-executed `run_cli` fixture.

## Implementation boundary

To keep the owning Rust module below the repository line limit, move its record
and lane types into
`tools/xtask/src/physical_sound_registry_command/neural_data_plane/evidence_record.rs`.
This is an internal layout refactor only; command syntax and V2 semantics do not
change.

No `unsafe`, new dependency, network access, source download, model library,
runtime crate or public contract is allowed. Outputs remain external and atomic
empty-directory-only. Maximum rows, reports and bytes remain the V2 bounds.

## Checks and stop rules

Required before D0 result:

1. `cargo fmt --check`;
2. focused `neural_data_plane` tests, including the twice-exact V3 `run_cli`;
3. `cargo clippy -p xtask --all-targets -- -D warnings`;
4. `cargo run -p xtask -- boundary-scan`, with the known unrelated
   `realimpact_transfer_fixture.rs` escape hatch reported separately if still
   present;
5. two clean focused executions with no repository artifact.

Any failure before implementation commit keeps T0/X0/M0 values sealed. After
implementation commit, a D0 contract failure may be fixed only on new compact
contract fixtures; it cannot open or tune from teacher, real holdout or model
values. D0 pass authorizes only T0 and X0 protocols, not their data generation.
