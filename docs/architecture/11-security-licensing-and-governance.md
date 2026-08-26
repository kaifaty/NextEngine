# SPEC-11: Runtime safety и license hygiene

| Поле | Значение |
|---|---|
| ID | SPEC-11 |
| Статус | Accepted |
| Версия | 2.7 |
| Последняя проверка | 2026-08-26 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-10](10-gothic-importer-boundary.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-083](adr/083-public-creator-project-cli-vertical.md), [ADR-084](adr/084-public-creator-run-and-project-package-vertical.md), [ADR-085](adr/085-public-creator-project-inspect-and-diff-vertical.md), [ADR-086](adr/086-public-creator-rpg-starter-template.md) |
| Дополнительные зависимости V2.5 | [ADR-087](adr/087-public-creator-runtime-scenario-and-prefix-minimization.md) |
| Дополнительные зависимости V2.6 | [ADR-088](adr/088-public-replay-first-divergence-and-domain-inspection.md) |
| Заменяет | SPEC-11 2.6; adds future remote generative-authoring privacy, spend, resume and rights guardrails without changing current network/runtime policy |

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

Public creator package принимает только отсутствующий destination и публикует
после полной проверки sibling staging directory. Manifest, inventory и каждый
file path/size/hash bounded и exact; symbolic links, traversal, unknown/changed
files, unsupported format, activation/run mismatch и missing/empty root
`NOTICE` отклоняют весь package. Проверка package никогда не выдаёт gameplay
capability и не доверяет записанному run proof без повторного production run.

Public creator template также принимает только отсутствующий destination под
существующим real parent. Он пишет фиксированный built-in CC0 file set в
private sibling staging, production-loads/cooks его до rename и не принимает
arbitrary local/remote templates, merge/overwrite или identity override.
Existing file/directory/symlink и publication race оставляют caller bytes
неизменными.

Public inspect/diff принимает только полностью validated authoring или package
operand. Package operand проходит тот же exact inventory/NOTICE/activation и
run-proof rerun до projection. Report может содержать stable IDs, schema facts,
exact hashes, provenance/license roots and granted capability IDs, but never raw
path, source span/property value, private generation layout or mutable runtime
object. Failure одного operand прекращает diff целиком; partial projection не
публикуется. Valid `different = true` является observation, а не input failure.

Public creator scenario reads one regular non-link file up to 1 MiB, probes its
current format before nested use and validates action/assertion counts, IDs,
ordering, hashes, tick budget and exact project identity before scenario world
creation. Minimize may publish only an absent regular file under an existing
real parent after the sibling-staged candidate decodes and reproduces the same
assertion failure identity. It cannot weaken an assertion, overwrite/link an
existing destination, inject an arbitrary command/fault or expose a raw path.

Public replay validation/inspection reads one regular non-link canonical file
up to 16 MiB, accepts only Replay V10 and one to 4,096 ticks, validates the full
owner/tick/compare-point closure and binds project/build/schema/content/
mechanics/tick compatibility before restore. Inspect completes production
replay before returning one tick/domain; descriptors may expose stable IDs,
versions, lengths and hashes but never canonical owner bytes, decoded private
snapshots, source/store paths or a partial projection after failure.

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
| Tools/agents | project-root allowlist, explicit operations, base-hash preconditions, atomic apply, path-free read-only projections, no generic ambient authority | stale/out-of-root operation rejected; invalid inspect/diff emits no partial view |

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

### Future generative-provider boundary

The `Proposed` SPEC-45/ADR-095 authoring path treats every prompt, reference,
remote response and generated output as untrusted developer input. Remote
submission is explicit opt-in and records the endpoint/adapter profile, classes
and exact hashes of content leaving the machine, declared retention/privacy
mode, positive billable-cost ceiling and attempt limit before transmission.
Prompt instructions are not security or spend enforcement.

Credentials, authorization headers, raw provider responses and opaque task
handles cannot enter source control, project/package content, default logs or
public reports. Protected Gothic-derived data, secrets, unknown-license input
and references without transmission/derivation rights cannot be submitted.

Every billable attempt has an idempotency key when the provider supports it.
After an ambiguous timeout without a safe status query, the tool records
`UnknownRemoteState` and forbids automatic resubmission. Downloaded output is
quarantined, hash-verified, bounded and scanned before structural validation.
Promotion additionally requires a positive rights/redistribution decision for
the complete reference/derivation chain; otherwise the candidate remains local
and excluded from package closure.

## License hygiene

Engine source распространяется под Apache-2.0. Для каждой shipped third-party
dependency или content/model asset должны быть известны exact version/hash,
source, license expression и redistribution status.

- Source и runtime dependency versions фиксируются lockfile; checksums и source
  registry проверяются воспроизводимо.
- Shipped packages включают требуемые license notices. SPDX/CycloneDX SBOM MAY
  генерироваться из lockfiles/package manifests, но не является отдельной
  архитектурной системой принятия решений.
- Current Creator Project Package V1 включает проверенный nonempty project
  `NOTICE`; отсутствие, link или несовпадение inventory исключает package до
  публикации. Это минимальный R6b content-distribution contract, не native
  shipping/SBOM claim.
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
Creator inspect/diff дополнительно проверяет tampered package, path redaction,
location-independent deterministic projection and all-or-none operand failure.
Creator scenario additionally checks retired/malformed/link input, exact
project mismatch, source/package proof parity, non-reproduced failure and
occupied output with no partial publication.
Новая public parser, WIT import, capability, IPC method или network endpoint
должна добавлять хотя бы один positive и один failure-path scenario. Эти
проверки дают локальную инженерную обратную связь и не создают отдельный
release status.
