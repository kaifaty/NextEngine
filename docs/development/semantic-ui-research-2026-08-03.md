# Research: Semantic UI (roadmap queue #1, R2)

| Поле | Значение |
|---|---|
| Статус | Research note; рабочий материал, не normative architecture |
| Дата | 2026-08-03 |
| Контекст | Roadmap "Ближайшая implementation queue", пакет 1: Semantic UI (`NEXT`) |

Этот документ фиксирует результат исследования перед реализацией пакета
**Semantic UI**: HUD, inventory/equipment, dialogue, quest journal,
pause/save/load flow и pseudo-locale. Нормативной authority остаются
Accepted SPEC/ADR; при расхождении действует precedence из
[architecture/README](../architecture/README.md).

## 1. Scope пакета по roadmap

Roadmap R2 объявляет для этого increment:

- semantic HUD, inventory/equipment, dialogue, quest journal;
- pause/save/load flow;
- source locale, deterministic text IDs, pseudo-locale и readable fallback.

Явные non-goals пакета (из R2 scope guard и roadmap): editor, advanced
renderer, controller profile (отдельный decision/item), extension
`ui.panel.register` из SPEC-18 (capability-scoped panels — отдельная работа,
здесь только не сломать будущую границу), audio (следующий пакет очереди).

Критерии успеха R2, на которые влияет пакет: новый пользователь проходит
movement → pickup/equip → combat → dialogue → quest → save/load loop без debug
commands; те же semantic actions в `game` и headless дают одинаковый
authoritative result; UI/camera faults не меняют gameplay hash.

## 2. Нормативная база (прочитана полностью)

Routing-строка: «Player interaction, UI, camera, localization, accessibility»
→ [SPEC-18](../architecture/18-player-interaction-ui-camera-localization-and-accessibility.md) + [ADR-019](../architecture/adr/019-canonical-player-actions-and-presentation-authority.md), check `play`.
Пакет также затрагивает строки «Public contracts» (SPEC-01/SPEC-02 + ADR-002,
check `fast`) и, при добавлении text content, «Assets/content catalog»
(SPEC-03/SPEC-24 + ADR-025/ADR-014, check `content-package`).

Прочитано полностью: INDEX-001 (v2.15), SPEC-00 (v2.1), GLOSSARY-001 (v2.3),
SPEC-18 (v2.3), ADR-019 (v1.1), SPEC-04 (v2.2), SPEC-30 (v2.0), SPEC-29
(v2.4), ADR-028 (v1.1), ADR-034 (v1.0), ADR-035 (v1.0), SPEC-08 (v1.8),
SPEC-24 (v1.0), SPEC-12 (v2.4), а также секции SPEC-17 о `ConfigurationClass`.

## 3. Что требует архитектура (обязательства для дизайна)

### 3.1. `UiSemanticSnapshot` (SPEC-18, glossary)

- Immutable projection: stable screen/panel/control IDs, semantic roles,
  enabled/visible/value/selection state, **text resource IDs с typed
  arguments**, focus graph, action/command affordances.
- Исключает: mutable aggregate references, widget/backend objects, localized
  strings как identity.
- Data flow: `PresentationSnapshotV2 / immutable query → UiSemanticSnapshot →
  rendered widgets → PlayerActionFrame → command candidate → WorldCommand
  validation → DomainEvent → new projection`. Обратного потока нет.
- UI projection — presentation work: задержка/отказ не меняет ingress,
  command order или authoritative state (REQ-093).

### 3.2. Место в `PresentationSnapshotV2` (SPEC-30)

- Snapshot уже содержит поле `semantic_ui_batches`; `SemanticUiPresentationRecordV1`
  ссылается на `UiSemanticSnapshot` values, localized text IDs и
  style/accessibility roles. Сортировка records: `(surface_id,
  semantic_path_id, element_id)`.
- В cue envelope существует closed kind `SemanticUi` с rank `4` — UI cues уже
  зарезервированы в canonical cue order.
- Batches следуют общему canonical batch rule (flatten → canonical sort →
  partition по lock-bound profile), worker order не входит.

