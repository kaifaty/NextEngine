# План применения оптимизаций из ресерча 2026-08-02

Дата: 2026-08-02. Статус: **superseded planning snapshot**, не ADR и не
roadmap commitment.
Источники: [optimization-approaches-research-2026-08-02](../development/optimization-approaches-research-2026-08-02.md)
(toolbox техник и алгоритмов), [performance-research-2026-08-01](../development/performance-research-2026-08-01.md)
(измеренные hotspot'ы), [2026-08-01-remaining-engine-performance-goal-prompt](2026-08-01-remaining-engine-performance-goal-prompt.md)
(decision protocol и фазы 1–7), `docs/roadmap.md` (этапы R0–R8, блокеры
B-04/B-06/B-07/B-12, отложенные workloads D1–D6).

> **Historical notice (2026-08-09).** R3 завершён через bounded consumer из
> ADR-051 без promotion SPEC-23 или generic scheduler/resource framework.
> Текущий R4 порядок задают `docs/roadmap.md`, SPEC-20 и ADR-052. Упоминания
> обязательной реализации SPEC-23 в R3, authoritative priority queue, JPS,
> flow fields или других алгоритмов ниже являются только исследовательским
> toolbox и не разрешают implementation до конкретного production consumer и
> измеренного ограничения.

## Принцип плана

Оптимизации применяются **внутри roadmap work packages**, а не отдельным
параллельным мега-проектом. Roadmap прямо требует: «Work package 4 не следует
начинать как универсальный scheduler design без package 3 и конкретного
streaming workload» — тот же принцип распространяется на весь toolbox:
каждая техника внедряется тогда, когда production workload её требует и
измеряет. Исключения — уровень A (дисциплины нулевой стоимости) и уровень C
(отдельные evidence-driven increments по существующему goal prompt).

## Три уровня применения

### Уровень A — непрерывный фон (бесплатные дисциплины, начиная сейчас)

Не требуют отдельных work packages, применяются как review checklist при
любой работе над затронутым кодом:

1. **Каноническая итерация.** Никакой итерации по HashMap в authoritative
   порядке; BTreeMap или explicit sort по ID до обхода (ресерч §4).
2. **Никакого FMA/fast-math/target-cpu=native** в authoritative codegen;
   float-операции, влияющие на state root, — IEEE-точные, без contraction
   (ресерч §8).
3. **Батчинг вместо per-entity callbacks** на новых системных путях
   (events, projections, queries) (ресерч §1, §8).
4. **Auto-vectorization-friendly форма циклов** в новых числодробилках:
   итераторы, `chunks_exact`, SoA-колонки, без index-loop bounds checks
   (ресерч §8).
5. **Никаких ручных SIMD intrinsics / unsafe** без отдельного Accepted ADR
   (существующая граница workspace).
6. **Канонический tie-break в любом поиске** (pathfinding, planner,
   scheduling): равные стоимости — по (ID, direction order), не по порядку
   обхода (ресерч §7).
7. **Канонизация порядка пар/результатов до merge/solver** — сортировка по
   ключу ID на commit boundary, worker count не входит в результат
   (ресерч §3, §6).

Контроль: code review checklist; никаких отдельных commits «причесать весь
репозиторий» — только вместе с material changes в затронутой области.

### Уровень B — встраивание в roadmap work packages

Таблица привязки техник к планируемым пакетам (roadmap «Ближайшая очередь» и
этапы R2–R5):

| Roadmap package | Техники из ресерча | Секция | Gate для применения |
|---|---|---|---|
| Playable alpha project (R2) | Dirty tracking/change detection на presentation extraction и save delta; allocation traffic (arena, SmallVec, hashbrown) по profiler evidence | §1, §4 | Production 20–30 min slice существует; ordinary tick attribution из goal prompt фаз 1–3 |
| Jobs/resources vertical (B-04, R3) | Deterministic reduction: per-shard immutable buffers + canonical merge на commit stage; persistent bounded worker pool (возврат parallel I/O только здесь, не per-checkpoint threads — negative result 08-01) | §3 | Content cook/stream use case как первый клиент; SPEC-23 классы реализованы production owners |
| General partition (B-06, R3) | Interest-rank как canonical input (не wall clock); staged streaming с due-эпохами | §2 | Multi-region save/restart scenario |
| Living-world vertical (B-07 → R4 100-NPC) | Entity sleeping + due-очередь (Factorio 40×); time slicing `agent_id % N`; simulation LOD tiers; event-driven off-screen advance | §2 | Integrated 100-NPC scenario (16 active / 32 near / 52 background), ADR-016 budgets 8 000/12 000 µs |
| Baseline navigation (R4) | Deterministic graph/tile baseline (roadmap decision) → JPS на grid-слоях, flow fields для групп с общей целью, path caching, async pathfinding через job classes | §7 | Navigation content/query baseline существует (hard blocker R4) |
| Population/schedules (R4) | Hungarian assignment для work/roles; Fenwick/2D BIT для influence/economy aggregates; difference arrays для зон эффектов по времени | §5 | Faction/membership operations и authored schedules в scenario |
| Physics stack (R5) | Broadphase (SAP с temporal coherence либо LBVH+Morton), канонизация пар до solver, DSU sleeping islands, warm starting на 120 Hz substep | §6 | Production physics/motor stack и physics owner state в save/replay |
| Content cooker/tools (R3, R6) | SCC+topo validation графов зависимостей (quest/recipes/load order) при cook = fail closed; Aho-Corasick для text pipeline; Mo's algorithm / sqrt decomposition для offline replay-аналитики (OfflineTool class) | §5.9, §5.10, §5.13 | Cook/validate pipeline и инспекторы существуют |
| Renderer при real content (R3+) | GPU culling/bindless (из заметки 08-01, тир 2) | — | R2+ контент делает renderer измеримым bottleneck; сейчас GPU p95 10 µs — не трогать |
| Codegen (после D5) | ThinLTO → PGO по performance-codegen.md | 08-01 §5 | Ten-run hard calibration (D5) |

### Уровень C — отдельные evidence-driven performance increments

Продолжение существующего goal prompt (2026-08-01), фазы 3–6: итеративное
снятие hotspot'ов ordinary tick, checkpoint, worker/render handoff по
decision protocol (3 before/3 after, правило 5%/10%, exact roots, один
atomic commit на гипотезу). Toolbox уровня B пополняет список кандидатов для
этих фаз (allocation traffic, dirty tracking, encode caching), но не меняет
сам protocol.

## Поэтапный план

### Этап 0 — сейчас (R1, Semantic UI не блокируется)

1. Принять уровень A как review checklist (этот документ — фиксация).
2. Инструментация: Tracy/puffin-зоны по 12 нормативным стадиям SPEC-02 в
   performance tooling как telemetry (SPEC-23: measured ≠ authoritative).
   Отдельный commit `perf(tooling): stage-level tracing zones`; checks:
   `fast`, profiler on/off root parity, host-check.
3. Оценить iai-callgrind/gungraun (instruction counts) как шумоустойчивое
   дополнение к paired same-hour A/B на нагруженной reference-системе.
   Результат — короткая заметка в `docs/development/`, не gate.

Ничего больше на этом этапе не оптимизировать: R2–R5 workloads `NOT_RUN`,
любые микро-оптимизации — риск шума (дважды подтверждено в 08-01).

### Этап R2 — playable alpha project

Вместе с пакетом «Playable alpha project»:

1. Dirty tracking на presentation extraction (версии компонентов/строк) —
   O(dirty) вместо O(world) на snapshot; кандидат в goal prompt фазу 3.
2. Allocation traffic audit на новом контенте: per-tick arena (bumpalo) для
   transient tick data, SmallVec на events/contacts, hashbrown/foldhash на
   hot maps — только по profiler evidence, один hotspot = один commit.
3. Активировать D1 (R2 alpha render workload) когда slice существует.

Критерий выхода: ordinary tick и checkpoint не деградируют на реальном
контенте против текущего fixture; D1 даёт честный `REPORT_ONLY`/`PASS`.

### Этап R3 — jobs/resources vertical + general partition (B-04, B-06)

1. Реализовать SPEC-23 substrate с deterministic reduction из ресерча §3:
   dispatch parallel, per-shard immutable results, merge только на declared
   commit stage в exact `(commit_stage, owner_id, request_id, result_hash)`
   порядке (уже норма SPEC-21/23 — план фиксирует приём реализации).
2. Persistent bounded worker pool по ADR-026; только после него пересмотреть
   parallel encode/read в checkpoint path (отложенный negative result из
   приложения 2 заметки 08-01).
3. JOB-P1/RESOURCE-MEMORY-P1/RESOURCE-RESIDENCY-P1/IO-BACKPRESSURE-P1 как
   выходные checks пакета (уже в SPEC-23); workers 1/2/8/16 permutations с
   exact roots.
4. Активировать D2 (R3 streaming workload) с budgets: host ≤ 12 GiB,
   device ≤ 5.5 GiB, commit p95 ≤ 2 000 µs.

Критерий выхода: B-04 и B-06 закрыты по roadmap-критериям; canonical merge
доказан replay parity на worker permutations.

### Этап R4 — living-world vertical → integrated 100-NPC (B-07)

Порядок внутри этапа — от дешёвого к дорогому, каждый шаг отдельным
increment с `play` + `persistence-replay`:

1. **Sleeping/due-очередь** для population agents и периодических mechanics:
   сущность без работы до эпохи X живёт в priority queue due-эпох
   (authoritative, канонический tie-break по ID), не в per-tick итерации.
   Ожидаемый эффект по аналогии Factorio — порядок величины на фоновом
   населении.
2. **Time slicing:** perception/planner/path recompute размазаны
   `slot % N == tick % N`; declared deterministic cadence reduction по
   ADR-016, никогда не starvation (R4 критерий).
3. **Simulation LOD tiers:** `Dormant/Abstract/Simulated/Active` уже в R4
   scope — реализовать tier-переходы как canonical решения из
   world-interest rank (SPEC-25), proxy-симуляция фоновых NPC по
   O'Halloran-паттерну (ресерч §2.1); unsupported abstract outcome upgrades
   or defers, never fabricates (R4 критерий).
4. **Event-driven off-screen advance:** мир вне интереса — скачки к
   следующему due-событию; stepped и bulk world time сходятся на одинаковых
   boundaries (R4 критерий).
5. **Navigation:** deterministic graph/tile baseline → JPS на grid-слоях
   (10× класс), flow fields для групп с общей целью (O(1) на агента),
   path caching, recompute только по изменению nav-данных; pathfinding как
   AuthoritativeCompute job с каноническим tie-break.
6. **Доменные структуры:** Hungarian для work assignment (малые n), Fenwick/
   2D BIT для influence/economy aggregates, difference arrays для зон
   эффектов по времени — внедрять по одному, когда соответствующая mechanics
   feature входит в scenario.
7. Активировать D3 (100-NPC workload): population mix 16/32/52, integrated
   p95 ≤ 8 000 µs, p99 ≤ 12 000 µs, exact roots across restart/worker
   permutations.

Критерий выхода: все R4 «Критерии успеха» из roadmap; D3 `PASS` (или честный
`REPORT_ONLY` до D5).

### Этап R5 — physics/motor

1. Broadphase: SAP с temporal coherence (инкрементальная сортировка) либо
   LBVH+Morton для детерминированного rebuild; выбор — по измерению на
   representative scene, за engine-owned API (SPEC-26 technology neutrality).
2. Канонизация порядка контактных пар (sort по `(id_a, id_b)`) до
   narrowphase/solver — обязательна независимо от broadphase.
3. DSU sleeping islands + velocity thresholds на фиксированном substep;
   warm starting/contact caching как часть deterministic physics state
   (SPEC-26 canonical snapshots).
4. Spatial queries (ClosestPoint и далее) на BVH с early-out при росте мира.
5. Активировать D4 (16-avatar workload): physics p95 ≤ 4 000 µs,
   p99 ≤ 6 000 µs.

### Этап R6–R7 — cooker/tools/codegen

1. SCC+topo validation при cook: цикл в обязательных зависимостях
   (quest graph, recipes, package load order) = стабильная ошибка до
   publication.
2. Aho-Corasick в text-canonical pipeline/tools; Mo's/sqrt для offline
   replay-аналитики (OfflineTool).
3. D5 ten-run calibration → затем D6 ThinLTO/PGO promotion по
   performance-codegen.md (отдельный commit, никогда не смешивать с
   алгоритмическими оптимизациями).
4. GPU culling/bindless — только когда R2+ контент сделал renderer
   измеримым bottleneck.

## Per-change protocol (для всех уровней B и C)

Используется существующий Performance decision protocol из goal prompt
2026-08-01 без изменений:

1. Baseline + profiler attribution на production path; synthetic microbench —
   только локализация, не verdict.
2. Один hotspot = одна гипотеза = один changeset = один atomic commit.
3. Минимум 3 before / 3 after `REPORT_ONLY` runs одинаковой методикой; на
   нагруженной reference-системе — paired same-hour A/B; outlier removal и
   retry-to-green запрещены.
4. Acceptance: targeted median run-p95 ≥ 5% improvement, либо exact
   allocation/I/O work −10% при timing в noise band <2%; ни одна critical
   p95 не хуже ≥5%.
5. Exact authoritative roots, ledger/archive parity, profiler on/off parity,
   save/replay equivalence — обязательны независимо от timing.
6. Не прошёл правило — аккуратный откат собственных изменений, запись вывода
   в research-заметку, следующий hotspot.
7. Изменение Accepted semantics (encoding, snapshot format, cadence, merge
   order) — сначала superseding ADR + SPEC/schema/migration/traceability в
   том же change, реализация отдельным commit.
8. Product checks по затронутой области: `fast` всегда; `play` для gameplay/
   runtime; `persistence-replay` для state/scheduling; `content-package` для
   cooker; `performance` для material hot-path; `platform` для renderer/OS.

## Матрица рисков детерминизма

| Риск | Техника | Закрытие |
|---|---|---|
| Worker count влияет на результат | Параллелизм (R3) | Canonical merge на commit stage; JOB-P1 permutations 1/2/8/16 с exact roots |
| Порядок контактов зависит от истории вставок | Physics (R5) | Sort пар по ID до solver; canonical snapshots |
| Пути NPC расходятся replay/live | Navigation (R4) | Tie-break по (node_id, direction order); JPS/flow field как чистые функции |
| FMA меняет float между хостами | SIMD/codegen | Запрет +fma в authoritative crates; fixed-point где нужна бит-идентичность |
| Sleeping/LOD фабрикует результат | Population (R4) | Tier-правила canonical; unsupported outcome → upgrade/defer, never fabricate |
| Arrival order/wall clock как input | Jobs/streaming | SPEC-21/23 уже запрещают; review checklist уровня A |
| Кэш меняет accepted result | Любые кэши | Только reconstructible state; restart/corruption tests обходят cache |

## Scope guard / non-goals

- Не создавать отдельный «optimization epic» вне roadmap packages; не
  расширять bounded tasks ради закрытия чужих блокеров (AGENTS.md).
- Не оптимизировать renderer (GPU p95 10 µs), checkpoint pipeline сверх
  уже сделанного (08-01) и allocator counter path (candidate-6/7 FAIL,
  metric disabled) без новой material hypothesis.
- Не вводить unsafe/SIMD intrinsics, новый allocator, target-cpu=native,
  BOLT, default LTO/PGO без соответствующего Accepted ADR и evidence.
- Не синтезировать R2–R5 workloads в tooling раньше продукта (D1–D4
  prerequisites).
- Не применять CP-структуры «для красоты»: каждая — под конкретный горячий
  запрос production scenario; при отсутствии запроса — не внедряется.
- Не переводить `REPORT_ONLY` в `PASS` без полного ADR-036 протокола.

## Критерии завершения плана

План считается исполненным по факту этапов: уровень A действует постоянно;
уровень B закрыт вместе с roadmap-критериями соответствующих пакетов (B-04,
B-06, B-07, R4, R5) и активацией D1–D6; уровень C — по терминальному
условию goal prompt 2026-08-01. Roadmap обновляется обычным порядком после
завершения каждого этапа; этот план не переводит ни один gate и не закрывает
ни один blocker сам по себе.

## Traceability

| Техника | Ресерч § | Этап плана | Roadmap/блокер |
|---|---|---|---|
| Dirty tracking, SoA, chunk sizing | §1 | R2 | ordinary tick, checkpoint |
| Simulation LOD, sleeping, time slicing, event-driven | §2 | R4 | B-07, R4 100-NPC, ADR-016 |
| Deterministic reduction, canonical merge | §3 | R3 | B-04, SPEC-21/23 |
| Arena, hashbrown, SmallVec, interning | §4 | R2/уровень C | B-12 enabled metric |
| Fenwick, DSU(+rollback), sparse table, monotonic queue, diff array, binary lifting, Hungarian, Dinic, SCC/topo, Aho-Corasick, bitmask DP, CHT, Mo's, binary-search-on-answer | §5 | R4/R6 | faction/economy, physics islands, quest validation, tools |
| SAP/LBVH, islands, warm start | §6 | R5 | SPEC-26 physics stack |
| JPS/CJPS, flow fields, HPA*, caching | §7 | R4 | Baseline navigation |
| Auto-vec, safe SIMD, FMA ban | §8 | уровень A | все числодробилки |
| Tracy/puffin, iai, p95/p99, replay-bench | §9 | Этап 0 | B-12, THOTH |
