# Требования к обучаемому humanoid-персонажу

| Поле | Значение |
|---|---|
| Статус | Draft requirements baseline; product assumptions require confirmation, все acceptance checks `NotRun` |
| Дата | 2026-08-12 |
| Candidate | `HumanoidFlatRecoveryCandidateV1` (planning identity, не public schema ID) |
| Scope | Fixed-body flat-command locomotion, safe fall, recovery, get-up and command resume |
| Реализация | [План перестройки обучения humanoid motor policy](2026-08-12-humanoid-motor-training-rebuild.md) |
| Не является | ADR, public/current contract, доказательством качества модели или обещанием полного R5 humanoid |

Этот документ отвечает на вопрос **что должен уметь и каким ограничениям
должен соответствовать первый обученный персонаж**. Связанный план определяет
**как** построить, обучить и проверить этот результат. Если требование ниже
конфликтует с Accepted ADR/SPEC, действует архитектурный документ, а baseline
исправляется до следующего run.

Нормативная архитектурная опора: [SPEC-05](../architecture/05-physics-animation-and-motor-control.md),
[SPEC-14](../architecture/14-physical-archetypes-motor-skills-and-policy-lifecycle.md),
[SPEC-26](../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md),
[SPEC-27](../architecture/27-motor-observation-action-and-deterministic-inference.md),
[SPEC-28](../architecture/28-skeletal-animation-retargeting-and-ik.md),
[SPEC-34](../architecture/34-model-training-environments-trajectories-and-consolidation-lifecycle.md),
[SPEC-35](../architecture/35-deterministic-humanoid-training-substrate.md) и
[ADR-053](../architecture/adr/053-engine-native-model-training-and-immutable-artifact-boundary.md),
[ADR-058](../architecture/adr/058-physx-only-deterministic-humanoid-training-substrate.md),
[ADR-059](../architecture/adr/059-event-sourced-physx-continuation-reconstruction.md),
[ADR-064](../architecture/adr/064-canonical-flat-command-locomotion-environment.md),
[ADR-065](../architecture/adr/065-curriculum-flat-command-locomotion-profile.md),
[ADR-066](../architecture/adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md),
[ADR-067](../architecture/adr/067-stage0-profile-identity-and-curriculum-hash-closure.md) и
[ADR-068](../architecture/adr/068-static-morphology-cache-and-action-chunk-field-closure.md).

## 1. Требуемый продуктовый результат

Первый candidate — физически симулируемый fixed-body humanoid, который на
ровной поверхности:

1. стоит без заметного дрейфа и запрещённой опоры;
2. начинает движение с левой и правой ноги;
3. идёт вперёд с заданной скоростью и плавно меняет её;
4. поворачивает влево/вправо и следует заданному facing/yaw rate;
5. останавливается с обеих support phases;
6. выдерживает объявленные recoverable pushes;
7. при неизбежной потере баланса переходит в brace/safe-fall;
8. встаёт из prone, supine и обеих side reset classes;
9. после стабилизации возобновляет последнюю допустимую команду;
10. при несовместимости, non-finite output или иной declared fault атомарно
    уступает procedural/animation/ragdoll fallback.

Candidate публикуется как immutable bundle. Он MAY содержать отдельные
locomotion и recovery actors и детерминированный engine-owned supervisor, но на
одном motor tick ровно один route является источником полного action. Общая
анатомия, observation/action/safety/PD semantics и hash closure обязательны для
всех routes. Distillation в один actor разрешён, но не является требованием.

### 1.1. Граница утверждения

Прохождение этого baseline разрешает только утверждение:

> fixed-body flat-command locomotion and recovery candidate passed its declared
> held-out production-headless profile.

Оно **не** означает:

- полный `MOTOR-HUMANOID-MVP-P1`: terrain часть не входит в candidate;
- закрытие roadmap R5 или статус `Supported`;
- backward, strafe, run, crouch, crawl, slope, stairs or uneven terrain;
- carry, weapon, melee, manipulation, damage, fatigue or equipment adaptation;
- arbitrary morphology, cross-body transfer или runtime learning;
- biomechanical эквивалент конкретному реальному человеку.