### 3.3. Pause/menu policy (SPEC-18 §UI, SPEC-29)

- Admitted project composition MUST declare одно из трёх: simulation
  continues / request `Suspended` на fixed lifecycle boundary / separate
  non-authoritative menu timeline. UI испускает только declared
  action/request; widget/platform callback не может pause/resume/close
  authoritative state напрямую.
- Для reference project нужно явно выбрать и зафиксировать политику в
  composition (SPEC-17), вероятный выбор — request `Suspended` на lifecycle
  boundary через существующий `ApplicationLifecycleRequestV1` path.

### 3.4. UI actions и ingress (SPEC-18, ADR-019, ADR-034)

- Новые действия (pause, открыть inventory/journal, навигация/выбор в
  диалоге) — это новые stable action IDs в `ActionMapManifest` + InputContext
  stack (context для menu/dialogue с capture policy). Те же frames идут через
  тот же gateway в `game` и headless; replay V5 receipts уже покрывают
  composite frames.
- Dialogue choice/inventory action входит в simulation как versioned
  action-derived command candidate через common validation — mapper slots
  должны быть объявлены, target tick только из `IngressAssignmentV1`.
- Action-to-command mapping заряжен на существующую ADR-016
  `core-command-rpg` budget row; новой budget category не создаётся.

### 3.5. Localization (SPEC-18 §Localization)

- Durable references: stable text/voice/subtitle `AssetId` или namespaced
  text ID, never rendered text. Locale — BCP-47 tag; fallback chain declared
  by project content; typed formatting args; simulation calendar values
  вместо host locale/wall clock.
- Missing string/format/glyph → stable diagnostic
  `LOCALIZATION_RESOURCE_MISSING` + declared source/default locale или
  readable placeholder. Pseudo-locale — это ещё один catalog, не особый код.
- Authority table называет source of truth: «Cooked content registry by
  stable text ID + locale».

### 3.6. Preferences/accessibility (SPEC-18, SPEC-17)

- `PlayerPreferenceProfile` — versioned **local** data (PresentationOnly
  class по SPEC-17): bindings, text/UI scale, subtitles, reduced motion и т.п.
  Не save/domain state; corruption → quarantine + bounded defaults
  (`PLAYER_PREFERENCE_INVALID`). Для этого пакета достаточно минимального
  профиля (text scale/subtitle/reduced motion), если он вообще входит.

### 3.7. Failure semantics, релевантные пакету

- `UI_SCHEMA_INCOMPATIBLE`: optional panel disabled; **required UI rejects
  project pre-world**.
- `INPUT_CONTEXT_STALE`, `INPUT_BINDING_CONFLICT` — для новых contexts/maps.
- `LOCALIZATION_RESOURCE_MISSING`, `PLAYER_PREFERENCE_INVALID`.
- UI-P1 и ACCESS-P1 — исполняемые product checks (см. §7).

## 4. Текущее состояние кода (инвентаризация 2026-08-03)

### Уже есть

- **Input/contracts** (`crates/contracts/src/input/`): `ActionMapManifest`,
  InputContext stack, canonical `PlayerActionFrameV1`, ingress batches,
  profiles; action IDs: `move`, `interact`, `pickup`, `equip-use`, `melee`,
  `camera-orbit` (`input/constants.rs`). Resolver production-общий для live и
  headless (`crates/player/`).
- **Snapshot schema** (`crates/contracts/src/presentation.rs`):
  `PresentationSnapshotV2.semantic_ui_batches: Vec<ContentHash>` — поле
  существует, canonical hash учитывает, **всегда пустое**
  (`Vec::new()` в constructor).
- **Camera** (`crates/contracts/src/presentation/camera.rs`): полный typed
  integer/fixed-point third-person record.
- **Extraction** (`crates/presentation/src/lib.rs`): `PresentationExtractorV1`
  с `extract` / `extract_with_cameras`, recovery codec
  (`recovery_codec.rs`) прозрачно переносит `semantic_ui_batches`.
