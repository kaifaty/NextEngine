# Ресерч: подходы и практики оптимизации игровых движков, применимые к Next Engine

Дата: 2026-08-02. Статус: исследовательская заметка, не ADR и не roadmap commitment.

Связанный документ: [performance-research-2026-08-01](performance-research-2026-08-01.md)
отвечает на вопрос «где наши измеренные горячие точки и рычаги» (checkpoint
pipeline, B-04 job substrate, allocation traffic, PGO). Эта заметка отвечает на
соседний вопрос: «какой внешний toolbox техник и алгоритмов существует и куда
он ложится в наши подсистемы». Пересечения с заметкой 08-01 даны ссылками, без
дублирования. Замечание по методологии: QMD-поиск по `nextengine-docs` в этой
сессии был недоступен; использован задокументированный fallback — прямые чтения
SPEC-02, SPEC-23, roadmap и заметки 08-01.

## Карта применимости (сводка)

| Техника | Подсистема Next Engine | Связанный blocker / roadmap | Цена | Ожидаемый эффект | Риск детерминизма |
|---|---|---|---|---|---|
| SoA + chunk sizing + sparse-set для churn-компонентов | ECS storage (SPEC-02, facade) | R3–R5 systemic load | Средняя | ~2× throughput против OOP baseline | Нет (layout — приватная деталь) |
| Change detection / dirty flags | Проекции, save delta, presentation extraction | Checkpoint hot path | Низкая | Срезает O(world) до O(dirty) | Нет |
| Simulation LOD / interest tiers | SPEC-20 population, SPEC-25 world partition | B-06, R4 100-NPC | Высокая | Порядки на off-screen населении | Средний — нужны канонические tier-правила |
| Entity sleeping / wake-on-event | World sim, physics (SPEC-26), population | R4, checkpoint tick | Средняя | 40× на спящих (Factorio-практика) | Низкий — пороги сна детерминированы |
| Time slicing / staggered updates | AI planner, perception, schedules | R4 100-NPC | Низкая | Линейное размазывание пиков | Низкий — канонический round-robin |
| Детерминированный параллелизм + canonical merge | SPEC-23 job substrate | B-04 | Высокая | Кратный на multi-core | Решён design'ом SPEC-21/23 |
| Arena/pool/inline storage | Runtime, events, staging | B-12 enabled metric | Средняя | 2–5× на alloc-трафике, −N аллокаций | Нет |
| hashbrown/foldhash, SmallVec, bitvec | Весь core | Ordinary tick 2 278 µs | Низкая | ~2× на map-heavy путях | Нет (итерацию — только каноническую) |
| Auto-vectorization-friendly код, safe SIMD (`wide`) | Physics, queries, projections | Physics partial | Средняя | 4–8× на числодробилках | FMA-ловушка (см. §7) |
| SAP / spatial hash / LBVH broadphase | SPEC-26 physics world, queries | Physics gap | Высокая | 8×+ на collision pairs | Низкий — порядок пар канонизировать |
| Sleeping islands + warm starting | SPEC-26 solver | Physics gap | Средняя | ~2× CPU на покоящихся сценах | Низкий |
| JPS / CJPS / flow fields / HPA* | SPEC-08 navigation | Navigation catalog отсутствует | Высокая | 10× на grid pathfinding; O(1) на агента для flow field | Низкий — tie-break канонический |
| «Олимпиадные» СТ (Fenwick, DSU, sparse table и др.) | См. §10 по пунктам | Quest graph, economy, perception, tools | Низкая–средняя | O(n) → O(log n) на горячих запросах | Нет — все структуры чистые |
| Tracy/puffin + replay-бенчмарки | Observability (SPEC-09), THOTH | B-12, R2–R5 | Низкая | Видимость, защита от регрессий | Только telemetry (SPEC-23 это уже требует) |

## 1. Data-oriented ECS: что подтверждено внешней практикой

Наш facade (SPEC-02) не фиксирует storage layout — это свобода, которую стоит
использовать осознанно. Внешний консенсус по production ECS:

- **Archetype+SoA vs sparse-set — не вера, а trade-off по workload.**
  Archetype-таблицы выигрывают на итерации (contiguous columns, zero branching
  во внутреннем цикле), sparse-set — на частом add/remove компонентов.
  Production-ориентир — гибрид Bevy: стабильные компоненты в таблицах,
  churn-маркеры (баффы, transient теги) в sparse-set
  [socratopia ch.9; arXiv 2606.14919]. Свежий формальный benchmark: archetype+SoA
  ≈ 2× throughput против OOP baseline, выше в параллельном варианте
  [arXiv 2606.14919].
- **Chunk sizing под кэш, не под эстетику.** GOKe (deterministic ECS):
  фиксированные memory pages ~96 KB под L1, `[IDs][CompA][CompB]` внутри
  чанка, рост «ступеньками» малых аллокаций вместо doubling большого массива —
  убирает latency-спайки на миллионной сущности [goke README].
- **Generational IDs** (32-bit index / 32-bit generation) — стандарт защиты от
  stale references; у нас уже есть generational `RuntimeEntityId` (SPEC-02).
- **Bitmask signatures + кэш matched archetypes.** Query = bitwise AND по
  маскам; matched-set кэшируется один раз, а не пересчитывается на тик
  [oboe.com; wild.codes].
- **Deferred structural changes через command buffer на stage boundary** —
  у нас это уже норма архитектуры (WorldCommand transactions, stage 9/10).
- **Change detection / dirty tracking** (версии на компонент/строку) — самый
  недооценённый приём: проекции, save delta и presentation extraction должны
  платить O(dirty), а не O(world). Стыкуется с тир-1 checkpoint-рычагом
  заметки 08-01 (field/chunk-level dirty tracking для materialization).

## 2. Масштабирование симуляции системного RPG

Главная специфика нашего жанра: сотни NPC, schedules, off-screen население,
persistent мир. Классические практики:

### 2.1 Simulation LOD / interest management

Многоуровневая симуляция NPC: full sim рядом с игроком, дешёвый proxy
(statistical/aggregate) вне зоны интереса. Диссертация O'Halloran (TCD, 2016):
framework с per-entity LOD-switch через добавление/снятие компонентов; 1000
агентов на full LOD — 588 fps (1.7 ms/frame), 2500 агентов на proxy LOD —
real-time; ML-proxy предсказывал исход столкновений с ~85% accuracy; SetLOD
0.27 ms/call — предлагается оптимизировать object pooling [TCD-SCSS-2016-055].
Для нас это ложится прямо в SPEC-20 population tiers / ADR-021 residency и
SPEC-25 world-interest rank: tier-переходы у нас уже канонические решения, а не
ad-hoc радиусы — это сильнее, чем у типичных движков.

### 2.2 Entity sleeping / wake-on-event (Factorio-практика)

Factorio FFF-421: roboports «выключены», пока не нужны (робот на зарядку и
т.п.) — время на roboports в типичном сейве 1 ms → 0.025 ms/тик (40×).
Измерения сообщества: спящая assembler/фурнитура стоит 0 ns/тик, активная
~100–500 ns [forums.factorio.com t=108144]. Принцип: **сущность без pending
работы не должна обходиться schedule'ом вообще** — только wake-set по событиям.
Для нас: physics bodies (SPEC-26), population agents, периодические mechanics —
всё, что имеет понятие «нет работы до эпохи X», должно жить в priority queue
due-эпох, а не в per-tick итерации. Детерминизм сохраняется: пороги сна и
wake-события — функции authoritative state.

### 2.3 Time slicing / staggered updates

Не обновлять всё каждый тик: группа K из N обновляется на тике `tick % N == K`.
Практика navigation/AI: path recompute, perception, utility scoring размазываются
по тикам каноническим round-robin (agent_id % N), плюс budget на тик
[daydreamsoft navigation]. Совместимо с нашими fixed stages и ADR-016
budget matrix; порядок выбирается из `PersistentId`/slot, не из wall clock.

### 2.4 Event-driven simulation для off-screen мира

Discrete-event simulation: мир вне интереса продвигается не по тикам, а скачками
к следующему due-событию (прибытие каравана, истощение ресурса). Это
стандартный приём в 4X/симуляциях и прямое продолжение 2.2. Для replay
критично: due-очередь — часть authoritative state (каноническая куча с
tie-break по ID).

## 3. Параллелизм под детерминизм

