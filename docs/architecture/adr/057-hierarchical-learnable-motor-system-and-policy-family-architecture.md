# ADR-057: Hierarchical learnable Motor System and policy-family architecture

| Поле | Значение |
|---|---|
| ID | ADR-057 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-10 |
| Последняя проверка | 2026-08-10 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-28](../28-skeletal-animation-retargeting-and-ik.md), [ADR-013](013-self-contained-physical-avatar-boundary.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | полностью [ADR-009](009-pretrained-foundation-policies-and-progressive-motor-skills.md) и [ADR-055](055-mamba2-physical-motion-foundation-profile.md) |
| Заменён | не заменён |

## Контекст

Один low-level model не может одновременно быть владельцем gameplay goal,
contact planning, morphology interpretation, damage identification и joint
actuation. Такая схема смешивает разные cadence, authority и failure domains,
плохо управляется designer-facing commands и не имеет shipping-capable
fallback.

Предыдущий ADR-009 правильно закрепил immutable weights, proficiency-aware
route selection и procedural fallback, но не определил общий body contract,
planner-facing motion/contact boundary, explicit dynamics adaptation или
границу между близкими и физически разными morphology families. ADR-055 затем
выбрал Mamba-2 как целевую universal foundation hypothesis. Research evidence
не подтверждает Mamba как default для 30–120 Hz proprioceptive low-level
control и не подтверждает одну production policy для humanoid, serpentine,
aquatic и aerial regimes.

Нужна универсальная Motor **System**, а не универсальная Motor Model. Решение
должно сохранить ADR-013/ADR-027 physics authority, exact runtime contracts и
procedural/animation fallback, но разрешить независимо развивать skills,
motion generation, adaptation и policy families.

## Решение

### Authority и иерархия

Canonical target flow:

```text
Strategic Agent
  → Tactical Controller
  → Skill Orchestrator ── Contact/Affordance Planner
  → optional Motion Generator / authored motion matching
  → low-level Motor Policy
  → deterministic actuator and safety layer
  → Physics
```

- Physics остаётся единственным source of truth для pose, velocity, contacts,
  impulses, constraints и active topology. Model, generator, animation и
  planner публикуют только bounded proposals/references.
- Tactical Controller выбирает maneuver/target; Skill Orchestrator владеет
  skill phase, style, interruption и fallback intent; Contact Planner задаёт
  desired effectors/surfaces/windows; optional Motion Generator строит bounded
  pose/keypoint/contact/object horizon. Low-level policy выдаёт actuator targets
  текущего motor tick и не выбирает narrative/gameplay goal.
- Обычная velocity locomotion MAY обходиться без Motion Generator. Parkour,
  climbing, multi-contact manipulation, weapon trajectories и cooperative load
  SHOULD использовать explicit motion/contact horizon.
- Profile фиксирует integer cadence. Target ranges: skill/contact `5..20 Hz`,
  generator `10..30 Hz`, adaptation `5..20 Hz`, low-level policy `30..120 Hz`,
  actuator/physics `120..480 Hz`. Wall time и measured load не выбирают
  cadence, route, action или fallback.

### `BodySchema` и instance projection

`BodySchema` является единым immutable versioned physical-body contract, а не
model layer и не projection после actuator controller. Он содержит stable
schema-scoped body, joint, actuator, effector и attachment identities,
articulation, colliders, limits, actuator capabilities, symmetry и semantic
roles. Из одной exact revision детерминированно строятся:

- SPEC-26 physics bodies/joints/constraints;
- SPEC-27 observation/action layouts and masks;
- cached static morphology input;
- actuator/safety limits;
- serialization/replay compatibility and topology remap.

`BodySchema` не хранит current pose/contact/fatigue/damage. Effective body
строится как immutable revision-bound `BodyInstanceProjection` над exact
schema, morphology, equipment/loadout, stats, damage, fatigue и attachment/
topology inputs. Каждый source field сохраняет своего RPG/Mechanics/Physics
owner; projection не становится вторым mutable source. Physical Embodiment
владеет только compiled effective mass/inertia/ROM/actuator/sensor projection.

Accepted здесь semantics и derivation boundary. ADR-058/SPEC-35 subsequently
accept the exact `BodySchemaV1` and `BodyInstanceProjectionV1` subset consumed
by the fixed 23-DoF Stage 0 humanoid. Advanced overlays/families remain
Proposed until their own production consumers under ADR-046.

### Policy families, skills и composition

Общие `BodySchema`, command protocol, skill metadata, observation/action
principles и safety layer не означают общие weights для разных physical
regimes. Target families:

1. humanoid/biped;
2. general legged;
3. serpentine;
4. aquatic;
5. aerial;
6. bounded modular;
7. musculoskeletal research.

Одна learned policy MUST declare one bounded family/topology/training envelope.
Humanoid+snake+fish+bird, ground+swim+flight и zero-shot arbitrary topology не
являются supported universal route. Cross-regime creature использует separate
ground/flight/swim policies и explicit transition experts.

Skill composition происходит в trajectory, keypoint, contact, phase or latent
reference space, не смешиванием raw torques. Engine `PolicyResolver` остаётся
deterministic owner доступности skill/route. Один policy artifact MAY иметь
internal MoE/expert routing, hysteresis или soft blend, если это одна bounded
action proposal, все future-affecting router state explicit в
`PolicyStateRecordV1`, а routing не выдаёт gameplay capability. Arbitrary
cross-bundle action averaging и hidden joint ownership запрещены.

### Low-level action и первый learned profile

Default architecture для articulated learned route — residual joint position
targets through deterministic fixed PD/SPD, optional bounded velocity target
и только separately gated small residual torque. Direct torque, adaptive gains
и muscle activation не являются default route и требуют отдельного profile,
safety envelope и ProductCheck.

Первый Proposed learned profile, не v1 requirement:

- one fixed humanoid skeleton, approximately `20..30` controlled DoF;
- feed-forward MLP, initially approximately `1..3M` parameters;
- `60 Hz` policy and `240 Hz` physics/actuator profile;
- standing, walk/run/backward/strafe/turn/crouch/crawl, slopes/stairs, push
  recovery, ragdoll and get-up;
- authored/motion-matching reference when useful;
- deterministic PD/safety and shipping-capable animation/procedural fallback.

MLP является first low-level comparator. TCN or GRU является first adaptation
comparator. Mamba MAY участвовать только в equal-budget benchmark for long
history, terrain/perception tokens, motion generation or temporal planning;
она не становится default по факту export или model availability.

### Explicit adaptation, progression и immutable learning

- Known engine values — equipment mass/inertia, CoM shift, strength, fatigue,
  damage, ROM, friction, actuator latency/limits and sensor confidence — входят
  в exact projection/observation explicitly. Model не должен угадывать их.
- Unknown/effective dynamics MAY кодироваться bounded TCN/GRU/other history
  module из recent observation→action→response. History layout, cadence,
  latent/state bounds, reset/remap and persistence are explicit; hidden
  evaluator cache запрещён.
- Gameplay выполняет no-gradient adaptation только через explicit state and
  conditioning. PPO/backpropagation/runtime optimizer запрещены.
- RPG proficiency, style/school и personality принадлежат owning domain and
  condition skill/generator/policy. Novice behavior является намеренно
  evaluated repertoire/timing/safety behavior, не noise или OOD failure.
- Offline per-hero adapter всегда создаёт новый immutable child bundle, passes
  retention/quality/safety checks и активируется только новым exact project
  lock and session. Active weights, save и parent bundle не изменяются.

### Equipment, fatigue, damage и recovery

Equipment, stats, fatigue and damage меняют реальные effective mass/inertia,
CoM, ROM, torque-speed-power, latency/noise and contact/grip constraints. Hard
limits применяются deterministic safety layer, не reward.

Severe injury/topology mutation commits through SPEC-26 transaction. Stable
BodySchema identities map surviving nodes/joints; incompatible adaptation and
generator state resets; unsupported route switches to declared recovery,
crawl, one-limb, ragdoll or animation/procedural fallback. Emergency cancel
follows bounded `stabilize → brace → safe fall → ragdoll → get-up` route.

### Deterministic inference and replay

- A `Supported` evaluator MUST yield identical canonical applied
  `MotorActionV1`, complete `PolicyStateRecordV1` and route/fallback decision on
  Windows x86_64 and Linux x86_64 for the locked profile. Raw evaluator tensors
  MAY have separately declared correspondence tolerance only before canonical
  decode; different applied action/state is failure.
- Replay records canonical observations or source roots, accepted actions,
  complete policy/adaptation/generator/router state, contacts and periodic
  physics snapshots. Authoritative replay consumes and verifies recorded
  canonical actions/state chain; it never depends only on re-running model
  inference.
- Evaluator re-execution is a separate parity/correspondence check and MUST
  reproduce the recorded canonical action/state for a Supported artifact.

### Training and technology boundary

Production `headless` remains the canonical environment. External GPU
simulators are accelerated mirrors and require body/joint/axis/actuator/contact
correspondence plus final in-engine evaluation.

The first Proposed replaceable toolchain profile is Isaac Lab/PhysX +
ProtoMotions + RSL-RL + PyTorch, portable fixed-shape standard-op ONNX export
and a private ONNX Runtime adapter. MuJoCo/MJX or another trainer/evaluator MAY
replace it behind the same engine-owned schemas. Reference physics, canonical
headless, reference CPU evaluation and procedural controller remain fallbacks.
No framework, provider, device or vendor type enters public gameplay contracts.

Target development order is humanoid MVP → humanoid variations → equipment →
injuries → weapons → parkour → general-legged → bounded cross-family research.
These advanced profiles remain Proposed and create no current registry entry,
format, implementation obligation or stage-completion claim without their own
production consumer and checks.

## Product impact

R5 can ship capsule/procedural locomotion, skeletal presentation and basic IK
without learned weights, vendor physics or training hardware. Learned quality
can be added incrementally without changing physics authority or granting
first-party content a private route. Designers get bounded skill/contact/motion
interfaces instead of raw joint control, while save/replay retains exact
future-affecting policy state.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `BODY-SCHEMA-P1` (future) | schema→physics/tensor/safety derivation, overlays and topology remap | exact projection roots; no duplicate owner; invalid/remap fault publishes nothing | retain prior topology/projection and procedural tier |
| `MOTOR-HUMANOID-MVP-P1` (future) | one million steps plus flat/terrain/push/fall/get-up profile | 0 NaN/Inf, flat survival ≥99%, declared terrain success ≥95%, velocity RMSE ≤0.20 m/s, heading error ≤7°, bounded slip and command resume after get-up | procedural/animation/ragdoll route |
| `MOTOR-ADAPTATION-P1` (future) | abrupt equipment/damage/friction/latency changes | explicit values apply immediately; bounded history state improves declared metric without weight change | explicit-only controller and recovery route |
| `MOTOR-RETENTION-P1` (future) | add specialist, transitions, interruption and old-skill corpus | declared new skill passes without old-skill/transition regression | retain parent bundle or separate expert |
| `MOTOR-REPLAY-P1` (future) | save/load/replay and worker/target permutations | canonical action/full-state/snapshot chain exact; evaluator parity is independently exact after canonical decode | reject artifact/profile and use procedural route |
| applicable `MODEL-*` (future) | mirror, export, multi-seed quality, data governance and retention | immutable child bundle only after every applicable gate | canonical headless and prior/procedural bundle |

All rows are `NOT_RUN(NO_PRODUCTION_CONSUMER)` in this documentation-only
decision. Existing `fast`, `play`, `persistence-replay`, `content-package` and
conditional `performance` remain selected by affected behavior under ADR-030.

## Рассмотренные варианты

- One monolithic cross-morphology network — Rejected: different physical
  regimes, cadence and actuator semantics require separate families/fallbacks.
- Mamba-2 as mandatory low-level foundation — Rejected: insufficient
  low-level control evidence and unnecessary state/export complexity compared
  with MLP/TCN/GRU baselines.
- Raw torque as default action — Rejected: larger training, transfer and safety
  burden; fixed PD residual targets are the first profile.
- Runtime full-policy learning — Rejected: mutable hidden authority, compute,
  replay and catastrophic-forgetting risk.
- Animation-free learned-only shipping path — Rejected: OOD, hardware and
  model failure would make mandatory gameplay unavailable.
- GPU simulator as canonical physics — Rejected: production ownership and
  replay live in engine `headless`.

## Consequences and failure semantics

- SPEC-05/14/26/27/28 and Proposed SPEC-34 divide ownership described here and
  MUST NOT create a second body, pose, route, history or artifact authority.
- Missing/incompatible/unsafe learned component selects only a declared
  procedural/animation/recovery path; silent cross-family approximation is
  forbidden.
- Unknown topology, invalid stable-ID mapping, hidden state, non-finite output,
  broken action/state chain or target divergence rejects the complete learned
  path before physical mutation.
- Full target breadth is an R&D program, not an R5/v1 completion claim.

## Research provenance

The architecture is informed by
[DeepMimic](https://xbpeng.github.io/projects/DeepMimic/index.html),
[AMP](https://xbpeng.github.io/projects/AMP/index.html),
[PHC](https://github.com/ZhengyiLuo/PHC),
[DReCon](https://www.ubisoft.com/en-us/studio/laforge/news/VjEIwquaIyEZZSw5RZI0V/drecon-datadriven-responsive-control-of-physicsbased-characters),
[AnyBody](https://arxiv.org/abs/2606.29209),
[HANDOFF](https://arxiv.org/abs/2606.06493),
[PARC](https://arxiv.org/abs/2505.04002),
[MaskedMimic](https://xbpeng.github.io/projects/MaskedMimic/index.html),
[MetaMorph](https://arxiv.org/abs/2203.11931),
[ModuMorph](https://proceedings.mlr.press/v202/xiong23a.html),
[URMAv2](https://arxiv.org/abs/2509.02815),
[ReActor](https://arxiv.org/abs/2605.06593),
[RMA](https://arxiv.org/abs/2107.04034),
[Tired Actor](https://arxiv.org/abs/2608.03528),
[SafeFall](https://arxiv.org/abs/2511.18509),
[StableMimic](https://arxiv.org/abs/2608.02385),
[UMC](https://arxiv.org/abs/2502.03035),
[SkillMimic](https://arxiv.org/abs/2408.15270) and
[InterMimic](https://arxiv.org/abs/2502.20390),
[contact-rich interacting characters](https://arxiv.org/abs/2604.07984),
[Perceptive Humanoid Parkour](https://arxiv.org/abs/2602.15827) and the original
[Mamba](https://arxiv.org/abs/2312.00752) work. These sources motivate
boundaries and evaluation; they are not public APIs or proof that an advanced
profile is production-ready.

## Supersession

ADR-009 and ADR-055 are fully Superseded. Their immutable weights,
proficiency-aware deterministic resolver, explicit state, safety and fallback
semantics are retained and generalized here. ADR-013 and ADR-027 remain
Accepted and are not superseded.