- **Reference driver** (`crates/reference-game/src/live.rs`,
  `publish_presentation`, line ~718): публикует scene + camera через
  extractor; источник данных — `physics_snapshot` + bindings.
- **RPG domain** (`crates/contracts/src/rpg/aggregate.rs`):
  `RpgSnapshotV2` и payload'ы Character/Inventory/Equipment/Quest/Dialogue/
  Faction/Relationship/InteractiveObject — достаточный источник для UI
  projections.
- **SPEC-17**: `ConfigurationClass::PresentationOnly` для preferences;
  `ApplicationLifecycleRequestV1` path для pause/suspend (SPEC-29).

### Отсутствует (gap)

| Gap | Деталь |
|---|---|
| `UiSemanticSnapshot` schema | Нет типа, codec, validation, stable IDs в `crates/contracts`. |
| `SemanticUiPresentationRecordV1` | Нет record/codec; batches всегда пустые. |
| Semantic UI extraction | Extractor не принимает UI bindings; нет projection из RPG/dialogue/quest state. |
| UI action IDs и contexts | Нет pause/inventory/journal/dialogue-nav action IDs, нет menu/dialogue InputContext. Mapper slots для UI-derived commands отсутствуют. |
| Localization content | В SPEC-24 neutral catalog (10 схем) **нет text/localization record**; нет text ID registry, locale catalogs, fallback chain, pseudo-locale. |
| `PlayerPreferenceProfile` | Нет типа и local persistence. |
| Widget rendering | `crates/render` (lib + planner) и `crates/desktop-sdl-ash` не рендерят текст/UI; toolkit не выбран. |
| Pause/menu flow | Нет declared pause policy в reference project composition; нет menu → lifecycle request связки. |
| Dialogue choice через actions | Диалог сейчас driven by scenario/production commands; нет player-facing choice actions. |

## 5. Открытые решения (требуют решения до/в начале реализации)

1. **UI toolkit/backend** — roadmap decision «до R2 UI integration»:
   private replaceable adapter behind semantic UI; не вводить widget types в
   contracts. Для пакета нужен минимальный decision: engine-owned immediate
   2D UI pass в private desktop adapter (B0), contracts остаются
   widget-free. Формально это не меняет Accepted semantics (SPEC-18 явно не
   выбирает toolkit), значит ADR не требуется, но решение фиксируется в
   roadmap/dev-note.
2. **Text content model** — где живут localized resources:
   - вариант A: новый neutral schema (например
     `nextengine.content.text-catalog`) — semantic change к SPEC-24 → нужен
     короткий ADR + обновление SPEC-24/routing/traceability;
   - вариант B: project generic records вне SPEC-24 catalog с собственным
     registry — проверить, не нарушает ли «Cooked content registry by stable
     text ID + locale» и bundle/dependency семантику.
   Рекомендация: A, потому что text ID + locale требует exact cooked
   revisions, fallback chain и dependency closure как у остального контента.
3. **Pause policy** reference project: continue / Suspended / menu timeline.
   Выбор фиксируется в project composition (SPEC-17), вероятно
   `Suspended`-request через существующий lifecycle path; это configuration,
   не ADR.
4. **Граница пакета по preferences**: включить минимальный
   `PlayerPreferenceProfile` (text scale + pseudo-locale switch) или оставить
   ACCESS-P1 на следующий increment. По roadmap item формулировке
   («pseudo-locale») — pseudo-locale входит, preferences minimal.
5. **Headless UI**: UI-P1 требует прохождения inventory/dialogue/quest/menu
   flows через semantic projections и production actions **в headless**.
   Значит semantic UI projection и action mapping обязаны быть
   renderer-independent и запускаться в headless scenario producer.

## 6. Предлагаемая декомпозиция пакета (sub-increments)

Каждый sub-increment — отдельный product increment с focused checks, по
правилам roadmap («не начинать как универсальный framework»):

1. **UI contracts**: `UiSemanticSnapshotV1` + `SemanticUiPresentationRecordV1`
   в `crates/contracts` (stable IDs, roles, state, text refs с typed args,
   focus graph, affordances), canonical codec, bounds, fail-closed
   validation, `UI_SCHEMA_INCOMPATIBLE`. Checks: `fast`.
