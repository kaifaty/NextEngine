# SPEC-05: Physics, animation и motor control

| Поле | Значение |
|---|---|
| ID | SPEC-05 |
| Статус | Accepted |
| Версия | 2.1 |
| Последняя проверка | 2026-07-30 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-013](adr/013-self-contained-physical-avatar-boundary.md), [ADR-033](adr/033-physx-grounded-capsule-parity-ffi-boundary.md), [ADR-036](adr/036-thoth-reference-performance-profile.md) |
| Заменяет | SPEC-05 2.0 |

## Source of truth и ownership

Для physical LOD `FullArticulation` и `SimplifiedActiveRagdoll` Physical Embodiment physics world владеет engine-owned body pose, velocities, contacts, constraints и canonical snapshot; replaceable PhysicsBackend является private compute adapter, а не source of truth. Для `CapsuleAnimation` validated controller владеет collision transform, animation graph — visual local pose. Physical `Abstract` не является `WorldResidencyTier`: World Services отдельно владеет durable population tier/logical region/activity, а RPG Framework — aggregate outcomes. Renderer всегда читает immutable presentation projection. Ни backend, World Services, animation, AI, gameplay script, importer, renderer, UI/camera, ни LLM не могут напрямую записать active physics authority.

Physical Embodiment владеет backend-neutral descriptors, physics stepping, motor observation/action, policy safety/resolution/supervision, topology transactions, animation bridge, LOD coordinator и physical support checks. RPG skill proficiency и Agent habits остаются за пределами этого ownership.

Boundary является self-contained по ADR-013: внешние research документы не задают requirements, phases, public types или support semantics. Frozen annex сохраняется только как ненормативная provenance; PhysX/Jolt/Bullet остаются отдельными `Proposed` technology hypotheses за одним contract.

## Public boundary

Нормативная public boundary специализируется [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md) и [SPEC-28](28-skeletal-animation-retargeting-and-ik.md). Она использует только engine-owned body/shape/material/joint/query/contact/snapshot, motor tensor/action/state и skeleton/clip/graph/retarget/IK descriptors. Backend world/device/session handles, native task objects, model-runtime tensors и animation-graph implementation nodes остаются private; public contracts не содержат ECS, OS, vendor, importer либо backend types.

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
6. PhysicsBackend выполняет CPU substep и выдаёт raw contacts/state; adapter сначала строит `QuantizedPhysicsProjectionV1` по `PhysicsQuantizationProfileV1` [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), затем нормализует scene-query/contact records и сортирует contacts по определённому ниже полному canonical total key. Backend callback, worker completion, native handle, raw float bit pattern и manifold insertion order не участвуют в public ordering.
7. Outcome resolver использует contact continuity для suppress repeated-hit/resting-contact exploits и предлагает `Outcome` WorldCommand для общего stage-9 validator [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md); контакт сам не меняет health/quest, а backend callback не коммитит gameplay.
8. Pose bridge публикует RenderPose, telemetry и replay hash.

LLM, `ai-host`, network и filesystem запрещены на шагах 3–7. Default profile использует 120 Hz physics и 60 Hz motor inference; PD/safety layer применяет последний принятый MotorAction на промежуточном physics substep. ONNX Runtime CPU provider — `Proposed` для small in-process policy. Reference performance oracle: p99 ≤0.5 ms/avatar/inference или ≤2 ms для batch из 16 vertical policies. Wall-clock duration, worker completion order и measured CPU load MUST NOT выбирать authoritative MotorAction. Только manifest-declared logical motor tick либо canonical injected fault signal может объявить action unavailable: тогда safety layer удерживает last-safe action не дольше двух motor intervals и на следующей declared boundary включает recovery controller. Wall watchdog MAY остановить зависший evaluator и защитно включить safe controller только после маркировки exact run как `Fail(MOTOR_WALL_DEADLINE_NONCONFORMING)`; такой trace не может пройти replay, PHYS или MOTOR check.

## Backend и runtime/training parity

PhysX reduced-coordinate articulations — primary `Proposed`; Jolt, затем Bullet — fallbacks. CPU path является runtime source. GPU/batched simulator MAY обучать policy, если golden mapping доказывает одинаковые body frames, joint axes/limits, actuator/torque units, contacts, observation normalization и action semantics.

