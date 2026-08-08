# SPEC-03: Assets, current world streaming and persistence

| Поле | Значение |
|---|---|
| ID | SPEC-03 |
| Статус | Accepted |
| Версия | 2.0 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-026](adr/026-deterministic-work-resource-and-streaming-admission.md), [ADR-032](adr/032-grounded-capsule-physics-checkpoint-version-boundary.md), [ADR-034](adr/034-player-targeting-replay-v5-and-mapping-provenance.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-047](adr/047-simple-application-session-and-save-on-close.md), [ADR-048](adr/048-direct-exact-project-lock.md) |
| Заменяет | SPEC-03 version 1.16 resolver/migration/session-object/future narrative clauses |

## Sources of truth

Before cooking, `nextengine.project-authoring.v2` and referenced source files
are editable intent. After cooking, `ProjectLockV3` plus its exact
`SchemaRegistryManifestV2`, content/mechanics/world/render manifests and blobs
are the only runtime content source. Runtime never scans source directories,
resolves version ranges or parses importer formats.

Mutable save state owns progression and durable world deltas only. It does not
copy immutable asset/model payloads or create a second RPG/physics/content
owner. Assets owns validation, cooking, immutable publication, low-level fetch
and atomic save generations; subsystem owners define the meaning of their
segments.

## Authoring, cooking and content

Importer adapters and first-party tools produce bounded neutral records with
stable `AssetId`/schema identity, canonical units, exact references, source
hashes and provenance. Source-specific/importer/backend metadata cannot leak
into runtime contracts.

Cooking is deterministic for equal canonical inputs:

1. decode current authoring and source records under finite limits;
2. validate schema, identity, references, bounds, provenance/license and
   target-profile support;
3. canonicalize and hash each immutable record/blob;
4. construct current schema/content/world/mechanics/render manifests;
5. construct exact `ProjectLockV3`;
6. reopen and validate the complete staged closure;
7. atomically publish the package/output.

Platform-neutral artifacts are byte-identical across supported targets for
equal inputs. Target-specific GPU/audio payloads have distinct target keys and
hashes. Cooker caches are reconstructible and never authority.

## Current world streaming

`WorldPartitionManifestV1` binds neutral chunks to regions/cells and exact
dependencies. A chunk carries immutable definitions/content; durable placement
and tombstones stay in world owner state. `PersistentId` crosses chunk
boundaries; direct pointers and `RuntimeEntityId` do not.

The current R2 implementation proves a canonical two-chunk transition. Fetch/
decode staging is private. A staged group becomes active only after exact
schema/content/partition/revision, required dependencies, bounds and duplicate
durable IDs validate, then commits at the declared simulation boundary in
canonical order. Fault, stale result or corrupt dependency preserves the
previous active world generation. Unload must retain durable state before
despawn and cannot resurrect collected/changed RPG state on reload.

Generic scheduler, cancellation tree, pins/leases and eviction contracts are
not part of this Accepted SPEC. The future R3a vertical is Proposed in SPEC-23
and may add only primitives demonstrated by a real chunk consumer.

## Save

`SaveManifestV2` remains the generic exact segment envelope. It binds build and
project identity, `ProjectLockV3`/schema/content/world/mechanics/runtime
profiles, tick and named RNG state, loaded chunk revisions, owner segment table
and the complete world `CommandLedgerV2` closure. Each segment has one owner,
current schema/version, byte length and domain-separated hash.

Current world checkpoint composition is `RuntimeSnapshotV3` +
`RpgSnapshotV2` + `PhysicsWorldCheckpointV1` inside `WorldCheckpointV4`.
`PhysicsCanonicalSnapshotV2` is the current grounded-capsule physical state;
physics snapshot V1 is not a fallback.

Save publication is freeze at a logical commit point → snapshot all owners →
write inactive generation → reopen/validate exact closure → atomically update
the pointer. The two-slot `SaveStore` always retains a previously complete
generation on write/power fault. Manual Save and save-on-close are the only
durable world publications; Suspend does not save.

Application-session snapshots are governed separately by SPEC-29/ADR-047 and
contain no content-addressed session object archive or object packs.

## Load and replay

Load is legal only in `Suspended`, selects the newest compatible complete save,
validates the full closure before mutation, then atomically replaces world
state. It creates a fresh presentation epoch/sequence `0` and waits for Resume.
Corrupt or incompatible input leaves the current world and source bytes intact.

`ReplayManifestV5` binds the same owner segments, exact
`InputMappingReceiptV2`, targeting/query facts, closed ingress and command
admission batches, expected receipts/events and per-tick compare roots. Replay
is read-only production re-execution: it injects recorded authoritative inputs,
uses the ordinary validators/order and stops at the first divergence. There is
no branching/counterfactual replay API and no projection through Replay V4.

`CommandLedgerV2` wire semantics, identity, reservation/receipt window and
golden roots are unchanged.

## Compatibility

All pre-v1 asset/project/save/replay families are current-only under ADR-046.
A recognizable retired outer version returns its typed `UNSUPPORTED_*` before
nested decode or publication. Runtime applies no defaults, auto-upgrade or
in-place rewrite and never deletes rejected user data. Migration is introduced
only after a publicly supported v1 format has a real successor.

## Public boundary and failure semantics

Public contracts contain neutral records/manifests, `ProjectLockV3`, stable
IDs/hashes, immutable chunk requests/results, Save/Replay manifests, owner
segments and typed diagnostics. They exclude filesystem paths, importer/vendor
records, task/allocator handles, raw pointers, ECS storage and caches.

Missing required content, invalid/cyclic dependency, stale/corrupt chunk,
duplicate persistent ID, save/replay hash/version mismatch or publication fault
rejects the complete candidate and retains the prior project/chunk/save/world.
Missing optional content uses only an exact declared fallback.

## Product checks

- `content-package`: deterministic cook, complete direct-lock/package closure,
  malformed/cyclic/missing content and Luau/Wasm packages.
- `play`: R2 two-chunk transition/unload-return behavior through production
  input and world paths.
- `persistence-replay`: Save → change → Load/Resume rollback, power-fault
  generation safety, direct Replay V5 comparison and typed retired-format
  rejection.
- `performance`: conditional for a changed streaming/I/O hot path; current
  evidence uses Performance V4 resource counters.