2. **Semantic UI extraction**: расширение `PresentationExtractorV1` UI
   bindings + projection из RPG/dialogue/quest/inventory state в
   reference-game; публикация `semantic_ui_batches` с canonical sort
   `(surface_id, semantic_path_id, element_id)`. HUD read-only. Checks:
   `fast`, `play`.
3. **UI actions + pause flow**: новые action IDs и menu/dialogue
   InputContext в `ActionMapManifest`; mapper slots; pause → declared
   lifecycle request; save/load affordances через production paths.
   Checks: `fast`, `play`, `persistence-replay`.
4. **Localization**: text-catalog content schema (ADR + SPEC-24 update при
   варианте A), source locale catalog, fallback chain, pseudo-locale,
   `LOCALIZATION_RESOURCE_MISSING`. Checks: `content-package`, `fast`.
5. **Minimal widget adapter**: private 2D UI pass в desktop adapter
   (B0-safe), потребляющий semantic batches + text catalogs; headless
   unaffected. Checks: `play`, `platform` (conditional, desktop adapter).
6. **Preferences minimal**: `PlayerPreferenceProfile` local store,
   bounded defaults, text scale/pseudo-locale override как PresentationOnly.
   Checks: `fast`, `play`.

Порядок 4↔5 можно поменять; 3 зависит от 1–2; 5 зависит от 1–2 и (для
текста) 4.

## 7. Product checks план

| Check | Обоснование |
|---|---|
| `fast` (`cargo run -p xtask -- host-check`) | Каждый code change; contracts codec/validation negative tests. |
| `play` | Routing-строка UI; live + headless parity новых actions; UI-P1/ACCESS-P1 сценарии (inventory/dialogue/quest/menu flows, optional-panel fault, pseudo-locale + missing resource). |
| `persistence-replay` | Новые action-derived commands входят в ingress/replay V5; pause/save/load через production lifecycle. |
| `content-package` | Text-catalog content: cook/validate/load, fallback chain, malformed/missing locale. |
| `platform` (conditional) | Только если меняется desktop adapter (widget pass). |
| `performance` | Не ожидается; UI projection — presentation work вне hot authoritative path. |

## 9. Принятые решения (2026-08-03, owner)

Подтверждены варианты из вопросов к owner:

| # | Решение | Выбор |
|---|---|---|
| 1 | Scope экранов | **A**: HUD + inventory/equipment + dialogue + quest journal + pause-меню с save/load. |
| 2 | Text content model | **A**: новый neutral text-catalog schema (вариант с ADR + SPEC-24 update). |
| 3 | Pause policy | **B**: request `Suspended` на lifecycle boundary через существующий SPEC-29 path. |
| 4 | Локали | **A**: source locale (en) + pseudo-locale catalog + fallback chain + `LOCALIZATION_RESOURCE_MISSING`. |
| 5 | Preferences | **A**: минимальный `PlayerPreferenceProfile` (text scale + pseudo-locale switch), local PresentationOnly. |
| 6 | UI rendering | **A**: private immediate-mode 2D pass в desktop-sdl-ash (engine-owned, textured quads + bitmap font); contracts widget-free. |
| 7 | Dialogue choice | **A**: универсальные UI action IDs `ui-nav` / `ui-confirm` / `ui-back` через общий ActionMap/InputContext + production mapper; работает для диалога, inventory и меню. |
| 8 | Save/load в меню | **A**: save request + load из меню через production same-session restart/checkpoint path, без новых lifecycle edges. |

## 10. Риски и замечания

- **Не создавать второй input path**: UI actions обязаны идти через общий
  ingress/mapper; widget callbacks не мутируют state (ADR-019 rejected
  variant).
- **Localized strings не identity**: все durable references — text IDs;
  pseudo-locale не должен менять hashes/commands (REQ-093, ACCESS-P1).
- **ADR граница**: новый text-catalog schema — semantic change к SPEC-24;
  оформить короткий ADR и синхронно обновить SPEC-24, routing table и
  traceability в том же change (правило AGENTS.md).
