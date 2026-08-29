# SPEC-30: Presentation snapshot, camera, UI and render content

| Поле | Значение |
|---|---|
| ID | SPEC-30 |
| Статус | Accepted |
| Версия | 4.0 |
| Последняя проверка | 2026-08-29 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-28](28-skeletal-animation-retargeting-and-ik.md), [SPEC-29](29-platform-host-and-application-session.md), [ADR-019](adr/019-canonical-player-actions-and-presentation-authority.md), [ADR-028](adr/028-platform-session-and-presentation-authority.md), [ADR-035](adr/035-bounded-live-recovery-platform-host-and-presentation-cut.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md), [ADR-074](adr/074-systemic-strategic-agent-owner-vertical.md) |
| Дополнительные зависимости V3.5 | [SPEC-36](36-functional-tissue-condition-and-injury.md), [SPEC-37](37-character-embodiment-and-surface-deformation.md), [ADR-075](adr/075-product-grounded-functional-anatomy-and-character-embodiment.md) |
| Дополнительные зависимости V4.0 | [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-47](47-streaming-tts-and-spatial-speech-presentation.md), [ADR-099](adr/099-bounded-streaming-tts-through-ai-host-and-audio-scene.md) |
| Заменяет | SPEC-30 3.9; clarifies the current audio companion projection and bounded Proposed spatial-speech extension |

## Authority boundary

Presentation is a read-only projection of committed simulation state. Runtime
stages and atomically publishes one immutable `PresentationSnapshotV3` at the
fixed extraction boundary. Renderer and UI consume the newest complete
snapshot; they never write camera, visibility, widget or GPU state back into
gameplay.

Presentation cadence, interpolation, pixels, device capability, cache warmth,
Vulkan timestamps and shader compilation are excluded from gameplay/replay
roots. Authoritative targeting uses SPEC-18/26 queries, not screen coordinates,
depth buffers or camera matrices.

## `PresentationSnapshotV3`

The implemented snapshot is:

```text
PresentationSnapshotV3 {
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
  character_skinning_batches[],
  cue_batches[],
  environment_batch,
  canonical_hash
}
```

`snapshot_epoch` changes on project activation, authoritative Load/restart or
explicit extraction reset. Sequence starts at zero and increments once per
published snapshot. At 30 Hz simulation the application publishes complete
snapshots; a 30/60/144 Hz renderer may repeat the newest one without changing
authoritative output.

On Load/restart the first snapshot has a fresh epoch, sequence `0` and camera
previous/current samples equal, forcing a cut. The session remains `Suspended`
until explicit Resume.

All batches are bounded and canonical. Worker fragments are flattened, records
are validated and sorted by stable engine-owned identities, and only then are
they partitioned by the locked positive batch size. Worker identity,
completion order, input fragment size and hash-map order cannot change batch
boundaries or roots. Duplicate key with different bytes rejects the complete
candidate; publication failure retains the prior snapshot.

## Scene, camera and semantic UI

- `ScenePresentationRecordV2` binds stable presentation object identity,
  quantized current/previous transform samples, exact mesh/material references,
  bounds and presentation flags. `RuntimeEntityId`, ECS row and draw index are
  not durable identity.
- `CharacterSkinningPresentationRecordV1` binds that same stable object key to
  exact mesh/profile/skeleton/body-schema revisions, source animation-profile
  hash, `Sampled | HeldPresentationPose | BindPoseFallback` source mode,
  `FullCorrectives | ReducedCorrectives | BaseSkinningOnly | Culled`
  deformation LOD and a complete sorted render-joint local pose. A skinned
  scene record must have exactly one matching record in the same snapshot;
  mesh/object mismatch rejects the candidate. `Culled` is valid exactly when
  that matching scene record is invisible; a visible/cull mismatch rejects the
  complete candidate.
- `CameraPresentationRecordV2` carries typed integer/fixed-point camera intent
  and result, viewport, projection and cut/interpolation policy. Private
  renderer float matrices are derived caches.
- `SemanticUiPresentationRecordV1` carries semantic paths, stable element/text
  IDs, bounded values, style/accessibility roles and action affordances.
  Widget objects, localized rendered strings and glyph caches are private.
- `cue_batches` and `environment_batch` are bounded content hashes in the
  current snapshot. No public VFX cue lifecycle, acknowledgement ledger,
  continuous-instance recovery state or VFX cache checkpoint is implemented or
  required.

UI menus may compose a new immutable presentation snapshot while simulation is
suspended, but cannot mutate world state. Gameplay actions still enter through
the normalized action/command path.

R5f implemented the base SPEC-37 subprojection. It maps the existing
committed `PhysicalAnimationPoseV1` through one exact surface profile into the
current `PresentationSnapshotV3`; both reference player and NPC use the same
profile and retain distinct materials/body transforms. Joint palettes, CPU/GPU
buffers and deformed vertices remain renderer-owned reconstructible data.
R5g adds the small authored pose-corrective subset and explicit deformation
work selector. Held is a complete already-published local pose, not a hidden
animation cursor; renderer cadence is not serialized into the record and only
repeats a complete snapshot. Load/injury correctives, secondary motion and
severity/accessibility variants remain later cuts. Headless/null presentation
may omit all of this without changing command, physics, save, Replay or
gameplay roots.

R5i selects animation work before that unchanged R5g surface. Sampled, bounded
held and bind results map through one production reference composition helper;
`IntentOnly`, requested cull and exhausted held/bind fallback omit both the
character scene binding and skinning record from the next complete candidate.
An invalid visible-scene/culled-record closure rejects the whole candidate,
does not consume a presentation sequence and leaves the prior complete V3
snapshot available. The `animation-lod` matrix repeats that accepted snapshot
at 30/60/144 Hz and rotating target/cache revisions while exact Runtime,
ledger, Physics, RPG and physical-animation snapshots remain unchanged.

