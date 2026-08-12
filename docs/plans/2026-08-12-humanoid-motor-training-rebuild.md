# План перестройки обучения humanoid motor policy

| Поле | Значение |
|---|---|
| Статус | Draft implementation plan |
| Дата | 2026-08-12 |
| Scope | Новый fixed-humanoid путь: biomechanics → motion tracking → command locomotion → recovery → export |
| Не является | ADR, доказательством качества модели или разрешением пропустить ProductCheck |
| Архитектурная опора | SPEC-05, SPEC-14, SPEC-26, SPEC-27, SPEC-28, SPEC-34, SPEC-35, ADR-027, ADR-053, ADR-058, ADR-059, ADR-066 |

Нормативные источники для реализации:

- [SPEC-05 — physics, animation and motor control](../architecture/05-physics-animation-and-motor-control.md);
- [SPEC-14 — physical archetypes, motor skills and policy lifecycle](../architecture/14-physical-archetypes-motor-skills-and-policy-lifecycle.md);
- [SPEC-26 — physics world, contacts and canonical snapshots](../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md);
- [SPEC-27 — motor observation, action and deterministic inference](../architecture/27-motor-observation-action-and-deterministic-inference.md);
- [SPEC-28 — skeletal animation, retargeting and IK](../architecture/28-skeletal-animation-retargeting-and-ik.md);
- [SPEC-34 — training environments and trajectory lifecycle](../architecture/34-model-training-environments-trajectories-and-consolidation-lifecycle.md);
- [SPEC-35 — deterministic humanoid training substrate](../architecture/35-deterministic-humanoid-training-substrate.md);
- [ADR-027 — physics/motor/animation layering](../architecture/adr/027-physics-motor-and-animation-layering.md),
  [ADR-053 — immutable artifact boundary](../architecture/adr/053-engine-native-model-training-and-immutable-artifact-boundary.md),
  [ADR-058 — PhysX training substrate](../architecture/adr/058-physx-only-deterministic-humanoid-training-substrate.md),
  [ADR-059 — continuation reconstruction](../architecture/adr/059-event-sourced-physx-continuation-reconstruction.md) and
  [ADR-066 — contact-centric motor architecture](../architecture/adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md);
- [current roadmap and ProductCheck status](../roadmap.md).

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
  → portable immutable actor and final production-headless evaluation
```

PPO остаётся optimizer для closed-loop tracking, task fine-tuning и
robustness. Он больше не отвечает за одновременное изобретение анатомии,
походки, переходов, баланса и recovery с нуля.

План считается завершённым только после прохождения `TRAIN-9`. Создание новой
BodySchema, запуск trainer или наличие checkpoint сами по себе не закрывают ни
одну стадию.

## Решение по старой линии

Pure-PPO checkpoints, optimizer state, replay buffers, run directories,
TensorBoard logs, evaluation captures и экспортированные модели, созданные для
старого одинаково ограниченного 23-DoF тела, не являются входом, baseline или
rollback нового процесса.

Перед первым новым training run выполняется cleanup:

1. по внешним run manifests определяется полный список артефактов старых
   `isaac-rsl-rl-*` экспериментов и их производных;
2. exact paths проверяются read-only;
3. run/checkpoint/evaluation/capture payloads удаляются из внешнего training
   store;
4. active launch presets перестают выбирать старые standing/flat-command pure-
   PPO experiment profiles;
5. новый training generation имеет отдельный корень и новый BodySchema hash;
6. поиск по active config/run indexes подтверждает отсутствие ссылки на старую
   модель или checkpoint.

Удаление не распространяется на Git history, принятые ADR/SPEC и engine tests.
Они не участвуют в inference или training selection и остаются историей
решений/регрессий. Retire или supersede Accepted profile semantics можно только
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
  immutable export.

Не входит:

- сохранение качества или совместимости старых pure-PPO policies;
- direct torque, learned/adaptive PD gains или muscle activation;
- graph policy, arbitrary topology или cross-family transfer;
- weapons, manipulation, parkour, terrain beyond the explicitly staged
  locomotion set;
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
8. Stage gate фиксируется до quality run. Нельзя менять threshold после
   просмотра результата или выбирать лучший seed вместо полного набора.
9. Visual review обязателен, но не заменяет numeric/contact/safety gates.
10. Failed run сохраняет bounded manifest, metrics и минимальный reproducer;
    тяжёлые checkpoint/log/capture payloads после triage удаляются.

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
QualityChecks:  NotRun | Pass | Fail
VisualReview:   NotRun | Pass | Fail | NotApplicable
```

