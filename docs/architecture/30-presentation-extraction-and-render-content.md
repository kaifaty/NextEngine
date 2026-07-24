# SPEC-30: Presentation extraction and render content

| Поле | Значение |
|---|---|
| ID | SPEC-30 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Rendering Team |
| Требуемые согласующие | Repository Owner, Architecture Working Group, Runtime Team, Player Experience Team, Asset & Persistence Team, Physical Embodiment Team, Verification & Evidence Team, Release Engineering |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-28](28-skeletal-animation-retargeting-and-ik.md), [SPEC-29](29-platform-host-and-application-session.md), [ADR-019](adr/019-canonical-player-actions-and-presentation-authority.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md), [ADR-024](adr/024-requirement-gate-evidence-and-profile-closure.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md), [ADR-028](adr/028-platform-session-and-presentation-authority.md) |
| Заменяет | отсутствует |

## История принятия

SPEC-30 подготовлен как часть consolidated architecture packet 1.8. Он
фиксирует immutable presentation extraction, neutral material/shader/color
contracts, VFX cue semantics и reconstructible cache boundary. Принятие exact
root означает только architecture admission; оно не создаёт renderer,
implementation gate `PASS`, qualitative approval, `vertical-v1` или release
readiness.

## Назначение и invariants

- Rendering Team stages and atomically publishes one immutable
  `PresentationSnapshotV2` from Runtime-owned projections at the
  Runtime-declared extraction boundary. Renderer, UI, VFX and capture consume
  it read-only.
- Presentation state, visibility, camera, interpolation, device capability,
  shader compilation, GPU timing and cache state MUST NOT become gameplay
  authority or enter gameplay/replay hashes.
- Every object and cue has a stable engine-owned key. Backend handles and array
  positions are never identity.
- Material, shader-interface, pipeline-key, color and VFX schemas are neutral
  serialized contracts. Public data contains no ECS component/storage, native
  pointer, OS/window/surface object, GPU/device/command handle, compiler object,
  importer record or vendor/backend type.
- GPU/UI/VFX caches are reconstructible from locked content, the exact accepted
  presentation snapshot/profile and bounded CPU
  `PresentationConsumptionStateV1`. Cache/device loss never resets cue
  acknowledgment/pending/active state or writes back to simulation.
- Exact pixel comparison is required only on a pinned capture profile.
  Gameplay/domain output remains exact across composition roots; non-pinned
  physical/render numerics use explicitly declared tolerances.
- Observable visual/UI/camera/animation/physics/motor/VFX changes MUST use the
  SPEC-15 capture/evidence flow and signed `HumanReviewDecisionV2`.

## Source of truth и ownership

| State | Единственный owner/source of truth | Reconstructible/non-authority |
|---|---|---|
| Domain/RPG/world/physics result | owning simulation bounded context | presentation copy |
| Extracted immutable scene/camera/cue projection | Rendering Team stages and atomically publishes `PresentationSnapshotV2` at the Runtime-declared extraction boundary | renderer upload buffers |
| Semantic UI/camera intent | Player Experience Team | widget tree, camera matrices after interpolation |
| Neutral material/shader-interface/color definitions | Asset & Persistence Team cooked catalog | compiled pipeline and descriptor caches |
| Presentation rendering and VFX realization | Rendering Team | current frame, particle instances, temporal history |
| Cue consumption/acknowledgment state | Rendering Team engine-owned bounded CPU `PresentationConsumptionStateV1` | GPU/UI/VFX instances and cache generations |
| Skeletal/IK presentation result | Physical Embodiment Team contract, presentation branch | skinning/pose buffers |
| Capture comparison and review evidence | Verification & Evidence Team | local viewer/cache |
| Gameplay hash | Runtime/domain owners | frame/audio/presentation hashes are separate evidence roots |

## `PresentationSnapshotV2`

```text
PresentationSnapshotV2 {
  schema_version,
  snapshot_epoch,
  snapshot_sequence,
  simulation_tick,
  project_composition_lock_hash,
  content_manifest_hash,
  presentation_profile_hash,
  scene_batches[],
  camera_batches[],
  semantic_ui_batches[],
  cue_batches[],
  environment_batch,
  canonical_hash
}
```

The snapshot body omits `canonical_hash`; the enclosing envelope stores
`SHA-256("nextengine.presentation-snapshot.v2\0" || JCS(body))`.
`snapshot_epoch` changes on project activation, authoritative recovery or
explicit extraction reset. `snapshot_sequence` starts at zero and increments
exactly once for each published snapshot; wrap is forbidden.

The whole snapshot is staged and validated before atomic publication. Missing
required reference, duplicate/colliding key, stale content/profile revision,
invalid bounds or failed extraction rejects the complete candidate and retains
the prior snapshot. The renderer never reads a partially replaced batch.

### Object keys and canonical order

```text
PresentationObjectKeyV1 {
  snapshot_epoch,
  persistent_id,
  presentation_role,
  incarnation
}
```

`persistent_id` is the durable subject/object `PersistentId`;
`presentation_role` is a closed engine enum; `incarnation: u32` distinguishes
an explicit destroy/recreate inside one epoch. `RuntimeEntityId`, ECS row,
pointer, draw index and backend handle are forbidden.

Scene objects sort by:

```text
(presentation_layer, PresentationObjectKeyV1, asset_id, instance_ordinal)
```

Camera records sort by `(viewport_id, camera_role, camera_id)`. Semantic UI
records sort by `(surface_id, semantic_path_id, element_id)`. All cue families
sort by the common typed-envelope total key defined below. Exact duplicate
bytes collapse only where the schema declares idempotency; same key with
different bytes rejects the snapshot.