### Audio companion projection and Proposed speech streams

Current `AudioSceneSnapshotV1` from SPEC-08 is a separate immutable
presentation companion derived at the same committed boundary. It owns one
listener, clip-backed emitters/cues and deterministic acoustic-fact projection;
it does not enlarge `PresentationSnapshotV3` or write mixer/device state back
to simulation.

[SPEC-47](47-streaming-tts-and-spatial-speech-presentation.md) proposes a
current-only V2 audio-source successor for generated `SpeechStream` plus
acoustic scene/frame/result values. Presentation extraction resolves the
model-neutral mouth/head attachment into an ordinary quantized emitter
transform. PCM bytes, stream cursor, propagation cache, lip timing and device
counters remain outside `PresentationSnapshotV3`, save and gameplay roots.
Missing attachment falls back to the subject/root transform; missing TTS or
propagation falls back to authored text/voice and engine-native audio without
changing the accepted snapshot.

## Neutral render content

The current B0 content path uses exact neutral mesh, material, texture and
render-profile revisions in `RenderContentCatalogV1`. Cooking validates
canonical payloads, references, bounds, index/attribute consistency, color
interpretation and profile support before `ActivatedProjectV8` publication.

The implemented renderer consumes the locked B0 shader interface and derived
meshlet/indexed-indirect content. R5f performs bounded fixed-point linear blend
skinning from the exact profile and complete joint record. R5g first evaluates
selected sparse mesh-local correctives in canonical corrective/vertex order.
Driver weight is an exact clamped `u16` interpolation of the named local-joint
translation relative to its bind translation; weighted integer deltas truncate
toward zero before checked accumulation. Full selects Essential+Detail,
Reduced selects Essential, Base selects none and Culled emits no renderer work.
B0 hashes the resulting vertex stream and uploads position plus the locked
UV/normal template through a per-frame-slot host-visible Vulkan vertex ring.
Static draws keep the existing indexed-indirect path; skinned draws bind their
exact dynamic stream. Backend handles, descriptor sets, command buffers,
SPIR-V compiler objects and Vulkan structs remain private. Stable pipeline/
cache keys derive only from exact content/profile/interface inputs; cache state
is reconstructible.

Missing/invalid corrected output atomically retries the same sampled pose with
base LBS; invalid base skinning selects the complete authored bind mesh.
Explicit bind fallback bypasses correctives. Missing optional normals select
the declared authored/flat-normal path.
Missing material or unavailable optional shadow allocation selects only the
declared presentation fallback; the current R5f skinned draw intentionally
uses the optional no-shadow branch while the static shadow path remains
unchanged. These paths do not replace content identity or change world/ledger
roots. SDR color/alpha behavior and optional HDR fallback remain profile-owned
presentation behavior; exact pixels are required only by an explicit pinned
developer capture profile.

`RenderContentCatalogV1` and `PresentationSnapshotV3` are exact current-only
alpha contracts under ADR-046. R5g replaces their prior in-tree shapes; R5i
uses those shapes unchanged and introduces no compatibility alias, persisted
migration or second runtime catalog/snapshot family.

## Future generated material and asset preview

Under SPEC-46, generation intent such as real-world extent, texel density or
physical tile scale, map-channel/color-space/normal convention, pivot/axes and
LOD/collider expectation is pre-cook authoring metadata. It does not widen
`NeutralMaterialV1`, `NeutralTextureV1`, `NeutralMeshV1` or
`PresentationSnapshotV3` with prompts, provider types or receipts. Promotion
produces ordinary exact source and the current cooker remains responsible for
the final render records and B0 fallback closure.

A standardized material preview uses a pinned B0 profile and known sphere,
cube and plane; a static-prop preview adds a scale reference and turntable.
Camera, light/environment, extent, frame count and output quota are explicit.
Preview media may be hashed and compared as human/agent evidence, but exact
pixels and similarity scores are not content admission, gameplay identity or
renderer conformance. Structural record validation remains authoritative and
an unavailable preview uses the ordinary authored-source fallback rather than
promoting an invalid candidate.

## Device loss and failure semantics

Device or cache loss invalidates affected presentation caches, not simulation,
session save data or the accepted snapshot. Reconstruction uses the exact
project/content/profile/snapshot bindings and exposes a new cache generation
only when complete. Until then the renderer uses a declared fallback or reports
presentation unavailable.

Malformed/stale snapshot, object collision, invalid batch boundary, missing
required reference, incompatible render content/profile or cache-binding fault
rejects before exposure and preserves the previous complete snapshot/cache.
Diagnostics contain stable IDs/hashes; screenshot comparison alone is not a
correctness oracle.

## Product checks

- `play` proves snapshot cadence independence, camera/UI behavior and no
  reverse authority into gameplay; the focused R5g matrix repeats Full,
  Reduced, Base, Held and Culled publications at 30/60/144 Hz.
- `animation-lod` proves the current five-level animation-work planner across
  10,000 cadence/resource/publication transitions, including no-pose output,
  exact prior-snapshot retention and repeated B0 frame plans.
- `content-package` proves neutral render records, exact base-profile closure,
  authored correctives, shared player/NPC consumption, sampled deformation and
  the corrected → base LBS → bind fallback chain.
- `platform` is conditional for Vulkan/host/device changes.
- `visual-smoke` and captures are bounded human evidence, not architecture
  admission artifacts.
- `persistence-replay` proves that Save/Load/restart creates the fresh
  epoch/sequence-zero camera cut while authoritative and ledger roots remain
  exact.