Следующая стадия может начаться только при `Implementation=Complete`,
`ExactChecks=Pass`, required `QualityChecks=Pass` и required
`VisualReview=Pass`. Git commit является checkpoint кода, а не gate result.

Каждый gate report содержит:

- gate ID и implementation commit;
- BodySchema/compiled descriptor/environment/dataset/config hashes;
- exact commands, seed set и episode counts;
- passed/failed/not-run checks;
- primary и safety metrics без отбрасывания run/seed;
- stable first failure и минимальный reproducer;
- ссылки только на существующие external artifacts;
- решение `Advance`, `FixInStage`, `InvalidateDownstream` или `StopLane`.

## Сводка стадий

| Gate | Результат | Training разрешён после gate |
|---|---|---|
| `TRAIN-0` | старая artifact line удалена и изолирована | нет |
| `TRAIN-1` | принята exact biomechanics specification | нет |
| `TRAIN-2` | BodySchema компилируется в корректную PhysX articulation | нет |
| `TRAIN-3` | safety/contact/terminal semantics работают без ML | нет |
| `TRAIN-4` | motion corpus лицензирован, retargeted и физически валиден | tracker only |
| `TRAIN-5` | specialist reference tracker проходит held-out clips | command fine-tuning |
| `TRAIN-6` | start/stop/velocity/facing locomotion проходит gates | perturbation/recovery |
| `TRAIN-7` | brace/fall/get-up/resume route проходит gates | motion prior/multi-skill |
| `TRAIN-8` | optional prior/distilled actor не ухудшает базовые skills | export |
| `TRAIN-9` | portable actor совпадает с runtime и проходит headless suite | candidate publication |

## TRAIN-0 — закрытие старого эксперимента и чистая generation

### Реализация

1. Инвентаризировать external run roots по manifest/config/BodySchema hash.
2. Составить exact deletion list без glob по broad workspace root.
3. Удалить payloads старой линии и пустые run indexes.
4. Создать новый внешний generation root.
5. Ввести один active `training_generation_id` и запрет resume/import старых
   BodySchema/environment hashes.
6. Удалить старые experiment presets из default launch/view commands. Старый
   профиль может оставаться доступен только из explicit historical test path,
   пока Accepted architecture не superseded.
7. Добавить preflight, который fail-closed при старом model/checkpoint/config
   input.

### Exit criteria

- active generation index не содержит старых run/model references;
- old checkpoint переданный новому launcher получает stable incompatible-
  generation diagnostic до simulator creation;
- новый run root пуст и содержит только generation manifest;
- disk scan по exact inventoried paths подтверждает удаление payloads;
- source tree и ignored external store не смешиваются.

### Failure/rollback

Не выполнять recursive deletion по unresolved path, glob, `$HOME` или
workspace root. При неоднозначном manifest deletion останавливается до exact
target resolution. Rollback старой model line отсутствует: неуспех cleanup
блокирует новый training, но не восстанавливает старые checkpoints.

### Commit boundary

`chore(training): start clean humanoid motion generation`

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
- the accepted table and its canonical hash are frozen before compiler work.

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

## TRAIN-4 — motion corpus, retargeting and reference closure

### Minimum corpus

The first admitted corpus contains only motions needed by the next two stages:

- neutral idle and weight shift;
- start with left and right lead;
- slow and nominal forward walk;
- stop from left and right support phase;
- gradual left/right turn;
- brace and safe fall;
- prone and supine get-up.

Fast walk/run, backward, strafe, crouch and terrain motions are later corpus
revisions and cannot silently appear in the first evaluation split.

### Admission

For each source record capture:

- source owner/provider and exact source hash;
- license text/hash and intended training/model-output/distribution rights;
- consent where own capture includes identifiable people;
- source skeleton/coordinate/rate profile;
- conversion and cleanup tool hashes;
- permitted train/validation/held-out assignment.

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
  admitted clip.

