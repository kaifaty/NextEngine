# SPEC-03: Assets, current world streaming and persistence

| Поле | Значение |
|---|---|
| ID | SPEC-03 |
| Статус | Accepted |
| Версия | 2.3 |
| Последняя проверка | 2026-08-15 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-026](adr/026-deterministic-work-resource-and-streaming-admission.md), [ADR-032](adr/032-grounded-capsule-physics-checkpoint-version-boundary.md), [ADR-034](adr/034-player-targeting-replay-v5-and-mapping-provenance.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-047](adr/047-simple-application-session-and-save-on-close.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-051](adr/051-r3a-packaged-chunk-streaming-commit-boundary.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md) |
| Заменяет | SPEC-03 2.2; adds the R4a routine owner and current-only Replay V6 closure |

## Sources of truth

Before cooking, `nextengine.project-authoring.v3` and referenced source files
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

## Current packaged world streaming

`WorldPartitionManifestV1` binds neutral chunks to regions/cells and exact
dependencies. A chunk carries immutable definitions/content; durable placement
and tombstones stay in world owner state. `PersistentId` crosses chunk
boundaries; direct pointers and `RuntimeEntityId` do not.

The current R3 implementation applies the R3a production vertical to the
manifest-driven reference partition of four regions and 64 chunks. Exact
Rust-only reference roles still select `relay-station → frontier` for gameplay,
while the canonical multiregion route starts with the initial role and orders
the remaining bindings by `(region_id, chunk_id)`. Assets pins the exact
activated content generation, verifies `INDEX.v1` and performs
filesystem-path-opaque, bounded blob reads. Project activation remains eager
and returns the decoded project together with this pinned source.

World derives an immutable request from exact project/content/schema/partition
hashes, topology revision, world generation, target binding and ordered asset
revisions. Private workers fetch and decode `NeutralRecordV1`; merge and error
selection depend only on `AssetId`, not completion order. The unchanged
ADR-051 profile is
bounded to 64 assets, 1 MiB per blob, 16 MiB encoded total, at most four workers
(default two), channel capacity 64 and 1 MiB canonical decoded bytes.

A group becomes publishable only after file/index/blob hashes, canonical
encoding, schema and semantic class, asset/record identities, exact dependency
closure, external project-global dependencies and unique durable IDs validate.
Its opaque result hash binds the request, ordered revisions, record IDs and
logical resource counters.

Runtime evaluates the paired prepared Runtime/World generation at
`WorldStreamingCommit`, between `IngressCommit` and `PhysicalStep`. Final
publication is infallible. The first existing gameplay tick publishes
`Requested`; simulation advancement pauses for mandatory I/O; the next existing
tick publishes `Active`/`Unloaded`. Live snapshots never publish reconstructible
`Staged`/`Validated` bytes. Restore of pending work rebuilds and re-fetches the
request.

Fault, stale result, capacity denial, worker panic or corrupt dependency leaves
the declared `Requested` root, previous active chunks/generation and decoded
cache unchanged, so retry/restart is explicit. Unload retains durable state
before despawn and cannot resurrect collected/changed RPG state on reload.

R3b reuses this private path for all 64 bindings; a second transition while a
mandatory request is `Requested` receives the same stable busy rejection. It
does not add optional work, placement-catalog authority or residency/eviction.
Generic scheduler, cancellation tree, pins/leases, residency policy and
eviction contracts are not part of this Accepted SPEC. SPEC-23 remains Proposed
future intent; ADR-051 continues to govern the consumer-driven bounded path.

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

`ReplayManifestV6` binds the runtime, RPG, physics, world-streaming and optional
world-routine owner segments, exact
`InputMappingReceiptV2`, targeting/query facts, closed ingress and command
admission batches, typed streaming assignments, interaction availability,
expected receipts/events and per-tick full-tuple descriptors/application roots. Replay
is read-only production re-execution: it injects recorded authoritative inputs,
uses the ordinary validators/order and stops at the first divergence. There is
no branching/counterfactual replay API and no projection through Replay V5.

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

- `content-package`: deterministic cook and activation of the complete
  four-region/64-chunk, 114-entry closure plus malformed/cyclic/missing content
  and Luau/Wasm packages.
- `play`: exact initial-role → frontier-role → initial-role transition through
  the paired production Assets/World/Runtime path without changing the R2
  gameplay and ledger baseline.
- `persistence-replay`: Save in `Requested`, process restart, exact pinned
  reactivation and re-fetch converge with uninterrupted execution; ordinary
  Save/Load/Resume, Replay V6 and retired-format rejection remain exact.
- `performance --scenario smoke --mode report`: 1,000 real packaged transitions
  record Performance V5 `required_staging_bytes`; the 30-second limit remains
  report-only.
- `performance --scenario r3-multiregion-streaming --mode report`: 1,000
  transitions cycle over the canonical 64-chunk route with two workers and
  publish only the authoritative `streaming_world` root; the result is
  `REPORT_ONLY` while B-12 remains open.