- **File layout**: соблюдать действующий лимит ~1000 строк/файл в contracts
  (см. прецедент переноса helpers в roadmap notes) и
  `docs/development/source-layout.md`.
- **B-03/B-11**: пакет не закрывает lawful representative content; UI
  текст/placeholder контент для reference slice — engine-owned, provenance
  фиксируется как обычно.
- **R1/B-01** остаётся открытым параллельно; этот пакет — Windows-first и не
  зависит от Linux evidence, но `v1-closure` продолжит честно сообщать Linux
  `NOT_RUN`.

## 11. Статус реализации

- **Sub-increment 1 (UI contracts) — DONE (2026-08-03).**
  `crates/contracts/src/presentation/ui.rs` + `ui/tests.rs`:
  `UiSemanticSnapshotV1`, `UiSemanticPanelV1`, `UiSemanticElementV1`,
  `UiTextRefV1`/`UiTextArgumentV1`, `UiElementValueV1`,
  `UiActionAffordanceV1`, `UiElementFocusKeyV1`, closed enums
  role/style/accessibility, `SemanticUiPresentationRecordV1`,
  `SemanticUiPresentationBatchV1`, `build/validate_semantic_ui_batches`
  (batch kind `"SemanticUi"`, domain `nextengine.presentation-batch.v1`,
  limit 4096). Checks: `host-check` PASS.
- **Sub-increment 2 (semantic UI extraction) — DONE (2026-08-03).**
  - `PresentationSnapshotV2.semantic_ui_batches`:
    `Vec<ContentHash>` (placeholder) → typed
    `Vec<SemanticUiPresentationBatchV1>`; новый конструктор
    `new_with_camera_and_semantic_ui_records` (старые конструкторы
    делегируют с пустым UI набором — canonical hash пустого случая не
    изменился); `validate()` вызывает `validate_semantic_ui_batches`;
    accessor `semantic_ui_records()`.
  - Recovery codec v2 (`nextengine.presentation-snapshot-recovery.v2`):
    field 12 — полный typed record codec
    (`nextengine.semantic-ui-presentation-recovery-record.v1`, nested
    `nextengine.ui-text-ref-recovery.v1`), field 16 —
    `max_semantic_ui_records_per_batch`; fail-closed, byte-exact
    round-trip. Codec разделён: `recovery_codec/mod.rs` +
    `recovery_codec/semantic_ui.rs`.
  - `PresentationExtractorV1`: поле `max_semantic_ui_records_per_batch`,
    ctor `new_with_snapshot_epoch_and_ui_batch_limits`,
    `extract_with_cameras_and_semantic_ui(Vec<SemanticUiPresentationRecordV1>)`,
    accessor `snapshot_epoch()`; старые extract* делегируют с пустым UI.
  - `PreparedRuntimeTick::rpg_snapshot()` — read-only staged RPG
    projection для UI probes на той же publication boundary.
  - Reference-game projection `crates/reference-game/src/ui.rs`: HUD
    surface `nextengine.ui.surface.hud`, panel
    `nextengine.ui.panel.hud.status`; health `Meter` из
    `CORE_CHARACTER_HEALTH_RESOURCE_ID` + quest `Label` с
    `TextId(state_id)` аргументом; `source_snapshot_hash` =
    `domain_hash("nextengine.ui-source.rpg-snapshot.v1",
    rpg.canonical_bytes())`. Пустой RPG snapshot (non-interactive) →
    ноль UI records. Wire-in в `publish_presentation` и `stage_advance`.
  - Integration test
    `live_presentation_publishes_typed_semantic_ui_hud_from_rpg_state`:
    typed HUD records, recovery restore с UI batches, пустой
    non-interactive случай.
  - Checks: `host-check` PASS, `play` PASS; `persistence-replay`,
    `content-package`, `platform`, `performance` — NOT_RUN (не
    затрагиваются этим increment: save/load flows и text catalogs ещё
    не реализованы).

