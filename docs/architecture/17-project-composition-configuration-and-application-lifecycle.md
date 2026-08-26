# SPEC-17: Direct project composition and activation

| Поле | Значение |
|---|---|
| ID | SPEC-17 |
| Статус | Accepted |
| Версия | 3.2 |
| Последняя проверка | 2026-08-26 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-018](adr/018-authoritative-project-composition-and-configuration.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-047](adr/047-simple-application-session-and-save-on-close.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md), [ADR-074](adr/074-systemic-strategic-agent-owner-vertical.md), [ADR-083](adr/083-public-creator-project-cli-vertical.md), [ADR-084](adr/084-public-creator-run-and-project-package-vertical.md), [ADR-085](adr/085-public-creator-project-inspect-and-diff-vertical.md) |
| Дополнительные зависимости V3.0 | [ADR-086](adr/086-public-creator-rpg-starter-template.md) |
| Дополнительные зависимости V3.1 | [ADR-087](adr/087-public-creator-runtime-scenario-and-prefix-minimization.md) |
| Заменяет | SPEC-17 3.1; records future generated-candidate promotion before direct exact project composition without changing current activation formats |

## Назначение

Current project path is deliberately direct:

```text
nextengine.project-authoring.v7
  → cook and validate exact artifacts
  → ProjectLockV3
  → validate complete hash closure
  → ActivatedProjectV8
```

There is no runtime dependency resolver. `ProjectManifestV1`, requirements,
semantic-version ranges, package catalog records/snapshots, selected records,
resolver profiles, `project.json`, `catalog.json` and
`composition-lock.json` are retired alpha formats.

## Sources of truth

| State | Source of truth | Not authority |
|---|---|---|
| Editable project intent | `nextengine.project-authoring.v7` source | cooked cache or runtime defaults |
| Exact runnable closure | `ProjectLockV3` | filesystem discovery or catalog selection |
| Current schema set | `SchemaRegistryManifestV2` referenced by the lock | dynamic registration |
| Cooked content/world/mechanics | exact manifests and blobs referenced by the lock | source paths or importer records |
| Published runtime project | atomically validated `ActivatedProjectV8` | partially decoded candidate |
| Application session | SPEC-29 session contracts | project authoring or lock policy fields |

Authoring describes only project-owned composition intent. Session recovery,
storage layout and shutdown policy do not belong to authoring or project lock;
save-on-close is the current application behavior from ADR-047.

ADR-083 exposes this same path as `next project validate` and
`next project cook`. ADR-084 adds authoring/package run and package publication;
the CLI does not override authoring identity or construct a reference-game
source/aggregate in Rust. ADR-085 adds source-neutral read-only inspect/diff;
the projection is diagnostic and never replaces the lock or activated project.
ADR-086 adds one fresh-output built-in starter; it production-loads/cooks before
publication and every later operation follows this same direct path.
ADR-087 adds a path-free exact-project-bound scenario view over authoring or
package. It validates the same closure before executing any scenario tick and
does not become a second project lock, resolver or mutable project authority.

## `ProjectLockV3`

`ProjectLockV3` contains exactly the project ID/revision, authoring hash,
schema/content/world/mechanics hashes, runtime/launch/platform profile hashes,
allowed presentation targets and its own canonical hash:

```text
ProjectLockV3 {
  project_id,
  project_revision,
  authoring_sha256,
  schema_registry_manifest_sha256,
  content_manifest_sha256,
  world_partition_manifest_sha256,
  mechanics_lock_sha256,
  runtime_determinism_profile_sha256,
  launch_profiles_sha256,
  platform_capability_profile_sha256,
  platform_timebase_profile_sha256,
  allowed_presentation_targets[],
  project_lock_sha256
}
```

Every hash is exact and non-zero; the target set is closed, sorted and unique.
The lock contains no requirements, version ranges, selected catalog records,
resolver trace, recovery policy, shutdown policy, storage policy, native handle
or backend-owned value.

## Cooking and package output

The cooker reads one bounded current authoring document and publishes:

- `project-lock.json`;
- `SchemaRegistryManifestV2`;
- content manifest;
- world partition manifest;
- mechanics lock and package payloads;
- render content catalog;
- exact referenced blobs and notices.

All outputs are staged privately. The cooker validates schemas, bounds,
canonical bytes, provenance and the complete reference graph before replacing
the output directory/current package. Failure leaves the prior output intact.
Cooker caches are reconstructible and do not enter lock identity unless their
produced artifact hash is explicitly referenced.

Every authored relative source/notice/span resolves beneath the selected
project root, including after symbolic-link resolution. Public cook accepts
only a new/empty or recognizable ContentStore root and uses its atomic
generation plus `CURRENT` switch; unrelated or symlink output fails closed.
Every individual authoring/source/notice read is bounded to 16 MiB.