## 2. Зафиксированные рабочие решения

До начала `TRAIN-1` product owner подтверждает или меняет следующие решения.
Изменение создаёт новую revision этого baseline и invalidates только уже
запущенные зависимые gates.

| ID | Рабочее решение |
|---|---|
| `DEC-HUM-01` | Первый результат — `Candidate`, а не `Supported` и не полный R5/MVP. |
| `DEC-HUM-02` | Mandatory locomotion envelope — только forward + turn на flat floor; backward/strafe/terrain deferred. |
| `DEC-HUM-03` | Multi-specialist bundle допустим; single distilled actor optional. |
| `DEC-HUM-04` | Side recovery обязателен; v1 может использовать admitted side-to-prone/supine transition перед get-up. |
| `DEC-HUM-05` | Тело представляет один явно выбранный anthropometric target profile, а не усреднённого «человека вообще». |
| `DEC-HUM-06` | Числовые operating, quality и runtime budgets разделов 3 и 6 являются candidate pass/fail envelope, а не report-only targets. |

## 3. Operating envelope

Все числа этого раздела являются входом evaluation profile, а не curriculum
подсказкой. Boundary values включены. Actor работает на `60 Hz`, physics/PD —
на `240 Hz`.

### 3.1. Среда и начальное состояние

| Параметр | Mandatory acceptance envelope |
|---|---|
| Поверхность | горизонтальная plane; slope `0°`; без ступеней, препятствий и moving platforms |
| Effective static friction | `0.65`, `0.80`, `1.00` evaluation cells |
| Effective dynamic friction | соответственно `0.55`, `0.70`, `0.90`; всегда `mu_dynamic <= mu_static` |
| Payload/equipment | отсутствуют |
| External wind/drag | отсутствуют, кроме declared push cells |
| Episode | `60 s` / `3 600` motor ticks, если scenario не имеет более короткого explicit completion |
| Initial state | admitted idle/reference state, finite and penetration-free; evaluation reset не выбирается из training trajectory |
| Randomization | exact held-out seeds and cell values frozen before acceptance run; unmanifested randomization запрещена |

Friction table задаёт evaluation coverage, но не public material contract. Exact
PhysX material pairs/combine mode входят в environment manifest; изменение
эффективного результата создаёт новый environment hash.

### 3.2. Команды locomotion

| Параметр | Mandatory acceptance envelope |
|---|---|
| Forward speed target | `0.00`, `0.25`, `0.50`, `0.75`, `1.00`, `1.25 m/s` |
| Lateral/backward target | строго `0.00 m/s` |
| Yaw-rate target | `-0.60`, `-0.30`, `0.00`, `0.30`, `0.60 rad/s` |
| Combined cells | forward `0.50` and `1.00 m/s` × every non-zero yaw-rate target |
| Linear command acceleration | `<= 0.50 m/s^2` |
| Yaw command acceleration | `<= 0.50 rad/s^2` |
| Plateau duration | `>= 4.0 s` после command ramp |
| Transition window | первые `1.5 s` после конца command ramp; отдельно измеряется, но исключается из steady-state RMSE |

Required matrix включает idle, left/right lead start, every steady speed,
left/right stop support phase, speed-up, slow-down, left/right turn, combined
forward+turn и interruption около single/double support. Один aggregate score
не заменяет результат каждой cell.

### 3.3. Recovery perturbations

Recoverable push задаётся mass-normalized impulse `J / body_mass`, чтобы
BodySchema mass не меняла смысл класса:

| Параметр | Mandatory acceptance envelope |
|---|---|
| Horizontal direction | forward, backward, left, right in root-local frame |
| `J / body_mass` | `0.15 m/s` and `0.30 m/s` cells |
| Application point | declared upper-torso body at its BodySchema local CoM |
| Duration | `<= 0.10 s`, exact tick-aligned force profile frozen in manifest |
| Command during push | idle, `0.50 m/s` forward, `1.00 m/s` forward |
| Fallen reset classes | prone, supine, left side, right side |

Более сильный push может завершиться declared safe fall и не называется
recoverable-push success. Push magnitude/direction нельзя переклассифицировать
по фактическому исходу episode.

