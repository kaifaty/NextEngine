# План перестройки обучения humanoid motor policy

| Поле | Значение |
|---|---|
| Статус | In execution: `TRAIN-3` remains advanced; `TRAIN-4` remains reopened. V18 passes `27/27` profile-local validation and `12815/12815` native poses, but exhaustive R14 still has `204/12518` required-safety failed cases. Bounded V7 passes R47 offline and R49 fresh `17/17` with required safety `0` and no control regressions. Its explicit decision permits only a clip-global prototype because window-local overlaps disagree and the prior full `cmu16` solve fails complete-clip bounds. ADR-070 fresh-scene authority is retained; partial reset is report-only. Full V19, optimizer execution, multi-seed, `TRAIN-5` Advance and `TRAIN-6` remain forbidden. |
| Дата | 2026-08-14 |
| Scope | Новый fixed-humanoid путь: biomechanics → motion tracking → command locomotion → recovery → export |
| Не является | ADR, доказательством качества модели или разрешением пропустить ProductCheck |
| Архитектурная опора | SPEC-05, SPEC-14, SPEC-26, SPEC-27, SPEC-28, SPEC-34, SPEC-35, ADR-027, ADR-030, ADR-046, ADR-053, ADR-058, ADR-059, ADR-064..070 |
| Требования candidate | [Humanoid motor requirements baseline](2026-08-12-humanoid-motor-requirements.md) |
| Anthropometric target | [Young-adult male gait target](2026-08-12-humanoid-biomechanics-target.md) |
| Frozen biomechanics profile | [Fixed humanoid biomechanics profile V1](2026-08-12-humanoid-biomechanics-profile-v1.md) |
| Frozen profile SHA-256 | `968ceb82ac2a40af872b496fdd43e32ba2ce3ffd90b2b3a33f38461c8269aabc` |
| TRAIN-3 safety/contact profile | [Humanoid safety and contact profile V2](2026-08-13-humanoid-safety-contact-profile-v2.md) |
| TRAIN-3 safety/contact SHA-256 | `ba9d368e075f389a4dbff4a0ed9299b737edf4907be10ae6cf3aeb60b348729f` |
| TRAIN-4 rejected baseline corpus profile | [Humanoid motion corpus dynamic reserve V3](../../lab/profiles/humanoid-motion-corpus-cmu-dynamic-reserve.v3.json), SHA-256 `1229eb18b8efea2daff50cf73a733fe67b804f507ef739bc88f1dee30584bb0c` |
| TRAIN-4 latest diagnostic corpus profile | [Humanoid motion corpus ankle-pitch velocity closure V18](../../lab/profiles/humanoid-motion-corpus-cmu-ankle-pitch-velocity-closure.v18.json), SHA-256 `91d60064444f155084ec0b793876b208336747b7dcaa03712290ca1ef9bdb4b0` |
| TRAIN-4 latest diagnostic corpus manifest canonical SHA-256 | `bd6164180a33585ed9231fd6a41cb3943b20e51da8b0e06ac0c978406e204456` |
| TRAIN-4 latest diagnostic tracker | [Humanoid reference tracker ankle-pitch velocity closure V11](../../lab/profiles/humanoid-reference-tracker-ankle-pitch-velocity-closure.v11.json), SHA-256 `19a8e5266f12c5c1ae9e307bd0f2f1a76bb6ea23a6ae9912e1ccec401eeaf28c` |
| TRAIN-4 causal decision | [Contact-consistent reference/reset research decision](../development/humanoid-train4-causal-research-2026-08-14.md) |
| TRAIN-4 contact-boundary decision | [Post-smoothing and projection-domain research](../development/humanoid-train4-contact-boundary-research-2026-08-14.md) |
| TRAIN-4 support-authorization decision | [Support-authorized quantized-clearance research](../development/humanoid-train4-support-authorization-research-2026-08-14.md) |
| TRAIN-5 current base tracker profile | [Humanoid reference tracker physics/velocity guard V4](../../lab/profiles/humanoid-reference-tracker-physics-velocity-guard.v4.json) |
| TRAIN-5 current base tracker SHA-256 | `7061e43bc59097312c10e90ea566485116bca4b5ec40ab1e93e22919b2160b5d` |
| TRAIN-5 rejected optimization child profiles | soft ROM `c482e68f05ad574b74ba037412a5d8b1d378966ac788de308b457885f7b0c35b`; predictive ROM `2640aa58886b00c901240f9f2b8912cfcff74e5a8490e846ad69b35b4fedc3b5`; realized contact impact margin `6a8b7c5871c200377cec4895ebefe370861f83c20a060e77ea9055f88e82ca06` |

Нормативные источники для реализации:

- [SPEC-05 — physics, animation and motor control](../architecture/05-physics-animation-and-motor-control.md);
- [SPEC-14 — physical archetypes, motor skills and policy lifecycle](../architecture/14-physical-archetypes-motor-skills-and-policy-lifecycle.md);
- [SPEC-26 — physics world, contacts and canonical snapshots](../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md);
- [SPEC-27 — motor observation, action and deterministic inference](../architecture/27-motor-observation-action-and-deterministic-inference.md);
- [SPEC-28 — skeletal animation, retargeting and IK](../architecture/28-skeletal-animation-retargeting-and-ik.md);
- [SPEC-34 — training environments and trajectory lifecycle](../architecture/34-model-training-environments-trajectories-and-consolidation-lifecycle.md);
- [SPEC-35 — deterministic humanoid training substrate](../architecture/35-deterministic-humanoid-training-substrate.md);
- [ADR-027 — physics/motor/animation layering](../architecture/adr/027-physics-motor-and-animation-layering.md),
  [ADR-030 — product-first workflow and ProductChecks](../architecture/adr/030-product-first-development-and-lightweight-validation.md),
  [ADR-046 — consumer-driven current-only contracts](../architecture/adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md),
  [ADR-053 — immutable artifact boundary](../architecture/adr/053-engine-native-model-training-and-immutable-artifact-boundary.md),
  [ADR-058 — PhysX training substrate](../architecture/adr/058-physx-only-deterministic-humanoid-training-substrate.md),
  [ADR-059 — continuation reconstruction](../architecture/adr/059-event-sourced-physx-continuation-reconstruction.md) and
  [ADR-064 — flat-command environment](../architecture/adr/064-canonical-flat-command-locomotion-environment.md),
  [ADR-065 — curriculum profile](../architecture/adr/065-curriculum-flat-command-locomotion-profile.md),
  [ADR-066 — contact-centric motor architecture](../architecture/adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md),
  [ADR-067 — profile identity/hash closure](../architecture/adr/067-stage0-profile-identity-and-curriculum-hash-closure.md),
  [ADR-068 — morphology cache/action chunk closure](../architecture/adr/068-static-morphology-cache-and-action-chunk-field-closure.md),
  [ADR-069 — biomechanics BodySchema V2](../architecture/adr/069-biomechanics-body-schema-v2-and-solver-projection.md) and
  [ADR-070 — reference-tracking environment](../architecture/adr/070-biomechanics-reference-tracking-training-environment.md);
- [current roadmap and ProductCheck status](../roadmap.md).

SPEC-34 and ADR-053 remain `Proposed`: this plan may use them as an experiment
shape, but does not promote their schemas or lifecycle to a shipped/default
contract. Accepted SPEC-35 and ADR-064..070 govern the current bounded
substrate. ADR-070 admits only the TRAIN-5 training environment; new
public/current command-locomotion, recovery or export semantics still require
a consumer-backed ADR/SPEC update before implementation.

## Цель

Заменить закрытую экспериментальную линию «pure PPO из neutral pose учится
изобретать походку по velocity reward» на последовательный физический pipeline:

```text
engine-owned biomechanical BodySchema
  → validated articulation, colliders, contacts and fixed PD/safety
  → licensed motion ingestion and deterministic retargeting
  → specialist reference-motion tracker
  → reference-conditioned command locomotion
  → separate brace/fall/recovery/get-up route
  → optional motion prior and specialist distillation
  → portable immutable candidate bundle and final production-headless evaluation
```

PPO остаётся optimizer для closed-loop tracking, task fine-tuning и
robustness. Он больше не отвечает за одновременное изобретение анатомии,
походки, переходов, баланса и recovery с нуля.

План считается завершённым после прохождения required `TRAIN-0..7` и
`TRAIN-9`. Optional `TRAIN-8` либо проходит собственный gate, либо получает
заранее зафиксированный disposition `Skipped(NotNeededForCandidate)` и
`TRAIN-9` использует immutable advancing candidate из `TRAIN-7`. Создание новой
BodySchema, запуск trainer или наличие checkpoint сами по себе не закрывают ни
одну стадию. Functional, safety, portability and claim requirements, а также
report-only quality/runtime targets задаются связанным requirements baseline;
этот документ не подменяет их описанием training steps.

## Решение по старой линии

Pure-PPO checkpoints, optimizer state, replay buffers и экспортированные
модели старого одинаково ограниченного 23-DoF тела не являются resume/import
input или rollback нового процесса. При этом immutable V1 profile identities и
bounded historical manifests/metrics остаются comparison and failure-analysis
evidence; их сохранение не делает старую policy active baseline нового
candidate.

Перед первым новым training run старая линия исключается из active path:

1. по внешним run manifests определяется полный список артефактов старых
   `isaac-rsl-rl-*` экспериментов и их производных;
2. exact paths проверяются read-only;
3. active indexes, default launch presets and resume selectors перестают
   выбирать старые pure-PPO run/model/checkpoint identities; immutable V1
   environment/profile IDs не удаляются и не меняются;
4. сохраняются bounded manifests, hashes, metrics и минимальные repro records,
   необходимые для provenance and failure comparison;
5. heavy checkpoints/logs/captures могут быть удалены только после triage, по
   exact validated paths и с отдельным явным разрешением на destructive purge;
6. новый training generation имеет отдельный корень и новый BodySchema hash;
7. поиск по active config/run indexes подтверждает отсутствие ссылки на старую
   модель или checkpoint.

Удаление не распространяется на Git history, принятые ADR/SPEC и engine tests.
Они не участвуют в inference или training selection и остаются историей
решений/регрессий. Физическое удаление historical payloads не является gate
condition. Retire или supersede Accepted profile semantics можно только
отдельным ADR. Новый pipeline, однако, не обязан сохранять executable backward
compatibility со старыми model artifacts.

## Границы scope

Входит в этот план:

- один фиксированный humanoid примерно с текущими 23 управляемыми DoF;
- биомеханически правдоподобные joint axes, frames, ROM, actuator и PD profiles;
- правдоподобные mass, inertia, CoM, colliders и стопы;
- full-body contact classification и отдельные contact rules для locomotion и
  recovery;
- authored/licensed idle, start, walk, stop, turn и get-up motions;
- deterministic retargeting и contact annotation;
- reference tracking, command locomotion, perturbation/recovery;
- fixed 60 Hz actor / 240 Hz physics and engine-owned fixed PD/safety;
- external training store, reproducible manifests, evaluation и portable
  immutable candidate bundle export.

Не входит:

- сохранение качества или совместимости старых pure-PPO policies;
- direct torque, learned/adaptive PD gains или muscle activation;
- graph policy, arbitrary topology или cross-family transfer;
- weapons, manipulation, parkour и любой non-flat terrain;
- runtime training, hot swapping или изменение model weights в active session;
- natural language, video/depth input или LLM в physical runtime;
- обязательное продвижение learned route в v1: procedural/animation fallback
  остаётся отдельной product requirement.