### Canonical batch rule

```text
CanonicalPresentationBatchV1 {
  batch_kind,
  batch_index,
  first_global_ordinal,
  record_count,
  records[],
  records_root
}
```

Worker fragments and worker-local batch boundaries are never observable.
Before publication, Rendering Team flattens all fragments independently for
scene, camera, semantic-UI and cue record families, validates the complete
family, applies its canonical sort and only then partitions the result using
the positive finite `max_records_per_batch` for that family from the locked
`PresentationBatchProfileV1`. Partition `i` is exactly the half-open ordinal
range `[i * max_records_per_batch, min((i + 1) * max_records_per_batch,
record_count))`; empty batches are forbidden and an empty family has zero
batches. Every batch records `batch_kind`, `batch_index`,
`first_global_ordinal`, `record_count` and `records_root`. Worker identity,
completion order, input fragment size, thread count and cache state are
forbidden inputs. Consequently both batch boundaries and batch roots are exact
for all worker/extraction permutations. `records_root` is
`SHA-256("nextengine.presentation-batch.v1\0" || JCS({batch_kind,
batch_index, first_global_ordinal, record_count, ordered_record_hashes}))`;
the body of each record omits its enclosing hash.

### Scene, camera and cue batches

`ScenePresentationRecordV2` contains object key, quantized current/previous
transform samples, mesh/material/skeleton/animation references, visibility
class, presentation bounds, physical-presentation projection reference and
closed feature flags. It never owns physical pose/contact or RPG state.

`CameraPresentationRecordV2` contains camera ID/role, viewport, projection
profile, quantized intent/result samples, exposure profile reference and
cut/interpolation policy. Camera output cannot feed targeting; authoritative
targeting remains SPEC-18 query/command data.

`SemanticUiPresentationRecordV1` references `UiSemanticSnapshot` values,
localized text IDs and style/accessibility roles. Rendered strings, widget
objects and glyph caches are not identifiers.

`PresentationCueBatchV1` contains only bounded
`PresentationCueEnvelopeV1` records:

```text
PresentationCueEnvelopeV1 {
  schema_version,
  cue_stream_id,
  cue_ordinal,
  cue_kind,
  cue_kind_rank,
  activation_simulation_tick,
  source_event_id,
  source_event_slot,
  source_cue_slot,
  kind_local_key,
  payload_schema_id,
  payload_schema_version,
  payload_hash,
  payload,
  canonical_hash
}
```

The closed rank is `Audio = 0`, `Vfx = 1`, `Decal = 2`, `Camera = 3`,
`SemanticUi = 4`; serialized `cue_kind_rank` MUST equal the rank of
`cue_kind`. `kind_local_key: Hash256` is the engine-owned audio cue ID,
`VfxCueV1.cue_id`, decal cue ID, camera cue ID or semantic-UI cue ID for that
kind. Unknown kinds/ranks, a rank mismatch or a backend-derived key reject the
complete snapshot.

Runtime projection enumerates committed `DomainEvent` records in its canonical
publication order. Each registered event projection assigns
`source_cue_slot: u32` from zero in schema-defined canonical cue expansion
after deterministic reducer merge; packages/adapters cannot choose it and a
duplicate slot rejects. Within one event Runtime sorts candidate envelopes by
`(activation_simulation_tick, cue_kind_rank, source_cue_slot)` and assigns
contiguous checked `cue_ordinal: u64` values in one `cue_stream_id`. Filtering
that order to VFX assigns contiguous `VfxCueV1.cue_sequence`, after which all
kind-local IDs/hashes are derived; there is no ID/order circularity.
Re-extraction/replay assigns the same slots, ordinals and sequences. The exact
total canonical envelope key is:

```text
(
  cue_stream_id canonical bytes,
  cue_ordinal,
  cue_kind_rank,
  kind_local_key canonical bytes
)
```

`payload_hash` is
`SHA-256("nextengine.presentation-cue-payload.v1\0" ||
JCS({payload_schema_id, payload_schema_version, payload}))`.
`canonical_hash` is
`SHA-256("nextengine.presentation-cue-envelope.v1\0" || JCS(body))`, where
`body` omits only `canonical_hash`. The same stream/ordinal or full envelope
key with different identity, payload or bytes is a collision; exact duplicate
envelopes collapse. Thus audio, VFX, decal, camera and semantic-UI order and
batch boundaries are independent of worker arrival or family grouping.

Filtering the common ordered envelopes to `cue_kind = Vfx` MUST preserve
strictly increasing contiguous `VfxCueV1.cue_sequence`; envelope
`cue_stream_id`, activation tick, source event/slot/cue-slot and
`kind_local_key` MUST equal the VFX payload fields. Each VFX portion is then
one contiguous `CueStreamSliceV1`:

```text
CueStreamSliceV1 {
  cue_stream_id,
  first_cue_sequence,
  prior_prefix_root,
  cues[],
  terminal_prefix_root
}
```

For each canonical cue, the next prefix is
`SHA-256("nextengine.vfx-cue-prefix.v1\0" || prior_prefix_root ||
u64_le(cue_sequence) || acknowledgment_key || canonical_hash)`. Sequences are
contiguous inside a slice; a gap, overlap with different bytes or invalid
prefix rejects the complete snapshot. Before sequence zero,
`prior_prefix_root` MUST equal
`SHA-256("nextengine.vfx-cue-prefix-empty.v1\0" || cue_stream_id)`. The slice
is a bounded projection of committed event facts and recovery continuity,
never a second authoritative event log.

