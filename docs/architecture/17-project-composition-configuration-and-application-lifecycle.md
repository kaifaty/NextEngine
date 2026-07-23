# SPEC-17: Project composition, configuration и application lifecycle

| Поле | Значение |
|---|---|
| ID | SPEC-17 |
| Статус | Proposed |
| Версия | 0.1.1 |
| Владелец | Repository Owner |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Asset & Persistence Team, Developer Experience Team, Security & Governance Team, Release Engineering |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md), [ADR-006](adr/006-scripting-and-plugin-model.md), [ADR-007](adr/007-identities-persistence-and-replay.md), [ADR-010](adr/010-artifact-first-headless-validation-and-review.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md) |
| Заменяет | отсутствует |

## Статус предложения

Этот документ входит в отдельный post-1.5 foundation-completeness proposal track. Он не изменяет Accepted packet 1.4, remediation candidate 1.5 или отдельный dialogue/model proposal SPEC-16/ADR-017. Нормативные слова задают future acceptance contract только после атомарного promotion всего foundation track. До этого lifecycle state — `AwaitingReview`.

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
| Exact resolved composition | Asset & Tool Chain `ProjectCompositionLock` | package-manager cache, installer database |
| Active session composition | Core Runtime exact lock hash | inspector views, crash capsule |
| Authoritative configuration | Resolved lock + `LaunchProfile` authoritative fields | environment, user preferences |
| Presentation preferences | Player Experience local profile | renderer/audio/UI cached settings |
| Developer-only instrumentation | Developer Experience launch overlay | telemetry/profiler adapters |
| Published bundles и schemas | SPEC-03 content registry | staging/download paths |

Repository Owner владеет schema и resolution semantics. Asset & Persistence Team владеет validation, atomic lock publication и compatibility with save/replay. Composition roots только потребляют validated result и не имеют собственных resolution rules.

## Public contracts

### ProjectManifest

`ProjectManifest` содержит:

| Field group | Contract |
|---|---|
| Identity | schema version, nominal project ID, project version, minimum/maximum engine contract range |
| Content | root `ContentManifest` references, startup world/chunk, neutral locale-independent definition IDs |
| Extensions | required/optional mechanic packages, Luau modules, Wasm plugins, exact capability requests |
| Physical/AI | physical archetype/policy/model-pack references только по `AssetId + ContentHash`; optional AI всегда имеет deterministic fallback |
| Profiles | allowed `LaunchProfile` IDs и default profile per composition root |
| Policy | configuration ceilings, network/telemetry defaults, package trust policy, save compatibility range |
| Provenance | publisher, source revision, schema/tool hashes; no credentials или local absolute paths |

Manifest выражает authored intent, но не является runtime lock. Version range, floating dependency, logical asset reference или optional capability MUST быть разрешены до activation.

### ProjectCompositionLock

`ProjectCompositionLock` является immutable closure и содержит exact:

- project/manifest identity и canonical hash;
- engine/schema registry/build compatibility;
- ContentManifest, MechanicsLock и package dependency DAG hashes;
- Luau/Wasm artifact hashes, WIT/API ranges и granted capability ceiling;
- physical archetype, policy/model catalog и required state schema hashes;
- ordered migration set, target-independent project data и target-specific presentation payload references;
- resolved fallback for every optional dependency;
- lock schema version и canonical closure hash.

Runtime получает только lock values and resolved assets, никогда package-manager handles или paths. Equal valid inputs MUST produce the same platform-neutral lock bytes on Windows/Linux. Target presentation payloads имеют отдельные hashes и не меняют domain subset.

### LaunchProfile и ConfigurationClass

`LaunchProfile` содержит composition root (`game`, `headless`, `tools`, `capture-worker`), target triple, declared capabilities, fixed tick/profile IDs, artifact roots и bounded operational options.

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

1. `Discovered/Parsed`: read bounded manifest bytes; syntax/schema failure не создаёт registry state.
2. `Resolved`: resolve exact versions/hashes and fallback closure with deterministic tie-breaks.
3. `Validated`: verify schemas, dependencies, capabilities, provenance, licenses, target compatibility и save/replay constraints.
4. `Staged`: create private immutable registries and composition-root adapters; no world state visible.
5. `Activated`: atomically publish exact lock hash and registries at pre-world boundary.
6. `Quiescing/Closed`: stop ingress, commit permitted save/artifacts, release adapters and retain crash/recovery metadata.

Failure before `Activated` discards staging. Failure after activation cannot silently replace lock; recovery either keeps exact active composition or closes session cleanly. Tool output uses staging + atomic publish per SPEC-03/09.

## Composition-root parity

- `game`, `headless` и `capture-worker` use identical schema registry, command validator, persistence/replay implementation, mechanics lock and authoritative configuration.
- `headless` omits PlatformHost/render/audio device adapters but not presentation-independent UI/action schemas required by scenario input.
- `capture-worker` adds exact CaptureJob presentation sink without re-resolving content or packages.
- `tools` MAY resolve/build a lock, but runtime roots cannot mutate the published lock.
- Compile-time feature differences cannot remove required domain schema or change command/event semantics.

