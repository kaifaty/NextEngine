# ADR-037: Packed session object storage

| Поле | Значение |
|---|---|
| ID | ADR-037 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-07-31 |
| Последняя проверка | 2026-07-31 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](../22-canonical-schema-registry-and-migration.md), [SPEC-29](../29-platform-host-and-application-session.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-025](025-schema-content-and-migration-authority.md), [ADR-035](035-bounded-live-recovery-platform-host-and-presentation-cut.md), [ADR-036](036-thoth-reference-performance-profile.md) |
| Заменяет | Узко заменяет physical-storage implication [ADR-035](035-bounded-live-recovery-platform-host-and-presentation-cut.md): current и previous остаются двумя complete logical session generations, но их logical objects MAY находиться в private canonical-indexed pack files вместо отдельного файла на каждый object. Cadence, rollback, recovery evidence и public wire semantics ADR-035 не меняются. |
| Заменён | не заменён |

## Контекст

Durable live checkpoint публикуется на tick `0`, каждые `30` committed ticks и
на forced suspend/close boundary. Прежний `SessionStore` создавал отдельный
синхронизируемый файл для каждого logical object. На Windows стоимость
`FlushFileBuffers` зависит прежде всего от количества durable files, поэтому
рост object closure превращал checkpoint в серию блокирующих filesystem
barriers и вызывал history-dependent frame stalls.

Исследовался fixed 4-KiB chunk pool: реальные adjacent checkpoints
переиспользовали большинство chunks и сокращали logical duplicate bytes.
Однако release `long-session-soak` показал противоположный latency-result:
несколько изменившихся chunks требовали нескольких durable file flushes.
Application checkpoint windows выросли примерно с `3/6/8 s` до `12/30/48 s`.
Поэтому отдельные durable chunk files отклонены, несмотря на storage reuse.

ADR-022 отделяет logical content-addressed map от private physical
packing/segmentation. Требуется использовать эту свободу для уменьшения числа
durability barriers без изменения authoritative identity.

## Решение

### 1. Logical identity не меняется

`SessionPublicationV1`, canonical session index, snapshot bytes, ordered logical
object hashes и `generation_id` сохраняют прежние semantics. Pack index, pack
ordinal, byte offset, filesystem path и validated in-process cache:

- не входят в authoritative state root, command ledger root или replay root;
- не меняют `WorldCheckpointV4`, `RuntimeSnapshotV3` и owner segments;
- не видимы за engine-owned public contracts;
- полностью восстанавливаются из logical object set.

Load MUST вернуть exact original bytes и проверить final logical hash каждого
object. Legacy generation с `objects/<logical_hash>.bin` остаётся допустимым
input без migration или перепубликации.

### 2. Bounded canonical object packs

New session generation хранит:

- прежние `session/index.bin` и `session/snapshot.bin`;
- private canonical `nextengine.session-object-pack-index.v1`;
- один или несколько `objects/pack-NNNN.bin`.

Pack index содержит sorted record для каждого logical object:
`object_hash`, `pack_ordinal`, `byte_offset`, `byte_length`. Objects
конкатенируются в canonical hash order. Pack не превышает прежний per-file
limit `256 MiB`; если следующий object не помещается, начинается новый pack.
Количество object records и packs ограничено прежним `4 096` object budget.
Packing не создаёт новый logical size rejection.

Physical pack bytes сами не являются новым logical content object.
`ContentStore` всё равно хэширует и валидирует каждый pack как generation file,
после чего `SessionStore` проверяет bounds, exact ordered object hashes и final
logical object hash.

Windows-local release `long-session-soak` после реализации дал application
checkpoint windows `2.784 / 4.977 / 7.298 s` против preceding
one-file-per-object implementation `3.228 / 5.874 / 7.933 s` при том же exact
state root. Это один `REPORT_ONLY` run, а не hard baseline verdict, но он
подтверждает выбранное направление и отклоняет chunk-file regression.

### 3. Atomic publication и retention

Generation publication использует прежний staging path:

1. проверить registry transition и current/previous closure;
2. записать pack files, pack index, snapshot и generation index в staging;
3. синхронизировать и полностью перечитать staged generation;
4. atomically rename generation и заменить `CURRENT`.

Current и immediately previous logical generations complete только при наличии
всех pack files либо всех legacy raw objects. Missing/corrupt pack, malformed
index, overlap/out-of-bounds location, unexpected mixed representation или final
object hash mismatch fail closed с `SESSION_STORAGE_UNAVAILABLE` до возврата
generation.

Post-commit cleanup по-прежнему удерживает current+previous generation
directories. Pack representation self-contained внутри generation directory:
экспорт/копирование не требует внешнего pool.

### 4. Validated in-process cache

Первый load в process полностью проверяет current и previous logical closures.
`SessionStore` MAY переиспользовать это immutable validated состояние в
последующих publications того же process, если exact `CURRENT` generation ID не
изменился.

Public `load_current()` всегда заново читает disk closure. Process restart,
changed `CURRENT` или новый store instance инвалидирует cache. Cache не является
source of truth и не сериализуется.

## Рассмотренные варианты

- **Продолжать one-file-per-object.** Отклонено: сохраняет последовательность
  expensive durable barriers.
- **Отдельные fixed-size chunk files.** Отклонено release soak: duplicate bytes
  меньше, но Windows checkpoint latency выросла в несколько раз.
- **Authoritative delta snapshot schema.** Отклонено: меняет recovery/migration
  surface и roots ради private storage concern.
- **Global mutable/append-only pack pool.** Отложено: требует отдельного
  crash-safe compaction/index protocol и усложняет complete closure.
- **Не проверять final object hashes после slicing.** Отклонено: pack integrity
  не доказывает правильные offsets/order.

## Последствия

- Обычная generation с любым числом небольших objects выполняет один object-data
  file flush вместо одного flush на object.
- Total bytes current+previous не дедуплицируются между generations; latency и
  простая self-contained recovery важнее этого storage win.
- Old raw generations читаются exact без migration.
- Optimization остаётся private Assets concern и не ослабляет cross-platform
  bit-exact replay requirements.

## Product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `fast` | legacy raw load, packed roundtrip, corrupt/missing pack, malformed pack index, pointer fault | Exact logical bytes; corruption rejects before return; prior current remains authoritative | Retain prior complete generation |
| `persistence-replay` | live checkpoint, restart and replay over packed storage | Runtime/RPG/physics/streaming/input/presentation bytes и all authoritative roots exact | Reject incompatible closure before activation |
| `performance` | release `long-session-soak` with 30-tick checkpoints | Fewer durability barriers, no regression from one-file-per-object baseline, unchanged roots/cadence | Do not accept physical packing that regresses checkpoint latency |
