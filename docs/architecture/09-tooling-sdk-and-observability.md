# SPEC-09: Tooling, SDK и observability

| Поле | Значение |
|---|---|
| ID | SPEC-09 |
| Статус | Accepted |
| Версия | 1.5 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-23 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-015](adr/015-evidence-trust-fixture-separation-and-attestation.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

Subsystem owners владеют semantics и emit structured diagnostics/metrics. Developer Experience Team владеет CLI grammar, diagnostic envelope, RunManifest/artifact layout, inspectors, profiling integration и local/future-CI conformance orchestration. Verification & Evidence Team владеет scenario/impact/evidence/review application services; authorized human reviewer владеет только decision. Observability/evidence storage не является gameplay source и отключается без изменения simulation outcome.

## Public boundary и data flow

Public tool boundary — versioned CLI arguments/exit codes/JSON, AuthoringContextBundle/AgentChangeSet, VerificationPolicy/TestScenario/ChangeImpact/CaptureJob/Evidence/HumanReview schemas, physical/policy manifests и certification reports, diagnostic/telemetry envelopes, RunManifest и documented artifact schemas. Optional MCP adapter является projection того же application service, не вторым source. Profiler SDK, CI/artifact-store/encoder/training-backend API, terminal/MCP library, OS crash API и internal Rust types не экспортируются. Поток `subsystem structured signal → bounded local collector → RunManifest/artifact/inspector/evidence` read-only относительно simulation; mutating development endpoint требует отдельной capability и делает run non-conforming.

## Обязательные CLI v1

Единый binary MAY называться `next` до product naming ADR. Exit codes: `0` success/pass, `2` validation/input failure, `3` incompatibility/unsupported capability, `4` conformance gate fail, `5` internal/tool crash. Human text локализуем; `--format json` schema стабилен в major CLI version.

| Команда | Обязательный контракт |
|---|---|
| `next cook --project <manifest> --target <profile> --out <dir>` | validate → deterministic cook → atomic publish; cache stats и manifest hash |
| `next validate assets|project|bundle|save|plugin|model <path>` | read-only validation, stable diagnostic codes/source locations |
| `next inspect import <neutral-manifest>` | provenance, namespace mappings, unsupported/dropped records; не читает proprietary source в engine repo |
| `next replay run <replay> [--headless] [--compare <oracle>]` | first divergent tick/owner/schema; nonzero при mismatch |
| `next inspect physical <run|live-endpoint>` | pose/joints/contacts/COM/support/LOD/motor/action safety и media export |
| `next inspect ai <run|live-endpoint>` | perception/intents/rejections/plans/memory provenance/ai-host deadlines без hidden mutable access |
| `next inspect mechanics <run|live-endpoint>` | package lock, ability phases, effects/statuses, reducer proposals/rejections, hook order и state revisions |
| `next physical new\|describe\|validate\|pack <archetype>` | public physical-archetype scaffold, closed-schema description, validation и immutable package build |
| `next policy inspect\|verify\|route-test\|transition-test\|certify <policy-or-project>` | compatibility/parity, deterministic resolver, switch supervisor и certification gates SPEC-14 |
| `lab generate-env\|train\|evaluate\|compare\|export-onnx\|certify <config>` | replaceable offline training application service; algorithm/backend не входит в engine ABI |
| `next test impact --changeset <manifest>` | deterministic affected suite/category/capture/review resolution + explanations |
| `next test run --impact <manifest> --profile <profile> --artifact-root <dir>` | executes exact ChangeImpactManifest; produces RunManifests/replays/diagnostics |
| `next scenario validate\|run\|minimize <input>` | generic scenario validation/run и same-failure replay minimization |
| `next test explain --run <manifest> --format json` | first divergence, owner, code, assertion/causal typed diff |
| `next capture render --job <manifest> --artifact-root <dir>` | short-lived displayless render/audio capture worker |
| `next evidence build\|verify\|diff <input>` | content-addressed evidence assembly/validation/comparison |
| `next review bundle <evidence> --out <dir>` | self-contained offline dossier rooted at `index.html`, no external resources |
| `next review record <bundle> --decision <value> --attest <key-id>` | human-only canonical `HumanReviewDecisionV1` через isolated signer adapter; CLI принимает key ID, но никогда raw key/path/seed; automatic failures cannot be approved |
| `next gate <gate-id|vertical-v1> --artifact-root <dir>` | executes exact pinned scenario, writes RunManifest и pass/fail |
| `next package --target windows-x86_64|linux-x86_64` | validates licenses/SBOM/content and creates reproducible package manifest |

Mechanic/mod/agent CLI family (`next sdk`, `next mod`, `next mechanic`, `next changeset`, `next agent serve`) определяется SPEC-13 и MUST использовать те же exit-code/JSON/diagnostic contracts.

Physical `AuthoringContextBundle` scope MUST включать exact body/morphology/topology/actuator schemas, skill catalog, policy compatibility keys, normalization, training config schemas, evaluation/transition suites, certification rules, provenance requirements и structured diagnostics. Model promotion, license/provenance change и certification decision всегда создают review-required AgentChangeSet; отсутствие training backend не мешает собрать `PrototypeFallback` package.

Tools MUST support `--help`, `--version`, `--format json`, explicit output path и non-interactive CI. Они MUST NOT upload telemetry/artifacts unless user supplies explicit endpoint/consent.

## Local bootstrap orchestration

До появления remote/CI repository admission выполняется локальными project-owned командами:

```text
cargo run -p xtask -- docs-check
cargo run -p xtask -- boundary-scan
cargo run -p xtask -- host-check
uv run --project lab python -m next_lab doctor
uv run --project lab python -m next_lab smoke --device auto
```

`host-check` MUST объединять format check, clippy, workspace tests, docs validator и boundary scan; Mac result имеет `DeveloperHostTier`, а не shipping status. Lab commands являются isolated development application service: Mac smoke MAY выбрать MPS или declared CPU fallback, но не закрывает `TRAIN-P1`, correspondence или certification. Local RunManifest фиксирует exact toolchain/device/fallback. Ни одна команда не требует remote, cloud account или CI token.

## Diagnostic envelope

Каждая diagnostic запись содержит schema version, stable code, severity, subsystem/owner, message key + rendered message, process/build, optional tick/entity PersistentId/AssetId/source span, first divergent tick/event/schema/assertion, typed expected/actual, causal command/event chain, remediation key, replay/minimization reference и redaction classification. Raw vendor error MAY прикладываться как debug field, но stable code остаётся engine-owned.

Severity: `info`, `warning`, `error` (operation fails), `fatal` (process/world unsafe). `warning` не может скрывать нарушение MUST gate.

## Telemetry envelope

Metrics/traces используют monotonic timestamp + simulation tick when applicable, subsystem, scenario/run ID, build/config hashes, typed attributes и units. PersistentId MAY логироваться в local artifacts; user paths, dialogue text, prompts, voices и imported asset bytes default-redacted. Sampling не меняет control flow и не блокирует fixed tick.

## RunManifest и artifact layout

```text
run-root/
  run-manifest.json
  impact/
  replay/
  metrics/
  traces/
  logs/
  reports/
  media/
  crash/
```

RunManifest MUST быть JCS-canonical и содержать schema version; run/scenario/suite/profile ID; VerificationPolicy/ChangeImpact/CaptureJob hashes when applicable; UTC creation time только как metadata; engine/git/build/toolchain/platform/hardware; project/content/save/replay/model/plugin/backend hashes; physical archetype, body revision, proficiency band, resolved route, policy state schema и certification hashes when applicable; seeds/tick rates; exact command; configs; automatic gate thresholds/results; first-failure/minimized-replay status; artifact list с relative path, media type, bytes, SHA-256 и role; redaction/fixture classification; parent/base/candidate/baseline run IDs.

Для HumanReviewRequired media manifest MUST фиксировать scenario/CapturePlan identity, base/candidate, camera, semantic tick window, render/audio settings, playback speed/FPS, overlays, raw frame/audio root, encoder/decoder manifests и reason выбора every worst/failure episode. Physical specialization дополнительно фиксирует policy compatibility/route/transition. Обязательные roles определены SPEC-15 и SPEC-05. Media generation failure записывается как AwaitingCapability/fail согласно required role; artifact нельзя молча исключить.

## Inspectors и live access

Inspectors по умолчанию читают saved RunManifest/artifacts. Verification inspector показывает impact reasons, required/actual gates, first divergence, causal IDs, baseline/media/review status и missing capability без mutation. Physical inspector показывает deterministic resolver inputs, `MotorCapabilityView`, supervisor phase, previous/candidate route, shadow outputs, policy state/reset reason и proficiency event без mutable control. Live endpoint доступен только development build, loopback/local authenticated channel, read-only snapshots; mutating debug command требует отдельной unsafe-development capability и делает run non-conforming. Inspector schema versioned и не раскрывает vendor handles. Coding agents получают bounded saved views/context bundles; live access/interactive runtime не является условием полного authoring workflow.

## Crash isolation

Каждый process пишет bounded crash capsule: build/config/content hashes, last completed tick, recent command/event IDs, owned subsystem health, redacted stack/minidump reference и replay checkpoint pointer. Crash handler не пытается сериализовать arbitrary corrupted world. `ai-host`, importer и tools crash не должны уронить `game`/повредить published output.

## Profiling hooks

CPU spans, task/schedule stages, allocator counters, GPU timestamps, physics/motor timings, streaming I/O, script/plugin budgets и ai-host latency MUST иметь stable category names. Profiling off/on MUST давать одинаковые accepted command/state hashes. External profilers MAY подключаться через platform-specific adapters.

## SDK stability

V1 public SDK ограничен documented CLI JSON schemas, scenario/impact/capture/evidence/review, mechanics/package/changeset/context schemas, cooked/save/replay manifests, Luau capability API, WIT worlds, `ai-host` IPC и neutral asset/import schemas. Rust internal crates, CI scheduler/artifact store, encoder implementation, MCP implementation library и live inspector protocol имеют `unstable` marker до отдельного ADR. SemVer applies to published schemas/packages; breaking major requires migration/compatibility note.

## Future CI design (не является bootstrap dependency)

Bootstrap repository MUST NOT требовать `.github/workflows/`, CI vendor или remote. Будущий monorepo CI MUST проецировать те же local commands/application services и иметь stages: format/lint/license → impact resolution → agent-fast schema/unit/scenario → changeset replay/fault → displayless capture/media → evidence validation/human review when required → backend/long/security matrices → vertical slice → package/SBOM. Windows/Linux artifacts объединяются только после одинакового schema/content check. GPU/long gates MAY идти на tagged runners; отсутствие capability создаёт AwaitingCapability и никогда не считается pass.

## Failure semantics

- Tool internal crash → exit 5, crash capsule, no atomic publish.
- Artifact disk/full/hash failure → gate fails; prior artifact remains.
- Unknown CLI/schema major → exit 3 before mutation.
- Observability sink unavailable → local bounded buffer/drop counter; gameplay continues, но required gate с incomplete evidence fails.
- Inspector corrupt input → read-only error, no repair unless explicit separate command.
- MCP adapter unavailable/incompatible → complete CLI/JSON path; MCP error не меняет project/runtime state.
- Stale/unsafe AgentChangeSet → reject before filesystem mutation, emit violated precondition/path/capability.
- Incomplete/inconsistent physical certification claim → exit 4, package remains `PrototypeFallback` or previous certified revision remains published.
- Training backend absent/incompatible → diagnostic + engine-owned headless-lab fallback; validation, route tests и prototype packaging remain available.
- Impact resolver unknown/failed → maximal affected suite + HumanReviewRequired; no author-selected fallback.
- GPU/encoder/reviewer capability unavailable → structured AwaitingCapability; completed CPU evidence preserved, changeset not admitted.
- Evidence/review hash, trust-manifest/revocation snapshot, role/scope или attestation mismatch → reject before project mutation; old decision cannot be reused.
- Deterministic retry divergence → gate failure `NONDETERMINISTIC_RESULT`; CI retry is diagnostic only.
- MPS unavailable/unsupported operation → stable capability diagnostic + CPU smoke fallback; RunManifest не утверждает MPS pass.
- Local RTX profile unavailable → `TRAIN-RTX-01=AwaitingCapability`; prototype validation доступна, certification promotion blocked.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback |
|---|---|---|---|---|
| TOOL-01 | golden CLI success/error fixtures Win/Linux | exact stable codes/JSON schema; 0 partial publish | CLI report | release block |
| TOOL-02 | inspect every v1 artifact/schema N/N-1 | 100% supported readable; incompatible clearly rejected | compatibility report | ship matching standalone inspector |
| OBS-01 | profiling/telemetry off vs max | exact gameplay hashes; runtime overhead p95 <3% CPU excluding media capture | benchmark/replay | reduce sampling/hooks |
| OBS-02 | crash injection processes/write points | crash capsule produced ≥99%; published/source files never partial | fault matrix | harden atomic boundary |
| MANIFEST-01 | validate all gate runs | 100% required fields/artifacts/hashes exist and verify | manifest validator report | gate fails |
| PRIVACY-01 | sensitive fixture scan | 0 raw user paths/secrets/prompts/voice/imported bytes in default artifacts | scanner report | redact and regenerate |
| PRIVACY-02 | import-smoke cleanup + neutral evidence closure | 0 protected bytes in repository/cache/build/package/final/evidence/media/log roots; mixed fixture class rejected | cleanup/source-access/privacy reports | quarantine; no publish/review |
| TOOL-03 | CLI/JSON application service projection parity | mechanics/context/changeset outputs identical across direct CLI and adapters for all golden fixtures | schema/result diff | disable divergent adapter |
| TOOL-04 | cold physical author workflow | public context + CLI create/validate/pack neutral prototype and prepare a certification changeset; all references close; 0 private crate/backend knowledge required | context closure, CLI transcript, AgentChangeSet, manifests | fix context/CLI; retain PrototypeFallback |
| TOOL-05 | scenario/impact/evidence/review CLI projection parity | direct services и CLI/JSON produce exact TestScenario/ChangeImpact/CaptureJob/Evidence/HumanReview schemas/results for golden corpus; no interactive/runtime-only step | schema/result diff, command transcript | disable divergent adapter; fix CLI before admission |