## Неподвижные принципы реализации

1. `BodySchema` является единственным источником joint topology, axes, limits,
   mass/inertia/CoM, colliders, actuator bounds и nominal PD facts.
2. Hard anatomy, target-rate, velocity, effort, power и contact safety работают
   вне reward. Reward не исправляет физически разрешённый неправильный сустав.
3. Policy выдаёт bounded residual target относительно authored/reference pose,
   а не world pose, teleport или backend torque buffer.
4. Каждый motor tick остаётся closed-loop: observation строится из committed
   physics; action проходит тот же engine-owned clamp и PD; только Physics
   определяет итоговую pose/contact.
5. Locomotion и recovery имеют разные skill/contact contracts. Опора рукой не
   является успешной locomotion; она разрешена в brace/get-up.
6. Isaac Lab/ProtoMotions/RSL-RL/PyTorch являются replaceable training tools.
   Engine descriptor, environment semantics, reward/termination identities и
   final production evaluation остаются engine-owned.
7. Любое изменение BodySchema, retarget profile, observation/action, reward,
   termination, seed derivation или dataset closure создаёт новый hash. Resume
   через несовместимую границу запрещён.
8. Evaluation manifest фиксируется до quality run. Нельзя менять reference
   target после просмотра результата или выбирать лучший seed вместо полного
   набора.
9. Visual review обязателен, но не заменяет functional/contact/safety gates;
   numeric quality/runtime targets остаются `ReportOnly`.
10. Failed run сохраняет bounded manifest, metrics и минимальный reproducer;
    тяжёлые checkpoint/log/capture payloads после triage удаляются.

Текущий `nextengine.body.humanoid-stage0.v1` revision `1` с pelvis root
`1.050 m` остаётся frozen identity для standing/flat-command V1 runs. Новый
биомеханический профиль TRAIN-1/2 получает отдельные BodySchema ID/revision и
environment/training generation; значение `1.095 m` или любая другая правка
анатомии не может быть записана поверх V1.

## Reproducibility classes and trainer continuation

План различает три уровня доказательств:

1. canonical CPU environment, descriptor, reset, checkpoint and replay roots
   должны быть byte-exact в принятом deterministic profile;
2. Isaac correspondence сравнивается по exact fields и заранее объявленным
   per-field numeric tolerances, без общего неявного epsilon;
3. stochastic GPU optimization является statistical reproducibility class, а
   не обещанием byte-identical model weights между machines/runs.

Каждый training/evaluation manifest фиксирует master seed и независимо
выведенные именованные streams как минимум для environment/reset, domain
randomization, policy sampling, worker assignment, minibatch shuffle and
evaluation. Worker/rank/order permutation не может менять identity эпизода или
seed stream; изменение derivation создаёт новый training-config hash.

Resume checkpoint включает actor, critic, optimizer, schedulers, observation
normalization, action-distribution state, curriculum position, sample/update
counters, named RNG states and sampler/minibatch position. Resume разрешён
только на объявленной optimizer-update boundary. GPU kernel/autotune/provider
state остаётся tool fingerprint, а не скрытым checkpoint state; поэтому
byte-exact GPU optimizer continuation не заявляется без отдельного passing
gate. Final candidate identity определяется immutable actor bytes и заново
выполненной deterministic held-out evaluation, а не утверждением, что trainer
повторит те же weights побитно.

## Артефактная политика новой линии

Новый внешний корень:

```text
/home/kaifaty/NextEngine-training/generations/humanoid-motion-v1/
  corpus/        admitted source and retargeted motion bytes
  runs/          active training runs
  candidates/    stage-gate candidates only
  evaluations/   bounded manifests, metrics and temporary captures
  exports/       portable immutable candidate bytes
```

Этот путь не является public contract и не входит в Git. Exact location
передаётся tool configuration; runtime manifests не получают filesystem path.

Retention policy:

| Состояние | Что сохраняется | Что удаляется |
|---|---|---|
| Active run | latest resume checkpoint, текущий optimizer state, metrics | более старые промежуточные checkpoints после проверки latest |
| Gate candidate | immutable actor checkpoint, run/config/dataset hashes, evaluation manifest | replay buffer, profiler scratch, redundant periodic checkpoints |
| Failed run | run/config hashes, seeds, metrics, stable failure, минимальный replay/trajectory prefix | `.pt`, optimizer, replay buffer, полные logs и captures после triage |
| Superseded candidate | только lineage/hash в новом manifest, если он требуется | старые model/checkpoint bytes после прохождения successor gate |
| Published export | exact model bytes, export/parity manifests, license/provenance closure | source optimizer и training-only caches |

Ни dataset bytes, ни checkpoints, ни generated media не добавляются в
repository. Bounded generated golden vectors разрешены только когда они нужны
для engine contract tests и не содержат защищённые motion data.

## Gate model

У каждой стадии четыре независимых статуса:

```text
Implementation: NotStarted | InProgress | Complete
ExactChecks:    NotRun | Pass | Fail
MetricReports:  NotRun | ReportOnlyComplete | NotApplicable
VisualReview:   NotRun | Pass | Fail | NotApplicable
```

`ExactChecks` покрывает required functional, interface, safety, determinism,
provenance and portability requirements. `MetricReports` означает полноту
заранее объявленных quality/runtime измерений; достижение reference target не
даёт `Pass`, а недостижение не даёт `Fail` и само по себе не запрещает
`Advance`.

Кроме них gate имеет stage и requirement dispositions:

```text
StageDisposition:       Required | OptionalSelected | Skipped(NotNeededForCandidate)
RequirementDisposition: RequiredCheck | ReportOnly
```

`Skipped` допустим только для `TRAIN-8`, фиксируется после `Advance` в
`TRAIN-7`
до начала `TRAIN-9` и указывает точный `TRAIN-7` candidate. Он не означает
`Pass` и не может скрывать начатый или провалившийся prior/distillation run.

Следующая стадия может начаться только при `Implementation=Complete`,
`ExactChecks=Pass`, applicable `MetricReports=ReportOnlyComplete` и required
`VisualReview=Pass`. Git commit является checkpoint кода, а не gate result.

Каждый gate report содержит:

- gate ID и implementation commit;
- requirements-baseline revision/hash и disposition каждого применимого
  `REQ-HUM-*`;
- BodySchema/compiled descriptor/environment/dataset/config hashes;
- exact commands, seed set и episode counts;
- passed/failed/not-run required checks and complete/not-run report-only
  measurements;
- primary и safety metrics без отбрасывания run/seed;
- stable first failure и минимальный reproducer;
- ссылки только на существующие external artifacts;
- решение `Advance`, `FixInStage`, `InvalidateDownstream` или `StopLane`.

## Statistical evaluation protocol

До первого evaluation run для `TRAIN-5..9` immutable evaluation manifest
обязан зафиксировать:

- candidate-selection rule и evaluation mode; production motor использует
  deterministic mean action, без exploration noise;
- disjoint train/validation/held-out seed sets и полный набор evaluation
  matrix cells;
- минимум training seeds, episodes per seed/cell and total sample budget,
  выбранные pilot variance/power analysis, а не после просмотра результата;
- estimator, confidence level/interval and interpretation rule for percentage,
  error and non-inferiority reference targets; они не дают candidate
  `Pass`/`Fail`;
- baseline candidate/hash, practical non-inferiority margin and effect-size
  report для сравнений;
- обработку timeout/truncation, missing/crashed seed and partial episode как
  failure/reportable fact, без исключения неудобных samples;
- все per-seed/per-cell results, mean, dispersion, confidence interval,
  median/range and sample-efficiency curve at declared budgets.

Exploratory runs могут быть дешевле, но не дают gate result. Один seed, лучший
checkpoint среди незаявленных evaluation points или point estimate без
предварительно выбранного sample count/confidence rule не может продвинуть
candidate. Hard safety condition `exactly 0` применяется также к raw event
count: confidence interval не превращает наблюдаемое нарушение в success.

## Сводка стадий

| Gate | Результат | Training разрешён после gate |
|---|---|---|
| `TRAIN-0` | старая artifact line исключена из active selection/resume | нет |
| `TRAIN-1` | принята exact biomechanics specification | нет |
| `TRAIN-2` | BodySchema компилируется в корректную PhysX articulation | нет |
| `TRAIN-3` | safety/contact/terminal semantics работают без ML | нет |
| `TRAIN-4` | motion corpus лицензирован, retargeted и проходит kinematic, visual и full-horizon dynamic-reference-feasibility gates | tracker only |
| `TRAIN-5` | specialist tracker проходит required checks и завершает held-out metric report | command fine-tuning |
| `TRAIN-6` | start/stop/velocity/facing проходят required checks; quality matrix полностью reported | perturbation/recovery |
| `TRAIN-7` | mandatory side-inclusive recovery route проходит required checks; quality matrix полностью reported | motion prior/multi-skill |
| `TRAIN-8` | optional prior/distilled actor проходит required checks и завершает retention report либо stage заранее skipped | export; при skip используется TRAIN-7 candidate |
| `TRAIN-9` | portable candidate совпадает с runtime, проходит required checks и завершает headless reports | candidate publication |

Текущее выполнение:

- `TRAIN-0`: `Advance`, implementation commit `9148b776db2c1be63d1393f62f1cbfa782eb11cc`;
- `TRAIN-1`: `Advance`, revision `2`, BodySchema hash
  `e2460e7dc4af93538ae4b0b68a9e1bf74b2b7990161e08e441d58687e953c43d`,
  gate report SHA-256
  `4d94d28f64c81d953e75804db4a0807d85b1e2e7e57af0deb747b0446c2f7fde`;
- `TRAIN-2`: `Advance`, BodySchema hash
  `e2460e7dc4af93538ae4b0b68a9e1bf74b2b7990161e08e441d58687e953c43d`,
  compiled descriptor hash
  `b6f8b1260b24ad401453942c9a4303d99ce378a8f71c6490db79bb78d85a1782`,
  gate report SHA-256
  `b814d7830188abbc83378e83b6fb8ff4da5fdf3c0c30d0606913a05692a71d32`;
- `TRAIN-3`: `Advance`; exact Rust/Isaac actuator, four-substep contact and
  directed terminal correspondence are closed. Gate report SHA-256 —
  `53c9f6b055007418c399f9fe9746b0784924b2a51cd5ee790ab158d3ae2824e5`;
- `TRAIN-4`: `Reopened / FailedDynamicReferenceFeasibility`. Прежний data-only
  `Advance` superseded; dynamic-reserve profile SHA-256
  `1229eb18b8efea2daff50cf73a733fe67b804f507ef739bc88f1dee30584bb0c`,
  corpus manifest SHA-256
  `33546488a73db25557c23fdb1a54b066ac3d02384aaca9acb529dab5d4cc81fd`,
  previous gate report SHA-256
  `261dce77dcc36448831373569c0a6d033ad8260769583e29228f706877c00778`;
  remediation gate report SHA-256
  `2fc1d6c3d312a1e88a3820bdd851fd0eecf3230795a1162b55f13dd65d45cdb6`.
  latest V18 corpus profile SHA-256 is
  `91d60064444f155084ec0b793876b208336747b7dcaa03712290ca1ef9bdb4b0`;
  deterministic rebuilds produced canonical manifest SHA-256
  `bd6164180a33585ed9231fd6a41cb3943b20e51da8b0e06ac0c978406e204456`.
  The corpus passes `27/27` profile-local and `12815/12815` native pose checks,
  but exhaustive R14 coverage is `12518/12518` with `204` required-safety
  failed cases, so this identity also does not advance the gate. Bounded causal
  causal research and the R29–R49 bounded contact cycle are complete. V7/R49
  passes selected fresh scenes `17/17`, but authorizes only one clip-global
  solve per selected clip, not optimizer work or a full corpus identity;