### 3.4. Measurement semantics

- `completion` означает достижение ожидаемого terminal/plateau state и
  сохранение safety/contact conditions до конца scenario; timeout, crash,
  missing sample, fallback activation or undeclared recovery дают failure;
- planar velocity измеряется из committed root velocity в command-local frame;
  steady RMSE использует все motor ticks после transition window, без выбора
  «красивого» подпериода;
- facing error — shortest wrapped angle между committed root facing и
  интегрированным heading target; median и p95 считаются по всем eligible ticks;
- support interval начинается/заканчивается по frozen sole-contact classifier;
  slip использует committed tangential sole velocity and displacement только
  внутри этого interval, но не отбрасывает плохой landing tick после начала
  valid contact;
- positive mechanical work — сумма `max(torque * joint_velocity, 0) * dt` по
  actuator channels; distance не включает root teleport или external push;
- `FallEvent` в nominal locomotion — любой вход в brace/safe-fall/fallen/get-up
  route, forbidden non-sole support или terminal balance loss;
- `StableCommandState` означает `StableIdle` при zero command и
  `StableLocomotion` при non-zero command.

Metric implementation, units, aggregation and quantization входят в frozen
evaluation-profile hash. Изменение формулы после run равнозначно новому
profile, а не reinterpretation существующего результата.

## 4. Требования к телу и данным

### 4.1. `REQ-HUM-BODY-*` — физическое тело

- `REQ-HUM-BODY-001` — существует ровно один exact anthropometric target
  profile с authored standing height, total mass, segment proportions, mass
  distribution, joint centers/axes/frames, hard and soft ROM, collider and foot
  geometry. Все значения хранятся в SI units.
- `REQ-HUM-BODY-002` — каждая source/derivation assumption имеет provenance;
  PhysX/simulator defaults не заполняют отсутствующие значения.
- `REQ-HUM-BODY-003` — сумма segment masses равна authored total mass в
  canonical representation; inertia finite, positive and consistent with the
  declared collider/body frame.
- `REQ-HUM-BODY-004` — left/right symmetry является authored mapping. Любая
  асимметрия имеет explicit reason и validator coverage.
- `REQ-HUM-BODY-005` — knee/elbow не hyperextend, torso/hip/shoulder axes
  соответствуют объявленной anatomical semantics, neutral pose не имеет
  self-penetration, soles образуют положительную support area.
- `REQ-HUM-BODY-006` — actuator effort, velocity, rate, power/energy, PD and
  residual-action bounds заданы per joint group и работают hard outside reward.
- `REQ-HUM-BODY-007` — anatomy review имеет zero open Blocker/Major defects для
  spine, hips, knees, ankles, shoulders and elbows.

Exact height, mass and segment table намеренно не придумываются в этом
planning baseline. Их выбор и frozen hash — blocking output `TRAIN-1`; пока
они отсутствуют, `REQ-HUM-BODY-*` и весь candidate не могут получить `Pass`.
Слово «биомеханически правдоподобный» без этой таблицы не является требованием
или evidence.

### 4.2. `REQ-HUM-DATA-*` — motion corpus

- `REQ-HUM-DATA-001` — mandatory classes: neutral idle/weight shift,
  left/right start, slow/nominal forward walk, left/right-phase stop,
  gradual left/right turn, four-direction brace/safe-fall, prone/supine
  get-up and left/right side-to-prone-or-supine transition.
- `REQ-HUM-DATA-002` — каждая learned mandatory class имеет минимум три
  независимых `split_group_id`: train, validation and held-out. Производные
  crop/cleanup/retime/mirror одного source group никогда не пересекают split.
- `REQ-HUM-DATA-003` — cyclic locomotion group содержит минимум четыре полных
  gait cycles; transition class содержит declared entry and terminal state.
- `REQ-HUM-DATA-004` — source hash, owner/provider, license and training/model-
  output/distribution rights проверены до conversion. Unknown or incompatible
  rights означают exclusion.
- `REQ-HUM-DATA-005` — deterministic re-import/retarget даёт identical output
  hash; 100% admitted clips проходят ROM, penetration, contact, phase and
  visual-overlay checks.