## Neutral material and shader-interface contracts

### `MaterialDefinitionV1`

```text
MaterialDefinitionV1 {
  schema_version,
  material_definition_id,
  shading_model_id,
  parameter_schema[],
  texture_slot_schema[],
  render_state_profile_id,
  shader_interface_manifest_hash,
  permitted_feature_set[],
  fallback_material_id,
  canonical_hash
}
```

Each parameter has immutable numeric field ID, semantic, closed value kind,
unit/color-space tag, finite range, default and override rule. Texture slots
declare neutral dimensionality, sample semantic, color interpretation,
required/optional status and exact fallback asset. Unknown/duplicate field IDs,
NaN/infinity, incompatible unit/color, missing required slot or recursive
fallback rejects the definition.

### `MaterialInstanceV1`

```text
MaterialInstanceV1 {
  schema_version,
  material_instance_id,
  definition_id,
  definition_hash,
  sorted_parameter_values[],
  sorted_texture_bindings[],
  variant_key,
  canonical_hash
}
```

Instance values are canonical fixed-width scalars/vectors or declared color
values and must match the exact definition. No instance may add bindings,
change shader interface/render state, override an immutable field or include a
compiled pipeline/device object. Equal canonical definitions and values produce
equal instance hashes independent of cooker/platform order.

### `ShaderInterfaceManifestV1`

```text
ShaderInterfaceManifestV1 {
  schema_version,
  interface_id,
  stage_interfaces[],
  vertex_or_mesh_inputs[],
  resource_bindings[],
  push_or_inline_fields[],
  specialization_fields[],
  render_target_interfaces[],
  source_map_hash,
  canonical_hash
}
```

The manifest describes semantics, numeric layouts, access, array bounds and
cross-stage compatibility only. It contains no source-language AST, compiler
object, binary-module handle or API-specific binding type. Offline toolchains
may produce target variants only when reflection validates exact equality to
the locked manifest. `SHADER-P1` remains the existing compiler/interface gate;
this SPEC references it and does not redefine its descriptor.

### Deterministic pipeline key

`PresentationPipelineKeyV1` is:

```text
SHA-256(
  "nextengine.presentation-pipeline-key.v1\0"
  || JCS({
       shader_interface_manifest_hash,
       target_shader_variant_hash,
       render_state_profile_hash,
       vertex_or_mesh_layout_hash,
       render_target_profile_hash,
       specialization_value_hash,
       capability_path_id
     })
)
```

All fields are exact locked hashes/IDs. Cache warmth, compile time, worker,
device pointer and insertion order are forbidden. A compile/cache miss may
delay or use the definition’s validated fallback pipeline; it cannot choose a
different material/gameplay outcome.

## Exact SDR color and alpha profile

`SdrColorProfileV1` is mandatory and fixed:

- chromaticities and white point: IEC sRGB primaries with D65;
- authored encoded components: `u16` UNORM in `[0, 65535]`;
- decode/encode transfer: exact IEC sRGB piecewise function identified by
  `SrgbTransferV1`, evaluated by the pinned implementation profile;
- working/compositing space: linear sRGB;
- alpha boundary representation: straight `u16` UNORM;
- compositing representation: premultiplied linear RGB with alpha;
- transparent canonicalization: when alpha is zero, RGB is exactly zero;
- blend operation: declared Porter-Duff operation plus explicit rounding mode;
- tone/output mapping: exact `SdrOutputTransformV1` hash in the presentation
  profile; no host display profile is implicit.

Material color fields declare whether the value is encoded sRGB or linear data.
Normal/roughness/metalness/mask/data textures never receive implicit color
conversion. Missing or ambiguous tag rejects cooking/publication.

A pinned capture profile fixes numeric precision, operation order, rounding,
output format and normalized frame hash, making SDR pixels exact for that
profile. Other conforming devices may be tolerance-classified only where the
profile says so; they cannot claim pinned-pixel equality.

HDR is optional and separated by `HdrPresentationBoundaryV1`, which declares
working primaries/white, transfer, luminance range, metadata and deterministic
SDR fallback transform. Absence or failure of HDR uses that exact fallback and
MUST NOT change material IDs, authoritative state, visibility/perception or SDR
reference evidence.

## VFX cue contract

```text
VfxCueV1 {
  schema_version,
  cue_stream_id,
  cue_sequence,
  cue_id,
  acknowledgment_key,
  source_event_id,
  source_event_slot,
  source_cue_slot,
  subject_persistent_id_or_none,
  effect_asset_id,
  effect_asset_hash,
  activation_simulation_tick,
  lifecycle,
  continuous_instance_id_or_none,
  target_cue_id_or_none,
  parameter_values[],
  attachment,
  fallback_effect_id_or_none,
  canonical_hash
}
```

For identity formulas in this section, every SHA-256 output is exactly 32 bytes
and is rendered only as lowercase 64-hex at text/JSON boundaries. JCS preimages
encode `Hash256` as lowercase 64-hex strings, nominal 128-bit IDs as lowercase
32-hex strings, `u64`/`u32` as fixed-width lowercase 16-/8-hex strings, closed
enums as their exact case-sensitive tokens and an absent optional ID as JSON
`null`. Concatenation outside JCS uses raw 32-byte hashes and explicit
`u64_le`/`u32_le`; no locale, native integer width or textual number is
permitted.

