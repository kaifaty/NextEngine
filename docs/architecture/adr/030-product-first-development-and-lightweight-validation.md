# ADR-030: Product-first development и lightweight validation

| Поле | Значение |
|---|---|
| ID | ADR-030 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-07-25 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md) |
| Заменяет | полностью [ADR-010](010-artifact-first-headless-validation-and-review.md), [ADR-015](015-evidence-trust-fixture-separation-and-attestation.md), [ADR-023](023-human-review-decision-v2-and-offline-attestation.md), [ADR-024](024-requirement-gate-evidence-and-profile-closure.md); частично governance/evidence/certification/signature sections [ADR-001](001-product-repository-license-and-platforms.md), [ADR-009](009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-011](011-macos-developer-host-local-verification-and-staged-training.md), [ADR-013](013-self-contained-physical-avatar-boundary.md), [ADR-014](014-deterministic-extensions-and-package-trust.md) |
| Заменён | не заменён |

## История принятия

ADR принят прямым запросом на архитектурное изменение 2026-07-25. Он упрощает процесс
разработки, не меняя техническую модель authoritative state, deterministic
runtime или security boundary. Отдельный exact-root promotion, signed review
artifact, team quorum или повторное approval для этого решения не требуются.

## Контекст

Предыдущая baseline превращала разработку продукта в certification pipeline:
даже обычное изменение могло требовать `ImpactResolver`, `EvidenceBundle`,
capture artifacts, human review, offline signatures, trust/revocation closure,
полного requirement graph и агрегата из пятнадцати vertical gates. Эти
механизмы усложняют ранний engine bootstrap, замедляют игровой feedback loop и
не улучшают сами по себе command validation, save safety или replay correctness.

Next Engine нужен короткий путь от изменения к работающему игровому циклу.
Проверки должны соответствовать риску изменения и давать разработчику полезный
сигнал, а не производить admission artifacts.

## Решение

### Product-first priority

Порядок приоритетов:

1. работающий и понятный player-facing loop;
2. сохранность authoritative world state и возможность продолжить игру;
3. deterministic replay/debug для authoritative поведения;
4. безопасная загрузка content, packages, saves и внешних inputs;
5. производительность и platform-specific качество там, где изменение их
   действительно затрагивает.

Architecture и tooling SHOULD помогать быстро проверять эти свойства. Они MUST
NOT требовать ceremony, не влияющую на продуктовый риск.

### Сохраняемые технические инварианты

Этот ADR не ослабляет следующие требования:

- каждый mutable authoritative field имеет ровно один source of truth;
- gameplay state меняется только через validated engine-owned
  `WorldCommand`/transaction boundary; committed `DomainEvent` и immutable
  projections остаются единственным внешним наблюдением результата;
- fixed simulation stages, deterministic commit points, command identity,
  save/load atomicity и replay equivalence сохраняются;
- contracts остаются engine-owned; ECS, OS, vendor, database, importer и VM
  types не выходят в public boundaries;
- untrusted input проверяется до mutation: schema/version, bounds, canonical
  encoding, hashes, capabilities, references и resource limits;
- corrupt или incompatible authoritative data fail-closed без partial mutation;
- scripts/plugins/packages работают через immutable views и proposal/command
  sinks с capability checks, instruction/fuel, allocation, memory, host-call и
  output limits; trap или overrun не публикует partial state;
- network, LLM, renderer timing и optional services не определяют correctness
  mandatory gameplay; deterministic fallback сохраняется;
- credentials, private keys, signing material, protected/imported assets,
  datasets, checkpoints и generated runs не попадают в Git или distributed
  engine artifacts;
- source/import provenance и обычные license/third-party notices сохраняются.

### Удаляемые обязательные процессы

Следующие механизмы больше не являются architecture, merge, package, model,
baseline или release admission requirements:

- `EvidenceBundle`, `EvidenceBundleManifest`, mandatory capture dossier и
  hash-bound media baseline;
- `HumanReviewDecisionV1/V2`, `AttestationEnvelopeV1/V2`, reviewer trust
  manifest, revocation snapshot, signer isolation и cryptographic admission;
- обязательный human approval для visual, UI, camera, animation, physics,
  motor, audio или иных изменений;
- `RequirementGraphV1`, `GateDescriptorV1`, `TRACE-01`, reverse gate closure и
  обязательное owner→gate→evidence→VS/profile отображение;
- engine-owned mandatory `ImpactResolver`, запрет автору выбирать разумный
  focused test set и автоматическое расширение каждого changeset до dossier;
- требование `15/15 PASS`, blocking `VS-01`…`VS-15`, team quorum,
  `required approvers`, certification committee или capability-based
  `AwaitingCapability` как условие обычной разработки;
- `PhysicalCertified` и аналогичные certification labels как обязательный
  lifecycle или право использовать content;