- `TRAIN-5`: `FailedSafetyGate / InvalidatedByUpstreamData`. Input/reset and
  reproducible h10 tiny sanity passed, but base, contact-impact-margin and
  deterministic phase-prefix curricula all failed hard safety. Every produced
  curriculum checkpoint is rejected; failure decision SHA-256
  `2832f50a6ab2b02ee1bd41acec1bf3f0d247555e22015ea396c50bb5c1da9c92`;
- `TRAIN-6..9`: `NotRun`; no downstream optimizer or publication work is
  authorized.

## TRAIN-0 — retirement/isolation старого эксперимента и чистая generation

### Реализация

1. Инвентаризировать external run roots по manifest/config/BodySchema hash.
2. Составить exact artifact inventory без glob по broad workspace root.
3. Удалить ссылки старой линии из active indexes/default selection/resume
   closure; historical records пометить `Retired`.
4. Создать новый внешний generation root.
5. Ввести один active `training_generation_id` и запрет resume/import старых
   BodySchema/environment hashes.
6. Удалить старые model/checkpoint experiment presets из default launch/view
   commands. Standing/flat-command V1 environment profiles остаются current
   comparison/test contracts с неизменными identities, пока Accepted
   architecture не superseded.
7. Добавить preflight, который fail-closed при старом model/checkpoint/config
   input.
8. После bounded evidence extraction отдельно сформировать optional exact purge
   list для heavy bytes. Его выполнение не входит в gate и требует явного
   destructive authorization.

### Exit criteria

- active generation index не содержит старых run/model references;
- old checkpoint переданный новому launcher получает stable incompatible-
  generation diagnostic до simulator creation;
- новый run root пуст и содержит только generation manifest;
- active config/run/resume scan подтверждает отсутствие selectable old
  payloads, а retained historical inventory остаётся readable and hash-bound;
- source tree и ignored external store не смешиваются.

### Failure/rollback

Не выполнять recursive deletion по unresolved path, glob, `$HOME` или
workspace root. TRAIN-0 по умолчанию изменяет selection/identity metadata, а не
стирает historical evidence. При неоднозначном manifest isolation или optional
purge операция останавливается до exact target resolution. Rollback старой
model line как active input отсутствует; metadata isolation должна быть
recoverable до публикации новой generation.

### Commit boundary

`chore(training): isolate retired humanoid experiment generation`

## TRAIN-1 — biomechanics specification

### Реализация

Создать engine-owned humanoid biomechanics profile. Для каждого DoF таблица
обязана определить:

- stable joint/parent/child IDs;
- anatomical semantic (`yaw`, `pitch`, `roll`, hinge flexion);
- normalized axis and parent/child local frames;
- asymmetric hard ROM and inner soft ROM;
- neutral position;
- maximum joint velocity;
- maximum effort, effort-rate, power/energy where supported;
- nominal `Kp/Kd`;
- residual action scale;
- maximum negative/positive target delta per motor tick;
- left/right mirror/sign rule.

Отдельная таблица body nodes определяет:

- bind pose, dimensions and semantic role;
- mass, local CoM and principal inertia;
- collider type/pose/material/layer/mask;
- self-collision exclusions only for declared adjacent pairs;
- foot support area and sole contact features;
- allowed support/manipulation roles.

Profile реализует один exact anthropometric target по
`REQ-HUM-BODY-001..007`: authored standing height, total mass, proportions and
source/derivation provenance являются blocking inputs, а не значениями,
которые выбирает trainer. До freeze также заполняются exact per-joint and
per-contact hard bounds, на которые ссылаются candidate safety/impact gates.
Selected planning input is the linked Rajagopal-based `1.700 m` young-adult
male gait target with exact source-model mass `75.337 kg`. Its source-to-target
mapping, virtual serial-axis carriers, ROM, colliders and actuators remain
blocking TRAIN-1 work; selecting the source does not pass the gate.

Начальный fixed humanoid может оставаться цепочкой revolute hinges. Hip,
shoulder и spine multi-axis motion представляются несколькими hinges с
различными correctly rotated frames. Ball/SixDof или новое public collision-
filter schema вводятся только если текущий accepted contract не способен
выразить требуемое тело; тогда до кода нужен новый superseding ADR и
синхронное обновление SPEC/contracts.

### Review checklist

- knee/elbow do not hyperextend through their anatomical back side;
- torso extension/twist/side-bend имеют консервативный ограниченный ROM;
- left/right limits and frames are intentional, not copied by sign accident;
- ankles have enough pitch/roll for balance but no unrestricted twist;
- arms cannot pass freely through torso/head under the declared filters;
- neutral pose has positive sole clearance and no self-penetration;
- actuator strengths/gains differ by joint group and are dimensionally valid;
- no value is taken from a simulator default.

### Exit criteria

- every body/joint/actuator/collider has a complete authored row;
- schema validator rejects missing, duplicate, unnormalized, inverted and
  asymmetric-without-declaration cases;
- an independent anatomy review records `Pass` for spine, knees, elbows,
  ankles, hips and shoulders;
- the gate-reviewed profile table and its canonical hash are frozen before
  compiler work. Это не меняет architecture status: current/public semantics
  требуют отдельного Accepted ADR/SPEC change.

### Failure/rollback

An unresolved anatomical choice blocks only the affected table/profile; it is
not filled by trainer or PhysX defaults. Any later table change invalidates
`TRAIN-2` and every downstream run.

### Commit boundary

`docs(motor): define fixed humanoid biomechanics profile`

Если exact schema semantics меняются, schema/ADR commit precedes implementation
and is not combined with generated fixtures.

## TRAIN-2 — BodySchema compiler and physical articulation

### Реализация

1. Extend BodySchema compilation from X-axis/identity-only hinges to every
   axis/frame admitted by `TRAIN-1`.
2. Compile per-joint limits, per-actuator gains/bounds and target rates without
   shared fallback constants.
3. Compile capsule/box/sphere colliders, explicit mass/CoM/inertia and declared
   collision filters.
4. Build flat sole geometry with stable left/right foot effectors.
5. Enable self-collision only under the declared filter/exclusion profile.
6. Preserve exact BodySchema → physics slot → motor channel mappings and hashes.
7. Update Isaac mirror descriptor generation from the same engine descriptor;
   no Python copy of anatomy values is allowed.

### Exact tests

- golden joint-axis/frame/limit mapping for every DoF;
- bind-pose compile and fresh-scene round trip;
- individual joint sweep to both hard limits while all other joints are held;
- invalid axis, frame, limit, collider, inertia and mapping negative corpus;
- left/right symmetry and intentional asymmetry corpus;
- descriptor/root equality under declaration and worker permutations;
- CPU/Isaac descriptor correspondence before any GPU optimization run.

### Passive physical tests

- neutral articulation has no ground or self penetration;
- feet, not ankle spheres, provide the support polygon;
- zero/neutral procedural action does not create unbounded energy;
- each joint reaches its declared range and cannot cross it;
- head/torso/limb colliders generate classifiable contacts;
- 10,000 reset/settle cycles produce no NaN/Inf, broken mapping or partial
  articulation publication.

### Exit criteria

- `BODY-SCHEMA-P1` and `PHYS-JOINT-P1` focused successor corpus passes for the
  new revision;
- canonical action/physics roots are exact under tested permutations;
- mirror reads all physical values from one generated descriptor;
- manual articulation inspection confirms axes and colliders correspond to the
  authored table;
- no model has been trained yet.

### Failure/rollback

Compiler or mirror mismatch retains no partially published body. Fix remains
inside this stage. Rollback target is the last passing new-generation compiler
commit, not the old BodySchema model line.

### Commit boundaries

1. `feat(motor): compile anatomical joint frames and limits`
2. `feat(motor): compile humanoid mass and collision geometry`
3. `test(motor): cover biomechanics compiler correspondence`

## TRAIN-3 — action safety, contacts and failure semantics

### Action path

The initial tracker action is defined as:

```text
candidate[j] = q_reference[j, phase]
             + residual_scale[j] * tanh(policy_output[j])

applied[j] = intersect_and_clamp(
  hard_joint_range[j],
  soft_safety_envelope[j],
  previous_applied[j] ± target_delta_per_tick[j],
  current skill/contact envelope[j]
)
```

PD then applies the fixed BodySchema-owned `Kp/Kd`, effort, effort-rate,
velocity and power limits at 240 Hz. Reference pose does not bypass safety.

### Contact classification

Add full-body contact sensing/projection and engine-owned classes:

- `SoleSupport`: declared foot sole contact;
- `TransientAllowed`: skill/profile-specific bounded brush;
- `ForbiddenLocomotion`: hand, forearm, elbow, knee, pelvis, torso or head
  ground contact above the declared impulse/duration threshold;
- `BraceSupport`: hand/forearm contact allowed only in brace/fall/recovery;
- `GetUpSupport`: hand/knee/forearm contact allowed only in get-up;
- `SelfCollisionViolation` and `JointSafetyViolation`.

Locomotion terminal conditions include root-height, bad orientation, sustained
forbidden contact, hard joint/safety violation, non-finite state and world
bounds. One-frame sensor noise uses an exact tick/impulse grace profile; an
indefinite hand prop is never `Running` locomotion.

### Non-ML scenarios

- scripted action step from neutral confirms target slew limiting;
- maximum action on each channel cannot exceed hard/soft ROM;
- standing procedural controller survives the declared neutral scenario;
- deliberate hand/knee/torso contacts terminate locomotion with stable reasons;
- identical contacts remain allowed under explicit recovery profile;
- fall/terminal and timeout/truncation are distinct;
- reset clears contact continuity, prior effort and policy action state exactly.

### Exit criteria

- every unsafe candidate is clamped or rejected before physical mutation;
- forbidden-contact scenarios have zero false `Running` results after grace;
- allowed sole and recovery contacts have zero profile misclassification in
  the golden corpus;
- exact safety/contact/done roots match between canonical CPU and Isaac mirror;
- passive and scripted visual review shows no reverse spine fold, knee/elbow
  hyperextension or sustained hand-prop locomotion.

### Failure/rollback

Safety or contact correspondence failure blocks all data collection and
training. Reward tuning is not an accepted fix for this gate.

### Commit boundaries

1. `feat(motor): enforce per-joint target slew and safety envelopes`
2. `feat(motor): classify full-body skill contacts`
3. `test(motor): cover locomotion and recovery terminal semantics`

### Gate result (2026-08-12)

`Advance` на validated commit
`8f2d07518a90d7e49b0829267c64a99c3261ee90`. Engine-owned 23-channel action path
атомарно пересекает hard/soft/skill/slew envelopes и применяет fixed PD с
effort/rate/power/work limits; post-step joint facts проверяются отдельно.
Shape-owned contact classifier, four-substep grace, skill-specific support и
terminal priority совпали в Rust golden и независимом Python mirror для семи
сценариев. Native PhysX standing review прошёл `1800` motor ticks / `7200`
physics substeps (`30 s`) с sole-only support, без hard ROM, hard-impact или
non-sole support violations и завершился exact `Truncated/terminal.timeout`.