- **Sub-increment 3 (UI actions + pause flow) — DONE (2026-08-03).**
  - Contracts (`crates/contracts/src/input/`): action IDs
    `nextengine.action.{ui-nav,ui-confirm,ui-back}`, control paths
    keyboard `up/down/left/right/return/escape`, context IDs
    `nextengine.input-context.{ui-menu,ui-dialogue}` + stack IDs.
    `core_keyboard_mouse_v1()` — 9 actions: ui-nav (Vector2Q15, стрелки,
    contexts ui-menu/ui-dialogue), ui-confirm (Digital, Return, те же
    contexts), ui-back (Digital, Escape, contexts
    gameplay/ui-menu/ui-dialogue). `InputContextStackV1::{ui_menu_v1,
    ui_dialogue_v1}` (priority 200, CaptureAll — модальный слой);
    gameplay stack допускает ui-back (pause из gameplay).
  - Mapper (`runtime/engine/ingress.rs`): ui-nav/ui-confirm/ui-back —
    validate-only slots (как camera-orbit): receipts Accepted, мировых
    команд нет. UI actions идут через общий ingress в canonical frame —
    replayable evidence (решение 7A; pause — lifecycle request, не world
    command). Modal transition в production — revisioned reconfiguration
    того же stack identity через `activate_player_input_configuration`
    (отдельные stack IDs — для отдельных controller bindings).
  - Reference-game driver: `ui_suspend_causal_hash` детектирует
    committed ui-back(Started, Digital(true)) в resolved frame; hash =
    sha256 canonical frame — causal evidence на
    `ValidatedReferenceGameAdvance`. Pause-menu records
    (`reference-game/src/ui.rs::pause_menu_semantic_ui_records`):
    surface `nextengine.ui.surface.pause-menu`, title Label + кнопки
    resume/save/load (Button) с affordances ui-confirm/ui-back/ui-nav;
    публикуются в snapshot приостанавливающего тика.
  - Coordinator (решение 3B):
    `suspend_from_committed_ui_action_with_prepared_run` — тот же
    `ApplicationLifecycleRequestV1` → `Suspended` через существующий
    transition machine + forced durable checkpoint, causal =
    `CausalInputSourceKindV1::PlayerAction` (существующее closed value)
    + canonical frame hash, reason `nextengine.session.ui-pause-requested`.
    Wired в оба live advance paths (`advance_reference_game_live_admitted`,
    `..._presentation_shared_observed_admitted`). Новых lifecycle edges нет.
  - Save/load (решение 8A): save = forced checkpoint на suspend
    (production, автоматически); resume — существующий platform
    ResumeRequested path (в Suspended тиков нет); load — существующий
    session restore из durable checkpoint. Affordances save/load
    задекларированы в pause-menu records; активация пунктов меню —
    host/platform обязанность (context switching — sub 4 scope).
  - Tests: contracts 18/18 (ui actions/contexts, modal stacks);
    runtime mapper test (ui-back Accepted + ValueOutOfProfile negative,
    menu stack rev-2 activation, ui-confirm/ui-nav Accepted без
    команд); application integration
    `ui_back_player_action_suspends_through_the_declared_lifecycle_path`
    (Escape → Suspended + forced checkpoint, causal PlayerAction,
    pause-menu records в snapshot, resume → Active → advance).
  - Deferred (честный scope): dialogue-choice command
    (ui-confirm→AdvanceDialogue) — требует арбитрации контекста, sub 4;
    pause policy declaration в admitted project composition (SPEC-18
    §UI) — в composition contracts поля нет, это semantic change к
    SPEC-17 с ADR, отложено на sub 4/5.
  - Checks: `host-check` PASS, `play` PASS, `persistence-replay` PASS;
    `content-package`, `platform`, `performance` — NOT_RUN (не
    затрагиваются этим increment).