`cue_stream_id: Hash256` is
`SHA-256("nextengine.presentation-cue-stream.v1\0" ||
JCS({project_composition_lock_hash, session_id}))`. It remains stable across
snapshot-epoch changes and never uses a backend/device identity. Runtime
projection assigns `cue_sequence: u64` in canonical committed-event order; it
is contiguous within the VFX subsequence, begins at zero and cannot wrap.
Re-extraction and replay assign the same sequence.

`cue_id: Hash256` is
`SHA-256("nextengine.vfx-cue-id.v1\0" ||
JCS({project_composition_lock_hash, cue_stream_id, cue_sequence,
source_event_id, source_event_slot, source_cue_slot, effect_asset_id,
subject_persistent_id_or_none}))`; an absent subject is canonical JSON `null`.
It is independent of `snapshot_epoch` and `snapshot_sequence`.
`VfxAcknowledgmentKeyV1`, stored in `acknowledgment_key`, is
`SHA-256("nextengine.vfx-acknowledgment.v1\0" || cue_stream_id ||
u64_le(cue_sequence) || cue_id)`. It is therefore stable across extraction
reset, device loss and cache-generation changes. `acknowledgment_key`,
`continuous_instance_id_or_none` when present and `target_cue_id_or_none` when
present are also `Hash256`. None of these IDs may be supplied by a package or
adapter. `VfxCueV1.canonical_hash` is
`SHA-256("nextengine.vfx-cue.v1\0" || JCS(body))`, where `body` omits only
`canonical_hash`.

`lifecycle` is
`OneShot | StartContinuous | UpdateContinuous | StopContinuous | Cancel`.
For a `StartContinuous` cue the only valid present instance ID is
`SHA-256("nextengine.vfx-continuous-instance.v1\0" ||
acknowledgment_key)`.
Presence and compatibility are closed:

| Lifecycle | `continuous_instance_id_or_none` | `target_cue_id_or_none` | Required relation |
|---|---|---|---|
| `OneShot` | absent | absent | one terminal at-most-once consumption outcome |
| `StartContinuous` | present | absent | ID equals the derived start instance ID and no active instance has that ID |
| `UpdateContinuous` | present | present | target is the current head cue of that exact active instance |
| `StopContinuous` | present | present | target is the current head cue of that exact active instance; terminal removal is atomic |
| `Cancel` of pending one-shot | absent | present | target is the exact checkpointed or newly staged pending one-shot and has not reached its deterministic activation transition |
| `Cancel` of continuous instance | present | present | target is the current head cue of that exact active continuous instance; terminal removal is atomic |

Every target has a lower `cue_sequence`, belongs to the same `cue_stream_id`
and resolves before publication. An update/stop/continuous-cancel must preserve
the start record’s effect asset/hash, subject and attachment identity; it may
change only fields explicitly mutable in the effect parameter schema.
Every `OneShot` first enters the bounded checkpointed pending map described
below. A one-shot `Cancel` may therefore arrive in a later extraction prefix
but MUST target that exact still-pending entry. An already acknowledged,
omitted, cancelled or otherwise non-pending one-shot target is stale and
rejects. A continuous `Cancel` with an absent instance or a non-current target
also rejects.

Canonical VFX order is
`(cue_stream_id, cue_sequence, activation_simulation_tick, source_event_id,
source_event_slot, cue_id)`. The sequence is authoritative for presentation
consumption order; the remaining fields are checked tie-break/provenance data,
not a worker-selected reorder. The same `(cue_stream_id, cue_sequence)`,
`cue_id` or `acknowledgment_key` with different provenance/canonical bytes is a
collision and rejects the complete snapshot and uncommitted consumption-state
update. Competing `StartContinuous` cues that derive the same
`continuous_instance_id_or_none` from different start identities/bytes are a
start collision. By contrast, `UpdateContinuous`, `StopContinuous` and
`Cancel` MUST legally reuse the active instance ID; they are accepted only
when their target equals the checkpointed or unique earlier-staged current head
and their immutable start binding matches the active history. Two different
next transitions that both target one current head are competing-head
collisions. Absent optional IDs are not identity values. Exact duplicate
canonical bytes collapse before batching.

The cue consumer:

1. validates cue-stream continuity, prefix roots, lifecycle presence,
   target compatibility and all identity derivations before any realization;
2. applies the canonical order and removes only exact duplicate bytes;
3. rejects cue identity, sequence, acknowledgment, competing-start or
   competing-head collisions before publication;
4. applies `UpdateContinuous`, `StopContinuous` and `Cancel` only to the
   validated pending one-shot or named current continuous-instance head;
5. commits acknowledgment/active-instance state before a one-shot or
   continuous realization becomes eligible for a cache generation, so recovery
   can omit but never repeat an acknowledged one-shot;
6. uses only the exact declared effect fallback when content/capability is
   unavailable;
7. records missing/omitted/fallback disposition and separate observed
   realization evidence without changing the source
   `DomainEvent` or simulation state.

Continuous VFX state is reconstructible from
`PresentationConsumptionStateV1`, the current accepted snapshot and locked
content. Unbounded particle state, random device seed or wall time is not
persisted. Any stochastic presentation uses a presentation-only named seed
recorded in capture evidence and excluded from gameplay hashes.

## Cue acknowledgment and bounded CPU consumption state

`PresentationConsumptionStateV1` is engine-owned CPU state and is explicitly
outside every invalidatable GPU/UI/VFX cache:

```text
PresentationConsumptionStateV1 {
  schema_version,
  session_id,
  project_composition_lock_hash,
  presentation_profile_hash,
  cue_stream_id,
  next_expected_cue_sequence,
  consumed_cue_prefix_root,
  acknowledged_one_shot_count,
  one_shot_acknowledgment_root,
  pending_one_shots[],
  pending_one_shot_root,
  active_continuous_instances[],
  active_continuous_instance_root,
  last_snapshot_epoch,
  last_snapshot_sequence,
  state_revision,
  canonical_hash
}
```

The initial `one_shot_acknowledgment_root` is
`SHA-256("nextengine.vfx-one-shot-root-empty.v1\0" || cue_stream_id)`.
For zero-based acknowledged one-shot ordinal `n`, the next root is exactly
`SHA-256("nextengine.vfx-one-shot-root-step.v1\0" || prior_root ||
u64_le(n) || acknowledgment_key || u8(consumption_disposition_tag))`, where
`consumption_disposition` is the closed pre-backend enum
`PrimaryEligible | FallbackEligible | Omitted | Cancelled`.
Its exact binary tags are respectively `0x00`, `0x01`, `0x02`, `0x03`;
unknown tags reject. `prior_root` and `acknowledgment_key` are raw 32-byte
values, so this preimage has no textual or variable-width field.
The checked `u64` count and one hash-chain root are constant-size, so prior
acknowledgment keys do not require an unbounded set.
`next_expected_cue_sequence` plus `consumed_cue_prefix_root` is the
epoch-independent replay barrier: a snapshot-epoch reset cannot reset or
advance it.

Every new `OneShot` is staged into `pending_one_shots` before any activation
decision. Entries contain cue/acknowledgment IDs, activation tick, exact
effect/fallback/content hashes, bounded parameters/attachment and source
provenance. The exact pending order is
`(activation_simulation_tick, cue_id canonical bytes, acknowledgment_key
canonical bytes)`, and `pending_one_shot_root` is
`SHA-256("nextengine.vfx-pending-one-shots.v1\0" ||
JCS(pending_one_shots))` over that list.

For an accepted snapshot, all VFX slices across physical cue batches are first
reassembled and validated as one closed contiguous prefix; no consumption
commit occurs at an individual batch boundary. The transition applies every
new cue/cancel in canonical sequence, then, only after the Runtime-declared
same-tick extraction cutoff is closed, matures pending entries whose
`activation_simulation_tick <= snapshot.simulation_tick` in pending order.
Maturation appends exactly one `PrimaryEligible`, `FallbackEligible` or
`Omitted` disposition and removes the entry. A valid one-shot `Cancel` removes
its target and appends `Cancelled` in the same atomic state revision.
Consequently worker/batch/extraction segmentation cannot lose a pending target
or choose whether same-cutoff cancellation wins; a cue assigned after the
cutoff belongs to the next tick and cannot retroactively cancel an acknowledged
one-shot.

Each `active_continuous_instances` entry contains the instance ID,
immutable start identity tuple, start/current cue IDs and acknowledgment keys,
last consumed sequence, bounded canonical parameter state, presentation-only
seed and terminal-free lifecycle state. Its maximum count, parameter bytes and
cue look-ahead, pending-one-shot count and pending parameter bytes are finite
values in the locked presentation profile. Capacity, counter or sequence
overflow rejects the candidate; the implementation never evicts a pending
one-shot/active instance or resets the acknowledgment root to make room.
Before hashing, the exact total ordering is ordering by
`continuous_instance_id` canonical bytes, then `start_cue_id` canonical bytes,
then `start_acknowledgment_key` canonical bytes. A duplicate full key with
different entry bytes rejects.
`active_continuous_instance_root` is
`SHA-256("nextengine.vfx-active-instances.v1\0" ||
JCS(active_continuous_instances))` over that exact sorted list.
`canonical_hash` is
`SHA-256("nextengine.presentation-consumption-state.v1\0" || JCS(body))`,
where `body` omits only `canonical_hash`.

For one contiguous cue prefix, Rendering Team privately stages:

- the next stream prefix/root and sequence;
- all pending-one-shot insert/cancel/maturation and acknowledgment-root
  updates;
- the exact resulting active-instance map;
- a realization plan and typed fallback/omission outcomes.

It validates the whole transition, then atomically publishes one incremented
state revision, its `PresentationConsumptionCheckpointV1` generation and a
complete eligible realization plan. No backend work is eligible before this
CPU/checkpoint commit. A crash/device fault after the commit may omit the visual
outcome and records that fact, but recovery MUST NOT replay its one-shot. A
fault before the commit exposes neither the plan, checkpoint nor a partial
state revision. Consumption-state changes never enter gameplay/replay hashes
or write to simulation.

Acknowledgment records only at-most-once consumption disposition. Backend
submission/display success is not retroactively written into that root.
`PresentationRealizationOutcomeV1` records the separate closed observed result:

```text
PresentationRealizationOutcomeV1 {
  schema_version,
  cue_stream_id,
  cue_sequence,
  cue_id,
  acknowledgment_key,
  consumption_state_revision,
  consumption_state_hash,
  presentation_snapshot_hash,
  cache_generation_or_none,
  realization_route,
  outcome,
  diagnostic_code_or_none,
  canonical_hash
}
```

`realization_route` is `Primary | DeclaredFallback | None`; `None` is valid
only for an omitted/cancelled operation. `outcome` is
`Realized | SubmittedNotObserved | OmittedAtCommit | CancelledAtCommit |
DeviceFaultAfterCommit`, with exact binary tags `0x00`, `0x01`, `0x02`,
`0x03`, `0x04` in that order. The record canonical hash is
`SHA-256("nextengine.presentation-realization-outcome.v1\0" || JCS(body))`,
where `body` omits only `canonical_hash`; JCS uses the identity encoding
profile above and the exact enum token. Records sort by
`(consumption_state_revision, cue_stream_id canonical bytes, cue_sequence,
acknowledgment_key canonical bytes)`. The same sort key with different
cue/state/snapshot binding or outcome rejects.