Visual review front/side snapshots at `0/10/20/30 s` и contact-semantics
matrix: `Pass`, open Blocker/Major defects `0`. Полный clean-worktree
`cargo run -p xtask -- host-check` прошёл. Exact external gate report:
`/home/kaifaty/NextEngine-training/gates/TRAIN-3/gate-report.json`.
Stage не запускал optimizer, не создал policy/checkpoint и не разрешает ML до
лицензионного, retarget и physical-validity closure `TRAIN-4`.

## TRAIN-4 — motion corpus, retargeting and reference closure

### Minimum corpus

The first admitted corpus contains only motions needed by the next two stages:

- neutral idle and weight shift;
- start with left and right lead;
- slow and nominal forward walk;
- stop from left and right support phase;
- gradual left/right turn;
- brace and safe fall;
- prone and supine get-up;
- left/right side-to-prone-or-supine transitions для обязательных side reset
  classes.

Fast walk/run, backward, strafe, crouch and terrain motions are later corpus
revisions and cannot silently appear in the first evaluation split.
Nominal locomotion clips feed `TRAIN-5/6`; brace/fall/get-up clips remain a
separate recovery partition for `TRAIN-7` and cannot make nominal tracker
metrics easier.

### Admission

For each source record capture:

- source owner/provider and exact source hash;
- license text/hash and intended training/model-output/distribution rights;
- consent where own capture includes identifiable people;
- source skeleton/coordinate/rate profile;
- conversion and cleanup tool hashes;
- permitted train/validation/held-out assignment;
- `split_group_id`, объединяющий одного source performer/session/original clip
  со всеми cropped, cleaned, retimed and mirrored derivatives.

Unknown, noncommercial-only, no-derivatives or otherwise incompatible input is
excluded before conversion. Source and retargeted bytes remain external.

### Deterministic retargeting

1. Map by stable joint keys/roles through an explicit profile; no runtime name
   guessing.
2. Convert units, axes and handedness before reference construction.
3. Solve target joint rotations within BodySchema soft ROM.
4. Reconstruct root translation/yaw, velocities and phase.
5. Detect sole/hand/knee/body contact intervals from declared geometry and
   thresholds, then allow a bounded human correction layer recorded in the
   retarget manifest.
6. Produce root/CoM, joint pose/velocity, effector trajectories, contact
   schedule and phase features at exact 60 Hz reference boundaries.
7. Mirror left/right clips only through the BodySchema symmetry mapping and
   record derived provenance.

Before implementation chooses a new current wire contract, add a consumer-
backed ADR that defines the smallest reference-tracking environment/profile
evolution. Likely affected records are the training environment manifest,
reference motion/phase inputs, reward/termination closure and correspondence
profile. Framework tensors and source filesystem paths do not enter it.

### Validation

- no retargeted joint leaves soft/hard ROM;
- no non-foot penetration appears in locomotion clips;
- declared contacts align with sole distance/velocity and support phase;
- root velocity/facing derived from the clip agrees with the labelled skill;
- loop clips have bounded pose/velocity discontinuity at wrap;
- start/stop clips have explicit non-looping entry/exit states;
- mirror-twice returns the exact canonical source reference;
- visual overlay of source target and BodySchema retarget passes for every
  admitted clip;
- a frozen scripted reference-following baseline covers every valid start phase
  of every admitted locomotion clip for the full declared tracker horizon under
  the exact BodySchema/USD, fixed PD/safety, contact/terminal and PhysX
  identities, with zero required hard-safety events.

### Exit criteria

- all corpus entries have compatible rights and complete provenance;
- deterministic re-import produces identical retargeted hashes;
- train/validation/held-out splits are frozen by `split_group_id`; original,
  mirrored and otherwise derived siblings никогда не пересекают split, а
  held-out source/performer/clip family не использовался для training;
- 100% admitted clips pass anatomy, penetration, contact and phase checks;
- a generated kinematic preview is visually accepted before policy training;
- `REQ-HUM-DATA-007` receives `Pass`; its tracking/fidelity metrics are complete
  `ReportOnly` evidence and cannot waive a hard-safety failure.

### Failure/rollback

One invalid clip is excluded and produces a stable corpus diagnostic; it does
not relax BodySchema limits. Any correction creates a new retarget/corpus hash
and invalidates dependent runs. A full-horizon scripted-reference safety
failure is classified as `DataDynamicReferenceFeasibility`, returns work to
retarget/contact timing/root/joint trajectory and forbids optimizer work until
a new `TRAIN-4 Advance`.

### Commit boundaries

1. `docs(architecture): admit reference tracking environment contract` when
   a public/current semantic change is required
2. `feat(animation): retarget physical humanoid references`
3. `feat(training): validate motion corpus contacts and provenance`

### Реализация и gate closure

Deterministic ASF/AMC import, explicit semantic BodySchema retarget, velocity
projection, ground/contact reconstruction, mirrors, canonical NPZ
serialization and executable provenance/split/coverage audits реализованы и
затем исправлены по результатам native physical reset audit. Ранний corpus на
`52` locomotion/recovery artifacts признан superseded evidence: recovery poses
не были физически допустимы и не входят в advancing manifest.

Ранее admitted, теперь superseded внешний immutable locomotion corpus:
`/home/kaifaty/NextEngine-training/generations/humanoid-motor-rebuild-v1/corpus/f281f73773f32ddb-7b59986973b39870`.
Его canonical manifest SHA-256 —
`6f76c1c7d60457d1b10833b6fb840afbb50cd502315f250fd6e44a40d1c0dcbc`.
Все `23` base и `5` derived clips (`28/28`) прошли kinematic checks; splits —
`10/9/9` train/validation/held-out; все шесть locomotion class families имеют
независимые split groups. Repeat import воспроизвёл тот же manifest.

Native PhysX reset audit прошёл `5961/5961` locomotion poses с нулевой active
self-penetration, нулевым active self-contact impulse и нулевой начальной joint
velocity. Source/BodySchema overlays получили `28/28` visual `Pass`, open
Major/Blocker defects `0`. Retarget явно сохраняет hip/shoulder collision
projection, `20°` ankle-pitch reserve и минимум `15°` bilateral ankle-roll
reserve до hard ROM. Ankle-roll soft-boundary fraction равен `0`; доля
затронутых ankle-roll samples `0.443182..1.0` сохранена как ReportOnly
retarget-fidelity metric. Knee/elbow lower-boundary fraction также равен `0`,
а minimum hard-ROM reserve равен `1°`. Recovery candidate audit сохранил
`11465/11536` failures и maximum penetration `241889 µm`; recovery selection и
`TRAIN-7` запрещены до нового retarget/profile/hash.

Exact external gate report:
`/home/kaifaty/NextEngine-training/gates/TRAIN-4/locomotion-r6-gate-report.json`,
SHA-256
`38f33f63c2d187509128eca1469ab6b114547402bb769e311b8717a4c2ed18c4`.
Decision: `Advance` только для `TRAIN-5` specialist locomotion tracker; более
поздние training stages и runtime publication не разрешены. Этот decision
впоследствии superseded после `TRAIN-5`: exhaustive optimizer-free reference
audit показал `4009/4117` full-horizon failed episodes при `0` reset-window
safety failures. Corpus остаётся kinematically valid, но не доказан dynamically
feasible; требуется новая `TRAIN-4` lineage.

### Temporal/contact V4 remediation result

Новая data-only lineage реализует symmetric whole-clip smoothing, усиленное
сглаживание защищённых ankle channels, bounded joint reserves, minimum-dwell
support/contact intervals, conservative stance knee and hip clearance,
numerical stance-sole leveling, swing-knee lift, collision-free
velocity-bounded root-height majorant и bounded endpoint-tapered planar-root
correction. BodySchema ROM, actuator, contact and safety limits не менялись;
optimizer, checkpoint и learned policy не загружались. `cmu16-walk-nominal-a`
исключён, потому что после подавления contact chatter не выполняет требование
четырёх полных gait cycles; `cmu16-walk-slow` hash-bound к более длинному crop
того же train split group. Это новая corpus identity, а не перезапись прежнего
артефакта.

Frozen profile:
`lab/profiles/humanoid-motion-corpus-cmu-temporal-contact.v4.json`, SHA-256
`17fc25739318ffd3c7634b5e2d1cca093c99fcc8686aaec09c7f79158cd2b19c`.
Две полные сборки дали один и тот же corpus root
`/home/kaifaty/NextEngine-training/generations/humanoid-motor-rebuild-v1/corpus/17fc25739318ffd3-9273bc2a36285de4`,
canonical manifest SHA-256
`5f570cdbc724cf7db51530c07b82fbafba2b73f333e1a2684e48aeb8ae0d5195`
и byte SHA-256
`f4792bd1118a86780ff00582bfdfe557730306698cce92ba142a7fa8faddad43`.
Все `22` base и `5` derived clips (`27/27`) прошли автоматическую validation;
splits равны `9/9/9`. Native pose audit прошёл `5859/5859` poses, failures `0`;
report SHA-256
`7e93c338ede7db90d6a5540b86920192fb795334399b9d15d937ca00d6a7decb`.
Tracker V6 SHA-256
`d7131e909586a140dc541d3c8580744dede485b1e9693fa48153d8ab30c16281`
прошёл all-artifact/input/reward closure (`27` artifacts, physics episodes
`0`, optimizer steps `0`); report SHA-256
`d9592d79b2aa04c0207ea8ce116403ecb34079f3c683d1f13da0b6de45a3630f`.
Preliminary review of representative idle/start/walk/turn/stop/weight-shift
previews found no gross inversion or broken geometry, but formal per-clip human
visual review remains `27 pending` and no visual `Pass` is claimed.

This `VALIDATED` status is local to the diagnostic V4 profile, not a
requirements admission result. V4 permits `174533 urad` (`10 degrees`)
bilateral ankle-roll reserve so that the sole-leveling diagnostic can explore
the impact mechanism, while `REQ-HUM-DATA-005` requires `261800 urad`
(`15 degrees`). The requirement is not relaxed: V4 therefore also fails
`REQ-HUM-DATA-005` and cannot advance even independently of the dynamic result.

Exhaustive dynamic audit r8 covered all `5562/5562` valid start phases at
horizon `11` and completed every case. It failed `2876` cases: `2475`
hard-impact, `257` hard-ROM, `293` joint-safety, `171` joint-velocity, `125`
effort-envelope and `31` self-collision; fall, forbidden-contact, world-bounds
and non-finite counts are zero. `2686` cases reached reference completion.
Contact precision/recall are `0.610893/0.890920` and remain `ReportOnly`.
Impact pairs are still almost entirely bilateral ankle-ground (`1186` left,
`1317` right; three hip-pair events), while ankle pitch/roll remain the main
joint channels. Compared with the rejected V3 manifest-wide audit, raw failed
cases decreased `4329 -> 2876` (`-33.6%`) and failure rate decreased from
`76.6%` to `51.7%`; the denominators differ (`5653` versus `5562`) because V4
removed the invalid three-cycle clip and changed the admitted slow crop.