Creator Project Package V1 wraps one complete immutable ContentStore
publication, canonical exact inventory, nonempty root `NOTICE` and the
production run proof. Its outer layout is public only for this current creator
envelope; the nested ContentStore generation layout remains private. Package
build accepts an absent destination, stages beside its resolved parent,
revalidates and reruns the staged bytes, then renames the complete directory.
It is not the native target package from R7.

Creator Project Projection V1 is a tool-owned read-only view over this exact
closure. Authoring produces it only after current load/cook validation; package
produces it only after exact inventory/NOTICE/activation and recorded-run rerun.
It exposes stable IDs, roots, schema/asset/dependency/chunk/package facts and
capabilities without paths, properties or private storage layout. Base→candidate
diff compares these keyed immutable facts and treats valid drift as successful
data, never as mutation or activation authority.

## Future generated-candidate promotion

SPEC-45/ADR-095 promotion occurs strictly before the current authoring → cook
boundary. A provider adapter cannot write `project.authoring.json`, a
ContentStore generation, `ProjectLockV3` or an active package directly.
Promotion selects one immutable hash-verified candidate, requires structural
and rights validation, stages ordinary project-source bytes beside the
destination, reopens them, and publishes only to an absent destination or one
matching the declared exact base hash.

After publication, normal authoring treats the bytes exactly like hand-authored
or importer-produced untrusted source. Their hashes participate in the current
cook and therefore change `ProjectLockV3` when content changes. Prompt, receipt,
provider session and discarded candidates do not participate in activation.
Failure or provider absence retains the prior project source and the current
ordinary creator workflow remains complete. This is a future constraint and
adds no current project/lock/package version.

## Atomic activation

Activation accepts `ProjectLockV3` directly and must:

1. validate its current format and canonical hash;
2. fetch every exact referenced manifest/blob from the supplied package;
3. validate schema, content, world, mechanics, runtime, launch and platform
   profile hashes plus all nested references;
4. validate neutral records, localization fallback closure, animation/skeleton
   relations, the exact `BodySchemaAssetV1` generation/compiler profile and
   render catalog against the content manifest;
5. build one immutable `ActivatedProjectV8` candidate, including exact
   body-schema, world-routine, 100-record population/navigation and required
   cognition/activity catalog closure;
6. publish it atomically only after the complete closure passes.

No component may substitute a compatible-looking record, resolve a range,
scan another directory or activate a partial closure. `game`, `headless` and
runtime-bearing tools use the same activation implementation and exact lock.

The current generic creator runtime derives its bootstrap identity from that
lock, binds the activated project's RPG and world-service definitions,
activates its authored initial population node and executes exactly one
production headless tick. It then follows the ordinary Application Session
save-on-close path. This proves a runnable project closure without requiring
reference-alpha roles or promising an interactive/scenario/resumable creator
session.

The separate creator-scenario entry keeps that one-tick command unchanged. It
admits 1–256 explicit tick actions, installs only the engine-owned routine/
population principal and command-stream routes required by the project, commits
each Runtime + World Services generation atomically and then uses the same
owner-complete save-on-close boundary. Assertions read only the final public
proof; minimization changes only the authored action prefix.

## Current-only formats

Until the first publicly supported v1 format is declared, authoring, lock,
registry and package families are current-only under ADR-046. A recognizable
retired format returns its typed `UNSUPPORTED_*` before nested decode or
publication. There are no defaults, aliases, automatic upgrades or in-place
rewrites. Source bytes remain untouched so a user can open them with the
matching older build or an explicit future export tool.

## Failure semantics and checks

Malformed/noncanonical authoring, hash mismatch, missing reference, duplicate
identity, dependency cycle, forbidden target, unsupported format or nested
closure mismatch rejects the complete candidate and preserves the previously
published project. Diagnostics include stable code plus expected/actual hashes
or identities.

`content-package` is the governing ProductCheck: cook the reference project,
reopen the package, validate all 37 roots/123 entries and activate the same
`ActivatedProjectV8` in required roots; then independently load/cook/publish/
activate `creator-smoke` with 16 roots/18 entries and three chunks, execute its
generic one-tick application path, build/reopen its exact creator package and
match the authoring/package runtime proofs. Focused `next_cli` tests cover exact
reports, unsupported/tampered package failure, output confinement, generation
preservation, location-independent inspect, empty authoring/package diff and
localized controlled drift plus scenario source/package parity and
failure-preserving prefix minimization. Native target packaging remains a separate
`v1-package`/R7 concern; launch/session changes additionally run active-host
`platform` as selected by SPEC-12.
