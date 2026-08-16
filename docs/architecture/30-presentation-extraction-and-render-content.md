# SPEC-30: Presentation snapshot, camera, UI and render content

| Поле | Значение |
|---|---|
| ID | SPEC-30 |
| Статус | Accepted |
| Версия | 3.3 |
| Последняя проверка | 2026-08-16 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-28](28-skeletal-animation-retargeting-and-ik.md), [SPEC-29](29-platform-host-and-application-session.md), [ADR-019](adr/019-canonical-player-actions-and-presentation-authority.md), [ADR-028](adr/028-platform-session-and-presentation-authority.md), [ADR-035](adr/035-bounded-live-recovery-platform-host-and-presentation-cut.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md) |
| Заменяет | SPEC-30 3.2; updates the current atomic project activation type after R4c |

## Authority boundary

Presentation is a read-only projection of committed simulation state. Runtime
stages and atomically publishes one immutable `PresentationSnapshotV2` at the
fixed extraction boundary. Renderer and UI consume the newest complete
snapshot; they never write camera, visibility, widget or GPU state back into
gameplay.

Presentation cadence, interpolation, pixels, device capability, cache warmth,
Vulkan timestamps and shader compilation are excluded from gameplay/replay
roots. Authoritative targeting uses SPEC-18/26 queries, not screen coordinates,
depth buffers or camera matrices.

## `PresentationSnapshotV2`

The implemented snapshot is:

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

## Neutral render content

The current B0 content path uses exact neutral mesh, material, texture and
render-profile revisions in `RenderContentCatalogV1`. Cooking validates
canonical payloads, references, bounds, index/attribute consistency, color
interpretation and profile support before `ActivatedProjectV6` publication.

The implemented renderer consumes the locked B0 shader interface and derived
meshlet/indexed-indirect content. Backend handles, descriptor sets, command
buffers, SPIR-V compiler objects and Vulkan structs remain private. Stable
pipeline/cache keys derive only from exact content/profile/interface inputs;
cache state is reconstructible.

Missing optional normals select the declared authored/flat-normal path. Missing
material or unavailable optional shadow allocation selects only the declared
presentation fallback. These paths do not replace content identity or change
world/ledger roots. SDR color/alpha behavior and optional HDR fallback remain
profile-owned presentation behavior; exact pixels are required only by an
explicit pinned developer capture profile.

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
  reverse authority into gameplay.
- `content-package` proves neutral render records, catalog closure and declared
  fallbacks.
- `platform` is conditional for Vulkan/host/device changes.
- `visual-smoke` and captures are bounded human evidence, not architecture
  admission artifacts.
- `persistence-replay` proves that Save/Load/restart creates the fresh
  epoch/sequence-zero camera cut while authoritative and ledger roots remain
  exact.