- signature/trust tier как обязательное условие загрузки local package;
- exact-root architecture promotion и signed/hash-bound historical review
  packet как источник текущей authority.

Старые gate IDs, scenarios, captures, manifests и review schemas MAY оставаться
полезными test recipes или historical audit data. Их отсутствие, неполнота или
статус не блокируют изменение сами по себе. Недоступная GPU, encoder, RTX
machine или reviewer capability означает только, что соответствующая
необязательная проверка не запускалась.

### Lightweight product checks

Изменение проходит минимальный релевантный набор:

| Check | Когда обязателен | Минимальный результат |
|---|---|---|
| `fast` | Каждый code change | format, compile/typecheck, lint, focused/unit tests и boundary scan для изменённой области |
| `play` | Gameplay, runtime, UI, renderer, input или composition change | приложение либо representative headless scenario запускается, загружает проект и выполняет затронутый игровой цикл без crash или явной regression |
| `persistence-replay` | Authoritative state, commands, IDs, save schema, scheduling или migrations | save→load продолжает мир; exact replay либо focused deterministic comparison проходит; corrupt/incompatible input отвергается до mutation |
| `content-package` | Asset schema, cooker, importer boundary, package/plugin или distribution change | representative content/package validates, cooks/loads where applicable; bounds/hash/version checks и scan на protected data/basic license notices проходят |
| `platform` / `performance` | Только когда изменение materially затрагивает frame/tick budget, memory, I/O, renderer, physics или OS integration | targeted benchmark/smoke на доступной релевантной platform; отсутствие необязательного hardware не блокирует unrelated work |

Repository-owned commands SHOULD объединять эти проверки в быстрые,
диагностируемые entry points. Focused checks запускаются во время итерации;
более широкий local check запускается перед handoff, когда он существует и
соответствует изменению. Flaky retry-to-green, wall-clock gameplay assertions и
скрытые mutable test backdoors остаются запрещены.

Ни один check не требует signed artifact, evidence bundle или human approval.
Обычный code review, playtesting и release judgement MAY использоваться как
рабочая практика, но не создают отдельный serialized authority contract.

### Package, model и physical content

- Exact content hash, compatibility metadata, input limits, sandbox,
  capabilities и deterministic fallback остаются обязательными technical
  controls.
- Package PKI и signer lifecycle не входят в engine runtime contracts;
  distribution-specific wrapping при необходимости остаётся внешней задачей.
- Learned policy/model MAY использоваться после relevant runtime smoke,
  compatibility и safety checks. Отдельный certification статус, media dossier
  или RTX ceremony не требуется.
- Недоступный backend/model использует declared deterministic fallback; это
  product behavior, а не certification state.

### Architecture и historical records

Architecture меняется обычным repository workflow через новый ADR при
семантическом решении. Прямого запроса на архитектурное изменение достаточно. Не требуются
exact-root confirmation, `architecture.promote`, signed approval или
синхронное обновление generated evidence graph.

Старые architecture review packets, evidence registers, traceability tables,
gate catalogs, attestations и certification records являются historical-only
reference. Они не могут разрешать либо запрещать текущий merge/release и не
обязаны поддерживаться executable verifier.

Если Accepted SPEC или более старый ADR требует удалённый этим решением
artifact, review, signature, trust, gate-closure, quorum или certification flow,
применяется ADR-030. Остальная technical semantics такого документа сохраняется
до отдельного superseding решения.

## Рассмотренные варианты

- Сохранить полный pipeline только «на будущее» — `Rejected`: формально
  обязательный неиспользуемый процесс создаёт постоянную ложную
  non-conformance и отвлекает от продукта.
- Удалить вместе с ceremony determinism и input validation — `Rejected`:
  именно эти свойства защищают saves, replay и пользовательские данные.
- Требовать human approval только для observable changes — `Rejected`:
  playtesting полезен, но serialized approval и signer infrastructure не нужны
  для разработки engine.
- Требовать подпись для всех packages — `Rejected`: local-first modding
  нуждается в hashes, sandbox и capabilities, а не в собственной PKI.

## Последствия

- Изменения быстрее доходят до работающего play loop.
- Architecture documents перестают быть certification checklist; trace/evidence
  catalogs могут постепенно упрощаться отдельными changesets.
- Команда самостоятельно выбирает focused checks пропорционально риску и
  сообщает, что действительно запускалось.
- Ошибка выбора слишком узкого test set становится обычным engineering risk,
  компенсируемым regression tests и broader checks, а не requirement-graph
  infrastructure.
- Technical determinism, persistence, ownership, sandboxing, bounds и
  protected-data boundaries остаются обязательными.

## Supersession

Любое возвращение mandatory signed review, attestation PKI, requirement/gate
graph admission, fixed all-gates aggregate, team quorum или certification
lifecycle требует нового ADR с конкретной product problem, стоимостью
поддержки и lightweight fallback.
