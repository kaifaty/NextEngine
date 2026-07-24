# ADR-014: Deterministic extensions и package trust

| Поле | Значение |
|---|---|
| ID | ADR-014 |
| Статус | Accepted |
| Версия | 1.1 |
| Владелец | RPG Framework Team + Security & Governance Team |
| Требуемые согласующие | Architecture Working Group, RPG Framework Team, Gameplay Extensibility Team, Security & Governance Team |
| Дата решения | 2026-07-23 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md) |
| Заменяет | [ADR-006](006-scripting-and-plugin-model.md) |
| Заменён | не заменён |

## История принятия

ADR принят атомарно в packet 1.5 и заменяет ADR-006. Он сохраняет разделение Luau/Wasm, capability-scoped immutable views и validated command sink, делая overrun и trust outcomes воспроизводимыми. Принятие contract не объявляет SCRIPT/PLUGIN/MOD gates пройденными.

## Контекст

Wall duration не является authoritative budget: одинаковый callback может пройти или получить strike в зависимости от host load. Требование подписи только для plugins также оставляет package trust двусмысленным, а signature без policy ошибочно воспринимается как capability grant.

## Execution model

- Luau остаётся gameplay/content scripting layer; Wasm Component Model с versioned WIT остаётся isolated plugin ABI model.
- Script, mechanic package и plugin получают только capability-scoped immutable views и command/proposal sink. Прямой mutable ECS/world/physics access запрещён.
- Callback/VM transaction накапливает proposals отдельно. Trap, denial, authoritative quota overrun или host watchdog discard all uncommitted proposals; committed state не откатывается частично и host simulation не падает.
- VM/runtime version, package content hash, policy hash, capability set и authoritative quota config входят в save/replay compatibility metadata.

## Deterministic authoritative budgets

Единственными authoritative criteria являются counters, заданные `ExtensionBudgetPolicyV1`:

| Counter | Contract |
|---|---|
| Instructions / Wasm fuel | Exact maximum per callback и per gameplay tick; counter semantics pinned VM/runtime compatibility manifest |
| Allocated bytes | Exact cumulative allocation quota per callback и exact live-memory ceiling per instance |
| Host calls | Exact per-call cost units из versioned registry; denied call не расходует mutation budget и всё равно auditируется |
| Proposed command bytes/count | Exact limits до command admission |

Quota comparison выполняется в integer units. Exhaustion создаёт stable diagnostic `EXTENSION_BUDGET_EXCEEDED` с package/plugin ID, callback ID, counter kind, limit, observed count и tick. Одни и те же accepted inputs/version/config MUST давать один и тот же overrun tick, rejection и strike независимо от CPU speed, worker count или load.

Wall watchdog остаётся только host-health guard. При его срабатывании host MUST:

1. abort callback/VM instance;
2. discard uncommitted proposals;
3. пометить весь run `NonConforming(EXTENSION_WALL_WATCHDOG)`;
4. сохранить host-health diagnostic и load profile;
5. не создавать обычный gameplay strike, event или deterministic outcome;
6. запретить PASS replay/performance/conformance gate для этого run.

Watchdog duration не входит в replay и не сравнивается как gameplay fact. Нормативный test не использует sleep или retry-to-green.

## Circuit breaker

Violation ledger является authoritative state, входит в save/snapshot/replay и индексируется по package/plugin principal. Default policy отключает issuer после третьего authoritative violation в inclusive sliding window `1_800` gameplay ticks. Violation — только deterministic quota exhaustion, trap, invalid command/capability attempt или schema violation из versioned policy. Wall watchdog не добавляет violation.

Window вычисляется по simulation tick. На tick нового violation runtime вычисляет `window_start_tick = current_tick.saturating_sub(1_799)`, удаляет entries с `entry_tick < window_start_tick`, добавляет current entry, затем сравнивает count. На ticks `0…1_798` lower bound равен `0`; начиная с tick `1_799` inclusive window всегда содержит не более `1_800` gameplay ticks. Disabled issuer не выполняется и получает `EXTENSION_CIRCUIT_OPEN`; reset возможен только declared admin/load/migration command с audit. Project override versioned, hash-bound и не может использовать wall seconds.

## Package identity, signatures и capabilities

- SHA-256 `ContentHash` обязателен для каждого Luau package, mechanic/mod package и Wasm plugin, включая local development.
- Official/trusted distribution и eligibility для trusted capabilities требуют valid signature, admitted signer и exact content/manifest/policy hashes.
- Unsigned local package разрешён только после explicit user consent, bound к exact content hash и project. Он всегда остаётся под untrusted capability ceiling.
- Invalid, malformed, expired или revoked signature закрывается ошибкой и не downgrade-ится автоматически в unsigned local mode.
- Signature доказывает identity/integrity, но сама не выдаёт capability. Effective capabilities равны пересечению package request, API compatibility, signer scope, project policy, user consent и hard security ceiling.
- Agent, script, plugin и capture worker не могут создавать consent, trust root или trusted signature.
- Capability denial fail-closed до host call/mutation и входит в deterministic audit/result.

`PackageTrustManifestV1` является JCS-canonical public manifest по ADR-022 encoding contract. Он содержит package ID/type/content hash, manifest hash, signer key ID или explicit unsigned marker, requested capabilities, effective trust tier, policy hash и compatibility hashes. Private signing material не входит в workspace, runner schema, package или evidence.

## Gate synchronization

| Gate | Blocking contract | Required permutations/evidence |
|---|---|---|
| `SCRIPT-P1` | Luau capability isolation и deterministic API | Denied calls, immutable views, command validation, state roots |
| `SCRIPT-P2` | Exact instruction/allocation/host-call quota outcomes | Boundary vectors at limit-1/limit/limit+1, save/reload ledger |
| `SCRIPT-P5` | CPU/load/worker/watchdog variation не меняет authoritative outcome; watchdog всегда non-conforming | Same corpus under declared load profiles, VM counters, watchdog diagnostic, no PASS on trip |
| `PLUGIN-P1` | Fuel/memory/table/instance limits fail isolated and atomic | Trap/overrun corpus, RSS cleanup, proposal discard audit |
| `PLUGIN-P2` | WIT N/N-1 negotiation fail-closed | Compatibility matrix and diagnostics |
| `MOD-P2` | Signed/unsigned/invalid signature capability matrix | Exact content/policy hashes, consent records, signer scope and effective capability report |
| `VS-06` | Vertical extension scenario passes all required SCRIPT/PLUGIN/MOD gates | Hash-bound automatic evidence only; watchdog trip blocks closure |

`SCRIPT-P5` использует одинаковый command corpus и expected authoritative ledger на idle, saturated non-authoritative worker pool и varied host scheduling. Instruction/fuel/allocation outcomes должны совпасть exact. Injected wall trip должен всегда дать `NonConforming`, а не gameplay outcome.

## Последствия и синхронизация

Packet 1.5 присвоил ADR-006 статус `Superseded`; SPEC-07, SPEC-11, SPEC-13, SCRIPT/PLUGIN/MOD/VS-06 rows, glossary и traceability обновлены атомарно. Конкретные Luau/Wasmtime/signature libraries остаются replaceable choices за engine-owned contract.
