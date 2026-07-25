# ADR-009: Pretrained foundation policies и progressive motor skills

| Поле | Значение |
|---|---|
| ID | ADR-009 |
| Статус | Accepted |
| Версия | 1.0.2 |
| Дата решения | 2026-07-22 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-013](013-self-contained-physical-avatar-boundary.md) |
| Заменяет | отсутствует |
| Заменён | частично [ADR-030](030-product-first-development-and-lightweight-validation.md) |

## Частичное supersession ADR-030

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежние process clauses. Immutable runtime weights,
proficiency-aware deterministic routing, safety clamps, compatibility metadata
и procedural fallback сохраняются. Relevant runtime, parity and safety checks
выбираются по риску изменения.

## Контекст

SPEC-05 определяет body descriptors, motor observation/action, ONNX parity и physics LOD, но не отвечает, как добавить новое существо, переиспользовать locomotion между навыками или безопасно изменить controller после изучения игрового skill. Один model на каждую комбинацию creature × weapon × proficiency создаёт combinatorial growth; произвольное использование expert model для novice вызывает out-of-distribution behavior, которое нельзя считать дизайном сложности.

Исследования composable primitives, skill-conditioned representations и teacher/student distillation показывают practicable directions для переиспользования motor skills: [MCP](https://arxiv.org/abs/1905.09808), [ASE](https://arxiv.org/abs/2205.01906), [PULSE](https://arxiv.org/abs/2310.04582), [MaskedMimic](https://arxiv.org/abs/2409.14393) и [InterMimic](https://arxiv.org/abs/2502.20390). Эти работы являются technical references, а не выбранным runtime API или обязательным training algorithm.

## Решение

- Каждая morphology family использует pretrained foundation controller для balance, locomotion и recovery; skill добавляется как conditioned mode, bounded residual adapter либо один exclusive full-body expert.
- Game runtime не обучает neural weights. `LearnSkill` изменяет authoritative RPG proficiency и разрешённые routes; weights остаются immutable content assets.
- Untrained action использует только специально evaluated novice/generic или procedural route. Expert вне declared proficiency/training envelope не запускается.
- Новый creature package MAY использовать procedural route без learned controller. Learned route включается только после relevant compatibility, parity, safety and performance checks.
- `PolicyResolver` является deterministic engine component. ML router не определяет доступность skill и не является source of truth.
- Переключение policy допускается только внутри неизменной body topology через `PolicySupervisor`; v1 не поддерживает polymorph, mounts или body replacement.
- Arbitrary output blending запрещён. V1 допускает один full-body expert и residual adapter только для declared joint mask с engine safety clamps.
- Habits, tactical preferences и planner decisions принадлежат Agent Runtime; motor policy исполняет `PhysicalAvatarIntent`, но не выбирает narrative/gameplay goal.

## Рассмотренные варианты

- Один monolithic model на creature со всеми skills — `Rejected`: добавление skill требует полного retrain и связывает независимые packages.
- Полный model на каждый skill — `Rejected`: дублирует locomotion/recovery и усложняет safe handoff.
- Live runtime learning — `Rejected`: weights становятся mutable gameplay state, раздувают saves, нарушают replay и усложняют safety validation.
- Запуск trained expert у untrained персонажа с искусственными stat penalties — `Rejected`: физическое поведение остаётся OOD и не доказывает novice proficiency.
- Автоматическое смешивание нескольких policy outputs — `Rejected`: ownership joint actions и переходная устойчивость становятся неявными.
- Обязательная trained policy до загрузки любого prototype — `Rejected`: делает community authoring недоступным и смешивает content iteration с production readiness.

## Последствия

Engine поддерживает registry физических archetypes, policy manifests,
proficiency-aware resolver и supervisor transitions. Content package может
перейти с procedural route на learned route без смены identity.
Foundation/expert composition снижает количество models, но требует
compatibility keys, held-out scenarios и explicit performance envelopes.

RPG progression становится наблюдаема в физическом исполнении, однако damage/stamina/quest effects остаются в Mechanics Runtime. Сохранения фиксируют proficiency, active route и exact model artifacts, но не mutable learned weights.

## Product checks

Isaac Lab является `Proposed` training backend, а не частью runtime contract.

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| Runtime policy | Run novice/expert routes and transition boundaries on representative bodies | Resolver follows proficiency, outputs are finite/bounded and fallback remains deterministic | Use the procedural route |
| Training backend | Train/export a small representative policy and compare runtime inference | Artifact carries exact compatibility hashes and parity is within the declared profile | Use the engine-owned headless lab or another trainer |

## Supersession

Runtime learning, arbitrary multi-expert blending, topology-changing policy
transition или объединение habits с motor controller требует нового ADR.
Замена training algorithm/backend при сохранении runtime contracts требует
только relevant product checks.