- `REQ-HUM-DATA-006` — runtime candidate не читает corpus, source path или
  training-only annotation.

## 5. Функциональные требования

| ID | Требование | Failure behavior |
|---|---|---|
| `REQ-HUM-F-001` | Из stable idle выполнять declared start и переходить к speed hold с правильным left/right lead. | Recovery or safe-stop; episode fail. |
| `REQ-HUM-F-002` | Следовать forward-speed и yaw-rate commands во всём mandatory envelope. | Recovery; no unsafe target chasing. |
| `REQ-HUM-F-003` | Выполнять bounded speed change, turn, combined command and stop без teleport/root authority. | Recovery or safe-stop. |
| `REQ-HUM-F-004` | Сохранять locomotion support только declared sole contacts; hand/knee/body support не считается locomotion success. | Enter recovery and fail locomotion cell. |
| `REQ-HUM-F-005` | Определять unstable state только из canonical physics/motor facts и выбирать Brace/SafeFall/Fallen route deterministically. | Declared ragdoll/safe-stop fallback. |
| `REQ-HUM-F-006` | Возвращаться из prone, supine, left-side and right-side reset через declared recovery phases. | Remain safe ragdoll; no floor locomotion. |
| `REQ-HUM-F-007` | После `Stabilize` возобновлять prior command, если она всё ещё внутри envelope; иначе idle. | Procedural idle fallback. |
| `REQ-HUM-F-008` | При model/schema/state/evaluator fault не публиковать partial action/state и активировать declared compatible fallback. | Atomic rejection and fallback. |
| `REQ-HUM-F-009` | Exported bundle работать без trainer, optimizer, mutable weights, network or motion corpus. | Candidate rejected before activation. |
| `REQ-HUM-F-010` | Save/load/replay сохранять complete authoritative action/route/policy state required by accepted contracts. | Reject learned route; retain compatible source/fallback. |

### 5.1. Interface requirements

- `REQ-HUM-I-001` — mandatory product command содержит только typed local
  forward-speed and yaw-rate targets из section 3.2; heading target получается
  deterministic integration. Lateral/backward/text/token/embedding inputs не
  являются скрытыми каналами candidate.
- `REQ-HUM-I-002` — actor observation строится только из committed canonical
  physics/motor facts, previous applied action и объявленного bounded reference
  context. Renderer, camera, wall time and trainer-only facts отсутствуют.
- `REQ-HUM-I-003` — выбранный route публикует один complete bounded residual
  joint-target action для всех controlled DoF. Partial action, world pose,
  teleport and direct torque output запрещены.
- `REQ-HUM-I-004` — action производится на exact `60 Hz` motor boundary и
  применяется engine-owned fixed PD/safety на четырёх `240 Hz` physics
  substeps; stale/unavailable action следует accepted hold/fallback semantics.
- `REQ-HUM-I-005` — supervisor route, phase and any recurrent/adaptation state
  являются explicit bounded policy state и участвуют в save/load/replay/hash
  closure.
- `REQ-HUM-I-006` — reference horizon, contact annotations, normalization and
  action decode имеют exact schema/profile identities; incompatible identity
  rejects bundle до первого action.

Эти требования задают candidate outcome. Если для них требуется новый
public/current command, route or reference schema, implementation plan сначала
проводит consumer-backed ADR/SPEC change; planning ID не становится public
contract автоматически.

`StableIdle` для evaluation означает непрерывное окно `0.5 s`: оба declared
support soles имеют valid support contact, planar root speed `<= 0.10 m/s`,
absolute root roll/pitch `<= 10°`, root height находится в пределах `±5%` от
BodySchema nominal idle height, отсутствуют forbidden contact and safety
events. `StableLocomotion` использует те же orientation/height/safety условия,
но допускает declared gait support schedule и commanded planar speed.

## 6. Нефункциональные и качественные требования

### 6.1. `MotorPerformanceEnvelope` candidate

Каждое percentage/error требование применяется per required cell и aggregate
по заранее frozen statistical protocol плана. Missing/crashed/timeout episode
считается failure, а observed hard-safety event нельзя скрыть confidence
interval.