ADR-033 добавляет более узкий реализованный experiment: PhysX 5.9.0 только
для upright grounded capsule против static Box. `PhysicsWorldBackend` и
`PhysicsWorldCheckpointV1` остаются engine-owned, reference backend —
обязательный default/oracle, а PhysX feature — optional и `Proposed`.
`ReferenceOnly`, `PreferPhysXThenReference` и `RequirePhysX` выбираются только
до activation. После создания мира backend fatal/mismatch abort-ит staging и
не разрешает mid-tick switch либо silent approximation.

Export pipeline MUST записывать model SHA-256, ONNX opset, input/output schema hashes, normalization constants, training simulator/build/config и golden observation/action corpus. Runtime отказывается загружать mismatch/non-finite/unsupported model и включает deterministic controller fallback.

## Deterministic numeric boundary

Используемые здесь reusable numeric schemas определены SPEC-21 как refinement этой owned physical boundary; canonical stage-9 ledger/encoding semantics определены superseding ADR-022.

Raw backend floats остаются private operational state выбранного PhysicsBackend и MAY использоваться только для declared per-field backend/runtime-training correspondence. До любого gameplay comparison, tie-break, scene-query result, contact continuity/classification, PhysicalOutcome, DomainEvent payload либо cross-target hash adapter MUST преобразовать decision-relevant values через `PhysicsQuantizationProfileV1`/`PhysicsQuantizationRuleV1` SPEC-21. Rule фиксирует source IEEE format, canonical unit, destination `FixedPointDescriptorV1`, exact scale numerator/denominator, offset и bounds. Adapter декодирует finite IEEE bits как exact rational, нормализует negative zero и округляет `NearestTiesToEven`; NaN, infinity и undeclared rule fail closed до outcome proposal, а out-of-bounds/intermediate overflow атомарно abort-ит transaction как `NUMERIC_OVERFLOW`. Tolerance не выбирает branch и не превращает exact mismatch в `Pass`.

Adapter сначала упорядочивает участников contact по `(PersistentId canonical bytes, body_slot)` и при перестановке канонически меняет направление participant-relative vector fields. `contact_feature_id` является engine-owned stable shape-feature ID из validated descriptor/asset mapping; raw backend feature index не может попасть в key без exact mapping. Полный ключ contact равен `(physics_tick, substep, participant_low, participant_high, contact_feature_id, quantized point, quantized normal, quantized relative velocity, quantized impulse bounds, quantized effective mass, canonical material tags)`. Все numeric elements ключа являются raw fixed-point integers после SPEC-21 quantization. Exact duplicate normalized records дедуплицируются до continuity classification. `contact_id` и begin/persist/end выводятся только после этой сортировки из canonical participant/feature continuity, а не из backend handle или callback ordinal.

На одинаковых target triple/build/config два successful run MUST иметь exact full authoritative `state_root`, exact contact sequence и exact outcome sequence. Между Windows x86_64 и Linux x86_64 `physics_projection_root` над `CanonicalBinaryV1` bytes `QuantizedPhysicsProjectionV1`, canonical contact/query order и gameplay outcomes MUST совпадать byte-for-byte; raw backend samples сравниваются отдельно только по registered per-field correspondence tolerance и не входят в cross-target exact projection. Для `MotorAction` versioned action schema отвергает non-finite output, декодирует каждый finite raw IEEE value как exact rational, применяет safety clamp по exact bounds и конвертирует его round-to-nearest-ties-to-even через named `FixedPointDescriptorV1` из `AuthoritativeNumericProfileV1` до actuation. MOTOR-P1 tolerance сравнивает raw model evaluators, но не разрешает различие applied canonical action.

## Policy lifecycle specialization

Game-time learning не изменяет model bytes. PolicyResolver и PolicySupervisor SPEC-14 выбирают immutable foundation/skill route по body, equipment, proficiency, topology mask и intent. Switch выполняется только на motor-tick safe point, никогда не копирует physics pose и не совмещается с topology/LOD transaction. До atomic commit previous route остаётся единственным actuator source.

## Animation bridge

Animation assets задают reference motions, intent features и presentation-only secondary layers через exact SPEC-28 schemas. В physical tiers animation MAY предлагать motor targets, но не final pose. Root motion становится revision-bound validated intent через production command/motor boundary, а не teleport. Physical IK решается внутри authoritative physics/motor ordering; presentation IK читает immutable pose и не меняет contacts/gameplay hashes. Graph, retarget, IK и LOD имеют fixed canonical order. В `CapsuleAnimation` collider motion следует validated locomotion controller, а visual pose — animation graph; расхождение ограничено project-locked leash и диагностируется.

## Physics LOD