- **Sub-increment 4 (Localization) — DONE (2026-08-03).**
  - ADR-044 (Accepted): typed `TextCatalogV1` как отдельный neutral content
    schema (решение 2A); catalog-declared fallback chains (ровно один
    source-locale root, уникальные locales, ацикличность, fail-closed);
    `PresentationOnly` semantic class; deterministic resolution (chain walk
    first-hit; SignedInteger → plain decimal; TextId → рекурсия depth ≤ 8 +
    cycle detection; missing → `LOCALIZATION_RESOURCE_MISSING` + placeholder
    `[{text_id}]`). SPEC-24 обновлён до v1.1 (секция
    `nextengine.content.text-catalog`, maximums 4096 entries / 1024 bytes
    template); routing table и traceability (4.2) обновлены в том же change.
  - Contracts `crates/contracts/src/localization.rs`: `TextCatalogV1`
    (schema `nextengine.content.text-catalog`, role NeutralContent,
    CanonicalBinaryV1, owner `nextengine.assets`), `TextLocaleTagV1`
    (lowercase ASCII BCP-47-shape, fail-closed без нормализации регистра),
    `TextCatalogEntryV1` (NFC template, placeholders `{0}`..`{15}` без
    escape-формы в v1), `TextCatalogErrorV1`, canonical codec
    (7 полей: version/asset/revision/locale/optional fallback/entries/
    content_hash) с round-trip и tamper проверками. Tests 8/8.
  - Cook (`next_project::cook_project_v1`):
    `NeutralProjectSourceV1.text_catalogs`; blobs + manifest entries с
    `ContentSemanticClassV1::PresentationOnly` (без dependency edges);
    schema registry пополняется text-catalog schema ref только при наличии
    catalogs; `validate_text_catalog_closure` (unique locale, ровно один
    root, fallback targets существуют, ацикличность) →
    `ProjectCookError::LocalizationClosureInvalid`
    (`LOCALIZATION_CLOSURE_INVALID`) / `MissingReference` /
    `DuplicateIdentity`. Activation декодирует text-catalog blobs через
    typed branch, проверяет asset/schema_ref/hash/semantic_class и несёт их
    в `ActivatedProjectV2.text_catalogs` с повторной closure-валидацией в
    `validate()` (defense in depth).
  - Reference catalogs (`reference-game/src/source.rs`): `en` root
    (8 entries: 6 UI text IDs HUD/pause-menu + 2 quest-state TextId target)
    и `qps-ploc` pseudo-locale (7 entries, fallback `en`; намеренно опущен
    `pause-menu.load` для детерминированного fallback). Asset IDs
    0x91/0x92; cook reference project теперь публикует 22 manifest entries.
  - Resolver (`crates/presentation/src/text.rs`):
    `TextCatalogResolverV1::new(catalogs, requested_locale)` ре-валидирует
    closure при загрузке; `resolve(&UiTextRefV1) -> TextResolutionV1`
    (text + `Option<LocalizationDiagnosticV1>` со стабильным code);
    missing requested locale → старт с root + `requested_locale_missing()`;
    missing arg → literal placeholder + diagnostic; TextId recursion с
    path-based visited и depth limit 8. Tests 7/7. Integration smoke в
    `live_presentation_publishes_typed_semantic_ui_hud_from_rpg_state`:
    en resolver ("Health 100/100", "Quest: Available"), pseudo resolver
    ("⟦Ħēåłŧħ⟧ 100/100", fallback "Load game").
  - Deferred (честный scope): plural forms и date/unit formatting —
    будущая schema version (в v1 только SignedInteger/TextId args); выбор
    locale и `PlayerPreferenceProfile` — sub 6; потребление resolved text
    виджетами/rasterizer — sub 5; dialogue-choice arbitration и pause
    policy declaration (перенесены из sub 3 deferred) — sub 5.
  - Checks: `host-check` PASS, `content-package` PASS (records 22);
    `play`, `persistence-replay`, `platform`, `performance` — NOT_RUN
    (gameplay/persistence flows не затронуты; content-package включает
    prepared play frame).