`realization_outcome_root` is
`SHA-256("nextengine.presentation-realization-outcome-root.v1\0" ||
JCS(ordered_record_hashes))`, including the exact empty array when there are no
records. Cardinality for one atomic consumption transition is exact:

- each one-shot that matures or is cancelled produces exactly one record bound
  to that one-shot’s acknowledgment key; a newly inserted still-pending
  one-shot produces zero until its terminal transition;
- each non-duplicate `StartContinuous`, `UpdateContinuous`,
  `StopContinuous` and continuous `Cancel` produces exactly one record bound
  to that lifecycle cue’s acknowledgment key;
- an exact duplicate produces no additional record.

An evidence generation closes only after the observation cutoff fixed by the
locked presentation/capture profile. `SubmittedNotObserved` is the immutable
terminal statement for that cutoff, not a placeholder. No later generation may
replace an outcome for the same run/cue/state identity; a later observation is
a different run/evidence identity. Recovery evidence may report loss after
commit but cannot alter consumption state or reopen a cue.

`PresentationConsumptionCheckpointV1` stores the canonical state bytes/hash as
bounded presentation-session recovery metadata, separate from authoritative
game saves and cache manifests. Recovery loads the last complete checkpoint,
verifies the composition/profile/stream binding, canonical hash, cue prefix,
one-shot acknowledgment/pending roots and active-instance root, then may replay
only the contiguous canonical cue suffix beginning at
`next_expected_cue_sequence`.
Reconstruction publishes a state only when the recomputed canonical hash is
exact. If no valid checkpoint exists for a nonzero stream position, the
presentation path becomes unavailable and does not replay one-shots or infer
active instances.

Every transition/reconstruction emits
`PresentationConsumptionEvidenceV1` containing prior/new state revisions and
canonical hashes, consumed cue-slice roots, one-shot acknowledgment root,
pending-one-shot root, active-instance root, realization-outcome root and
recovery reason. Its
canonical evidence hash is
`SHA-256("nextengine.presentation-consumption-evidence.v1\0" || JCS(body))`.
It is separate from gameplay hashes and is required by `VFX-P1` and
`PRESENTATION-CACHE-P1`.

## Cache reconstruction and device loss

```text
PresentationCacheManifestV1 {
  schema_version,
  cache_generation,
  project_composition_lock_hash,
  content_manifest_hash,
  presentation_profile_hash,
  presentation_snapshot_hash,
  snapshot_epoch,
  snapshot_sequence,
  presentation_consumption_state_hash,
  presentation_consumption_state_revision,
  consumed_cue_prefix_root,
  cache_schema,
  deterministic_key_set[],
  producer_toolchain_hashes[],
  canonical_hash
}
```

`presentation_snapshot_hash`, `snapshot_epoch` and `snapshot_sequence` MUST
equal the exact currently accepted `PresentationSnapshotV2`.
`presentation_consumption_state_hash`,
`presentation_consumption_state_revision` and `consumed_cue_prefix_root` MUST
equal the exact current `PresentationConsumptionStateV1`. `canonical_hash` is
`SHA-256("nextengine.presentation-cache-manifest.v1\0" || JCS(body))`, where
`body` omits only `canonical_hash`.

Before any cache record is read, exposed or used as a rebuild input, the
consumer validates all six snapshot/consumption binding fields plus project,
content and profile hashes against one atomically observed accepted
snapshot/state pair. A mismatch invalidates the whole cache generation. A
rebuild captures that pair before staging and revalidates it immediately before
atomic exposure; if either revision/root advances, the staged generation is
discarded rather than rebound or partially exposed. The manifest is an
optimization index, not authoritative save data, and never derives
acknowledgment from cache contents.

On device loss or cache corruption:

1. invalidate every affected GPU/UI/VFX cache generation;
2. retain simulation, session, accepted snapshot and the engine-owned CPU
   `PresentationConsumptionStateV1` unchanged;
3. stop publishing incomplete frames;
4. validate/rebuild from the exact manifest-bound
   content/interface/profile/snapshot/consumption-state inputs;
5. use the consumption-state replay barrier to suppress acknowledged one-shots
   while preserving the exact pending-one-shot map and restoring only the
   exact active continuous-instance map;
6. atomically expose a complete new cache generation;
7. emit typed recovery evidence including prior/new cache roots and exact
   snapshot/consumption binding fields.

Failure uses a validated fallback material/effect/presentation path or reports
the presentation unavailable. It never mutates simulation, asks ECS for an
unversioned live pointer, changes command ordering or selects another
authoritative motor/physics result.

## Stable diagnostics

Required codes include:

