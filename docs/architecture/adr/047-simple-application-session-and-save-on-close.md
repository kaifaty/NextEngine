# ADR-047: Simple application session and save-on-close

| Поле | Значение |
|---|---|
| ID | ADR-047 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-08 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-29](../29-platform-host-and-application-session.md), [ADR-028](028-platform-session-and-presentation-authority.md), [ADR-035](035-bounded-live-recovery-platform-host-and-presentation-cut.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | Полностью [ADR-037](037-packed-session-object-storage.md). Частично ADR-028/035: active-run checkpoint cadence, recovery evidence/history, session object closure, retry/deadline/failure-disposition и final-save-ledger clauses. Authority split, platform-host admission, presentation immutability и восемь lifecycle-состояний сохраняются. |
| Заменён | не заменён |

## Контекст

R2 подтвердил рабочий Save/Load/Resume loop, но session durability стала второй
системой persistence поверх `SaveStore`: object packs, content-addressed index,
полный lifecycle archive, recovery evidence и 30-tick checkpoints. Для текущего
single-player alpha это повышало стоимость live и close paths, не улучшая
пользовательский результат, который уже выражается ручным Save и save-on-close.

## Решение

1. Сохраняются ровно восемь состояний:
   `Created → CompositionStaged → RuntimeStaged → Active ↔ Suspended → Quiescing → Finalizing → Closed`.
2. Текущие контракты — `ApplicationSessionManifestV2`,
   `ApplicationSessionStateV2`, lifecycle request/event V2,
   `CloseSessionRequestV2`, `CloseSessionJournalV2` и
   `CloseSessionReceiptV2`. Старые версии отклоняются как unsupported.
3. Session snapshot V4 хранит текущее состояние, только последний lifecycle
   request/event, один close request/journal/receipt и immutable prepared save
   image только на стадии `Prepared`.
4. Close journal имеет только `Prepared | SavePublished`. `Prepared` позволяет
   повторно опубликовать byte-identical `SaveImage`; `SavePublished` завершает
   close без второй save generation.
5. `SessionStore` — два чередующихся слота, `CURRENT` и ровно один
   `session.snapshot.v4.bin` в каждом занятом слоте. Session objects, object
   packs, indexes и recovery archives удаляются.
6. Durable world publication выполняется только ручным Save и save-on-close.
   Suspend не сохраняет мир. Crash теряет прогресс после последнего Save.
7. После crash live session открывается в `Suspended` и восстанавливает newest
   compatible save. `Load` разрешён только в `Suspended`, создаёт fresh
   presentation epoch/sequence `0` и требует явного Resume.
8. Wire shape `SaveManifestV2`, `WorldCheckpointV4`, `ReplayManifestV5` и
   двухслотового `SaveStore` не меняется. `CommandLedgerV2` и physics contracts
   не затрагиваются.

## Последствия

- Нет обещания восстановления несохранённых 29 ticks и полной lifecycle history.
- Close имеет одну policy: final save. Retry budget, deadline class и last-safe
  fallback не являются контрактами.
- Authoritative durable history остаётся в существующих Save/Replay форматах.

## Проверка

- contract/runtime/session-store/application focused tests;
- `play`, `persistence-replay`, `platform` и `host-check`;
- structural scan без recovery archive, session object pack, final-save ledger
  и 30-tick session checkpoint symbols.
