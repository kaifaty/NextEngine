# Physical sound PS-2N0 neural data plane

| Field | Value |
|---|---|
| Date | 2026-08-30 |
| Result | `PS_2N0_V2_IMPLEMENTED / SYNTHETIC_FIXTURE_PASS / REAL_PROJECTION_BYTE_IDENTICAL / MODEL_TRAINING_DISABLED / SHADOW_CONTENTS_NOT_MATERIALIZED` |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Plan | [Neural acoustic field implementation plan](../plans/2026-08-30-physical-sound-neural-acoustic-field-implementation-plan.md), N0.1 |
| Runtime effect | None; external research tooling only |

## Question and answer

The N0.1 question was whether the existing claim-scoped physical-sound
registry could feed a neural experiment without collapsing missing metadata,
mixing related objects across partitions or exposing validator shadow rows to
model-development commands.

The new `xtask physical-sound-registry neural-data-plane` command implements
that boundary. It accepts only an external hash-closed manifest, verifies every
referenced artifact and emits only to an empty external directory. It does not
train a model, authorize quality, open a holdout or change the game/audio
runtime.

## Frozen input contract

Schema:

`nextengine.experimental-physical-sound-neural-data-plane.manifest.v2`

The only supported task scope in V2 is
`exact_object_few_shot_impact_listener_field`. Every row declares:

- `context` or `query` sample role and `target` or `reject_parent` corpus role;
- source, family, object, recording-parent, condition and optional mutation
  groups;
- one or more typed lineage reports;
- audio and audio-provenance artifacts;
- explicit `recorded_impact_waveform` or
  `force_deconvolved_transfer_response` audio semantics;
- independently optional material, geometry, support, impact, listener and
  excitation claims.

Every published claim carries its own hash-closed evidence. Geometry also
carries a feature artifact. Impact/listener points are finite metres in an
explicit coordinate profile; impact normals must be unit length within the
frozen tolerance. Impact point and outward normal are independent, so an absent
normal cannot erase a published point. Excitation requires published impulse,
energy or a force profile. Missing axes remain absent and reduce the capability
count; an object or material label never fills them.

A lineage descriptor declares the expected report `schema` and `claim`. The
command parses the referenced JSON and requires the declared pair plus
`status = Validated`; an arbitrary hash-closed file cannot impersonate a typed
registry report.

## Partition and privacy boundary

The manifest must contain target context and query rows for all five roles:

1. `train`;
2. `development`;
3. `calibration`;
4. `method_holdout`;
5. `admission_shadow`.

The command rejects any cross-role reuse of object group, source group,
recording parent, condition group, mutation parent or identical audio SHA-256.
Family overlap is reported rather than rejected because the first few-shot
task intentionally evaluates new objects from a related family. Rows and
lineage descriptors must be canonically sorted, so repeated projection has a
stable byte representation.

Development receives only:

- `fit-projection.json` with train/development rows;
- `calibration-projection.json` with calibration rows;
- `sealed-role-commitments.json` with counts and canonical commitment hashes,
  but no method-holdout/admission-shadow row IDs or artifact hashes;
- `report.json` with capability/leakage aggregates and explicit
  `model_training_authorized = false`.

The two sealed roles are never materialized by this command. Opening either
role requires a later, separately governed command after candidate or
validator freeze.

## Synthetic verification

The focused tests synthesize ten tiny rows and their referenced artifacts in a
temporary directory outside the repository: one context and one query for
each role. All six conditioning axes are present in the positive fixture.

Verified controls:

- two independent projections produce byte-identical four-file outputs;
- sealed row IDs and audio hashes are absent from every development-facing
  projection/commitment file;
- cross-role object, source, recording, condition, mutation-parent and
  identical-audio leakage each fail closed;
- invalid normals, empty excitation claims, disabled sealing policy and false
  lineage semantics fail closed;
- deleting a geometry claim produces
  `DeclaredAxisCoverageIncomplete`, counts exactly nine geometry-complete rows
  and emits no fabricated geometry field.

Focused result: `4 passed; 0 failed`; the complete physical-sound registry
module result is `145 passed; 0 failed`. Formatting and all-target xtask Clippy
with warnings denied also pass. The broad `boundary-scan` remains red on the
pre-existing tracked `#[path = "transfer_calibration/dsp.rs"]` in
`realimpact_transfer_fixture.rs`; none of the new neural modules uses an escape
hatch and their physical line counts are `943`, `244` and `363`, below the
1,000-line limit.

## Real projection result

The [R0–R1 real evidence](physical-sound-neural-real-boundary-r0-r1-2026-08-30.md)
supersedes the earlier `REAL_PROJECTION_NOT_RUN` limitation. A 23-row,
five-role projection combines eight recorded impact waveforms and 15
force-deconvolved Green Goblet transfer responses. It reports impact/listener
coordinates for those 15 rows while leaving geometry, support and excitation
absent. Two runs emit byte-identical fit, calibration, report and sealed-role
commitment files. No method-holdout or admission-shadow row content is
materialized.

## Allowed claim and remaining work

Allowed now:

`PS_2N0_V2_DATA_CONTRACT_IMPLEMENTED / REAL_PROJECTION_REPEAT_PASS`

Not allowed:

- model-training, learned-quality or admission claims;
- method-holdout or admission-shadow access;
- public content/runtime schema or product readiness.

The smallest next action is N0.3: train one preregistered fixed-impact
listener-field pilot against the frozen transfer controls. If published data
cannot support the later impact axis, keep N0.4 blocked rather than inventing
that axis.
