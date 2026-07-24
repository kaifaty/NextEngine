# SPEC-17: Project composition, configuration и application lifecycle

| Поле | Значение |
|---|---|
| ID | SPEC-17 |
| Статус | Accepted |
| Версия | 1.2 |
| Владелец | Repository Owner |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Asset & Persistence Team, Developer Experience Team, Security & Governance Team, Release Engineering |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-018](adr/018-authoritative-project-composition-and-configuration.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |

## История принятия

SPEC-17 принят в architecture packet 1.7 как foundation contract authored project composition, deterministic resolution, configuration ownership и atomic activation. Принятие specification не создаёт runtime implementation, gate `PASS` или `vertical-v1` conformance.

## Назначение и invariants

SPEC-17 задаёт единственный engine-owned contract, который превращает authored project intent в точную composition для `game`, `headless`, `tools` и `capture-worker`.

- Active project composition MUST быть content-addressed и immutable на протяжении authoritative session.
- Required content, schemas, mechanics, scripts, plugins и physical/model references MUST пройти validation до world mutation.
- `game`, `headless` и `capture-worker` MUST использовать один `ProjectCompositionLock`; различаться могут только declared presentation adapters.
- User preferences MUST NOT менять authoritative configuration, content resolution, command validation, schedule или replay result.
- Filesystem paths, environment variables, OS handles, vendor configuration objects и downloader state MUST NOT входить в public contracts.
- Optional failure MUST иметь explicit bounded fallback. Required failure MUST завершать activation до частичного world state.

## Source of truth и ownership

| State | Единственный owner/source of truth | Реплики или не-authority |
|---|---|---|
| Authored project intent | Versioned `ProjectManifest` в project source | editor form, CLI flags, local cache |
| Resolution catalog input | Immutable `ProjectCatalogSnapshot` revision/hash | mutable registry, downloader, local package cache |
| Exact resolved composition | Asset & Tool Chain `ProjectCompositionLock` | package-manager cache, installer database |
| Active project revision | Core Runtime exact lock hash | process/window/session state, inspector views, crash capsule |
| Authoritative configuration | Resolved lock + `LaunchProfile` authoritative fields | environment, user preferences |
| Presentation preferences | Player Experience local profile | renderer/audio/UI cached settings |
| Developer-only instrumentation | Developer Experience launch overlay | telemetry/profiler adapters |
| Published schemas/content/partition | exact `SchemaRegistryManifestV1`, `ContentManifestV1`, resource-policy and `WorldPartitionManifestV1` hashes in the lock | staging/download/cache/residency views |

Repository Owner владеет schema и resolution semantics. Asset & Persistence Team владеет validation, atomic lock publication и compatibility with save/replay. Composition roots только потребляют validated result и не имеют собственных resolution rules.

## Public contracts

### ProjectManifest

`ProjectManifest` содержит:

| Field group | Contract |
|---|---|
| Identity | schema version, nominal project ID, project version, minimum/maximum engine contract range |
| Content | root `ContentManifestV1` and `WorldPartitionManifestV1` references, startup region/chunk, neutral locale-independent definition IDs |
| Dependencies | required/optional typed identities, SemVer ranges, exact prerelease admissions, declared fallback and compatibility constraints |
| Extensions | required/optional mechanic packages, Luau modules, Wasm plugins, exact capability requests |
| Physical/AI | physical archetype/policy/model-pack references только по `AssetId + ContentHash`; optional AI всегда имеет deterministic fallback |
| Profiles | allowed `LaunchProfile` IDs и default profile per composition root |
| Policy | configuration ceilings, network/telemetry defaults, package trust policy, budget policy and save compatibility range |
| Provenance | publisher, source revision, schema/tool hashes; no credentials или local absolute paths |

Manifest выражает authored intent, но не является runtime lock. Version range, floating dependency, logical asset reference или optional capability MUST быть разрешены до activation.

Canonical `SemVerRangeV1` не зависит от ecosystem-specific text parser. Он является non-empty OR-list comparator clauses; каждая clause является non-empty AND-set `Exact`, `GreaterThan`, `GreaterThanOrEqual`, `LessThan` or `LessThanOrEqual` над valid SemVer 2.0.0 values. Comparator/clauses canonical-sort and reject duplicates/contradictions before resolution. Range satisfaction использует SemVer precedence and ignores build metadata; optional `required_record_sha256` pins exact catalog record. Caret, tilde, wildcard or registry-specific range text MAY быть authoring projection, но MUST быть normalized в `SemVerRangeV1` before manifest hashing.

### ProjectCatalogSnapshot и deterministic resolver

`ProjectCatalogSnapshot` является immutable JCS-canonical input resolver. Он состоит из closed `body` и внешнего `catalog_snapshot_sha256 = SHA-256(JCS(body))`; hash field не входит в собственный preimage. Body содержит:

- catalog schema/version, resolver profile ID/version and provenance hash;
- bounded records с typed dependency identity, full SemVer, exact artifact/manifest/schema/content hashes, transitive requirements, target/engine compatibility, capabilities, trust/license/budget metadata, fallback metadata and `yanked` flag;
- `record_sha256 = SHA-256(JCS(record_body))`, где внешний hash field record не входит в `record_body`;
- canonical ordering records по `(dependency kind, canonical identity bytes, full SemVer text, record_sha256)`.

Duplicate canonical record key с различными bytes, self-hash mismatch, unknown field/schema или hash collision делает snapshot invalid до resolution. URL registry, source path, download time, cache presence, current network state and authentication material не входят в snapshot или resolution result.

Resolver является pure deterministic application service:

```text
(ProjectManifest, exact ProjectCatalogSnapshot,
 engine/schema/content/resource/partition/target/trust/budget/config profile hashes)
    → ProjectCompositionLock | ProjectResolutionConflictReport
```

Resolution MUST выполнять один contract:

1. Root и transitive requirements сортируются по canonical `(dependency kind, identity)` bytes. Dependency cycle rejected; source declaration order не является tie-break.
2. Candidate eligible только когда он удовлетворяет всем accumulated SemVer/engine/schema/target/capability/trust/budget constraints и `yanked = false`.
3. Prerelease candidate не допускается broad range автоматически. `ProjectManifest` либо transitive requirement MUST перечислить его exact full prerelease SemVer в closed `prerelease_admissions`; отсутствие exact value допускает только stable releases.
4. Eligible candidates ранжируются по SemVer 2.0.0 precedence descending. Build metadata не меняет precedence; при равной precedence побеждает lexicographically smallest 32-byte `record_sha256`.
5. Resolver выбирает canonical smallest unresolved identity, перебирает candidates в этом order и deterministically backtrack-ит до первой полной compatible closure. Optional fallback рассматривается только после исчерпания primary candidates и только в declared form; undeclared substitution запрещена.
6. Success публикует one lock. Если полной closure нет, resolver публикует one canonical `ProjectResolutionConflictReport`; partial lock/registry не публикуется.

`ProjectResolutionConflictReport` содержит exact manifest/catalog/profile hashes и canonical ordered conflict groups: dependency identity, every contributing source/range/requiredness, inspected candidate SemVer/record hash, stable rejection reason and declared fallback result. Rejection reason является closed enum `RangeMismatch`, `Yanked`, `PrereleaseNotAdmitted`, `RecordHashMismatch`, `EngineIncompatible`, `SchemaIncompatible`, `TargetIncompatible`, `CapabilityDenied`, `TrustDenied`, `BudgetExceeded`, `Cycle` or `FallbackUnavailable`. Groups сортируются по identity, constraints — по source identity/range bytes, candidates — по resolver ranking. Equal valid inputs MUST produce byte-identical lock; equal invalid inputs MUST produce byte-identical report and diagnostic.

Runtime roots MUST NOT читать registry, network/cache state, выбирать package version или повторно разрешать floating range. Они только валидируют exact lock closure/hashes and activate it. New catalog snapshot, un-yank/yank state, admitted prerelease or changed range always requires a new resolution and a new lock hash.

### ProjectCompositionLock

`ProjectCompositionLock` является immutable closure и содержит exact:

- project/manifest identity и canonical hash;
- catalog snapshot, resolver profile and canonical resolution-trace hashes;
- engine build/contract, schema registry and runtime-determinism profile hashes; the schema-registry entry resolves exact `SchemaRegistryManifestV1` and its compatibility/migration graph;
- ContentManifest/content closure, MechanicsLock/package dependency DAG and selected catalog-record hashes; the content closure resolves exact `ContentManifestV1`, neutral bundle/target variants, `WorldPartitionManifestV1` and resource/streaming-admission policies;
- Luau/Wasm artifact, WIT/API compatibility, `PackageTrustManifestV1`, trust-policy and effective-capability hashes;
- `GameplayBudgetMatrix`, extension/resource budget policy and configuration-schema hashes; the resource policy binds exact job/cancellation/memory/residency/I/O/backpressure profiles;
- canonical authoritative configuration and selected `LaunchProfile` authoritative-subset hashes;
- physical archetype, policy/model catalog and required state-schema hashes; this closure additionally binds physics numeric/snapshot, motor observation/action/state/safety and required animation-state schema hashes;
- platform-capability/timebase, `ApplicationSessionManifestV1` recovery/shutdown policy and allowed presentation-target hashes;
- ordered migration set, target-independent project data и target-specific `PresentationSnapshotV2`/material/shader-interface/SDR/HDR/VFX/cache profile references;
- resolved fallback for every optional dependency;
- lock schema version и canonical closure hash.

Runtime получает только lock values and resolved assets, никогда ranges, catalog resolver/package-manager handles или paths. Equal valid inputs MUST produce the same platform-neutral lock bytes on Windows/Linux. Target presentation payloads имеют отдельные hashes и не меняют domain subset. PresentationOnly/DeveloperOnly current values не входят в domain hash; lock фиксирует class/owner registry и authoritative subset, а launch/run manifests отдельно записывают enabled non-authoritative values.

### LaunchProfile и ConfigurationClass

`LaunchProfile` содержит composition root (`game`, `headless`, `tools`, `capture-worker`), target triple, declared capabilities, fixed tick/profile IDs, artifact roots и bounded operational options. Resolved root is bound into [SPEC-29](29-platform-host-and-application-session.md) `ApplicationSessionManifestV1`; `headless` permits presentation target only `None`, while `capture-worker` permits only `DisplaylessOffscreen`. Interactive adapter availability cannot alter the authoritative subset.

Каждый configuration key принадлежит ровно одному class:

| Class | Может влиять на | Persistence/replay rule |
|---|---|---|
| `Authoritative` | schedule, validation, RNG, domain rules, content/model selection | exact value/hash входит в lock, save и replay compatibility |
| `PresentationOnly` | UI, camera, audio, graphics, input bindings | MAY храниться local; не входит в gameplay hash |
| `DeveloperOnly` | diagnostics, probes, profiler, artifact output | не меняет access sets/control flow; conformance metadata records enabled state |

Precedence: engine defaults → locked package defaults → `ProjectManifest` → selected `LaunchProfile`. User profile может перекрывать только allowlisted `PresentationOnly` keys. Explicit CLI может перекрывать только `DeveloperOnly` либо presentation keys; изменение authoritative key создаёт новый lock до activation. Environment variables не являются configuration source of truth.

Unknown key, duplicate owner, class mismatch или incompatible profile MUST fail validation. Runtime hot change authoritative key запрещён; development rebuild создаёт new lock/session и invalidates replay compatibility.

## Resolution и activation lifecycle

Нормативный state machine:

`Discovered → Parsed → Resolved → Validated → Staged → Activated → Quiescing → Closed`, с `Rejected` terminal для конкретной manifest/lock revision.

Это lifecycle exact project composition, а не [SPEC-29](29-platform-host-and-application-session.md) application-session lifecycle, OS process, user login, PlatformHost window, renderer/audio device или interactive session. Process MAY существовать без active project и MAY последовательно activate новую revision только после `Closed` предыдущей. Focus, suspend/resume, device loss, window recreation или process-session reconnect не меняют project lifecycle stage, не запускают resolution и не заменяют lock. They are typed session/platform facts. OS termination request only initiates the declared session `Quiescing → Finalizing → Closed` path; abrupt termination restores from the last atomic project/save/session publication.

1. `Discovered/Parsed`: read bounded manifest bytes; syntax/schema failure не создаёт registry state.
2. `Resolved`: consume one exact catalog snapshot and produce one exact lock or canonical conflict report.
3. `Validated`: verify lock/catalog hashes, schema registry/compatibility/migration
   closure, content/bundle/variant/partition dependencies, resource profiles,
   capabilities, trust, budgets, provenance, licenses, target compatibility и
   save/replay constraints.
4. `Staged`: create private immutable schema/content/partition registries,
   bounded resource queues and composition-root adapters; no world state
   visible.
5. `Activated`: atomically publish exact lock hash and registries at pre-world boundary.
6. `Quiescing/Closed`: stop ingress, commit permitted save/artifacts, release adapters and retain crash/recovery metadata.

Failure before `Activated` discards staging. Failure after activation cannot silently replace lock; recovery either keeps exact active composition or closes the project revision cleanly. Tool output uses staging + atomic publish per SPEC-03/09. Starting/stopping a presentation or OS adapter is subordinate to the active lock and never constitutes project activation by itself.

Session recovery after a terminal required-save failure MAY create a new live
application session only through the SPEC-29 `RecoverySessionLinkV1`, using the
same exact `ProjectCompositionLock`, the exact prior application-session
manifest/state hashes and verified last-safe save generation. This is session
reconnection under the existing `Activated` project revision: it is not another project activation.
The superseded session remains immutable non-live history, and no different
project revision may stage or activate until the recovered live session shuts
down and this project lifecycle reaches `Closed`.

## Composition-root parity

- `game`, `headless` и `capture-worker` use identical schema registry, content
  catalog, resource/streaming policy, partition manifest, command validator,
  persistence/replay implementation, mechanics lock and authoritative
  configuration.
- `headless` omits PlatformHost/render/audio device adapters but not presentation-independent UI/action schemas required by scenario input.
- `capture-worker` adds exact CaptureJob presentation sink without re-resolving content or packages.
- `tools` MAY resolve/build a lock, but runtime roots cannot mutate the published lock.
- Compile-time feature differences cannot remove required domain schema or change command/event semantics.

## Persistence, replay и recovery

SaveManifest и ReplayManifest reference exact `ProjectCompositionLock` hash.
Load first validates project ID; lock/catalog/schema-registry/migration,
content/bundle/variant, resource/partition, package/trust/budget/config
compatibility; and declared unique copy-on-write migrations. Runtime does not
re-resolve authored ranges during load/replay. Best-effort substitution of
required schema/package/model/content/partition data is forbidden. Optional
downgrade is allowed only when manifest predeclares a deterministic
copy-on-write migration and new lock/save hash.

Crash capsule records lock hash, lifecycle stage, last completed tick and last atomic publication. It does not serialize arbitrary staging objects. Last valid save and published project content remain immutable during recovery.

## Stable diagnostics и failure semantics

| Code / failure | Required outcome |
|---|---|
| `PROJECT_MANIFEST_INVALID` | Reject before resolution; no registry/world mutation |
| `PROJECT_CATALOG_SNAPSHOT_INVALID` | Reject snapshot before resolution; no cache/registry guess or partial lock |
| `PROJECT_DEPENDENCY_UNRESOLVED` | Emit canonical conflict report and reject required dependency; optional dependency uses only declared fallback |
| `PROJECT_LOCK_MISMATCH` | Reject activation/load on lock/catalog/schema/content/resource/partition closure mismatch; preserve current lock/save |
| `PROJECT_CONFIG_CLASS_VIOLATION` | Reject override; authoritative value remains unchanged |
| `PROJECT_CAPABILITY_DENIED` | Disable optional component or reject required project before activation |
| `PROJECT_ACTIVATION_INCOMPLETE` | Discard staging; no partially published registry |
| `PROJECT_LIFECYCLE_FAULT` | Produce crash capsule and clean close/recovery path; never invent new lock |
| `NONDETERMINISTIC_RESULT` | Gate fails with first divergent resolution/input; retry cannot turn result green |

## Verification gates

| Gate | Owner | Reproducible command/scenario | Pass threshold | Required evidence | Fallback |
|---|---|---|---|---|---|
| `PROJECT-P1` | Repository Owner + Asset & Persistence | `next gate PROJECT-P1 --scenario project-resolution-v1 --targets windows-x86_64,linux-x86_64 --fixtures 1000` | 1,000 manifest/catalog/order permutations produce byte-identical platform-neutral locks or conflict reports; every selected record is highest compatible non-yanked SemVer, prerelease requires exact admission and equal precedence uses canonical hash tie-break; exact schema/content/resource/partition hashes close; 100% invalid cycles/ranges/hash/trust/budget conflicts rejected; 0 ambient path/env/network dependency | manifest/catalog/schema/content/resource/partition corpus, snapshot/record/lock hashes, resolution traces, conflict reports, diagnostics | reject manifest; pin exact prior lock |
| `PROJECT-P2` | Core Runtime + Security | `next gate PROJECT-P2 --scenario project-activation-faults --all-stages` | every injected stage failure yields 0 partial registries/world commands/source/save mutation; optional fallback exactly declared; required failure pre-world | lifecycle trace, registry snapshots, file-access/write audit, fault matrix | discard staging; retain prior published project |
| `LIFECYCLE-P1` | Runtime + Release Engineering | `next gate LIFECYCLE-P1 --scenario composition-root-parity --roots game,headless,capture-worker` | exact lock/schema/content/resource/partition/accepted-command/final gameplay hashes across roots; presentation subset differences declared only; 1,000 open/close cycles leak-free | RunManifests, lock/schema/content/admission/partition hashes, replay diff, resource report | block incompatible root/package |
| `CONFIG-P1` | Developer Experience + Verification & Evidence | `next gate CONFIG-P1 --scenario configuration-classes --permutations 10000` | 100% unknown/duplicate/class-invalid overrides rejected; presentation/developer permutations produce 0 gameplay hash differences; authoritative change always creates new lock | config corpus, lock diffs, replay hashes, diagnostic envelopes | ignore/reject invalid override; use locked value |

## Requirements

| ID | Требование | Primary owner | Contributors | Blocking gates |
|---|---|---|---|---|
| REQ-087 | Authored `ProjectManifest` MUST deterministically resolve against one exact `ProjectCatalogSnapshot` to one immutable content-addressed `ProjectCompositionLock` binding exact schema/content/resource/partition closure; runtime roots MUST NOT resolve floating ranges. | Repository Owner | Asset & Persistence Team, Security & Governance Team | PROJECT-P1 |
| REQ-088 | Every configuration key MUST have exactly one owner/class, and every authoritative value, trust policy or budget change MUST produce a new lock before activation. | Repository Owner | Developer Experience Team, Runtime Team | CONFIG-P1 |
| REQ-089 | `game`, `headless` and `capture-worker` MUST share the exact lock, schema/content/resource/partition registries, validators, persistence/replay path and domain semantics. | Runtime Team | Asset & Persistence Team, Verification & Evidence Team | LIFECYCLE-P1 |
| REQ-090 | Resolution publication, startup activation, save/load and recovery MUST be atomic, preserve the last valid lock/save and fail closed before partial world state. | Asset & Persistence Team | Runtime Team, Release Engineering | PROJECT-P2 |

## Failure paths

| ID | Trigger | Required result | Primary owner | Contributors | Blocking gates |
|---|---|---|---|---|---|
| FAIL-031 | Invalid catalog snapshot, unresolved/yanked/unadmitted prerelease dependency, incompatible required composition or invalid authoritative configuration | Emit the exact canonical conflict/validation diagnostic and reject before registry/world mutation; use only a declared optional fallback. | Repository Owner | Asset & Persistence Team, Security & Governance Team | PROJECT-P1, PROJECT-P2, CONFIG-P1 |
| FAIL-032 | Resolution publication, activation, quiesce, close or process-lifecycle fault | Discard private staging, preserve the prior published lock/content/save, emit bounded recovery metadata and never re-resolve or invent a replacement lock. | Runtime Team | Asset & Persistence Team, Release Engineering | PROJECT-P2, LIFECYCLE-P1 |