Exact failure report:
`/home/kaifaty/NextEngine-training/generations/humanoid-motor-rebuild-v1/evaluations/TRAIN-4/exhaustive-dynamic-feasibility-temporal-contact-v4-r8.json`,
SHA-256
`8719a0a893824333e9b72b247bb90bd0ea503b0e3119f1cecdcedfd5e4b6759b`.
It has `optimizer_steps=0`, `training_runs=0`, learned-policy claim false and
does not create a new `Advance` report. `REQ-HUM-DATA-007` remains failed and
`TRAIN-4` remains reopened.

The next bounded remediation is a contact-constrained whole-stance-chain solve
that jointly treats stance hip/knee/ankle pose, sole position/normal and planar
root trajectory over transition windows. A local per-frame IK prototype is not
admitted: it reduced median support slip but worsened transition tails after
smoothing. The production increment therefore requires a temporally coupled,
bounded solve under a new profile/corpus identity, followed by the same
deterministic, native-pose, visual and exhaustive dynamic gates. Parameter or
reward tuning and optimizer execution are not accepted substitutes.

## TRAIN-5 — specialist reference-motion tracker

Execution input is frozen by [ADR-070](../architecture/adr/070-biomechanics-reference-tracking-training-environment.md)
and `lab/profiles/humanoid-reference-tracker.v1.json` with SHA-256
`4a898ccf67051b34b6266ec5293f74758103e7db260ded393e76161f72de527d`.
This freezes environment semantics but is not evidence that the sanity ladder
or training has passed.

### Environment

Every episode selects one admitted clip and deterministic reference phase.
Reset modes are mixed by a fixed profile:

- exact reference-state initialization over the complete phase range;
- small pose/velocity perturbations around a reference state;
- neutral entry for start/idle clips;
- bounded near-failure states only after nominal tracking passes.

Observation includes current root-local kinematics, joint state, previous
applied action, contact/support facts, current reference phase and a short
bounded reference horizon. Reference features are root/CoM, joint or keypoint,
effector and contact targets in declared frames. Actor output remains only the
bounded per-joint residual target from `TRAIN-3`.

### Locked PPO/GAE profile

Первый optimizer остаётся PPO with GAE. Его config hash связывает не только
learning rate, но и:

- actor/critic topology and parameter sharing, initialization and optimizer;
- rollout horizon, batch/minibatch size, epoch/sample reuse and shuffle rule;
- discount, GAE lambda, terminated-vs-truncated bootstrap semantics;
- advantage/return and observation normalization scope, epsilon and frozen
  evaluation statistics;
- clipped policy/value objectives, entropy/value coefficients, gradient clip,
  target KL/early-stop and learning-rate schedule;
- Gaussian mean/log-std parameterization, log-std bounds, sampling mode and
  exact action transform.

Policy density и PPO ratio соответствуют фактическому transformed action:
если residual использует squashed Gaussian, `tanh` Jacobian входит в log-prob.
Нельзя вычислить log-prob для одного action, а затем скрыто заменить его
environment clamp. Trajectory записывает sampled pre-safety proposal, его
log-prob и canonical applied action после engine safety; deterministic
evaluation использует transformed mean без sampling noise.

`terminated` не bootstraps value; declared time-limit `truncated` bootstraps из
следующего valid observation. Critic может читать только явно перечисленные и
hash-bound training facts. Любой privileged critic input остаётся training-only,
не попадает в actor/export/replay authority и не может менять environment
state. Required diagnostics: episodic/task metrics, policy/value losses,
explained variance, approximate KL, clip fraction, entropy/action std,
gradient norm, clamp incidence, NaN/Inf and sample-efficiency curve.

До full run выполняется pre-acceptance sanity ladder:

1. random action, zero residual and scripted/reference-following baselines
   подтверждают reset/action/reward/termination wiring;
2. reward-component distribution and scale измерены отдельно; constant,
   non-finite, dominating or wrong-sign component блокирует training;
3. tiny deterministic batch reproduцируется, затем одна phase/clip намеренно
   overfit до ожидаемого поведения;
4. только после этого запускаются curriculum and multi-seed experiments.

Если sanity ladder не проходит, сначала исправляются environment,
observation/action, reward or reset semantics. Смена PPO на более сложный
algorithm не является диагностикой wiring failure.

Текущий new-lineage input audit SHA-256
`784f09d4b9e86c3053120df022f0e2eb422b85b264fb11964a930feb78dc26df`
замкнул profile/corpus/gate/artifacts и прошёл с `optimizer_steps = 0`. Он не
является learned-policy claim. Checkpoint с superseded corpus hash запрещён как
initialization input.

Randomized phase-reset audit SHA-256
`8cbd9c33b01ca98956b377f242628623ca943e11adce75178e20508ba145cb6d`
покрыл все `169/169` допустимых start phases в `4105` zero-residual episodes:
первый motor tick дал `0` hard-ROM, forbidden-contact and non-finite failures.
Полный 11-tick untrained baseline сохранил `690` hard-ROM, `209`
forbidden-contact и `589` tracking-loss events как ReportOnly diagnostic; это
не learned-policy quality result и не ослабляет final safety requirement.

Новый bounded-horizon tiny overfit выполнен дважды с нуля как
`tiny-cmu104-h11-r6-seed120812-r1/r2`: оба run дали deterministic completion
`0/64 -> 64/64`, mean episode length `10.953125 -> 11.0`, final failure count
`0`, `327680` samples и `1135` optimizer steps. Metrics SHA-256
`00b8719f15e3620aa4cbe88dc1ebe3baac0cfbe65de2e9065b00dfa3752aecd5`
и checkpoint SHA-256
`bda3df1d4770649e3f1cbf976997cb0d76a0a60ce88657f47c3e2a650e0a157b`
совпали побитово. Reproducibility report SHA-256
`b90dc9953c5ecd45c5c6c291e9e36053ce7f5a82a7861fb0de700b80df000742`
разрешает только curriculum и multi-seed. Новый frozen curriculum profile
SHA-256 — `d97c93adb39a32694fc2f882732e59561949c726e5827c13a844f8edb45d01d1`;
его evaluation использует outcome-independent `fixed-vector-waves-v1` matrix.

Корректный curriculum run
`curriculum-start-phase-h11-r6-seed120812-r4-wave-matrix` сравнил одну и ту же
matrix SHA-256
`61c6fccbf2d70cc94aec2b7f96df8aa12d4a6a466b8f31a43d930b55dc94f9da`.
Completion вырос с `166/256` до `184/256`, mean episode length — с
`9.69921875` до `10.16015625`, поэтому узкий profile acceptance равен `PASS`.
Но final matrix сохранила `37` hard-ROM, `6` forbidden-contact и `34`
tracking-loss failures: hard safety gate равен `FAIL`, multi-seed и `TRAIN-5`
Advance запрещены. Optimizer-free checkpoint diagnostic SHA-256
`a7d0e3b97dd1f60f1acedc60d2b5e78e11f929f8ebffa6148596e76db79532c5`
отнёс `27/37` hard-ROM failures к ankle channels. Optimization decision
SHA-256
`fce515e0b1010a74cb7be9103afe9f1d850852377622741be4f82a2434149d38`
классифицирует failure как `Optimization`, а не `Body` или `Data`.

Единственная заявленная следующая гипотеза — immutable child environment
profile SHA-256
`c482e68f05ad574b74ba037412a5d8b1d378966ac788de308b457885f7b0c35b`:
он добавляет `reward.soft-rom-excursion-cost` с coefficient Q16 `-65536`.
Cost равен нулю внутри descriptor soft ROM и линейно возрастает до единицы
между soft и hard bound; hard termination, action, PD/safety, PPO и curriculum
не меняются. Optimizer-free audit SHA-256
`087cbef5e3618784c86ba2ce71d3b334c5ee4a999abaf181ca4727797b7a3b27`
прошёл с `optimizer_steps = 0`: cost нулевой на natural/reference probes и
достигает unit scale на directed probe. Новый frozen tiny profile SHA-256 —
`5315f47aec64e827ffb336e7a380c0177d21616d4e789553b4fb3b319692fc14`.
Два run с нуля
`tiny-cmu104-h11-r6-soft-rom-cost-seed120812-r1/r2` дали одинаковые
`0/64 -> 64/64`, final failure count `0`, `327680` samples и `1159` optimizer
steps. Metrics SHA-256
`4beeb84ca0db3ee2413b5e9887e3a56caa2546b938b55b443d261ff9c808abab`
и checkpoint SHA-256
`30778aa6b8b4d3c1b0b5ff87118a025969dc035df461dedff9c03eccbbebe8e2`
совпали побитово. Reproducibility report SHA-256
`744db4ffbfa04cc479ceb0e60c82f55e7a6d4d750b0c122e47cb8102aa9a78e2`
разрешает только новый curriculum. Frozen soft-ROM-cost curriculum profile
SHA-256 — `9f19e257595ac38a005cc7c46b08b194851af1376e71289106e78db2c685e9c5`.
Его exact fixed-matrix run улучшил completion `156/256 -> 178/256` и
forbidden contacts `37 -> 9`, но hard-ROM ухудшился `34 -> 43`, включая `29`
ankle-channel events. Поэтому гипотеза rejected, checkpoint SHA-256
`856de3c9aed5887176ae1ab16b7588c62619ac53b5db28b1c06c41127f8dead0`
не admitted, multi-seed запрещён. Failure decision сохраняет exact report и
не превращает узкий curriculum `PASS` в safety claim.
Следующая отдельная hypothesis identity использует one-motor-tick predictive
ROM excursion вместо rejected realized-state cost. Environment overlay SHA-256
`2640aa58886b00c901240f9f2b8912cfcff74e5a8490e846ad69b35b4fedc3b5`,
tiny overlay SHA-256
`e90a4bb042bd4e62a7269d974a686e718e40ddc0623ee81da0091f2434747a75`.
Optimizer-free audit SHA-256
`9307ba4df987dfef8d96d2fc0b5ac98321c1ebc2e9b48eb90fac15f2f327cd6d`
прошёл: natural/reference median cost равен `0`, directed probe достигает
unit scale, optimizer steps равны `0`. До curriculum снова обязательны два
побитово воспроизводимых tiny run с нуля. Runs
`tiny-cmu104-h11-r6-predictive-rom-cost-seed120812-r1/r2` прошли `0/64 ->
64/64`, final failure count `0`; metrics SHA-256
`4e57075148959184c2f2ff3b9b18a8ac691d44fdbab8e0102c6d9f0a647fdd8f`
и checkpoint SHA-256
`29e1e703c36f368c6bc1ae50ba4416f497f8a1b5673739e2299827891d4369ed`
совпали побитово. Reproducibility report SHA-256
`1ea3e41e286963651b9f2959bef364c858b53907cf3de9caeaa0a27b5018ba2b`
разрешает только curriculum. Curriculum overlay SHA-256 —
`b658f8ecf35c6677293a8aa37f9003799e040388f0a2cf10975b5d6613224d87`.

Predictive fixed-matrix curriculum
`curriculum-start-phase-h11-r6-predictive-rom-cost-seed120812-r1-wave-matrix`
улучшил completion `167/256 -> 186/256` и forbidden contacts `33 -> 4`, но
hard-ROM ухудшился `28 -> 37`, tracking loss — `29 -> 34`. Final checkpoint
SHA-256
`d1ae1263e41466634e190e3a5d8c8b6230899a3ca0c8824b5aacb71d4068d360`
не admitted; failure decision SHA-256
`2a8f8d9c96fb601e6ee8e55402d9845406e1f63b281ff85010a5e18b7a92e8d1`
закрыл обе reward-only hypotheses и запретил multi-seed.