## Persistence, replay и recovery

SaveManifest и ReplayManifest reference exact `ProjectCompositionLock` hash. Load first validates project ID, lock/schema/content compatibility and declared migrations. Best-effort substitution of required package/model/content is forbidden. Optional downgrade is allowed only when manifest predeclares a deterministic copy-on-write migration and new lock/save hash.

Crash capsule records lock hash, lifecycle stage, last completed tick and last atomic publication. It does not serialize arbitrary staging objects. Last valid save and published project content remain immutable during recovery.

## Stable diagnostics и failure semantics

| Code / failure | Required outcome |
|---|---|
| `PROJECT_MANIFEST_INVALID` | Reject before resolution; no registry/world mutation |
| `PROJECT_DEPENDENCY_UNRESOLVED` | Reject required dependency; optional dependency uses only declared fallback |
| `PROJECT_LOCK_MISMATCH` | Reject activation/load; preserve current lock/save |
| `PROJECT_CONFIG_CLASS_VIOLATION` | Reject override; authoritative value remains unchanged |
| `PROJECT_CAPABILITY_DENIED` | Disable optional component or reject required project before activation |
| `PROJECT_ACTIVATION_INCOMPLETE` | Discard staging; no partially published registry |
| `PROJECT_LIFECYCLE_FAULT` | Produce crash capsule and clean close/recovery path; never invent new lock |
| `NONDETERMINISTIC_RESULT` | Gate fails with first divergent resolution/input; retry cannot turn result green |

## Proposed gates

| Gate | Owner | Reproducible command/scenario | Pass threshold | Required evidence | Fallback |
|---|---|---|---|---|---|
| `PROJECT-P1` | Repository Owner + Asset & Persistence | `next gate PROJECT-P1 --scenario project-resolution-v1 --targets windows-x86_64,linux-x86_64 --fixtures 1000` | 1,000 valid/permuted manifests produce byte-identical platform-neutral locks; 100% invalid cycles/ranges/hash conflicts rejected; 0 ambient path/env dependency | manifest corpus, locks/hashes, resolution traces, diagnostics | reject manifest; pin exact prior lock |
| `PROJECT-P2` | Core Runtime + Security | `next gate PROJECT-P2 --scenario project-activation-faults --all-stages` | every injected stage failure yields 0 partial registries/world commands/source/save mutation; optional fallback exactly declared; required failure pre-world | lifecycle trace, registry snapshots, file-access/write audit, fault matrix | discard staging; retain prior published project |
| `LIFECYCLE-P1` | Runtime + Release Engineering | `next gate LIFECYCLE-P1 --scenario composition-root-parity --roots game,headless,capture-worker` | exact lock/schema/accepted-command/final gameplay hashes across roots; presentation subset differences declared only; 1,000 open/close cycles leak-free | RunManifests, lock/schema hashes, replay diff, resource report | block incompatible root/package |
| `CONFIG-P1` | Developer Experience + Verification & Evidence | `next gate CONFIG-P1 --scenario configuration-classes --permutations 10000` | 100% unknown/duplicate/class-invalid overrides rejected; presentation/developer permutations produce 0 gameplay hash differences; authoritative change always creates new lock | config corpus, lock diffs, replay hashes, diagnostic envelopes | ignore/reject invalid override; use locked value |

## Foundation requirement aliases

| Alias | Primary owner | Future acceptance contract |
|---|---|---|
| `FND-PROJECT-R1` | Repository Owner | ProjectManifest resolves to one exact content-addressed ProjectCompositionLock |
| `FND-PROJECT-R2` | Repository Owner | Configuration keys have one class/owner and authoritative changes require a new lock |
| `FND-PROJECT-R3` | Runtime Team | Game/headless/capture-worker share lock, validators and domain semantics |
| `FND-PROJECT-R4` | Asset & Persistence | Startup, save/load and recovery are atomic and fail closed |
| `FND-PROJECT-F1` | Repository Owner | Invalid/incompatible required composition rejects before world mutation |
| `FND-PROJECT-F2` | Runtime Team | Activation/lifecycle fault discards staging and preserves prior published state |

## Promotion contract

Promotion occurs only with SPEC-18…20 and ADR-018…021 in one reviewed transaction after all required approvals. It updates SPEC-00/01/02/03/07/09/11/12/15, glossary and traceability; maps PROJECT/LIFECYCLE/CONFIG gates into VS-01/02/08/11/12; and preserves exactly fifteen VS gates.

Global row allocation is conditional and deterministic: if SPEC-16/ADR-017 is Accepted first, foundation aliases use REQ-087…102 and FAIL-031…038; if that track is formally Rejected/withdrawn, they use REQ-079…094 and FAIL-025…032. While SPEC-16 remains `AwaitingReview`, this track may be reviewed but cannot be promoted. No external technology row is added.