- `PRESENTATION_SNAPSHOT_INVALID`;
- `PRESENTATION_SNAPSHOT_STALE`;
- `PRESENTATION_OBJECT_KEY_COLLISION`;
- `PRESENTATION_BATCH_BOUNDARY_INVALID`;
- `PRESENTATION_CUE_KIND_INVALID`;
- `PRESENTATION_CUE_ORDER_INVALID`;
- `PRESENTATION_CUE_ENVELOPE_COLLISION`;
- `PRESENTATION_REFERENCE_MISSING`;
- `MATERIAL_SCHEMA_INVALID`;
- `MATERIAL_INSTANCE_INCOMPATIBLE`;
- `SHADER_INTERFACE_MISMATCH`;
- `PRESENTATION_PIPELINE_KEY_INVALID`;
- `COLOR_PROFILE_INVALID`;
- `HDR_FALLBACK_INVALID`;
- `VFX_CUE_IDENTITY_COLLISION`;
- `VFX_CUE_STREAM_GAP`;
- `VFX_CUE_PREFIX_MISMATCH`;
- `VFX_CUE_LIFECYCLE_INVALID`;
- `VFX_CUE_TARGET_INVALID`;
- `VFX_ACKNOWLEDGMENT_KEY_COLLISION`;
- `VFX_CONTINUOUS_START_COLLISION`;
- `VFX_CONTINUOUS_HISTORY_MISMATCH`;
- `VFX_COMPETING_HEAD_TRANSITION`;
- `VFX_PENDING_ONE_SHOT_INVALID`;
- `VFX_CUE_REFERENCE_MISSING`;
- `PRESENTATION_CONSUMPTION_STATE_CORRUPT`;
- `PRESENTATION_CONSUMPTION_STATE_OVERFLOW`;
- `PRESENTATION_CACHE_CORRUPT`;
- `PRESENTATION_CACHE_BINDING_MISMATCH`;
- `PRESENTATION_DEVICE_RECOVERY_FAILED`.

All diagnostics use stable structured envelopes and exact causal/content/
snapshot identifiers. Text logs and rendered pixels alone are never the oracle.

## Verification gates

Each row is the canonical `GateDescriptorV1` source for its gate.

| Gate | Primary owner | Contributors | Reproducible command/scenario | Pass threshold | Required evidence | Fallback | VS closure |
|---|---|---|---|---|---|---|---|
| `PRESENTATION-P1` | Rendering Team | Runtime Team, Player Experience Team | `next gate PRESENTATION-P1 --scenario extraction-parity-v2 --roots game,headless,capture-worker --permutations all` | Exact closed inputs produce byte-identical snapshot/object/camera/UI/common-cue-envelope roots and exact canonical batch boundaries across extraction-order, cue-family grouping, fragment-size, thread-count and worker permutations; 100% cue-kind/rank/ordinal/local-key/payload-hash/order tamper vectors reject; 0 partial publication, reverse write or gameplay-hash dependency | snapshot/batch/profile manifests, typed cue-envelope corpus, canonical record/batch roots, root-parity, mutation and publication audits | retain prior complete snapshot or publish explicit presentation-unavailable result | VS-05, VS-07, VS-11, VS-15 |
| `MATERIAL-P1` | Asset & Persistence Team | Rendering Team | `next gate MATERIAL-P1 --scenario neutral-material-shader-interface --corpus all` | 100% definition/instance/interface/pipeline-key positive corpus canonicalizes identically; 100% unknown field, unit/color, binding, reflection, fallback and key faults reject before publication | content/material/interface manifests, canonical hashes, reflection and negative-corpus reports; existing SHADER-P1 result | reject candidate and retain previous material/interface generation or exact fallback | VS-01, VS-07, VS-10, VS-15 |
| `COLOR-P1` | Rendering Team | Asset & Persistence Team, Verification & Evidence Team | `next gate COLOR-P1 --scenario sdr-color-alpha-v1 --vectors all --capture-profile pinned` | All transfer/composite/rounding vectors match exact expected values; transparent canonicalization has 0 violation; pinned SDR frame roots exact; HDR absence/fault produces exact SDR fallback | color/output/HDR manifests, golden vectors, normalized pinned frames and fallback report | reject incompatible profile; use exact locked SDR path | VS-07, VS-15 |
| `VFX-P1` | Rendering Team | Asset & Persistence Team, Verification & Evidence Team | `next gate VFX-P1 --scenario vfx-cue-lifecycle --cycles 10000 --faults all` | 10,000 dedupe/pending/activation/start/update/stop/cancel/epoch-reset/recovery cycles and all worker/batch permutations produce exact cue-prefix, acknowledgment, pending-one-shot, active-instance, consumption-state and evidence roots; legal update/stop/cancel reuse of one active instance passes, while 100% competing-start/head, immutable-history, sequence/prefix/ack-key/target/presence/canonical-byte and cutoff-boundary tamper vectors reject atomically; every crash boundary before/after CPU-state commit causes 0 repeated one-shot, lost pending target, collision admission, unbounded state or simulation write; fallback exact | cue/content/profile/batch manifests, consumption checkpoints, one-shot/pending/active-instance/lifecycle/evidence roots, activation/cancel/cutoff and tamper corpus, crash/fault/recovery traces and required media evidence | cancel pending cue, omit exact cue or use only declared effect fallback; corrupt/missing noninitial checkpoint makes presentation unavailable; source event/state unchanged | VS-05, VS-07, VS-15 |
| `PRESENTATION-CACHE-P1` | Rendering Team | Runtime Team, Player Experience Team | `next gate PRESENTATION-CACHE-P1 --scenario presentation-cache-reconstruction --fault-boundaries all --cycles 10000` | Every cache/device fault retains exact simulation/session/snapshot and CPU consumption-state roots and atomically rebuilds only manifest-bound pending/active presentation state or reports unavailable; 100% presentation_snapshot_hash/epoch/sequence/presentation_consumption_state_hash/revision/prefix-root and checkpoint/acknowledgment/pending/active-map tamper vectors reject before read, rebuild or exposure; 0 stale/partial exposure, acknowledged one-shot replay or cache-derived acknowledgment | cache/snapshot manifests, exact binding matrix, consumption checkpoints/evidence hashes, one-shot/pending/active-instance roots, before/after authoritative roots, tamper/fault matrix and atomic-publication audit | invalidate cache generation and rebuild/use validated fallback without simulation write; stale/invalid binding or consumption state fails presentation unavailable without replay | VS-02, VS-05, VS-11, VS-15 |