Optimizer-free state diagnostic на clean commit
`da9c98aa548358c4b9d04757e6af9904998a23c2` воспроизвёл те же `37`
hard-ROM episode failures; exact report SHA-256 —
`5cb40e4d3ba9a7061fb25c0931fc96d226a87fb454a39cac417f7c791b79cb83`.
В `45/45` violating channel events applied target находился внутри hard ROM и
был направлен обратно внутрь диапазона; `45/45` pre-physics states ещё
находились в разрешённом `10 µrad` observed tolerance, `37/45` имели outward
velocity, `29/45` пересекали границу по one-tick linear projection и `7/45`
уже превышали descriptor maximum velocity.

Source correspondence audit установил, что Rust TRAIN-3 controller проверяет
hard ROM/maximum velocity до effort publication и пересекает fixed-PD effort
с effort/rate/power/work limits, тогда как проверенная Isaac TRAIN-5
реализация применяла только effort/rate, не имела maximum-velocity failure и
не публиковала полный `terminal.joint-safety`. Decision artifact SHA-256
`92d4c45e29dbb8b146c9d3030bb06a030dc86893f3fa6cad8b62def3309bd81a`
классифицирует blocker как `EnvironmentCorrespondence`. Поэтому все
перечисленные выше TRAIN-5 input/reset/tiny/curriculum результаты остаются
historical diagnostic evidence, но больше не разрешают downstream work.
Эта remediation впоследствии завершена: exact actuator/terminal semantics
восстановлены в Isaac, `TRAIN-3` получил `Advance`, а dynamic-reserve corpus
закрыл `TRAIN-4`. Текущая correspondence lineage начинается с environment
profile SHA-256
`7061e43bc59097312c10e90ea566485116bca4b5ec40ab1e93e22919b2160b5d`
и corpus manifest SHA-256
`33546488a73db25557c23fdb1a54b066ac3d02384aaca9acb529dab5d4cc81fd`.
Optimizer-free input audit SHA-256
`f6993f565ec04871e56c116c9799581619c349bcf9124c20a3b52c2db78b8bfc`
и phase-reset audit SHA-256
`f1a8447711a7bb7b67ae4e8a584e44f2c2d613198710f16c4a93767b76f5fa99`
прошли; reset audit покрыл `169/169` phases и не зарегистрировал hard-safety
failure в reset window.

Первый h11 tiny sanity завершался tracking-loss на 11-м motor tick и был
отклонён. Единственное bounded изменение horizon до h10 затем прошло два
fresh run: `0/64 -> 64/64`, final failure count `0`; metrics SHA-256
`261315f4aef6e7a81685f9d847621704aa2286550b24f8d457cfa6bb526b3c34`
и checkpoint SHA-256
`16c33959cafe3e4e7a7a52f043ea974385cd5fe98de287b67a25fff16fd9343e`
совпали побитово. Reproducibility report SHA-256
`80130ec5e04b8b4f2e689189caf1447e67f46bdca321b76978f15ca2c398de21`
разрешил один phase-randomized curriculum run. Он улучшил completion
`37/256 -> 94/256`, но сохранил `162` final failed episodes, включая `99`
hard-impact and `44` hard-ROM failures. Failure decision SHA-256
`95602ddd82666f6dba626ca7335eb7e98d9bd8294c94a4269ca48d9a8b51851a`
отклонил checkpoint и разрешил проверить только одну contact-impact hypothesis.

Immutable contact-impact-margin child SHA-256
`6a8b7c5871c200377cec4895ebefe370861f83c20a060e77ea9055f88e82ca06`
добавил realized maximum contact-pair margin cost с warning boundary `9500`
basis points, не меняя hard limits или terminals. Input audit SHA-256
`34c8686b331e434dfa9430f4a8a1c22a863a90617485e4f33b5571f7ef9d250d`
и phase-reset audit SHA-256
`f6c7c819520627ef880b518b93ad0c58e971b76967c63576efeea3b45cc1c209`
прошли с `optimizer_steps = 0`. Два fresh h10 tiny run воспроизвели `64/64`
completion, zero final hard-safety, metrics SHA-256
`26822f4aa9d46b9ccb32727f2c4c41ebad1ebf26929d8313aa2489e935192d7b`
и checkpoint SHA-256
`087afb5a03bad41595e3e588d9dd61e30fe811a8095b59387434b7163ad543ba`.
Reproducibility report SHA-256
`a1dd7f718566f093549a3a3b15fe812280c1f41572c1773a8db6c7a754f6e33a`
разрешил ровно один curriculum run.

Contact-impact-margin curriculum сохранил fixed matrix SHA-256
`61c6fccbf2d70cc94aec2b7f96df8aa12d4a6a466b8f31a43d930b55dc94f9da`
и узко улучшил completion `34/256 -> 69/256` и mean episode length
`5.60546875 -> 7.75390625`, но final evaluation сохранила `187` failures и
`168` hard-safety failed episodes: `111` hard impact, `48` hard ROM, `34`
joint safety, `24` effort, `11` joint velocity и `3` self-collision. На той же
matrix это хуже base curriculum: hard impact `99 -> 111`, completion
`94 -> 69`, all failures `162 -> 187`. Optimizer-free terminal diagnostic
SHA-256
`b5d9cc190eaed9f67385c69b7ef8df6732ddc21dc927019f57b57abdddf245f1`
отнёс `109` impact episodes к right sole и `29/48` hard-ROM episodes к right
ankle pitch. Failure decision SHA-256
`f496f3e20c0f1743e7fa5094a7d13128df77b714e3256a64634a5a2c517ae7c5`
отклоняет reward hypothesis и checkpoint; multi-seed, `TRAIN-5` Advance и
`TRAIN-6` запрещены.

Последняя declared hypothesis,
`curriculum.deterministic-phase-prefix-expansion.v1`, была реализована и прошла
optimizer-free audit SHA-256
`4ff66bc23884a5e3b5953b9a067bdb595a1179f4ae6026afdc49bfb1c900a7aa`:
четыре 80-iteration ступени точно покрыли prefixes `42`, `84`, `126`, `169`,
повтор был byte-identical, evaluation отключила prefix. Единственный
hash-closed run улучшил completion `37/256 -> 78/256`, но сохранил `178`
failed episodes, из них `157` с hard-safety failure: `102` hard impact, `50`
hard ROM, `35` joint safety, `17` joint velocity, `19` effort и `3`
self-collision. Это хуже base curriculum (`94` completions, `162` failures).
Checkpoint SHA-256
`9f05a0a8307527585e35a7efecbbacbe9ef6700d9c76ada54f16d719539eaef6`
rejected. Optimizer-free terminal diagnostic SHA-256
`bdc6fbf5320a1f46e473018a083e48a63cd93936bab2005bd591b23e7eba5df4`
подтвердил прежний dominant right-sole impact/right-ankle pattern; diagnostic
repeat насчитал `104` impact reasons против `102` в source run, поэтому exact
deterministic safety claim также запрещён.

Общий failure decision SHA-256
`2832f50a6ab2b02ee1bd41acec1bf3f0d247555e22015ea396c50bb5c1da9c92`
классифицирует результат как `DataDynamicReferenceFeasibility`, а не новую
optimization hypothesis. До обучения exact-reference baseline уже имел
`4009/4117` full-horizon failed episodes: `3022` hard-impact, `618` hard-ROM,
`1102` joint-safety, `792` joint-velocity, `373` effort-envelope и `22`
self-collision reasons при нулевых reset-window safety failures. Поэтому
`REQ-HUM-DATA-007` required, прежний `TRAIN-4 Advance` superseded, `TRAIN-5`
checkpoint/resume и все новые optimizer runs запрещены до новой corpus/profile
identity и полного optimizer-free dynamic-feasibility `Pass`. Exact reopened
`TRAIN-4` gate report:
`/home/kaifaty/NextEngine-training/gates/TRAIN-4/locomotion-dynamic-feasibility-r9-gate-report.json`,
SHA-256
`2fc1d6c3d312a1e88a3820bdd851fd0eecf3230795a1162b55f13dd65d45cdb6`.

Manifest-wide optimizer-free audit r2 then enumerated all `28` admitted
locomotion clips and all `5653` valid start phases at horizon `11`; coverage is
`5653/5653`, but only `1324` cases completed without required safety failure and
`4329` failed. Required terminal counts are `3100` hard-impact, `846` hard-ROM,
`919` joint-safety, `610` joint-velocity, `339` effort-envelope and `93`
self-collision; fall, forbidden-contact, world-bounds and non-finite counts are
zero. Twenty cases fail in the first motor tick. Contact precision/recall are
`0.495467/0.854583` and remain `ReportOnly`.

The failure is corpus-wide: every clip has failed phases, including
`cmu140-idle` at `109/109`. Foot-ground impact is bilateral
(`ground:body.right-ankle-roll` `1668`, left `1572`); hard-ROM is dominated by
right/left ankle pitch (`410/397`), velocity by left/right ankle roll
(`319/274`), effort by the four ankle channels (`317/339`), and all `93`
self-collisions name the left/right hip-yaw pair. This fixes the remediation
order: first use a temporally coupled ankle trajectory solve, then a
contact-aware root/stance/swing-foot solve with stable contact intervals, and
then preserve bilateral hip clearance. Each correction must create a new
retarget/corpus identity before the same exhaustive audit is rerun.

The external report is
`/home/kaifaty/NextEngine-training/generations/humanoid-motor-rebuild-v1/evaluations/TRAIN-4/exhaustive-dynamic-feasibility-old-corpus-r2.json`,
SHA-256
`85e729e89406beb650ca2c62bc68590be576b0b7defe7ac60ec15bda97e18d09`.
It is diagnostic failure evidence only: `optimizer_steps=0`,
`training_runs=0`, learned-policy claim false. `TRAIN-4` remains reopened.

Следующие результаты сохранены только как historical failure/diagnostic
evidence старой corpus lineage и не продвигают текущий `TRAIN-5`: input/reward audit
SHA-256
`ccb6e3e78c76846dd3714258e09f102a69335879264625c3ee268816ecb00b5a`
проверил 84 observation/perfect-reward, 84 phase-offset, 84 cost и 84 terminal
fixtures. Все девять tracking-компонентов меняются на corpus probes; максимум
одного positive component равен `2857` basis points от positive scale при
блокирующем пороге `5000`. Native PhysX matrix SHA-256
`dc56b411c8ada67afbe8dddc1215bd4eda8c0fa55c581ad01d9c81a7b221c7ab`
содержит девять optimizer-free baselines: zero-residual idle завершил reference,
а слабые dynamic/random baselines достигли заявленных tracking/contact/safety
terminal branches. Решение разрешает только следующий tiny deterministic
overfit; learned-policy quality, held-out quality и сам `TRAIN-5 Advance` ещё
не заявлены.