### Exit criteria

- all corpus entries have compatible rights and complete provenance;
- deterministic re-import produces identical retargeted hashes;
- train/validation/held-out splits are frozen by clip/actor/source identity;
- 100% admitted clips pass anatomy, penetration, contact and phase checks;
- a generated kinematic preview is visually accepted before policy training.

### Failure/rollback

One invalid clip is excluded and produces a stable corpus diagnostic; it does
not relax BodySchema limits. Any correction creates a new retarget/corpus hash
and invalidates dependent runs.

### Commit boundaries

1. `docs(architecture): admit reference tracking environment contract` when
   a public/current semantic change is required
2. `feat(animation): retarget physical humanoid references`
3. `feat(training): validate motion corpus contacts and provenance`

## TRAIN-5 — specialist reference-motion tracker

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
- terminal failure outside the tracking reward.

Every component cites engine facts and is manifest/hash bound. Tracking weights
are fixed before the run; a visual symptom is not patched by an unrecorded
Python coefficient.

### Preliminary quality gate

On held-out phases and held-out clips of every admitted locomotion class:

- full-reference completion at least 95%;
- forbidden locomotion contact rate exactly 0;
- mean joint-limit clamp incidence below 0.1% of motor ticks;
- root/CoM and mean per-joint position errors reported with a locked threshold
  chosen in the pre-run evaluation manifest;
- sole contact precision and recall each at least 0.90;
- no NaN/Inf and no safety violation;
- visual review accepts posture, spine direction, knee/elbow motion, foot
  planting and absence of skating/propping.

The exact pose-error thresholds are frozen from retarget resolution and body
scale before the first acceptance run, not selected after it.

### Failure/rollback

Failure is classified before another run:

- `Body` — axis/ROM/mass/collider/PD error; return to `TRAIN-1..3` and invalidate
  downstream artifacts;
- `Data` — retarget/contact/phase error; return to `TRAIN-4` with new corpus
  hash;
- `Mirror` — CPU/Isaac divergence; stop GPU training until correspondence;
- `Optimization` — architecture/reward/curriculum issue; change one declared
  hypothesis and start a new run ID;
- `VisualOnly` — numeric gates pass but motion is unacceptable; stage remains
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
  → idle/start/walk/stop/turn reference or bounded chunk
  → closed-loop residual tracker
  → fixed safety + PD
  → Physics