Полностью разобран в заметке 08-01 (тир 1, п. 2: B-04 substrate, canonical
merge, Almost Deterministic Work Stealing). Добавления с этой волны:

- **Deterministic reduction:** параллельный map — свободно; reduce — только с
  каноническим порядком слияния (sort по shard index / ID до merge), тогда
  worker count не входит в результат. SPEC-23 это уже требует («dispatch MAY be
  parallel... merge only at the declared commit stage in exact order») — приём
  реализации: per-shard частичные результаты в immutable buffers, merge на
  commit stage.
- **Negative result, зафиксированный 08-01:** per-checkpoint thread spawning —
  анти-паттерн на нагруженной системе; ждём persistent bounded worker pool по
  B-04. Не возвращаться.
- Для replay-parity каждую параллелизованную систему гонять через существующий
  RUNTIME-01-стиль сценарий (10 000 ticks × 100 seeds, два повтора) до и после.

## 4. Память и коллекции в Rust

Дополняет тир-2 п. 4 заметки 08-01 (снижение N аллокаций, а не стоимости одной):

- **Arena (bumpalo) для per-tick transient.** Измерения: 2–5× против global
  allocator на batch-lifetime аллокациях [oneuptime.com]; в serde-gauntlet —
  до 5× на миллионе итераций [nickb.dev]. Reset на commit point. Осторожно:
  лайфтаймы арены «виральны» по сигнатурам — держать на границах стадий, не в
  public contracts (SPEC-23 это и так запрещает).
- **hashbrown/foldhash.** Текущий std HashMap = hashbrown+foldhash; при
  использовании hashbrown напрямую — ~2× против старого std, 1 байт overhead
  на entry, SIMD-lookup [rust-lang/hashbrown]. Для hot maps с integer keys
  (PersistentId → slot) — foldhash/fx-подобные hasher'ы.
- **SmallVec/ArrayVec** для маленьких коллекций на hot path (events per
  command, contacts per pair) — inline storage без heap alloc в типичном случае.
- **bitvec/u64-массивы** для visited/occupancy флагов (pathfinding, perception
  grids) — 64× плотнее bool-vec, дружелюбно к SIMD через popcount/AND.
- **Interning строк** (internment) для text-canonical pipeline: один alloc на
  уникальную строку, сравнения по указателю.
- **Каноническая итерация — дисциплина:** любая итерация по HashMap, влияющая
  на authoritative результат, запрещена; либо BTreeMap, либо explicit sort по
  ID до обхода. Это не оптимизация, а защита replay contract.

## 5. «Олимпиадные» структуры данных → игровые применения

Самая недооценённая категория: структуры из спортивного программирования дают
O(n) → O(log n) ровно там, где движки обычно пишут линейные обходы «и так
сойдёт». Все структуры ниже — чистые функциональные/мутабельные алгоритмы без
рандома и wall clock, т.е. по построению детерминированные.

### 5.1 Fenwick (BIT) и segment tree — агрегаты по диапазонам

- **Influence maps / heat maps:** сумма урона, опасности, присутствия фракций
  по прямоугольнику grid-карты за O(log n) (2D BIT/seg tree), точечное
  обновление O(log n). Альтернатива — пересчёт префиксных сумм O(n) на тик.
- **Экономика/логистика фракций:** запасы ресурсов по регионам с range-update
  (поставка каравана = += на отрезке пути).
- **Event analytics по replay:** сумма событий типа X на интервале тиков —
  BIT поверх event log для инспекторов (tools, PresentationOnly).
- Segment tree с lazy propagation — range-add + range-query (баффы/дебаффы по
  зонам). Persistent segment tree — версионные запросы «состояние агрегата на
  тике T» без хранения снапшотов (естественно для replay/inspection).

### 5.2 DSU (disjoint set union) — связность

- **Physics sleeping islands:** DSU по графу контактов = острова за
  O(α(n)); классика Bullet («automatic de-activation», islands). 
- **Разрушаемость/структуры:** отвалившиеся чанки зданий = connected
  components после удаления рёбер.
- **World partition connectivity:** проходимость регионов для population sim.
- **Rollback DSU** (без path compression, union by rank + стек undo):
  откат за O(1) на операцию — идеально ложится на нашу модель транзакций:
  speculative раскладка внутри команды с откатом без полной копии.
  Обычный DSU с path compression откатывать нельзя — это важный нюанс
  реализации.

