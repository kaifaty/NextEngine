# SPEC-05: Physics, animation и motor control

| Поле | Значение |
|---|---|
| ID | SPEC-05 |
| Статус | Accepted |
| Версия | 1.2.1 |
| Владелец | Physical Embodiment Team |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [ADR-004](adr/004-physics-avatar-backend-boundary.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [RESEARCH-001](research/physical-avatar-research-spec.md) |
| Связанные документы | [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [research provenance](../../MIGRATION_PROVENANCE.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

Для LOD `FullArticulation` и `SimplifiedActiveRagdoll` PhysicsBackend владеет body pose, velocities, contacts и articulation topology. Для `CapsuleAnimation` capsule controller владеет collision transform, animation graph — visual local pose. Для `Abstract` RPG/world simulation владеет logical location. Renderer всегда читает immutable RenderPose. Ни animation, AI, gameplay script, importer, renderer, ни LLM не могут напрямую записать active physics transforms.

Physical Embodiment Team владеет backend-neutral descriptors, physics stepping, motor observation/action, policy safety/resolution/supervision, topology transactions, animation bridge, LOD coordinator и physical certification. RPG skill proficiency и Agent habits остаются за пределами этого ownership.

## Public boundary

`PhysicsBackend` MUST поддерживать: world create/destroy; immutable shape/material/body/articulation descriptors; instance handles opaque backend; force/impulse/kinematic-request API с declared semantics; scene queries; normalized ContactEvent stream; state snapshot/restore for gates; topology mutation transaction; statistics/errors.

Основные engine-owned data contracts:

| Contract | Обязательные поля/semantics |
|---|---|
| `PhysicalBodyDescriptor` | PersistentId, parent relation, mass/inertia, collision geometry AssetId, material tags, canonical frame/axis, limits, actuator bounds |
| `PhysicalAvatarIntent` | intent ID, issuer, start/expiry tick, desired locomotion velocity/facing/posture/manipulation target, priority, safety constraints |
| `MotorObservation` | schema/model version, normalized root/joint state, target features, contacts/support/terrain features, previous action, masks |
| `MotorAction` | schema/model version, joint targets/torques/controller gains, confidence/validity flags |
| `ContactEvent` | continuity `contact_id`, participants PersistentId/body slot, point/normal, relative velocity, impulse bounds/effective mass, material tags, begin/persist/end, physics tick |
| `PhysicalOutcome` | normalized fall/recover/traverse/hit/grab completion candidate, доказанный physics state |
| `RenderPose` | source physics ticks, hierarchy-local transforms, interpolation policy, topology revision |

Physical archetype, policy manifest, skill/proficiency-aware resolver, ActivePolicyRoute и transition contracts специализируются [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md); они не меняют PhysicsBackend ABI или MotorObservation/MotorAction ownership.

Units/right-handed axes соответствуют SPEC-03. Vendor enumerations и handles не входят в serialized schemas.

## Physics step и data flow

1. Gameplay/tactical layer публикует bounded PhysicalAvatarIntent.
2. LOD coordinator выбирает tier только на permitted transition point.
3. PolicySupervisor предоставляет committed compatible route; Observation builder читает current physics state и immutable allowed context.
4. Motor controller (heuristic, foundation/skill policy либо recovery fallback) создаёт MotorAction без blocking I/O.
5. Safety layer проверяет finite values, schema/model hash, action age, joint/torque/rate/energy limits и contact guards; invalid action заменяется safe hold/recovery action.
6. PhysicsBackend выполняет CPU substep и выдаёт normalized contacts/state.
7. Outcome resolver использует contact continuity для suppress repeated-hit/resting-contact exploits и предлагает internal WorldCommand для gameplay effects; контакт сам не меняет health/quest.
8. Pose bridge публикует RenderPose, telemetry и replay hash.

LLM, `ai-host`, network и filesystem запрещены на шагах 3–7. Default profile использует 120 Hz physics и 60 Hz motor inference; PD/safety layer применяет последний принятый MotorAction на промежуточном physics substep. ONNX Runtime CPU provider — `Proposed` для small in-process policy. Inference deadline на reference CPU: p99 ≤0.5 ms/avatar/inference или ≤2 ms для batch из 16 vertical policies; miss использует last-safe action не дольше 2 motor intervals, затем recovery controller.

## Backend и runtime/training parity

PhysX reduced-coordinate articulations — primary `Proposed`; Jolt, затем Bullet — fallbacks. CPU path является runtime source. GPU/batched simulator MAY обучать policy, если golden mapping доказывает одинаковые body frames, joint axes/limits, actuator/torque units, contacts, observation normalization и action semantics.

Export pipeline MUST записывать model SHA-256, ONNX opset, input/output schema hashes, normalization constants, training simulator/build/config и golden observation/action corpus. Runtime отказывается загружать mismatch/non-finite/unsupported model и включает deterministic controller fallback.

## Policy lifecycle specialization

Game-time learning не изменяет model bytes. PolicyResolver и PolicySupervisor SPEC-14 выбирают immutable foundation/skill route по body, equipment, proficiency, topology mask и intent. Switch выполняется только на motor-tick safe point, никогда не копирует physics pose и не совмещается с topology/LOD transaction. До atomic commit previous route остаётся единственным actuator source.

## Animation bridge

Animation assets задают reference motions, intent features и presentation-only secondary layers. В physical tiers animation MAY задавать motor targets, но не final pose. Root motion преобразуется в intent, а не teleports. Foot IK для presentation не может менять physics contacts. В CapsuleAnimation collider motion следует validated locomotion controller, а visual pose — animation graph; расхождение ограничено configurable leash и диагностируется.

## Physics LOD

| LOD | Simulation | Разрешённое применение |
|---|---|---|
| `FullArticulation` | полный articulated body + motor + contacts | player, near/important NPC, gate scenarios |
| `SimplifiedActiveRagdoll` | reduced body/actuators, обязательные root/support contacts | nearby background NPC under interaction |
| `CapsuleAnimation` | capsule collision/navigation + animation pose | distant visible NPC без physical interaction |
| `Abstract` | logical region/time/task state | unloaded/non-visible NPC |

LOD request зависит от distance, importance, interaction, visibility и CPU budget, но safety constraints имеют приоритет. Downgrade запрещён при external contact impulse, fall/recovery, grab, topology transaction, quest-critical physical interaction или unstable support.

Transition MUST быть explicit state machine `Prepare → Validate → Commit → Stabilize`:

- lower→higher создаёт bodies из canonical descriptor, проецирует pose/velocity, проверяет penetration/support и прогревает 2–8 hidden substeps;
- higher→lower фиксирует root/logical state, сохраняет required physical outcome, проверяет forbidden conditions и только затем удаляет bodies;
- failed validation оставляет исходный LOD без partial mutation;
- `Stabilize` ограничивает artificial energy; position jump ≤5 cm, facing ≤3°, linear velocity discontinuity ≤0.3 m/s для accepted transition.

## Topology changes

Dismemberment/breakable constraints выполняются как physics transaction: validate allowed joint and gameplay command → snapshot → backend mutation → remap body slots/topology revision → contact/render/observation schema update → commit. Policy, не поддерживающая topology mask, немедленно заменяется compatible recovery controller. Удалённые части получают отдельный PersistentId только если становятся durable gameplay objects.

## Failure semantics

- Invalid/stale model или parity mismatch → policy rejected до world activation; heuristic/recovery motor.
- Invalid proficiency/equipment/topology route или unsafe switch → candidate rejected/deferred; previous/novice/recovery route остаётся authoritative согласно SPEC-14.
- NaN/out-of-range action → action rejected, safe controller, structured event.
- Backend fatal/invariant loss → остановка simulation instance и crash capsule; продолжать с недостоверной pose запрещено.
- Missing contact telemetry required текущим gameplay scenario → backend gate failure, не silent approximation.
- LOD transition failure → остаётся source tier; после 3 failures entity pin-ится в safe tier и emits diagnostic.
- CPU budget pressure → downgrade только eligible avatars; player/critical physical interaction не теряет required tier.

## Verification gates

Все gates являются TestScenarioManifest specializations SPEC-15, используют fixed cameras/scenarios/seeds и [RunManifest из SPEC-09](09-tooling-sdk-and-observability.md). Любое material physics/motor/animation изменение автоматически имеет HumanReviewRequired; automatic contact/safety/replay gates проходят до human review.

| Gate | Сценарий | Threshold | Evidence | Fallback |
|---|---|---|---|---|
| PHYS-P1 | descriptor parity corpus | 100% bodies/axes/limits; mass/inertia ≤0.1%; torque conversion ≤1% | mapping report | next backend |
| PHYS-P2 | contact/topology suite | ≥99.9% required contacts ordered; 1 000 topology cycles, 0 invalid handle/leak | traces/sanitizers | next backend |
| PHYS-P3 | replay stability same target | final root ≤2 cm/1°, declared contact sequence exact, outcome exact over 100 repeats | replay report | tighten config/next backend |
| PHYS-P4 | 16 full avatars reference 8-core CPU | physics+motor p95 ≤4 ms, p99 ≤6 ms; no missed critical steps | timings | LOD budget tuning; backend fallback if one avatar gate fails |
| MOTOR-P1 | 10 000 golden observations | ONNX vs training max abs action error ≤1e-5; 0 schema mismatch accepted | parity report/model hashes | reference CPU evaluator/heuristic |
| PHYS-P5 | runtime/training golden trajectories | normalized RMSE ≤0.05; contact F1 ≥0.98; outcome pass rate delta ≤2 percentage points | correspondence report | retrain/map fix/backend fallback |
| PHYS-P6 | push, slope, stair, trip, carry, fall/recovery suite | success thresholds per scenario manifest; aggregate ≥95%; 0 safety violation | metrics + failures | recovery controller/retrain; release block |
| PHYS-P7 | every adjacent LOD transition, 1 000 cycles | 0 forbidden transition; jump/velocity thresholds выше; no lost durable outcome | transition report/video | pin higher safe tier |
| PHYS-P8 | visual artifact completeness | `baseline.gif`, `selected-policy.gif`, `failures.gif`, `comparison.gif`; MP4 where GIF obscures; overlays command/phase/COM/support polygon/contacts/target/gate metrics; SHA-256 and metadata 100% | media directory + RunManifest | gate fails; artifacts cannot be omitted |

Representative fixed-suite и worst/failed scenarios MUST публиковаться вместе; только showcase episodes не принимаются. Existing `baseline.gif`, `selected-policy.gif`, `failures.gif`, `comparison.gif` map to SPEC-15 base/candidate/failure/comparison roles и входят в EvidenceBundleManifest/HumanReviewDecision exact hashes.

Physical archetype, progressive skill, policy switching, certification и creature authoring проходят дополнительные EMB/POLICY/SKILL/CREATURE/TRAIN/AUTHOR gates SPEC-14.
