# SPEC-11: Runtime safety и license hygiene

| Поле | Значение |
|---|---|
| ID | SPEC-11 |
| Статус | Accepted |
| Версия | 2.1 |
| Последняя проверка | 2026-08-18 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-10](10-gothic-importer-boundary.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-083](adr/083-public-creator-project-cli-vertical.md) |
| Заменяет | SPEC-11 2.0; makes resolved project-root and creator output confinement explicit for the first public creator commands |

## Назначение

Этот документ задаёт минимальные технические правила, которые не дают
невалидному или враждебному вводу повредить authoritative state, выйти из
выданных возможностей либо случайно включить в проект чужие данные и секреты.
Он ограничен technical safety и license metadata и не определяет процесс
принятия изменений.

## Обязательные invariants

- Saves, replays, packages, scripts, plugins, models, importer output, IPC
  messages, project manifests и cooked content считаются untrusted.
- Ни один untrusted input не изменяет мир, project lock или active content до
  полной проверки schema, version, bounds, references и integrity.
- Authoritative gameplay mutation проходит только через production
  `WorldCommand` validation и атомарный commit.
- Luau, Wasm, importer/parser и optional `ai-host` не получают ambient
  filesystem, network, process или mutable-world authority.
- Offline game remains complete. Network and telemetry are opt-in and never
  determine simulation correctness.
- Secrets, credentials, private keys, game installations, imported/protected
  assets, datasets, checkpoints и generated model outputs не коммитятся в
  engine repository.

## Проверка untrusted inputs

Каждый public decoder/loader MUST до выделения крупных buffers и до mutation:

1. распознать точную schema и поддержанную version;
2. проверить byte length, element count, nesting depth, archive expansion
   ratio, string length, tensor shape и другие domain bounds;
3. проверить required SHA-256/checksum и exact referenced revisions;
4. отклонить duplicate IDs, missing references, запрещённые cycles и
   path traversal;
5. проверить enum values, numeric ranges, units и finite floating-point
   values;
6. построить полную staging generation и публиковать её только атомарно;
7. вернуть stable diagnostic code без частичного результата.

Для project-root allowlist проверка `path traversal` относится и к resolved
target: безопасно выглядящий relative path через symbolic link не может выйти
за выбранный project root. Public cook также не принимает symbolic-link output
или произвольный nonempty directory как content-store root.

Unknown version, hash mismatch, unsupported capability, malformed payload или
resource-limit violation являются typed failure, а не warning с попыткой
угадать данные. Предыдущая валидная generation остаётся активной.

Public contracts используют только engine-owned IDs и schemas. Native handles,
backend objects, parser internals, ECS storage и vendor types не пересекают
public boundary.

## Sandbox, capabilities и budgets

Default capability — deny. Эффективные права package/script/plugin равны
пересечению запрошенных прав, project policy и host allowlist.

| Boundary | Минимальные controls | Безопасный исход |
|---|---|---|
| Luau | isolated environment, allowlisted libraries, instruction/allocation/host-call budgets, no native/fs/network APIs | callback aborted; proposal and staged mutation discarded |
| Wasm plugin | versioned WIT surface, capability checks on every host call, fuel, memory/table/instance limits | instance stopped or disabled; authoritative state unchanged |
| Importer/parser | separate process, bounded reads/archives, normalized neutral output, no runtime linkage | input quarantined or rejected; no partial bundle |
| `ai-host` | bounded schema/size/deadline, immutable context, no mutable tools, deterministic in-process fallback | response rejected; gameplay continues through fallback |
| Model runtime | exact model/schema/hash compatibility, bounded tensors/operators, finite output check, common safety clamp | model route rejected; deterministic procedural fallback or safe stop |
| Tools/agents | project-root allowlist, explicit operations, base-hash preconditions, atomic apply, no generic ambient authority | stale/out-of-root operation rejected |

Fuel, instruction counts и deterministic logical deadlines MAY влиять на
authoritative outcome только через заранее объявленный deterministic fallback.
Wall-clock watchdog завершает зависший worker/process, но не выбирает gameplay
result и не превращает failed run в successful retry.

## Secrets и protected data

- Credentials, tokens, private keys и local machine paths не записываются в
  source, fixtures, crash reports или default logs.
- Gothic installations и их derived/imported bytes остаются во временном
  isolated importer root. Parent repository, engine packages, ordinary test
  fixtures и distributable artifacts содержат только independently licensed
  neutral content.
- Logs и diagnostics MUST не включать secret values или protected source
  bytes. Для воспроизведения используются stable IDs, hashes и bounded metadata.
- Обнаруженный secret/protected asset приводит к остановке текущей операции,
  очистке или quarantine generated output и повторной проверке repository
  boundaries.

## Network и local privacy

Default runtime profile выполняет zero outbound connections. Optional AI,
telemetry, voice или remote tools включаются явной настройкой, получают
минимальный endpoint scope и имеют локальный fallback. Их absence, timeout или
protocol mismatch не блокируют simulation tick и не меняют обязательный
gameplay outcome.

## License hygiene

Engine source распространяется под Apache-2.0. Для каждой shipped third-party
dependency или content/model asset должны быть известны exact version/hash,
source, license expression и redistribution status.

- Source и runtime dependency versions фиксируются lockfile; checksums и source
  registry проверяются воспроизводимо.
- Shipped packages включают требуемые license notices. SPDX/CycloneDX SBOM MAY
  генерироваться из lockfiles/package manifests, но не является отдельной
  архитектурной системой принятия решений.
- Code, content, fonts, datasets и model weights классифицируются раздельно;
  code license не распространяется автоматически на data or weights.
- Unknown, non-commercial, field-of-use, source-available или
  redistribution-incompatible artifact не включается в distributable package.
- Raw datasets, intermediate checkpoints и training runs не входят в engine
  repository или shipping package.
- Gothic importer работает только с user-provided local installation и не
  добавляет protected bytes либо legacy runtime types в engine distribution.

## Failure semantics

| Failure | Required behavior |
|---|---|
| Unsupported schema/version or malformed bounds | reject before allocation/mutation; stable diagnostic |
| Hash/reference/provenance mismatch | reject complete object graph; retain prior generation |
| Script/plugin budget or capability violation | abort callback/instance; discard staged proposals |
| Parser/archive/resource overflow | stop isolated worker; publish nothing partial |
| Invalid/non-finite model output | reject action/state pair; use declared safe fallback |
| Secret/protected asset found | stop operation; quarantine/clean generated output |
| Unknown or incompatible shipped license | exclude artifact from package |
| Optional network/AI service unavailable | remain offline and use deterministic local fallback |

## Проверка в продуктовой разработке

Негативные fixtures для malformed inputs, capability denial, budget exhaustion,
protected-data boundaries и incompatible licenses входят в соответствующие
`fast` или `content-package` ProductCheck из [SPEC-12](12-vertical-slice-conformance.md).
Новая public parser, WIT import, capability, IPC method или network endpoint
должна добавлять хотя бы один positive и один failure-path scenario. Эти
проверки дают локальную инженерную обратную связь и не создают отдельный
release status.