```

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

Evaluate each cell separately, never only aggregate episode length:

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

- every required matrix cell completes at least 95% of held-out episodes;
- aggregate flat fall rate is at most 1%;
- forbidden locomotion contact rate exactly 0;
- velocity RMSE at most `0.20 m/s` after the declared transition window;
- median absolute facing error at most `7°`;
- idle/start/walk/stop tracker retention does not regress beyond the
  pre-registered non-inferiority margin;
- no command change causes action target or actuator safety violation;
- visual review accepts both left/right leads and every transition class.

### Failure/rollback

If steady tracking passes but start fails, add/fix start references and stage
selection; do not weaken fall/contact termination. If policy fine-tuning
regresses tracker retention, restore the passing `TRAIN-5` teacher and retrain
the command layer with a new run/config. The old pure-PPO model is never a
rollback target.

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

Recovery training uses dedicated specialist data and reset distributions:

- near-balance-loss states around locomotion;
- controlled pushes that remain recoverable without falling;
- prone, supine and side poses sampled within anatomical/contact bounds;
- intermediate reference states from brace/get-up clips;
- contact-rich hand/knee support permitted only by recovery phase.

Perturbation, friction/mass/load and latency randomization starts narrow and is
expanded only after nominal get-up passes. It is fully manifest-bound.

### Exit criteria

- at least 95% of declared recoverable pushes return to stable locomotion
  without forbidden locomotion contact after recovery handoff;
- at least 90% of held-out prone/supine/side resets reach `Stabilize` within
  the declared time bound;
- at least 90% resume the prior bounded command after get-up;
- no recovery pose exceeds hard ROM/effort/velocity/contact-impact bounds;
- locomotion never silently reclassifies sustained hand/knee support as success;
- route/contact/action/state roots replay exactly;
- visual review accepts brace direction, limb motion, get-up posture and
  transition back to walking.

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
only after `TRAIN-5..7` pass without a prior.

Candidate work:

- AMP-style discriminator/style reward trained only from admitted motion data;
- structured motion inpainting/masked reference generation for future chunks;
- immutable specialist teacher trajectories;
- supervised distillation into one fixed-humanoid student;
- joint PPO fine-tuning with complete retention suite;
- morphology/family-conditioned critic remains training-only and is never
  exported into actor inputs.

### Exit criteria

- every `TRAIN-5..7` primary/safety threshold remains non-inferior;
- visual motion quality improves under a predeclared metric/review protocol;
- no new hidden state, runtime motion-data dependency or unsupported export op;
- student action/state/runtime cost fits the declared profile;
- all dataset, teacher and child lineage is exact and license-compatible.

### Failure/rollback

Reject and delete the prior/student payload. Retain the passing specialist
candidate from `TRAIN-7`; do not lower safety or retention thresholds to admit
distillation.

### Commit boundaries

1. `feat(training): add admitted humanoid motion prior`
2. `feat(training): distill humanoid motor specialists`
3. `test(training): enforce multi-skill retention`

## TRAIN-9 — export, runtime parity and publication candidate

### Export

1. Export a fixed-shape standard-op graph with explicit state if any.
2. Reopen and validate exact model bytes and capability closure.
3. Bind BodySchema, compiled descriptor, observation/action/state,
   normalization, safety, PD, dataset, training config and fallback hashes.
4. Compare trainer → exported evaluator → Windows/Linux runtime over one
   golden observation/action/state corpus.
5. Run final held-out suite in production `headless`; Isaac results alone are
   insufficient.
6. Activate only through a new immutable candidate bundle and explicit project
   lock in a new session.

### Final quality/safety gate

- flat locomotion survival at least 99% over the declared held-out suite;
- velocity RMSE at most `0.20 m/s` and facing error at most `7°`;
- all `TRAIN-5..7` contact, transition, fall/get-up and retention thresholds
  pass on the portable actor;
- canonical applied action and complete policy state are exact across trainer
  decode, exported evaluator and supported runtime targets;
- no NaN/Inf, safety violation, hidden evaluator state or runtime motion-corpus
  dependency;
- procedural/animation/ragdoll fallback activates under every declared model,
  schema, state and evaluator fault;
- inference latency/memory stay within the declared motor profile;
- final human visual review passes a fixed seed/command/perturbation playlist.

### Failure/rollback

Export/parity failure rejects exported bytes and retains only the passing
external trainer candidate long enough to diagnose/re-export. Runtime quality
failure returns to the owning stage. No partial project-lock activation or
active-session hot swap is allowed.

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
| Learned actor | future `MOTOR-HUMANOID-MVP-P1`, `MODEL-STATISTICS-P1`, retention/recovery checks |
| Export/runtime | `MODEL-EXPORT-P1`, `POLICY-01`, future `MOTOR-REPLAY-P1`, `play`, `persistence-replay`, conditional `platform`/`performance` |

Future checks remain `NotRun(NoProductionConsumer)` until their consumer and
exact profile exist; this plan does not declare them passed.

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

The first executable increment after this plan is `TRAIN-0`, then `TRAIN-1`.
No new PPO run is authorized before `TRAIN-4`. No command locomotion run is
authorized before the reference tracker passes `TRAIN-5`.

Roadmap status changes only after material implementation/check results. This
planning document alone does not close R5, B-08, B-12, Stage 0, GPU
correspondence, Linux parity or any learned-policy ProductCheck.

## Definition of done

The rebuilt process is complete when:

1. old model/run artifacts are absent from active and external retained
   training storage;
2. the new BodySchema passes anatomy/compiler/contact/safety gates without ML;
3. an admitted reproducible motion corpus drives deterministic retargeting;
4. specialist tracking, command locomotion and recovery each pass independent
   held-out, safety and visual gates;
5. optional prior/distillation, if used, passes full retention;
6. the portable immutable actor reproduces canonical actions/state and final
   quality in production `headless`;
7. every failure has a bounded reproducer/decision and no heavy failed payload
   remains without an explicit temporary reason;
8. runtime retains a declared procedural/animation/ragdoll fallback and never
   depends on trainer, dataset or mutable weights.