Первый полный tiny run manifest SHA-256
`20ef6279508c1e3731fbfa0869f6654209415b4f872e7e49e15f60e9ef64abc6`
завершил 160 iterations, 327680 samples и 1018 optimizer steps. Средняя
deterministic episode length выросла с `8.359375` до `12.0`, но completion
остался `0/64`, поэтому результат `FAIL`. Terminal diagnostic SHA-256
`c3f4eb57f6a1bbd6b823e17254624b8fe71d4d52282ff16c6af91a8f68bfefa4`
зафиксировал `64/64 terminal.reference-tracking-lost` и нули для hard ROM,
forbidden contact и non-finite. Следующий tiny retry использует 11 motor ticks:
это выше untrained maximum `10`, ниже первого learned tracking-loss tick `12`
и не ослабляет последующие curriculum/full-reference проверки. Success на
границе horizon учитывается только при отсутствии одновременного failure.

Historical bounded-horizon runs `tiny-cmu104-h11-seed120812-r1/r2` независимо прошли
acceptance: deterministic completion `0/64 -> 64/64`, mean episode length
`8.359375 -> 11.0`, final failure count `0`. Их metrics и checkpoints побитово
совпали. Reproducibility report SHA-256
`a6704af4736c2ae45da8cfb84e7adceb8187f3d65bf189789ece9a1a715b2c2e`
остаётся доказательством superseded corpus lineage и ничего не разрешает в
текущей lineage.

Initial optimization order:

1. idle/weight shift;
2. one walk cycle with randomized phase;
3. start and stop specialists;
4. turn specialist;
5. one combined locomotion tracker after specialists pass.

The first actor is a feed-forward MLP. Recurrent architecture, AMP and command
randomization are not introduced to rescue a failing nominal tracker.

### Reward groups

- root orientation/height/linear and angular velocity tracking;
- joint pose and velocity tracking normalized by per-joint ROM/velocity;
- root/CoM and end-effector trajectory tracking;
- contact schedule and support transition accuracy;
- foot slip, impact, energy/effort and action/acceleration smoothness;
- profile-owned soft-ROM excursion warning cost when a pre-registered
  optimization child explicitly activates it;
- terminal failure outside the tracking reward.

Every component cites engine facts and is manifest/hash bound. Tracking weights
are fixed before the run; a visual symptom is not patched by an unrecorded
Python coefficient.

### Preliminary quality evaluation

On disjoint held-out phases and held-out clips of every admitted nominal
locomotion class, under the common statistical protocol, report against these
reference targets:

- full-reference completion at least 95%;
- mean joint-limit clamp incidence below 0.1% of motor ticks;
- root/CoM and mean per-joint position errors within the locked reference
  targets from the requirements baseline;
- sole contact precision and recall each at least 0.90;

`MetricReports=ReportOnlyComplete` requires every declared clip/seed and metric,
including unsuccessful samples, to be present. Target misses are retained in
the report but do not block the next stage. Independently, required checks must
show:

- forbidden locomotion contact raw count exactly 0;
- no NaN/Inf and no safety violation;
- visual review accepts posture, spine direction, knee/elbow motion, foot
  planting and absence of skating/propping.

The reference targets are frozen before the first evaluation run, not selected
after it. This stage cannot claim that the tracker "passed quality"; it can
claim only that the quality evaluation is complete.

### Failure/rollback

Failure is classified before another run:

- `Body` — axis/ROM/mass/collider/PD error; return to `TRAIN-1..3` and invalidate
  downstream artifacts;
- `Data` — retarget/contact/phase error; return to `TRAIN-4` with new corpus
  hash;
- `Mirror` — CPU/Isaac divergence; stop GPU training until correspondence;
- `Optimization` — architecture/reward/curriculum issue; change one declared
  hypothesis and start a new run ID;
- `VisualOnly` — metric report is complete but motion is unacceptable; stage remains
  failed and adds a measurable diagnostic before rerun.

Failed payloads are removed after the failure manifest and reproducer are
closed. Best-checkpoint cherry-picking across undeclared evaluation points is
forbidden.

### Commit boundaries

1. `feat(training): add humanoid reference tracking environment`
2. `feat(training): add phase-randomized tracking curriculum`
3. `test(training): add held-out motion tracking evaluation`

## TRAIN-6 — reference-conditioned command locomotion

### Runtime/training shape

```text
velocity/facing command
  → deterministic skill/phase selector
  → idle/start/walk/stop/turn bounded reference horizon
  → closed-loop residual tracker
  → fixed safety + PD
  → Physics
```

Direct bounded reference conditioning is the fixed-humanoid baseline. A
`PhysicalActionChunkV1` consumer is introduced here only after a
consumer-backed ADR/profile fixes its exact fields, cadence and hash closure;
TRAIN-6 does not require the not-yet-consumed chunk path to learn command
locomotion.

The controller is not asked to jump directly from idle reference to arbitrary
2 m/s velocity. Command profiles are promoted in order:

1. idle only;
2. idle → start at 0.25–0.5 m/s;
3. steady forward speed bands;
4. walk → stop;
5. bounded speed changes;
6. gradual yaw/facing turns;
7. combined forward+turn;
8. backward/strafe only after separate reference data is admitted.

Initial linear command acceleration is at most `0.5 m/s²`. It may increase
only after start/stop gates pass. Hard action target slew remains independent
of command acceleration.

Training begins with tracker parameters frozen or a much smaller tracker
learning rate while command selection/conditioning stabilizes. Joint fine-
tuning is enabled only with a retention suite over all `TRAIN-5` clips.

### Evaluation matrix

Evaluate each cell from the exact operating envelope and command grid in the
requirements baseline separately, never only aggregate episode length:

- idle;
- start-left/start-right;
- slow/nominal speed hold;
- stop-left/stop-right;
- left/right turn;
- speed-up/slow-down;
- combined forward+turn;
- command interruption near single/double support.

Report completion, fall, forbidden contacts, velocity/facing error, start
latency, stop overshoot, support/contact timing, foot slip, impact, energy,
joint/action clamp incidence and visual result.

### Exit criteria

Все percentage/error/retention metrics применяются per cell and aggregate under
the common statistical protocol; exact episode counts and
confidence/non-inferiority interpretation фиксируются до evaluation run.
Следующие значения — report-only reference targets:

- every required matrix cell completes at least 95% of held-out episodes;
- aggregate flat fall rate is at most 1%;
- velocity RMSE at most `0.20 m/s` after the declared transition window;
- median absolute facing error at most `7°`;
- idle/start/walk/stop tracker retention does not regress beyond the
  pre-registered non-inferiority margin;

`MetricReports=ReportOnlyComplete` требует полный отчёт по каждой required
cell, включая start latency, stop settling/overshoot, slip, energy and
facing-tail из `REQ-HUM-Q-008..011` и `REQ-HUM-Q-015`; значения не определяют
`Advance`. Required exit conditions:

- every declared cell and left/right lead/transition class is executed with
  the pre-registered episode count;
- frozen canonical conformance scenarios for every function in
  `REQ-HUM-F-001..004` reach their declared route/terminal state;
- forbidden locomotion contact raw count exactly 0;
- no command change causes action target or actuator safety violation;
- visual review accepts both left/right leads and every transition class.

### Failure/rollback

If the required start-route check fails while the steady route works, add/fix
start references and stage selection; do not weaken fall/contact termination.
A report-only retention miss may motivate another command-layer run, but does
not roll back an otherwise advancing candidate. The old pure-PPO model is never
a rollback target.

### Commit boundaries

1. `feat(motor): select locomotion references from typed commands`
2. `feat(training): add start stop and facing curriculum`
3. `test(training): add command transition retention matrix`

## TRAIN-7 — brace, safe fall, recovery and get-up

### Separate route

Use an explicit motor state machine:

```text
Locomotion
  → Unstable
  → Brace | SafeFall
  → FallenProne | FallenSupine | FallenSide
  → GetUp
  → Stabilize
  → Locomotion
```

Transitions depend only on canonical orientation, root/CoM, contact, support,
joint safety and skill state. Renderer/wall time cannot choose the route.
Если этот graph, phase/contact contract или policy state становится
public/current production semantic, до его реализации нужен consumer-backed
ADR/SPEC update с точными replay/persistence/hash rules. Private trainer state
не может молча стать runtime authority.

Recovery training uses dedicated specialist data and reset distributions:

- near-balance-loss states around locomotion;
- controlled pushes that remain recoverable without falling;
- prone, supine and side poses sampled within anatomical/contact bounds;
- intermediate reference states from brace/get-up clips;
- contact-rich hand/knee support permitted only by recovery phase.

Exact push direction/magnitude/command cells, four fallen reset classes and
time bounds come from requirements sections 3.3 and 6.1. A side reset either
uses its admitted side transition to prone/supine or fails; it cannot jump to
an unobserved get-up phase.

Perturbation, friction/mass/load and latency randomization starts narrow and is
expanded only after nominal get-up passes. It is fully manifest-bound.

### Exit criteria

Percentage and time-bound metrics проходят общий statistical protocol по
каждой reset/push class и aggregate; evaluation seeds не используются для
curriculum or candidate selection. Следующие значения — report-only reference
targets:

- at least 95% of declared recoverable pushes return to stable locomotion
  without forbidden locomotion contact after recovery handoff;
- at least 90% of held-out prone/supine/side resets reach `Stabilize` within
  `6.0 s`;
- at least 90% resume the prior bounded command after get-up;

`MetricReports=ReportOnlyComplete` требует результаты для всех push cells и
четырёх reset classes. Target miss сохраняется в отчёте и не блокирует
`Advance`. Required exit conditions:

- every prone, supine, left-side and right-side reset class executes the
  declared route; side recovery may transition through admitted prone/supine
  state but may not be skipped or marked `NotApplicable`;
- frozen canonical conformance scenarios for every function in
  `REQ-HUM-F-005..007` reach their declared route/terminal state;
- no recovery pose exceeds hard ROM/effort/velocity/contact-impact bounds;
- locomotion never silently reclassifies sustained hand/knee support as success;
- route/contact/action/state roots replay exactly;
- visual review accepts brace direction, limb motion, get-up posture and
  transition back to walking.

The report-only reference bounds are `6.0 s` to `Stabilize` and `2.0 s` from
`Stabilize` to the resumed command band, per `REQ-HUM-Q-013..014`.

### Failure/rollback

Recovery failure leaves the subject in declared ragdoll/safe-stop behavior; it
does not keep chasing a locomotion reference from the floor. A failed combined
route retains the passing locomotion and recovery specialists separately until
handoff is fixed.

### Commit boundaries

1. `feat(motor): route fall brace recovery and get up phases`
2. `feat(training): train dedicated humanoid recovery specialists`
3. `test(motor): cover recovery handoff and command resume`

## TRAIN-8 — optional motion prior and specialist distillation

This stage is optional for the first useful locomotion candidate. It begins
only after `TRAIN-5..7` advance without a prior.

До первого TRAIN-8 experiment принимается одно из двух immutable решений:

- `OptionalSelected`: фиксируются hypothesis, baseline, budget and retention
  protocol, после чего failure нельзя переименовать в skip;
- `Skipped(NotNeededForCandidate)`: advancing TRAIN-7 candidate напрямую идёт в
  TRAIN-9, а prior/distillation не входит в candidate lineage.

Candidate work:

- AMP-style discriminator/style reward trained only from admitted motion data;
- structured motion inpainting/masked reference generation for future chunks;
- immutable specialist teacher trajectories;
- supervised distillation into one fixed-humanoid student;
- joint PPO fine-tuning with complete retention suite;
- morphology/family-conditioned critic remains training-only and is never
  exported into actor inputs.

### Exit criteria

