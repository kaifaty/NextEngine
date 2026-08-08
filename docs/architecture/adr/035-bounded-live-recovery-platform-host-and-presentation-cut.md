# ADR-035: Bounded live recovery, platform-host binding и presentation cut

| Поле | Значение |
|---|---|
| ID | ADR-035 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-07-30 |
| Последняя проверка | 2026-07-30 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](../18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-29](../29-platform-host-and-application-session.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [ADR-018](018-authoritative-project-composition-and-configuration.md), [ADR-019](019-canonical-player-actions-and-presentation-authority.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-025](025-schema-content-and-migration-authority.md), [ADR-028](028-platform-session-and-presentation-authority.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | Частично [ADR-028](028-platform-session-and-presentation-authority.md): задаёт exact bounded active-run cadence вместо неуказанной durable cadence; заменяет подразумеваемое неограниченное хранение raw prior-session generations на current+previous retention и bounded carry-forward recovery evidence closure; требует session-bound registration platform host/capability identity и exact authoritative presentation recovery cut. Closed lifecycle, exactly-once close/save, authority split, displayless headless и epoch-independent cue-consumption rules ADR-028 остаются Accepted. |
| Заменён | Recovery cadence/evidence/storage clauses заменены [ADR-047](047-simple-application-session-and-save-on-close.md); platform-host binding, input admission и presentation authority остаются Accepted |

## Контекст

ADR-028 правильно разделяет platform, application-session, authoritative
runtime и presentation authority, но оставляет implementation-critical
варианты открытыми:

- как часто active live run обязан публиковать durable checkpoint;
- сколько уже committed in-memory ticks разрешено потерять при process crash;
- является ли OS wall time после restart источником catch-up;
- какие canonical lifecycle request/event bytes нужны для exact retry после
  process restart и как ограничен их рост;
- как сохранить required-save recovery lineage после bounded pruning raw
  session generations;
- какой platform-adapter lifetime имеет право присылать новые lifecycle/input
  events текущей interactive session;
- может ли восстановленная camera timeline интерполировать через
  authoritative recovery boundary.

Без exact решения implementation либо публикует filesystem generation на
каждом tick, либо теряет unbounded active progress, хранит unbounded session
history, принимает stale event от старого host lifetime или визуально
интерполирует между двумя разными authoritative continuations.

## Решение

### 1. Live closure и durable cadence

Interactive live path работает на fixed `30 Hz` simulation cadence и на каждом
committed tick публикует один complete **in-memory** closure. Closure включает
Runtime/RPG/physics/world-streaming state, input resolver state, presentation
snapshot/camera state, logical sequences, revisions и exact roots. Renderer
MAY повторять последний complete snapshot с любой собственной cadence; partial
tick closure не наблюдаем.

Durable same-session checkpoint cadence фиксирована:

1. tick `0` публикуется до того, как active run считается restartable;
2. следующий checkpoint публикуется после каждого committed tick, кратного
   `30`;
3. suspend и close принудительно flush-ят newest complete closure до
   lifecycle progress.

Число `30` означает simulation ticks, а не wall-clock timeout. Между durable
boundaries не выполняется background timer save. Crash восстанавливает ту же
session из newest complete checkpoint и MAY откатить не более `29` committed
in-memory ticks. Volatile suffix не угадывается, renderer callbacks не
replay-ятся, а время process absence или suspend не превращается в simulation
catch-up.

### 2. Atomic lifecycle publication и exact archive

Lifecycle operation выполняется как stage → validate → atomic durable
publication → infallible in-memory commit. Suspend публикует forced live
closure и `Active → Suspended` transition как одну complete generation. Close
не начинает следующий journal-proven edge, пока forced closure не
опубликована. Publication fault возвращает session machine, prepared/live
closure и durable pointer к prior complete generation.

Каждый retained lifecycle request хранится не только как request/event hash:
его полные canonical request и event bytes находятся в content-addressed
objects, а durable archive связывает request ID с обоими object hashes.
Restart восстанавливает этот archive до допуска нового transition. Exact retry
сначала сравнивает полный request; changed bytes под тем же ID остаются
identity collision.

Archive ограничен:

- не более `1 040` lifecycle entries в одной durable application session;
- не более `1 024` platform-sourced lifecycle requests внутри этого archive.

Переполнение отклоняет новый transition до publication с
`SESSION_PLATFORM_LIFECYCLE_BUDGET_EXCEEDED`; completed exact retry не создаёт
новую entry. Bound не разрешает забывать старую identity и принимать её как
новую.

### 3. Bounded required-save recovery evidence

Assets session store удерживает только current и immediately previous complete
session generations. Это даёт atomic pointer rollback и один known-good
predecessor, но не обещает unbounded raw history.

Если terminal `RequireFinalSave` failure создаёт новую linked session, новая
durable generation carry-forward-ит ordered recovery evidence archive. Каждая
entry содержит:

- canonical `RecoverySessionLinkV1` hash;
- content hash объекта с полными canonical link bytes;
- content hash exact prior durable snapshot;
- canonical sorted list content hashes полного prior object closure, включая
  manifest, lifecycle request/event bytes, close request/journal/failed-ledger
  evidence и live-checkpoint closure, если он referenced prior snapshot.

Chain валидируется полностью до activation/recovery: object hashes и canonical
bytes должны совпасть; prior snapshot обязан содержать exact named
`Finalizing` state/manifest/close/failed ledger и непрерывный lifecycle archive;
project lock остаётся тем же; `prior.new_session_id` равен
`next.prior_session_id`; последний link обязан указывать на current session и
совпадать с её manifest. Missing object, gap, reorder или tamper fail closed.

Archive содержит не более `64` entries, `16 384` object references и `1 984`
уникальных carry-forward evidence objects. Первый исчерпанный bound отклоняет
required-save recovery до publication с
`SESSION_RECOVERY_EVIDENCE_BUDGET_EXCEEDED`; существующая failed
`Finalizing` session и prior generation остаются неизменны.

Таким образом prior immutable state/events остаются read-only canonical
evidence внутри явных bounds, хотя отдельные raw generation directories кроме
current+previous не удерживаются и history не растёт бесконечно. Это точечная
замена storage-retention implication ADR-028, а не новый lifecycle edge и не
разрешение переписывать failed session в `Closed`.

### 4. Session-bound platform host

Interactive coordinator регистрирует ровно один current platform-adapter
lifetime после проверки полного `PlatformCapabilitySetV1`:

- capability set canonical hash обязан совпасть с hash в immutable
  `ApplicationSessionManifestV1`;
- coordinator выдаёт fresh opaque `host_instance_id`, связанный с current
  session/generation и adapter lifetime;
- каждый новый `PlatformEventV1` обязан содержать этот host ID и exact
  capability-set hash;
- unregistered host отклоняется с `PLATFORM_CAPABILITY_REQUIRED`, а stale host
  или changed capability binding — с
  `PLATFORM_EVENT_IDENTITY_COLLISION`;
- `headless`/presentation target `None` не может зарегистрировать interactive
  host и не принимает его events.

Host ID ephemeral и non-authoritative: он не выбирает simulation tick,
command, save или gameplay result. После process restart adapter регистрируется
заново и получает fresh ID. Event старого host lifetime не может начать новый
transition; исключение — полный exact platform event, уже связанный с
архивированным lifecycle/close request. Такой retry возвращает либо продолжает
только journal-proven prior result и не меняет causal identity.

### 5. Authoritative presentation recovery cut

Persisted `PresentationSnapshotV2` при same-session active restart сначала
декодируется и проверяется как evidence: exact profile, project/content
bindings, authoritative tick и current camera state должны совпасть с
восстановленным closure.

После проверки он **не** продолжается как следующая запись старой presentation
timeline. Extractor:

1. выводит fresh `snapshot_epoch` из canonical hash persisted snapshot под
   отдельным authoritative-recovery domain;
2. публикует rebuilt current projection с `snapshot_sequence = 0`;
3. помечает каждую camera record как `cut = true`;
4. задаёт camera `previous_result_sample == current_result_sample`, запрещая
   interpolation через recovery boundary.

Renderer MUST discard cross-epoch interpolation history. Camera yaw/pitch
восстанавливаются только после exact validation и остаются presentation-only.
Snapshot epoch/sequence reset не сбрасывает epoch-independent cue stream,
acknowledgment prefix или `PresentationConsumptionStateV1` rules SPEC-30.
Recovered held controls отменяются до допуска input от freshly registered
host, поэтому старый adapter lifetime не продолжает незавершённое действие.

### 6. Persisted compatibility

Решение не добавляет новый engine-owned public contract. Existing
`PlatformCapabilitySetV1`, `PlatformEventV1`, lifecycle/close contracts,
`RecoverySessionLinkV1` и `PresentationSnapshotV2` сохраняют wire semantics.

Private application durable snapshot получает current schema с full lifecycle
archive и recovery-evidence archive. Legacy private generations MAY
декодироваться для bounded compatibility, но active restart, которому
необходима отсутствующая exact closure/archive, fail-closed; implementation не
фабрикует history и не дополняет historical bytes default values.

## Рассмотренные варианты

- **Durable publication every tick.** Отклонено: оно превращает fixed-step
  runtime в filesystem-latency path и не даёт продуктовой пользы относительно
  явно ограниченного rollback window.
- **Wall-time autosave и catch-up после resume.** Отклонено: host timing стал
  бы simulation authority.
- **Хранить все raw session generations бесконечно.** Отклонено: storage растёт
  без bound. Current+previous плюс bounded canonical recovery evidence closure
  сохраняют проверяемые prior state/events без unbounded generation history.
- **Забывать lifecycle request после pruning.** Отклонено: старый ID мог бы
  снова считаться новым и нарушить exactly-once.
- **Принимать любой self-validating event с capability hash.** Отклонено:
  stale process/adapter lifetime мог бы suspend, resume или close текущую
  session.
- **Продолжить presentation epoch/sequence после authoritative restart.**
  Отклонено: renderer мог бы интерполировать между двумя разными
  authoritative continuations.

## Последствия

- Crash-loss bound честно равен `0..=29` ticks; это recovery policy, а не
  обещание сохранения каждого live tick.
- Lifecycle и recovery evidence имеют явные конечные budgets и typed
  exhaustion вместо silent eviction.
- Platform host identity является доверенной session-scoped registration, но
  не входит в gameplay authority.
- Same-session restart сохраняет authoritative session/world continuity и
  одновременно создаёт явный presentation discontinuity.
- ADR-028 продолжает определять closed lifecycle, exactly-once close/save,
  composition-root parity и presentation authority во всех частях, прямо не
  заменённых этим ADR.

## Product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `fast` | tick `0/29/30/59/60`, crash, publication fault, archive budgets, tamper и exact retries | Durable cadence exact; no partial state; full request/event objects и recovery evidence closure восстанавливаются; overflow/tamper fail closed | Prior complete generation остаётся current |
| `play` | live/headless authoritative parity при renderer cadence `30/60/144 Hz` и active restart | Gameplay roots совпадают; live renderer повторяет only complete snapshots; restart теряет не более `29` ticks | Resume newest complete checkpoint без catch-up |
| `persistence-replay` | same-session checkpoint/restart и consecutive required-save recoveries | Runtime/RPG/physics/streaming/input roots exact; ordered links и their prior snapshot/object closures carry forward до bound | Reject missing/tampered closure before activation |
| `platform` | fresh host registration, stale/wrong-capability events, suspend/resume/close retry и recovered camera | Только current host создаёт new events; exact archived retry остаётся idempotent; recovery публикует new epoch, sequence `0` и camera cuts | Reject event/profile or keep presentation unavailable without gameplay mutation |

Acceptance и локальный Windows check result не закрывают native Linux
validation, R1/B-01, `native_gate_ready` или весь R2. PhysX, CI и remote
execution не входят в это решение.
