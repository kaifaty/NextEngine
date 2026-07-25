# ADR-014: Deterministic extensions and package integrity

| Поле | Значение |
|---|---|
| ID | ADR-014 |
| Статус | Accepted |
| Версия | 1.2 |
| Дата решения | 2026-07-23 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | [ADR-006](006-scripting-and-plugin-model.md) |
| Заменён | частично [ADR-030](030-product-first-development-and-lightweight-validation.md) |

## Частичное supersession ADR-030

[ADR-030](030-product-first-development-and-lightweight-validation.md)
удаляет прежний формальный package-admission lifecycle. Content hashes,
capability isolation, instruction/fuel/allocation/memory/host-call limits,
atomic proposal discard, deterministic circuit breaker и отсутствие secrets
сохраняются. Engine не определяет package PKI, signer trust или revocation
lifecycle; distribution-specific wrapping остаётся за пределами runtime
contracts и не выдаёт capability.

## Контекст

Wall duration не является authoritative budget: один callback может завершиться
или прерваться в зависимости от host load. Package permissions определяются
только runtime policy и не выводятся из publisher metadata.

## Execution model

- Luau остаётся gameplay/content scripting layer; Wasm Component Model с
  versioned WIT остаётся isolated plugin ABI.
- Script, mechanic package и plugin получают capability-scoped immutable views
  и command/proposal sink. Прямой mutable ECS/world/physics access запрещён.
- Callback/VM transaction накапливает proposals отдельно. Trap, denial,
  authoritative quota overrun или host watchdog отбрасывает все uncommitted
  proposals; committed state не откатывается частично, host simulation не
  падает.
- VM/runtime version, package content hash, policy hash, capability set и
  authoritative quota config входят в save/replay compatibility metadata.

## Deterministic authoritative budgets

Единственными authoritative criteria являются integer counters из
`ExtensionBudgetPolicyV1`:

| Counter | Contract |
|---|---|
| Instructions / Wasm fuel | Exact maximum per callback и per gameplay tick; counter semantics закреплены VM/runtime compatibility manifest |
| Allocated bytes | Exact cumulative allocation quota per callback и exact live-memory ceiling per instance |
| Host calls | Exact per-call cost units из versioned registry; denied call не расходует mutation budget |
| Proposed command bytes/count | Exact limits до command admission |

Quota exhaustion создаёт stable diagnostic `EXTENSION_BUDGET_EXCEEDED` с
package/plugin ID, callback ID, counter kind, limit, observed count и tick. Одни
и те же accepted inputs, version и config дают одинаковый overrun tick,
rejection и strike независимо от CPU speed, worker count или load.

Wall watchdog остаётся только host-health guard. При срабатывании host:

1. aborts callback/VM instance;
2. discards uncommitted proposals;
3. emits `EXTENSION_WALL_WATCHDOG` с load profile;
4. не создаёт gameplay strike, event или deterministic outcome;
5. исключает этот run из deterministic replay/performance comparison.

Watchdog duration не входит в replay и не сравнивается как gameplay fact.
Нормативный test не использует sleep или retry-to-green.

## Circuit breaker

Violation ledger является authoritative state, входит в save/snapshot/replay и
индексируется по package/plugin principal. Default policy отключает issuer после
третьего authoritative violation в inclusive sliding window `1_800` gameplay
ticks. Violation — только deterministic quota exhaustion, trap, invalid
command/capability attempt или schema violation из versioned policy. Wall
watchdog не добавляет violation.

На tick нового violation runtime вычисляет
`window_start_tick = current_tick.saturating_sub(1_799)`, удаляет entries с
`entry_tick < window_start_tick`, добавляет current entry, затем сравнивает
count. На ticks `0…1_798` lower bound равен `0`; начиная с tick `1_799`
inclusive window содержит не более `1_800` gameplay ticks. Disabled issuer не
выполняется и получает `EXTENSION_CIRCUIT_OPEN`; reset возможен только declared
admin/load/migration command. Project override versioned, hash-bound и не
использует wall seconds.

## Package identity and capabilities

- SHA-256 `ContentHash` обязателен для каждого Luau package, mechanic/mod package
  и Wasm plugin, включая local development.
- Local package разрешён, когда exact content hash и compatibility metadata
  валидны, а project policy допускает его capabilities.
- Effective capabilities равны пересечению package request, API compatibility,
  project policy и hard security ceiling.
- Capability denial происходит до host call/mutation и даёт deterministic
  result.
- Engine-owned package manifest связывает package ID/type, content and manifest
  hashes, requested/effective capabilities и compatibility hashes.
- Distribution-specific identity/wrapping metadata не входит в runtime
  compatibility contract и не меняет load result или capability set.

## Product checks

| Сценарий | Ожидаемый результат | Fallback |
|---|---|---|
| Luau callback получает denied capability и пытается изменить immutable view | Denial происходит до host call; ни одна proposal не публикуется; stable diagnostic содержит package и capability IDs | Отключить optional callback/package, сохранив base-game path |
| Instruction, allocation, host-call и command quotas проверяются на `limit-1`, `limit`, `limit+1`, затем после save/load | Overrun tick, rejection, strike и violation ledger exact для одинаковых inputs | Уменьшить optional workload либо использовать более простой authored mechanic |
| Тот же command corpus запускается при разном worker load; отдельно injects wall watchdog | Integer-budget outcomes совпадают; watchdog не создаёт authoritative result и отбрасывает proposals | Restart isolated VM; disable offending optional extension |
| Wasm trap, memory/fuel exhaustion и WIT N/N-1 mismatch | Failure изолирован, staging остаётся atomic, несовместимая ABI отвергается до execution | Отключить plugin либо загрузить совместимую immutable version |
| Valid, tampered и malformed packages запрашивают одинаковые capabilities | Exact content hash, compatibility и project policy определяют load; publisher metadata не меняет capability ceiling | Reject tampered/malformed package; retain the prior exact package lock |

## Последствия

ADR-006 остаётся `Superseded`. Конкретные Luau и Wasm runtime являются
replaceable adapters за engine-owned contracts. Extension
failure снижает optional functionality, но не повреждает authoritative world
state и не обрушает host simulation.