### 5.3 Sparse table — статический RMQ

Min/max на неизменном массиве за O(1) запрос после O(n log n) build. Применения:
высоты террейна на отрезке (LOS-оценки, ballistic clearance), статические
cost-таблицы навигации. Для изменяемых данных — seg tree/Fenwick.

### 5.4 Monotonic queue / stack, sliding window

- Sliding-window min/max по временным рядам (телеметрия THOTH, frame times,
  audio envelopes) за O(n).
- Next-greater-element (monotonic stack) — «ближайший следующий более высокий
  объект» для visibility/стайлинга препятствий на профиле террейна.

### 5.5 Difference array — массовые отложенные range-update

K обновлений «+= на [l, r]» за O(K + n) вместо O(K·n): зоны эффектов по
времени (aura началась/кончилась), плановые world events по тикам. Применяется
на stage boundary — канонично и дёшево.

### 5.6 Binary lifting и LCA — иерархии

- Scene/attachment-графы, skeleton hierarchies: k-й предок, LCA двух узлов за
  O(log n) после O(n log n) preprocess — например, «общий родитель квестовых
  узлов», наследование флагов по дереву регионов.
- Heavy-light decomposition: агрегаты «на пути в дереве» (влияние по цепочке
  командования фракции) за O(log² n).

### 5.7 Hungarian algorithm — назначение

O(n³) optimal assignment: распределение NPC по работам/ролям (RimWorld-style
work assignment), выбор целей в combat AI, размещение encounters. Для
относительно малых n (десятки) — практично даже каждый decision tick; для
больших n — greedy + локальные улучшения.

### 5.8 Max flow (Dinic) — логистика

Пропускная способность торговых путей, supply chains между поселениями,
эвакуация/миграция населения. Dinic — практичный выбор (простая реализация,
быстрый на игровых размерах). Запускать редко (world tick, time slicing).

### 5.9 SCC + топологическая сортировка — зависимости

Tarjan/Kosaraju SCC + topo sort: quest graph (циклы prerequisite'ов = fail
closed на валидации), crafting recipes, load order мод-пакетов, schedule
system DAG. У нас schedule graph уже сериализуется в build metadata (SPEC-02) —
SCC-валидация при cook: цикл в обязательных зависимостях = стабильная ошибка.

### 5.10 Aho-Corasick / rolling hash — текстовые pipeline

- Aho-Corasick: multi-pattern поиск за O(n + matches) — фильтры контента,
  поиск по диалогам/лору в tools, trigger-слова в text-canonical pipeline.
- Rolling hash (Rabin-Karp): дедупликация content chunks, delta-encoding
  диалоговых пакетов.

### 5.11 Bitmask DP / meet-in-the-middle — малые комбинаторики

- Build/loadout optimization (инвентарь под ограничения: weight, slots,
  set-bonuses) — bitmask DP при ≤ ~20 предметах; meet-in-the-middle при ~40.
- AI-планирование малых боевых сцен, выбор набора abilities под budget.
- Осторожно: exponential по построению — только для малых n, только в
  AuthoritativeCompute job'ах с fuel limits.

### 5.12 Convex hull trick / Li Chao tree — максимум по семейству прямых

Utility AI: score(action) = k·x + b по параметру x (расстояние/здоровье) —
выбор лучшего действия по всем кандидатам за O(log n) вместо O(n) на агента на
тик. Нишево, но при тысячах агентов и десятках скоринг-функций — измеримо.

### 5.13 Sqrt decomposition / Mo's algorithm — офлайн-аналитика

Запросы по диапазонам event/replay логов в инспекторах и tools (OfflineTool
class): Mo's algorithm отвечает на Q range-запросов за O((n+Q)·√n) —
достаточно для отладочной аналитики без построения индексов.

### 5.14 Binary search on the answer — подбор параметров

«Минимальный budget, при котором R4-сценарий проходит»; «максимальный spawn
count под frame budget»; автокалибровка difficulty/LOD-порогов. Параметрический
поиск по монотонному предикату за O(log range) прогонов — дешевле сетки.

## 6. Физика и пространственные запросы (SPEC-26, текущий gap)