| LOD | Simulation | Разрешённое применение |
|---|---|---|
| `FullArticulation` | полный articulated body + motor + contacts | player, near/important NPC, product-check scenarios |
| `SimplifiedActiveRagdoll` | reduced body/actuators, обязательные root/support contacts | nearby background NPC under interaction |
| `CapsuleAnimation` | capsule collision/navigation + animation pose | distant visible NPC без physical interaction |
| `Abstract` | logical region/time/task state | unloaded/non-visible NPC |

Physical LOD request выводится только из canonical simulation facts: quantized simulation distance, gameplay importance, interaction/contact state, committed WorldResidencyTier view, simulation-owned visibility fact, PersistentId и versioned integer budget tokens из manifest. Renderer camera/frustum/occlusion, presentation visibility, measured CPU time, wall clock, worker load и completion order MUST NOT влиять на tier. World Services может предложить residency/physical capability change, но LOD coordinator отдельно валидирует physical transition; unsupported abstract precise outcome детерминированно требует upgrade/defer и не телепортирует pose. Safety constraints имеют приоритет. Downgrade запрещён при external contact impulse, fall/recovery, grab, topology transaction, quest-critical physical interaction или unstable support.

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
- Missing contact telemetry required текущим gameplay scenario → backend product-check failure, не silent approximation.
- LOD transition failure → остаётся source tier; после 3 failures entity pin-ится в safe tier и emits diagnostic.
- Exhausted manifest-declared integer LOD budget → deterministic downgrade только eligible avatars по canonical order; player/critical physical interaction не теряет required tier.
- Measured CPU overrun → PHYS-P4/PERF-01 failure и diagnostic; он не меняет authoritative LOD, action или outcome текущего run.
- Inference wall-watchdog trip → защитный safe-controller path и `Fail(MOTOR_WALL_DEADLINE_NONCONFORMING)` для exact run; deterministic fallback проверяется только canonical logical fault signal.

## Product checks

Checks используют fixed scenarios/seeds и [RunManifest из SPEC-09](09-tooling-sdk-and-observability.md). Captures и timing reports являются диагностикой продукта и не влияют на authoritative simulation.

| ID | Сценарий | Ожидаемый результат | Fallback |
|---|---|---|---|
| PHYS-P1 | descriptor parity corpus | 100% bodies/axes/limits; mass/inertia ≤0.1%; torque conversion ≤1% | попробовать следующий backend через тот же contract |
| PHYS-P2 | contact/query/topology suite | 100% required contacts в canonical total order; query/contact output exact under callback, manifold, worker and registration permutations; все key-field vectors проходят `NUMERIC-P1`; 1 000 topology cycles без invalid handle/leak | отвергнуть backend и сохранить последнюю canonical world state |
| PHYS-P3 | same-target replay и cross-target quantized projection | full authoritative `state_root`, applied canonical actions, contact/outcome sequences exact over 100 repeats; Windows/Linux `physics_projection_root`, canonical contact/query order и gameplay outcomes byte-identical | исправить deterministic boundary или выбрать другой backend |
| PHYS-P4 | 16 full avatars на полном `ref-win-thoth-v1` без CPU affinity restriction | physics 120 Hz, motor 60 Hz; physics+motor p95 ≤4 ms, p99 ≤6 ms; no missed critical steps; inference p99 ≤0.5 ms/avatar или ≤2 ms для batch 16 | offline tuning следующего manifest integer LOD budget; safe-tier pin |
| MOTOR-P1 | 10 000 golden observations | ONNX vs training max abs raw action error ≤1e-5; applied safety-clamped/quantized `MotorAction` exact; 0 schema mismatch accepted | reference CPU evaluator или heuristic controller |
| PHYS-P5 | runtime/training golden trajectories | normalized RMSE ≤0.05; contact F1 ≥0.98; outcome pass-rate delta ≤2 percentage points | retrain, mapping fix или backend fallback |
| PHYS-P6 | push, slope, stair, trip, carry, fall/recovery | per-scenario thresholds; aggregate ≥95%; 0 safety violation | recovery controller; unsupported learned route остаётся отключён |
| PHYS-P7 | every adjacent LOD transition, 1 000 cycles | 0 forbidden transition; jump/velocity limits соблюдены; no lost durable outcome | pin higher safe tier |
| PHYS-P8 | optional local success/failure/comparison capture | fixed camera/profile; overlays command/phase/COM/support/contacts/target; when captured, media reproduces the exact run and contains no authority | сохранить diagnostics; gameplay check остаётся authoritative |

Physical archetype, progressive skill, policy switching и creature authoring проходят дополнительные product checks SPEC-14.