| ID | Метрика | Pass threshold |
|---|---|---|
| `REQ-HUM-Q-001` | TRAIN-5 full-reference completion | `>= 95%` per held-out nominal-locomotion motion class |
| `REQ-HUM-Q-002` | Normalized root/CoM position RMSE | `<= 0.05` of standing height |
| `REQ-HUM-Q-003` | Per-joint pose RMSE normalized by that joint soft-ROM span | frozen-weight mean `<= 0.05`; no joint `> 0.10` |
| `REQ-HUM-Q-004` | Sole contact precision and recall | each `>= 0.90` |
| `REQ-HUM-Q-005` | TRAIN-6 command-cell completion | `>= 95%` per cell |
| `REQ-HUM-Q-006` | Flat survival | `>= 99%` across complete `60 s` held-out suite |
| `REQ-HUM-Q-007` | Steady forward velocity RMSE | `<= 0.20 m/s` after transition window |
| `REQ-HUM-Q-008` | Facing error | median absolute `<= 7°`; p95 `<= 15°` |
| `REQ-HUM-Q-009` | Start latency | within `1.5 s` after ramp end, speed remains within `±0.10 m/s` of target for `0.5 s` |
| `REQ-HUM-Q-010` | Stop settling/overshoot | `StableIdle` within `1.5 s` after zero-target ramp; planar displacement after ramp `<= 0.30 m` |
| `REQ-HUM-Q-011` | Foot slip | during sole support, tangential speed p95 `<= 0.20 m/s` and stance displacement `<= 0.10 m` |
| `REQ-HUM-Q-012` | Recoverable pushes | `>= 95%` return to `StableCommandState` per direction/magnitude/command cell |
| `REQ-HUM-Q-013` | Fallen resets | `>= 90%` reach `Stabilize` within `6.0 s` per prone/supine/left/right class |
| `REQ-HUM-Q-014` | Command resume | `>= 90%` return to target band within `2.0 s` after `Stabilize` |
| `REQ-HUM-Q-015` | Energy efficiency | steady-cell positive mechanical work per meter p95 `<= 115%` of passing TRAIN-5 teacher on matched reference cells |
| `REQ-HUM-Q-016` | Reference-skill retention | no class exceeds its pre-registered non-inferiority margin after TRAIN-6/7/8/export |

Contact impact, per-joint effort/velocity/power/energy and target slew are hard
BodySchema/SafetyProfile bounds, not universal guessed numbers. `TRAIN-1/3`
must publish exact per-contact/per-joint values before ML; acceptance requires
zero violations and reports peak/p95 margin to every bound.

### 6.2. Safety and correctness

| ID | Требование |
|---|---|
| `REQ-HUM-S-001` | `0` NaN/Inf, hard ROM, effort, velocity, power/energy, target-rate or forbidden-impact violations in every gate run. |
| `REQ-HUM-S-002` | Forbidden locomotion contact raw count exactly `0`; recovery hand/knee contact valid only in declared recovery phases. |
| `REQ-HUM-S-003` | Mean joint-limit clamp incidence `< 0.1%` motor ticks in TRAIN-5 and final suite; every clamp remains reported, never reward-hidden. |
| `REQ-HUM-S-004` | No root pose write, teleport, direct backend torque buffer, learned/adaptive PD gain or reward-based safety substitute. |
| `REQ-HUM-S-005` | Route fault cannot publish partial action/state; fallback проходит тот же engine-owned action clamp. |
| `REQ-HUM-S-006` | Recovery failure ends in declared safe ragdoll/safe-stop and never continues locomotion reference from the floor. |

### 6.3. Determinism, portability and runtime budget