## Requirements

| ID | Требование | Primary owner | Contributors | Blocking gates | Required evidence | VS / profile closure |
|---|---|---|---|---|---|---|
| REQ-144 | Rendering Team MUST stage and atomically publish one immutable canonical `PresentationSnapshotV2` at the Runtime-declared boundary, with stable epoch/object keys, one typed total-order `PresentationCueEnvelopeV1` across all cue kinds, deterministic worker-independent batch boundaries and no reverse authority into simulation. | Rendering Team | Runtime Team, Player Experience Team | PRESENTATION-P1 | snapshot/batch/profile manifests, typed cue-envelope/order corpus, canonical/root-parity, permutation, mutation and publication audits | VS-05, VS-07, VS-11, VS-15 |
| REQ-145 | Neutral material definition/instance, shader-interface manifest, deterministic pipeline key and exact SDR color/alpha profile MUST be content-addressed, bounded and backend-free; optional HDR MUST have exact SDR fallback. | Asset & Persistence Team | Rendering Team, Verification & Evidence Team | MATERIAL-P1, COLOR-P1, SHADER-P1 | content/material/interface/color manifests, golden/reflection corpora, pipeline/frame roots and fallback report | VS-01, VS-07, VS-10, VS-15 |
| REQ-146 | VFX cues MUST use exact epoch-independent event-derived cue/acknowledgment/continuous-instance identity, contiguous canonical order, lifecycle-compatible target/history rules, bounded checkpointed pending one-shots, legal active-instance reuse, exact dedupe/cancel semantics and bounded atomic CPU consumption state plus declared fallback without changing source events or simulation. | Rendering Team | Asset & Persistence Team, Verification & Evidence Team | VFX-P1 | cue/content/batch manifests, consumption checkpoints, lifecycle/acknowledgment/pending/active-instance/evidence roots, activation/cancel/cutoff and tamper corpus, fallback report and media evidence | VS-05, VS-07, VS-15 |
| REQ-147 | GPU/UI/VFX caches MUST be reconstructible after corruption/device loss only from exact `PresentationCacheManifestV1` bindings to the accepted snapshot hash/epoch/sequence and bounded engine-owned CPU `PresentationConsumptionStateV1` hash/revision/prefix; consumption state remains outside cache invalidation, prevents acknowledged one-shot replay, preserves pending/active instances and authoritative roots, and only complete generations publish; observable output MUST follow SPEC-15 review. | Rendering Team | Runtime Team, Player Experience Team, Verification & Evidence Team | PRESENTATION-CACHE-P1, PRESENTATION-P1 | cache/snapshot manifests, exact binding corpus, consumption checkpoint/evidence hashes, acknowledgment/pending/active-instance roots, device-fault and tamper matrix, authoritative roots, capture bundle and V2 review decision when required | VS-02, VS-05, VS-07, VS-11, VS-15 |

## Failure paths

| ID | Trigger | Required result | Primary owner | Contributors | Blocking gates | Required evidence | VS / profile closure |
|---|---|---|---|---|---|---|---|
| FAIL-060 | Invalid/stale/colliding snapshot, object key or typed cue envelope/order; malformed/incompatible material, shader interface, pipeline key, color/HDR profile or required content reference | Reject the complete candidate before publication, retain the previous snapshot/content generation and never expose backend data or infer a replacement authoritative value. | Rendering Team | Runtime Team, Asset & Persistence Team, Player Experience Team | PRESENTATION-P1, MATERIAL-P1, COLOR-P1, SHADER-P1 | complete cue/schema negative corpus, prior/candidate roots, reflection/color and atomic-publication audits | VS-01, VS-05, VS-07, VS-10, VS-11, VS-15 |
| FAIL-061 | VFX cue sequence/prefix/identity/acknowledgment/start/history/head/target/lifecycle or pending/cutoff collision/tamper; missing content; consumption checkpoint corruption/overflow; cache binding/corruption, device loss or rebuild/publication fault | Reject the complete uncommitted transition; dedupe/cancel pending/omit or use only the exact declared presentation fallback; invalidate incomplete/stale cache while retaining valid bounded CPU consumption state; preserve simulation/session/snapshot roots and never replay acknowledged one-shots, lose/infer pending or active instances, or write presentation state back. A missing/corrupt noninitial consumption checkpoint or mismatched cache binding makes presentation unavailable until exact reconstruction succeeds. | Rendering Team | Runtime Team, Asset & Persistence Team, Verification & Evidence Team | VFX-P1, PRESENTATION-CACHE-P1 | cue/cache/binding/checkpoint/device tamper, activation/cutoff and crash-boundary matrix, before/after authoritative and consumption-state roots, acknowledgment/pending/active-instance/evidence hashes, atomic publication audit and media evidence | VS-02, VS-05, VS-07, VS-11, VS-15 |

## Technology neutrality

This contract selects no graphics API, renderer, render graph, shader language
or compiler, window system, UI toolkit, VFX runtime, image library or GPU
vendor. Adapter internals and caches remain private. Architecture admission
does not claim exact pixels outside the pinned capture profile or satisfy any
automatic/human/release gate by itself.