- **Sub-increment 5 (Minimal widget adapter) — DONE (2026-08-03).**
  - Форма (решение 6A): engine-owned immediate-mode overlay внутри
    `desktop-sdl-ash`, contracts widget-free. Semantic UI records → CPU
    rasterizer → одна fullscreen texture → textured quad draw поверх b0
    swapchain image. Никаких vendor UI toolkit зависимостей.
  - Presentation (`crates/presentation/src/ui_font.rs`,
    `ui_overlay.rs`): engine-owned 8x8 bitmap font (ASCII 0x20–0x7E из
    font8x8_basic, public domain, provenance в header; 16 composed glyphs
    псевдо-локали + placeholder box), `rasterize_semantic_ui(records,
    resolver, width, height) -> Option<UiOverlayImageV1>` с
    `content_hash()` (`nextengine.ui-overlay-image.v1`). Layout:
    canonical sort по (surface, panel, element), стек панелей от (8,8),
    cell 16px, text scale 2, Meter = text row + bar, selected →
    highlight, !enabled → dim, !visible → skip, source-over blend integer
    math. Golden hashes на HUD/pause fixtures; tests 26/26.
  - GPU (`desktop-sdl-ash/src/gpu_content/ui_overlay_gpu.rs`,
    `pipeline.rs`): `RasterFixedStateV1` (blend/depth/cull) + отдельный
    `UI_OVERLAY_RASTER_FIXED_STATE` (src-alpha blend, no depth, cull
    none); self-contained `UiOverlayGpu` (shared b0 shaders, identity
    frame uniform, NDC quad 6 verts, NEAREST sampler) и `UiOverlayState`
    (resolver + gpu + counters frames/updates/failures). Overlay key =
    domain_hash над extent + canonical record hashes; key match → skip
    re-raster; key принимается один раз даже при failure (bounded
    fallback, failures += 1, кадр не падает). Texture swap через
    device-idle recreate + upload с тем же leak-правилом, что и b0.
  - Wire-in: `DesktopRunOptions.ui_text_catalogs` / `ui_locale` (пустой
    список = overlay off), `DesktopRunReport.ui_overlay_frames /
    updates / failures`; `InteractiveWorkerReadyV1.text_catalogs`;
    `apps/game` передаёт activated catalogs (locale "en"); swapchain
    format change → `ui_overlay.recreate`; teardown перед destroy_device;
    allocation stats включают overlay.
  - Verification fixture: `hud_semantic_ui_records_for_ids` (pub из
    reference-game без session handle); play-check fixture извлекает
    snapshot через `extract_with_cameras_and_semantic_ui` с HUD records и
    несёт `PreparedGameFrameV1.text_catalogs`. Platform candidate и
    frame-timing smoke передают catalogs в options и требуют
    `ui_overlay_failures == 0`, `ui_overlay_frames == rendered frames`,
    `ui_overlay_updates >= 1` — реальный draw path (raster → upload →
    draw) покрыт обязательным check.
  - Инфраструктурный fix в том же change: debug soak harness
    (`performance --scenario interactive-frame-soak`) начал падать со
    stack overflow на main thread после роста inline state в Semantic UI
    серии (UiOverlayState ≈ 9 КБ в GraphicsContext ≈ 22 КБ; Windows MSVC
    default reserve 1 MiB). Бисect: 93e3519 (до серии) — проходит,
    7ac7cb5 — падает; промежуточные per-crate коммиты workspace не
    собирают. xtask `build.rs` теперь резервирует 8 MiB main stack
    (`/STACK:8388608`, только windows-msvc tooling binary; Linux
    неизменён). После fix soak проходит 240/240 frames с
    `vulkan_timestamp_queries = 480` и новыми overlay assertions.
  - Deferred (честный scope): centering/anchors и выбор text scale;
    mouse hit-testing; inventory/dialogue surfaces (records уже
    рендерятся generic path); locale selection и
    `PlayerPreferenceProfile` (sub 6); dialogue-choice arbitration и
    pause policy declaration (sub 6).
  - Checks: workspace tests PASS (desktop-sdl-ash 55, presentation 26);
    `host-check` PASS; `play` PASS; `platform` PASS
    (`sdl_ash_candidate: PASS` с `--features desktop-sdl-ash`, overlay
    counters asserted); `performance --scenario interactive-frame-soak`
    — workload завершается (240 frames, overlay assertions PASS),
    hard verdict честно NOT_RUN (debug, некалиброванный host).