- every `TRAIN-5..7` quality/retention metric is re-run and reported against
  its pre-registered reference target;
- visual review passes under a predeclared playlist and defect protocol;
- no new hidden state, runtime motion-data dependency or unsupported export op;
- student action/state remains exact and runtime cost is completely reported;
- all dataset, teacher and child lineage is exact and license-compatible.

Quality, retention and runtime target misses do not by themselves fail this
optional stage. If it advances, the choice of distilled or specialist route
must follow a selection rule frozen before evaluation and cannot be based on
post-hoc target crossing.

### Failure/rollback

Reject and delete the prior/student payload after a required-check or visual
failure. Retain the advancing specialist candidate from `TRAIN-7`; do not
rewrite reference targets or the predeclared route-selection rule to admit
distillation.

### Commit boundaries

1. `feat(training): add admitted humanoid motion prior`
2. `feat(training): distill humanoid motor specialists`
3. `test(training): enforce multi-skill retention`

## TRAIN-9 — export, runtime parity and publication candidate

### Export

1. Export every selected actor as a fixed-shape standard-op graph with explicit
   state if any, plus the deterministic engine-owned route manifest.
2. Reopen and validate exact model bytes, route manifest and capability closure.
3. Bind BodySchema, compiled descriptor, observation/action/state,
   normalization, safety, PD, dataset, training config and fallback hashes.
4. Compare trainer → exported evaluator → Windows/Linux runtime over one
   golden observation/action/state corpus.
5. Run final held-out suite in production `headless`; Isaac results alone are
   insufficient.
6. Activate only through a new immutable candidate bundle and explicit project
   lock in a new session.

### Final required gate and report-only evaluation

Report-only evidence on the portable candidate includes flat locomotion
survival against `99%`, velocity RMSE against `0.20 m/s`, facing error against
`7°`, all other `TRAIN-5..8` quality/retention targets, and declared inference
latency/model-size/memory budgets. Every cell/seed/result must be present, but
target crossing does not admit or reject the candidate.

Required checks are:

- all declared locomotion, transition, push and four fallen-reset classes are
  executed in the production-headless suite;
- forbidden contact and hard-safety violations are exactly zero;
- canonical applied action and complete policy state are exact across trainer
  decode, exported evaluator and supported runtime targets;
- no NaN/Inf, safety violation, hidden evaluator state or runtime motion-corpus
  dependency;
- procedural/animation/ragdoll fallback activates under every declared model,
  schema, state and evaluator fault;
- final human visual review passes a fixed seed/command/perturbation playlist.

Final publication claim is limited to section 1.1 of the requirements
baseline. Terrain/R5/`Supported`/full `MOTOR-HUMANOID-MVP-P1` cannot be inferred
from this gate. Multi-specialist and distilled-single-actor candidates run the
same final suite; exactly one route supplies the complete action on each tick.

### Failure/rollback

Export/parity, required safety/contract or visual failure rejects exported
bytes and retains only the advancing external trainer candidate long enough to
diagnose/re-export. A quality/runtime target miss is recorded and may motivate
a later candidate, but does not roll back this one. No partial project-lock
activation or active-session hot swap is allowed.

### Commit boundaries

1. `feat(motor): evaluate portable humanoid policy artifacts`
2. `feat(training): export fixed humanoid motor candidate`
3. `test(motor): verify runtime parity and fallback routes`

## Failure ledger

The repository stores no heavy run artifact, but the implementation work
maintains one bounded human-readable ledger for unresolved failures. One row:

| Field | Meaning |
|---|---|
| Failure ID | Stable `HUM-MOTOR-###` identifier |
| First failing gate | Earliest invalid stage, not the stage that merely exposed it |
| Classification | Body, Safety, Contact, Data, Mirror, Optimization, Handoff, Export, Runtime |
| Exact identity | implementation/config/body/dataset/run hashes and complete seed set |
| Expected/actual | Typed values and first divergent tick/contact/joint |
| Reproducer | Bounded command or external trajectory prefix reference |
| Decision | FixInStage, InvalidateDownstream, StopLane or Won'tDo with reason |
| Payload retention | deletion timestamp or explicitly justified temporary retention |

Rules:

- no retry-to-green;
- no deletion of a failing seed from the declared set;
- no promotion by mean episode length alone;
- no simultaneous body, reward and optimizer changes in one diagnostic run;
- same failure under a new config remains a new run linked to the original;
- changing a hard gate requires a documented plan/architecture revision before
  rerun, not an edit to a completed report.

## Dependency and invalidation rules

| Change | Invalidates |
|---|---|
| Body topology, joint frames/limits or colliders | `TRAIN-2..9` |
| Mass/CoM/inertia, actuator or PD/safety profile | `TRAIN-2..9` |
| Contact/termination semantics | `TRAIN-3..9` |
| Retarget profile or motion/contact correction | affected `TRAIN-4..9` corpus lineage |
| Observation/action/reference/state schema | `TRAIN-5..9` |
| Reward/curriculum/perturbation profile | dependent run and `TRAIN-5..9` evaluation |
| Command selector or skill phase graph | `TRAIN-6..9` |
| Recovery state/contacts | `TRAIN-7..9` |
| Export operator/numeric profile | `TRAIN-9` parity only if training schemas unchanged |

An invalidated gate returns to `NotRun`; an old report is not relabelled for a
new hash.

## ProductCheck mapping

During implementation use risk-scoped focused checks. Before a gate handoff,
report every applicable check honestly:

| Scope | Required checks |
|---|---|
| BodySchema/contracts/compiler | `fast`, `content-package`, `BODY-SCHEMA-P1`, `PHYS-JOINT-P1` |
| Physics/contact/safety/replay | `fast`, `play`, `persistence-replay`, `PHYS-COLLISION-P1`, `PHYS-SNAPSHOT-P1`, `MOTOR-SAFETY-P1`, `MOTOR-STATE-P1` |
| Animation/retarget/reference | `fast`, `play`, `content-package`, `ANIM-RETARGET-P1`, future `ANIM-HYBRID-P1` consumer gate |
| Training environment/mirror | `host-check`, `MODEL-DATAPLANE-P1`, `MODEL-MIRROR-P1` plus focused correspondence |
| Learned actor | `MODEL-STATISTICS-P1`, retention/recovery checks; future `MOTOR-HUMANOID-MVP-P1` only when claiming that broader profile |
| Export/runtime | `MODEL-EXPORT-P1`, `POLICY-01`, future `MOTOR-REPLAY-P1`, `play`, `persistence-replay`, conditional `platform`; `performance` is recorded as report-only evidence for this candidate |

Future checks remain `NotRun(NoProductionConsumer)` until their consumer and
exact profile exist; this plan does not declare them passed. A future
ProductCheck with mandatory quality/runtime thresholds is not weakened by this
candidate baseline: missing its thresholds only prevents the corresponding
broader ProductCheck/profile claim, not the narrower claim in requirements
section 1.1.

## Visual acceptance playlist

Every required visual review uses the same pinned viewer build, camera and
slow-motion option, and includes at least:

1. neutral front/side/back anatomy sweep;
2. each joint group at both limits;
3. idle and weight shift;
4. left/right start;
5. slow and nominal walk from front/side/rear;
6. left/right stop;
7. left/right turn and command interruption;
8. recoverable push from four horizontal directions;
9. brace/safe fall;
10. prone, supine and side get-up;
11. resume command after get-up;
12. deliberately forbidden hand-prop locomotion, which must terminate.

Review records structured defect IDs: anatomy, balance, contact, foot plant,
spine, limb direction, impact, transition, recovery or presentation-only.
Presentation defects cannot waive a physics defect and vice versa.

## Implementation order and WIP discipline

Only one gate is `InProgress`. Permitted preparatory parallel work is limited
to read-only research, motion-rights review and generated test design; it
cannot publish a later-stage contract or start training before its dependency
gate passes.

Manifest-wide optimizer-free `TRAIN-4` diagnosis now reaches V18/R14:
`12518/12518` cases complete, with `204` required-safety failed cases rather
than V4's `2876`, but the required result remains exact zero. The bounded
[causal research cycle](../development/humanoid-train4-causal-research-2026-08-14.md)
replayed the exact schedule under controller/reset counterfactuals, rejected
target lead/feed-forward and naive velocity zeroing, confirmed an insufficient
root link/CoM semantic defect, and selected contact-consistent reference/reset
as the next implementation direction.

The first bounded V19 prototype over three representative clips and matched
passing controls is complete and rejected by R27. It closes active sticking
points offline but permits the inactive `cmu139` swing-foot collider to start
`3.544..14.905 mm` below ground, regresses passing controls and proves indexed
running-scene reset non-equivalent. The detailed
[prototype decision](../development/humanoid-train4-v19-prototype-research-2026-08-14.md)
retains ADR-070 fresh-scene authority and makes partial reset diagnostic-only.

R34/R35 selected the V5 post-smoothing boundary closure. R39 then exposed an
unsupported delayed landing at `cmu16@249`, and R45 showed that a
one-microradian quantized correction can cross hard ROM at `cmu16@415` only in
the full inventory layout. The bounded
[support-authorization research](../development/humanoid-train4-support-authorization-research-2026-08-14.md)
therefore permits high swing clearance only with active support and freezes a
one-micrometre deadband above the unchanged collider floor. V7/R47 passes
offline `17/17`; V7/R49 passes fresh `17/17` with every required-safety count
zero and no passing-control regression.

This executable increment is still not a full corpus rebuild. Independently
solved overlapping windows disagree on the value of the same source frame, and
an 801-frame `cmu16` counterfactual fails complete-clip closure. Therefore even
a fresh `17/17` bounded result permits only a clip-global prototype. That successor
must solve every clip once, prove exact overlap identity and pass both selected
windows and complete-clip invariants before a full 27-clip V19 identity or
repeated local/native/visual/exhaustive gates are allowed. No new PPO run is
authorized before a new `TRAIN-4 Advance`; no command locomotion run is
authorized before `TRAIN-5` advances.

Roadmap status changes only after material implementation/check results. This
planning document alone does not close R5, B-08, B-12, Stage 0, GPU
correspondence, Linux parity or any learned-policy ProductCheck.

## Definition of done

The rebuilt process is complete when:

1. old model/run artifacts are absent from active selection, resume and import
   closure, while retained historical evidence is inventoried and hash-bound;
2. the new BodySchema passes anatomy/compiler/contact/safety gates without ML;
3. an admitted reproducible motion corpus drives deterministic retargeting;
4. specialist tracking, command locomotion and mandatory side-inclusive
   recovery each pass independent required safety/contract and visual gates,
   with complete held-out metric reports;
5. optional prior/distillation, if selected, passes required checks and
   completes its retention report; otherwise TRAIN-8 records
   `Skipped(NotNeededForCandidate)` before TRAIN-9;
6. the portable immutable candidate bundle reproduces canonical actions/state
   in production `headless` on Windows and Linux and has complete report-only
   quality/runtime measurements;
7. every failure has a bounded reproducer/decision and no heavy failed payload
   remains without an explicit temporary reason;
8. runtime retains a declared procedural/animation/ragdoll fallback and never
   depends on trainer, dataset or mutable weights;
9. every required requirement in the linked baseline has an evidence-backed
   `Pass`, every report-only requirement has `ReportOnlyComplete`, and
   publication makes no quality-threshold, terrain/R5 or `Supported` claim.