| ID | Требование |
|---|---|
| `REQ-HUM-NF-001` | Body/environment/dataset/observation/action/state/safety/PD/model/evaluator identities and hashes are complete and immutable. |
| `REQ-HUM-NF-002` | Canonical applied action and complete policy state are exact across trainer decode, exported evaluator, Windows and Linux supported runtimes. |
| `REQ-HUM-NF-003` | CPU inference including deterministic route selection has p99 `<= 0.5 ms/avatar` or `<= 2.0 ms/batch-of-16`. |
| `REQ-HUM-NF-004` | Every active actor is a fixed-shape standard-op graph with target size `1..3M` and hard maximum `3M` parameters; all model payloads in one bundle total `<= 64 MiB`. |
| `REQ-HUM-NF-005` | Shared evaluator plus batch-of-16 scratch has resident-memory delta `<= 128 MiB`; per-avatar persistent policy state `<= 256 KiB`. |
| `REQ-HUM-NF-006` | Actor tick performs no blocking I/O, allocation dependent on corpus size, network access or mutable-weight update. |
| `REQ-HUM-NF-007` | Worker completion order, measured CPU load and wall time never choose authoritative action, route or fallback. |
| `REQ-HUM-NF-008` | Published bundle has complete license/provenance/SBOM closure and no source-motion bytes. |

If platform or performance check is unavailable, status remains `NotRun`; это
не условный `Pass` и не разрешение публиковать candidate для этого target.

### 6.4. Visual quality

Visual review — project-local quality evidence, не новая architecture-wide
approval authority. Fixed playlist из implementation plan проходит на exact
candidate, seeds and commands.

| Severity | Определение | Gate disposition |
|---|---|---|
| Blocker | unsafe/anatomically impossible motion, wrong joint direction, penetration, prop/skating that invalidates skill, violent unexplained impact | zero open |
| Major | repeated visible balance/contact/transition defect, materially asymmetric left/right behavior, implausible get-up or command resume | zero open |
| Minor | presentation defect without physics/contact/safety or task impact | may remain only with defect ID, owner and explicit deferral |

Numeric pass не отменяет Blocker/Major, а субъективное одобрение не отменяет
metric/contact/safety failure.

## 7. Acceptance traceability

| Requirement group | Primary gate | Required evidence |
|---|---|---|
| `DEC-HUM-*`, claim boundary | before `TRAIN-1`, reconfirm at `TRAIN-9` | frozen baseline revision and candidate manifest claim |
| `REQ-HUM-BODY-*` | `TRAIN-1..3` | exact profile table/hash, validators, anatomy and passive/contact/safety reports |
| `REQ-HUM-DATA-*` | `TRAIN-4` | rights/provenance manifest, split audit, retarget hashes and preview review |
| `REQ-HUM-F-001..004` | `TRAIN-5..6` | held-out reference and command matrix |
| `REQ-HUM-F-005..007` | `TRAIN-7` | route replay, push/reset/resume matrix |
| `REQ-HUM-F-008..010` | `TRAIN-9` | fault/fallback, export, save/load/replay and no-runtime-corpus evidence |
| `REQ-HUM-I-*` | `TRAIN-3`, `TRAIN-6`, `TRAIN-9` | schema/profile validation, golden action/state corpus and runtime parity |
| `REQ-HUM-Q-*` | `TRAIN-5..9` | pre-registered statistical report, all seeds/cells, no cherry-picking |
| `REQ-HUM-S-*` | `TRAIN-3..9` | raw safety/contact events and exact failure reproducer |
| `REQ-HUM-NF-*` | `TRAIN-9` | parity roots, Windows/Linux results, performance/memory and package audit |
| Visual quality | every owning gate | fixed playlist, defect ledger and final disposition |

Every gate report lists applicable requirement IDs as `Pass`, `Fail`, `NotRun`
or `NotApplicable(reason)`. `NotApplicable` is valid only outside the declared
candidate scope; it cannot waive a mandatory threshold. A requirement change
after dependent training begins creates a new baseline revision, evaluation
manifest and affected candidate lineage.

## 8. Definition of ready and done

`ReadyForTRAIN-1` requires confirmed `DEC-HUM-01..06` and an owner/source plan
for the exact anthropometric profile. `ReadyForML` requires passing
`TRAIN-1..4`; unresolved body, safety, contact, data-rights or split requirement
blocks training.

Candidate is done only when all mandatory `REQ-HUM-*` have `Pass`, required
`TRAIN-0..7` and `TRAIN-9` pass, optional `TRAIN-8` passes or is explicitly
skipped before export, Windows/Linux production-headless parity/performance are
not `NotRun`, visual Blocker/Major count is zero, and the published claim stays
inside section 1.1.