Классический стек оптимизаций rigid-body движков (Bullet/ODE feature set:
SAP broadphase, auto de-activation, LCP warm starting [Bullet feature lists]):

- **Broadphase: SAP с temporal coherence** — инкрементальная сортировка по
  осям, обновление за почти O(n) между кадрами вместо полной сортировки;
  альтернативы: uniform grid (плотные равномерные миры), spatial hash
  (неравномерные размеры), LBVH поверх Morton codes (детерминированный rebuild
  за O(n log n), отлично параллелится; z-order curve = чистая битовая магия,
  дружественна к детерминизму).
- **Канонизация порядка пар:** broadphase выдаёт пары в порядке, зависящем от
  layout; до narrowphase/solver — sort по (body_id_a, body_id_b). Иначе
  порядок контактов (и результат solver'а) может зависеть от истории
  вставок — тихий источник replay-расхождений.
- **Sleeping islands** (см. 5.2 DSU) + velocity thresholds на фиксированном
  тике — fixed timestep делает пороги сна консистентными
  [game-mechanics-optimizations #6/#9].
- **Warm starting / contact caching:** solver стартует итерации с прошлого
  substep'а — сходимость за меньшее число итераций; у нас substep 120 Hz
  (SPEC-02) — coherence между substep'ами высокая, warm start особенно
  выгоден. Кэш контактов — часть deterministic state (SPEC-26 canonical
  snapshots это уже предполагают).
- **Spatial queries:** наш текущий `ClosestPoint` — exact scene query; при
  росте мира переводить на BVH с early-out по bounding volumes, а не обход
  всех shapes.

## 7. Навигация и pathfinding (SPEC-08, catalog пока отсутствует)

- **JPS (Jump Point Search)** — symmetry breaking на uniform-cost grids: в
  примерах open-list 18 узлов против 91 у A* [gamedev.net JPS]; улучшения
  Harabor/Grastien (block-based операции, offline preprocessing, новые pruning
  rules) — от нескольких факторов до >10× против базового JPS [ICAPS 2014].
  CJPS (Zhao et al., 2023) устраняет патологии JPS: до 7× на больших игровых
  картах и 14× на патологических случаях [arXiv 2306.15928].
- **Flow fields для массовых агентов:** один BFS/Dijkstra от цели = поле
  направлений; дальше O(1) на агента на тик. Flow field быстрее A* во всех
  тестовых сценариях tower-defense сравнения [ijmra]; идеален для толп NPC с
  общей целью (караваны, рейды, городская рутина).
- **HPA\* (hierarchical):** кластеризация карты, путь «кластер-кластер» +
  локальное уточнение; trade-off — неоптимальность пути и дорогой rebuild при
  динамике [arXiv 2602.04130 обзор].
- **Практики оркестровки:** path caching, инкрементальный recompute только при
  изменении nav-данных, async/off-tick вычисления через наш job substrate
  (ImmutableInput → proposal), LOD для навигации (дальние NPC — грубые пути)
  [daydreamsoft]. Всё это ложится на SPEC-23 классы: pathfinding —
  AuthoritativeCompute с каноническим tie-break (равные f — по (node_id,
  direction order)), иначе пути NPC разойдутся между replay и live.

## 8. SIMD и числодробилки под наш запрет unsafe

Workspace запрещает `unsafe_code` вне узких ADR-границ — это меняет меню:

- **Первый рычаг — код, дружественный к auto-vectorization LLVM:** итераторы
  вместо index loops (нет bounds checks), `chunks_exact(N)` + массивы
  фиксированного размера, SoA-колонки, отсутствие early-exit зависимостей,
  wrapping-арифметика там, где overflow-checks блокируют векторизацию
  [users.rust-lang.org t/43807; emschwartz.me; stackoverflow 73118583].
  Проверка через godbolt с `-O -C target-cpu=native`. matklad: «пишите код в
  форме, которую компилятор может векторизовать» — не ручные intrinsics.
- **Safe SIMD crates:** `wide` (stable, без unsafe в нашем коде, до 256-bit),
  `pulp` (runtime dispatch). Ручные `std::arch` intrinsics — unsafe → только
  через отдельный Accepted ADR, как FFI/backend boundary. До доказанного
  bottleneck — не трогать [kerkour.com советует то же].
- **FMA-ловушка детерминизма:** fused multiply-add меняет результат
  (одиночное округление вместо двойного). Если авторитетная математика f32/f64
  попадёт под FMA (target-feature=+fma, pulp dispatch на машине с FMA против
  машины без), replay разойдётся между хостами. Правила: не включать
  `+fma`-target-feature в authoritative crates; float-операции, влияющие на
  state root, — строго IEEE-точные без contraction; для кросс-платформенной
  бит-идентичности — fixed-point (мы уже используем typed integer/fixed-point
  camera; Subterrans запрещает float в sim целиком, координаты в 1/256 tile
  [subterrans.com]).
- **Батчинг вместо per-entity вызовов:** transforms, projections, physics
  integration — массивами через SoA; event streams батчами, не per-entity
  callbacks [wild.codes].

## 9. Профилирование и культура измерений

- **Frame profiler:** Tracy (`tracy-client`, `tracing-tracy`) — наносекундные
  зоны по стадиям тика; puffin — игровой frame profiler, минимальная
  интеграция [lib.rs profiling catalog]. Всё это — telemetry/diagnostic, что
  согласуется со SPEC-23 («measured... MAY be measured for conformance, but
  MUST NOT select an authoritative result») и существующей THOTH-инфрой.
- **Микробенчмарки:** criterion/divan; **детерминированные бенчмарки:**
  iai-callgrind/gungraun (instruction counts вместо wall time — устойчивы к
  шуму нагруженной машины; хорошее дополнение к paired same-hour A/B
  методологии из заметки 08-01); dhat — heap profiling для allocation-traffic
  работы; perf + inferno flamegraphs на Linux (LNX-005/006).
- **Метрики — перцентили, не средние:** p95/p99 frame/tick times; среднее
  скрывает рывки [rust forum Bevy profiling thread]. У нас уже p95 в THOTH —
  сохранять.
- **Replay как бенчмарк:** headless replay зафиксированных сценариев = готовый
  workload harness (мы это уже делаем soak-runs'ами). Расширить до R2–R5 —
  это и есть exit criterion B-12.
- **CI perf gates:** golden workloads с порогами (практика: p95 budget assert,
  2k entities сценарий) — ловит регрессии за день, а не за релиз [wild.codes].

## 10. Что НЕ делать в наших ограничениях

- Не вводить unsafe ради SIMD/структур данных без Accepted ADR (ADR-002/039).
- Не менять shipping allocator и не ослаблять 3% budget (ADR-036/039) —
  атакуем трафик, не аллокатор (см. заметку 08-01).
- Не включать FMA/fast-math/target-cpu=native в authoritative codegen —
  кросс-хост replay parity важнее процентов.
- Не параллелить без canonical merge; не спавнить потоки на tick path
  (negative result 08-01, приложение 2).
- Не итерировать HashMap в authoritative порядке; не использовать arrival
  order/worker count/wall clock как input решения (SPEC-21/23 уже запрещают —
  держать как review checklist).
- Не оптимизировать вслепую: до R2–R5 ten-run любые микро-оптимизации —
  риск погони за шумом (дважды подтверждено в заметке 08-01).
- Не переносить практики «молча»: presentation-only техники (LOD, culling) не
  должны приобретать gameplay authority (SPEC-00 инвариант).

## 11. Рекомендуемая очередь освоения toolbox'а

Привязка к roadmap/блокерам (не commitment, а предложение порядка изучения):

1. **К B-04 (job substrate, spec-only → implementation):** deterministic
   reduction patterns, per-shard immutable buffers, canonical merge (§3);
   Chase-Lev/ADWS — из заметки 08-01.
2. **К R4 100-NPC systemic load:** simulation LOD/tiers (§2.1), entity
   sleeping + due-очередь (§2.2), time slicing (§2.3), flow fields для
   группового движения (§7), Hungarian для work assignment (§5.7).
3. **К physics gap (SPEC-26):** broadphase SAP/LBVH + канонизация пар,
   sleeping islands через DSU, warm starting (§6).
4. **К navigation catalog (SPEC-08):** JPS/CJPS на grid-слоях, flow fields,
   path caching, async pathfinding через job classes (§7).
5. **К tick cost / B-12:** allocation traffic (bumpalo arena, SmallVec,
   hashbrown — §4) + iai-callgrind как шумоустойчивый бенч (§9).
6. **К quest/faction/content:** SCC+topo validation при cook, Fenwick для
   influence/economy aggregates, Aho-Corasick для text pipeline (§5).
7. **К observability:** Tracy/puffin зоны по 12 нормативным стадиям SPEC-02 —
   прямая проекция нашего system order в трейсинг (§9).

## 12. Источники (получены в этой сессии)

Локальные документы: SPEC-02, SPEC-23, SPEC-00, README (INDEX-001),
docs/roadmap.md, performance-research-2026-08-01.md.

Внешние:

- arXiv 2606.14919 — The Essence of ECS: формализация, SoA ~2× vs OOP,
  archetype storage (Unity DOTS/Bevy/Flecs).
- socratopia.app Game Code Anatomy ch.9 (archetype vs sparse-set, Bevy hybrid
  storage), ch.26 (replay/determinism disciplines), ch.27 (fixed timestep +
  substepping).
- github.com/kjkrol/goke — chunked SoA pages ~96 KB под L1, stepless growth,
  generational 32/32 IDs, archetype masks.
- oboe.com / wild.codes — bitmask queries, chunk sizing 32–128 rows, command
  buffers, CI perf gates, per-system counters.
- github.com/raduacg/game-mechanics-optimizations — fixed timestep (Gaffer),
  quantified impact, synergy с physics sleeping.
- bugnet.io — determinism для replays: fixed step, seeded RNG, ordered
  iteration, record inputs not state.
- subterrans.com — fixed 20 Hz, Mulberry32, integer fixed-point (1/256 tile),
  floats banned in sim.
- and.rew.gold (FPS bots) — shared deterministic core для training/gameplay,
  1.4M steps/s CUDA port с parity-тестами.
- TCD-SCSS-DISSERTATION-2016-055 (O'Halloran) — multi-tier simulation LOD:
  1000 agents @588 fps, 2500 proxy real-time, pooling для SetLOD.
- factorio.com/blog/post/fff-421 — roboports sleeping: 1 ms → 0.025 ms/тик;
  forums.factorio.com t=108144 — per-entity ns/tick измерения, sleeping = 0.
- gamedev.net JPS tutorial; ICAPS 2014 «Improving Jump Point Search»
  (Harabor/Grastien) — до >10×; arXiv 2306.15928 CJPS — до 7×/14× против JPS.
- MDPI Applied Sciences 12(11):5499 — систематический обзор pathfinding.
- ijmra.in v5i9 — flow field быстрее A* во всех сценариях; CVUT thesis —
  flow fields для массовых NPC (BFS-поле, O(1) на агента).
- arXiv 2602.04130 — Recast navmesh + multi-threaded A*, HPA* trade-offs,
  числа по узлам/времени.
- daydreamsoft.com — navigation optimization checklist (caching, LOD,
  async, hierarchical).
- oneuptime.com — arena allocators в Rust, 2–5×; nickb.dev — bumpalo serde
  gauntlet, до 5× на 1M итераций, bump-scope сравнение.
- rust-lang/hashbrown README/releases — foldhash default, ~2× vs старый std,
  1 byte/entry, SIMD lookup, NEON support.
- users.rust-lang.org t/43807, emschwartz.me (hamming auto-vec,
  chunks_exact), stackoverflow 73118583 — правила auto-vectorization в Rust.
- kerkour.com — меню SIMD в Rust (std::simd nightly, wide, pulp, arch),
  совет «не ручной SIMD без доказанного bottleneck».
- lib.rs/development-tools/profiling — каталог: criterion, divan,
  iai-callgrind, gungraun, dhat, puffin, tracy-client, tracing-tracy, inferno,
  perf-event, memory-stats, hotpath.
- users.rust-lang.org t/50062 — Bevy benchmarking обсуждение: p99 frame
  times, cycles/instructions, headless benchmark games.
- Bullet/ODE feature lists (CSDN-зеркала документации) — SAP broadphase,
  auto de-activation (sleeping), LCP warm starting, hash space.
- cp-algorithms-derived skill reference (skillsmp data-algo-competitive) —
  карта структур: segment trees, persistent, Li Chao, rollback DSU, HLD,
  Dinic, Hopcroft-Karp, Aho-Corasick, Mo's algorithm.
