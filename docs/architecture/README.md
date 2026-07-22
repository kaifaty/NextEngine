# Next Engine: нормативный пакет архитектурных спецификаций

| Поле | Значение |
|---|---|
| ID | INDEX-001 |
| Статус | Accepted |
| Версия | 1.4 |
| Review candidate | 1.5 |
| Владелец | Architecture Working Group |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | отсутствуют |
| Заменяет | отсутствует |

Этот каталог задаёт архитектурную baseline независимого AI-first RPG-движка, временно называемого **Next Engine**. Он не описывает перенос OpenGothic и не меняет контракты действующего C++20 runtime. Документы написаны так, чтобы после review пакет можно было перенести без смысловых изменений в отдельный engine monorepo.

Architecture packet version 1.4 объединяет перечисленные в индексе версии RFC/ADR в один review set. Статус `Accepted` фиксирует decision baseline, достаточную для standalone Mac-first bootstrap; он **не** утверждает, что ещё не созданная implementation прошла gates SPEC-12. Такой результат называется отдельно `vertical-v1 implementation conformance`.

В индексе также опубликован packet 1.5 remediation candidate: ADR-012…ADR-016 имеют статус `Proposed/AwaitingReview` и не входят в Accepted baseline. Они не изменяют смысл SPEC или ADR packet 1.4, пока Architecture Working Group, профильные owners и Security для trust-boundary решений не дадут явные approvals. Promotion выполняется только атомарной синхронизацией статусов, SPEC, glossary, evidence register и traceability; частично синхронизированный candidate не является baseline.

Отдельно опубликован post-1.5 dialogue/model proposal track: SPEC-16 и ADR-017 имеют статус `Proposed/AwaitingReview`, а RESEARCH-002 является ненормативным `Draft` snapshot. Этот track не входит ни в Accepted packet 1.4, ни в remediation transaction 1.5. Он фиксирует text-canonical dialogue/model-pack acceptance contract; перечисленные model rows остаются `Proposed` до прохождения собственных gates.

## Нормативный язык

Ключевые слова **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT** и **MAY** имеют нормативный смысл RFC 2119/8174 только когда написаны заглавными буквами:

- **MUST / MUST NOT** — обязательное условие conformance;
- **SHOULD / SHOULD NOT** — рекомендуемое условие, отступление требует записанного обоснования и компенсирующей проверки;
- **MAY** — допустимый, но необязательный вариант.

При конфликте действует следующий приоритет: более новый Accepted ADR, явно заменяющий старое решение → `12-vertical-slice-conformance.md` для release gates → профильный subsystem RFC → product contract → glossary. Evidence register сообщает факты, но сам по себе не принимает архитектурных решений.

## Статусы и изменение решений

Для RFC и ADR разрешены статусы `Draft`, `Proposed`, `Accepted`, `Rejected`, `Superseded`. Для внешней технологии дополнительно используется только один из тех же четырёх решающих статусов: `Accepted`, `Proposed`, `Rejected` или `Superseded`.

- `Draft` не входит в baseline.
- `Proposed` требует измеримого gate и заранее указанного fallback.
- `Accepted` обязателен для conforming implementation.
- `Rejected` хранит отрицательное решение и причину.
- `Superseded` содержит ссылку на заменяющий документ.

Accepted ADR не редактируется так, чтобы изменить смысл решения. Изменение оформляется новым ADR с полем `Заменяет`, затем синхронно обновляются зависимые RFC, evidence register и traceability. Редакционные исправления без изменения смысла разрешены с увеличением patch-версии.

Каждый RFC MUST содержать ID, статус, версию, владельца, дату проверки, нормативные зависимости и сведения о supersession. Каждый `Proposed` backend MUST иметь владельца gate, команду или воспроизводимый сценарий, числовой pass/fail threshold, перечень evidence artifacts и fallback.

## Порядок чтения

1. [Product contract](00-product-contract.md) и [system architecture](01-system-architecture.md).
2. [Glossary](glossary.md) — единые имена контрактов и идентификаторов.
3. RFC 02–11 в порядке индекса ниже.
4. [Gameplay mechanics, mod packages и agent authoring](13-gameplay-mechanics-mod-packages-and-agent-authoring.md).
5. [Physical archetypes, motor skills и policy lifecycle](14-physical-archetypes-motor-skills-and-policy-lifecycle.md).
6. [Headless testing, agent validation и human evidence](15-headless-testing-agent-validation-and-human-evidence.md).
7. [Vertical-slice conformance](12-vertical-slice-conformance.md).
8. [Traceability](traceability.md), [evidence register](evidence-register.md) и [ADR](adr/000-template.md).
9. Для review packet 1.5 — Proposed ADR-012…ADR-016. Их требования являются future acceptance contracts, а не заявлением runtime/gate PASS.
10. Для post-1.5 dialogue/model proposal — [RESEARCH-002](research/npc-dialogue-model-landscape.md), [SPEC-16](16-text-canonical-multimodal-dialogue-and-model-packs.md) и [ADR-017](adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md).

## Индекс нормативных документов

| ID | Документ | Статус | Владелец | Нормативные зависимости |
|---|---|---|---|---|
| SPEC-00 | [Продуктовый контракт](00-product-contract.md) | Accepted | Product Architecture | INDEX-001, ADR-001, ADR-008, ADR-009, ADR-010, ADR-011 |
| SPEC-01 | [Системная архитектура](01-system-architecture.md) | Accepted | Core Architecture | SPEC-00, SPEC-14, SPEC-15, ADR-001, ADR-002, ADR-005, ADR-008, ADR-009, ADR-010, ADR-011 |
| SPEC-02 | [Runtime, ECS и модель данных](02-runtime-ecs-and-data.md) | Accepted | Runtime Team | SPEC-01, SPEC-15, ADR-002, ADR-007 |
| SPEC-03 | [Assets, streaming и persistence](03-assets-world-streaming-and-persistence.md) | Accepted | Asset & Persistence Team | SPEC-02, SPEC-14, SPEC-15, ADR-007 |
| SPEC-04 | [Rendering и platform](04-rendering-and-platform.md) | Accepted | Rendering Team | SPEC-01, SPEC-03, SPEC-15, ADR-003, ADR-010 |
| SPEC-05 | [Physics, animation и motor control](05-physics-animation-and-motor-control.md) | Accepted | Physical Embodiment Team | SPEC-02, SPEC-14, SPEC-15, ADR-004, ADR-009 |
| SPEC-06 | [AI agents, perception и memory](06-ai-agents-perception-and-memory.md) | Accepted | Agent Intelligence Team | SPEC-02, SPEC-13, SPEC-14, ADR-005 |
| SPEC-07 | [RPG, scripting и plugins](07-rpg-scripting-and-plugins.md) | Accepted | RPG Framework Team | SPEC-02, SPEC-06, SPEC-14, ADR-006 |
| SPEC-08 | [Audio, navigation и world services](08-audio-navigation-and-world-services.md) | Accepted | World Services Team | SPEC-02, SPEC-05, SPEC-15 |
| SPEC-09 | [Tooling, SDK и observability](09-tooling-sdk-and-observability.md) | Accepted | Developer Experience Team | SPEC-02, SPEC-03, SPEC-13, SPEC-14, SPEC-15, ADR-010, ADR-011 |
| SPEC-10 | [Граница Gothic importer](10-gothic-importer-boundary.md) | Accepted | Importer Team | SPEC-03, ADR-001, ADR-007, ADR-011 |
| SPEC-11 | [Security, licensing и governance](11-security-licensing-and-governance.md) | Accepted | Security & Governance Team | все SPEC, ADR-001, ADR-006, ADR-008, ADR-009, ADR-010 |
| SPEC-12 | [Vertical-slice conformance](12-vertical-slice-conformance.md) | Accepted | Release Engineering | SPEC-00…SPEC-11, SPEC-13, SPEC-14, SPEC-15, ADR-011 |
| SPEC-13 | [Gameplay mechanics, mod packages и agent authoring](13-gameplay-mechanics-mod-packages-and-agent-authoring.md) | Accepted | Gameplay Extensibility Team | SPEC-02, SPEC-03, SPEC-05, SPEC-07, SPEC-09, SPEC-11, SPEC-14, SPEC-15, ADR-008, ADR-010 |
| SPEC-14 | [Physical archetypes, motor skills и policy lifecycle](14-physical-archetypes-motor-skills-and-policy-lifecycle.md) | Accepted | Physical Embodiment Team | SPEC-03, SPEC-05, SPEC-06, SPEC-07, SPEC-09, SPEC-13, SPEC-15, ADR-009, ADR-010, ADR-011 |
| SPEC-15 | [Headless testing, agent validation и human evidence](15-headless-testing-agent-validation-and-human-evidence.md) | Accepted | Verification & Evidence Team | SPEC-01, SPEC-02, SPEC-03, SPEC-04, SPEC-09, SPEC-11, SPEC-13, ADR-010 |
| SPEC-16 | [Text-canonical multimodal dialogue и model packs](16-text-canonical-multimodal-dialogue-and-model-packs.md) | Proposed | Agent Intelligence Team | SPEC-01, SPEC-03, SPEC-06, SPEC-07, SPEC-08, SPEC-09, SPEC-11, SPEC-12, SPEC-15, ADR-005, ADR-007, ADR-010 |
| GLOSSARY-001 | [Глоссарий](glossary.md) | Accepted | Architecture Working Group | INDEX-001 |
| EVIDENCE-001 | [Реестр доказательств](evidence-register.md) | Accepted | Architecture Working Group | профильные ADR |
| TRACE-001 | [Матрица трассируемости](traceability.md) | Accepted | Release Engineering | SPEC-12 |

## Индекс ADR

| ID | Решение | Статус |
|---|---|---|
| ADR-000 | [Шаблон ADR](adr/000-template.md) | Accepted |
| ADR-001 | [Продукт, репозитории, лицензия и платформы](adr/001-product-repository-license-and-platforms.md) | Accepted |
| ADR-002 | [Rust-first core, FFI и ECS facade](adr/002-rust-first-ffi-and-ecs-facade.md) | Accepted |
| ADR-003 | [Vulkan renderer и compiler-neutral shaders](adr/003-vulkan-renderer-and-shader-toolchain.md) | Accepted |
| ADR-004 | [Physics-authoritative avatars и replaceable backend](adr/004-physics-avatar-backend-boundary.md) | Accepted |
| ADR-005 | [Offline-first AI и process boundary](adr/005-offline-first-ai-process-boundary.md) | Accepted |
| ADR-006 | [Luau gameplay и Wasm plugins](adr/006-scripting-and-plugin-model.md) | Accepted |
| ADR-007 | [Stable IDs, persistence и replay](adr/007-identities-persistence-and-replay.md) | Accepted |
| ADR-008 | [Mechanics/mod package и agent-ready authoring](adr/008-mechanics-mod-package-and-agent-authoring-model.md) | Accepted |
| ADR-009 | [Pretrained foundation policies и progressive motor skills](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md) | Accepted |
| ADR-010 | [Artifact-first headless validation и human review](adr/010-artifact-first-headless-validation-and-review.md) | Accepted |
| ADR-011 | [macOS developer host, local verification и staged training capability](adr/011-macos-developer-host-local-verification-and-staged-training.md) | Accepted |
| ADR-012 | [Deterministic command identity, ordering и replay](adr/012-deterministic-command-identity-and-replay.md) | Proposed |
| ADR-013 | [Self-contained physical-avatar authority boundary](adr/013-self-contained-physical-avatar-boundary.md) | Proposed |
| ADR-014 | [Deterministic extensions и package trust](adr/014-deterministic-extensions-and-package-trust.md) | Proposed |
| ADR-015 | [Evidence trust, fixture separation и offline attestation](adr/015-evidence-trust-fixture-separation-and-attestation.md) | Proposed |
| ADR-016 | [Compositional gameplay budgets](adr/016-compositional-gameplay-budgets.md) | Proposed |
| ADR-017 | [Text-canonical multimodal dialogue и replaceable model packs](adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md) | Proposed |

## Ненормативные приложения

| ID | Документ | Статус | Назначение |
|---|---|---|---|
| RESEARCH-001 | [Frozen physical-avatar research annex](research/physical-avatar-research-spec.md) | Historical | Проверяемая provenance ADR-013; не является нормативной dependency или Next Engine contract |
| RESEARCH-002 | [NPC dialogue model landscape](research/npc-dialogue-model-landscape.md) | Draft | Current primary-source snapshot; не является technology acceptance или Next Engine contract |

## Packet summary

Summary считает indexed architecture documents без frozen research annex RESEARCH-001. Document/SPEC/ADR/technology counts включают явно помеченные Proposed/Draft additions; requirement, failure и vertical-gate counts сохраняют Accepted packet 1.4 до atomic promotion.

| Metric | Value |
|---|---:|
| Markdown documents | 40 |
| Subsystem SPEC files | 17 |
| Decision ADR files | 17 |
| Vertical gates | 15 |
| Requirements | 78 |
| Failure paths | 24 |
| Technology rows | 33 |
| Proposed technology rows | 26 |

## Post-1.5 dialogue/model proposal track

Этот track reviewable независимо от remediation packet 1.5 и не получает номер Accepted packet, пока 1.5 не разрешён. Он состоит из:

- RESEARCH-002 — ненормативный market/Skyrim AI snapshot с Russian-local, multilingual, high-end, remote-opt-in и text-only profiles;
- SPEC-16 — engine-owned `CanonicalUtterance`, turn/stream/model-pack/provider/capability contracts, failures, CLI projections и numerical gates;
- ADR-017 — решение text-as-canonical, per-role replaceability, optional downloadable packs, remote opt-in и replay-without-regeneration;
- десяти `Proposed` model rows в evidence register; cloud services остаются research-only до выбора exact adapter.

Promotion требует approvals всех owners и Security, синхронизацию SPEC-01/03/06/07/08/09/11/12/15, glossary и traceability, а также добавление reserved REQ-079…086 и FAIL-025…030 без создания VS-16. Protocol approval не переводит model candidate в `Accepted`: exact model files отдельно проходят DIALOGUE/MODEL/license/voice gates. До этого track lifecycle — `AwaitingReview`, а `TextOnlyFallback` остаётся единственным mandatory baseline.

## Packet 1.5 remediation closure

Эта таблица является review checklist candidate, а не evidence PASS:

| Finding | Нормативное закрытие candidate | Primary owner | Gate/failure evidence |
|---|---|---|---|
| 1. Command identity и arrival-independent order | ADR-012: tagged principal, stream ledger, collision policy и полный sort tuple | Runtime Team | `RUNTIME-06`; permutation/collision corpus, ledger, state roots |
| 2. Post-physics same-tick mutation | ADR-012: закрытая `Outcome` phase на stage 9 без re-entry | Runtime Team | `RUNTIME-06`; stage trace и `OUTCOME_REENTRY_FORBIDDEN` |
| 3. Runtime durable identity | ADR-012: causal-command-derived PersistentId, retry/tombstone/collision semantics | Persistence Team | `RUNTIME-07`; spawn vectors, save/reload и collision corpus |
| 4. Canonical bytes, hashes и determinism classes | ADR-012: JCS, `CanonicalBinaryV1`, path/domain/Merkle rules и cross-target classes | Runtime + Persistence Teams | `CANON-01`; Windows/Linux golden vectors |
| 5. Protected/imported fixture leakage | ADR-015: isolated import-smoke, CC0 neutral fixture, cleanup before VS-09 | Importer Team | `PRIVACY-02`; prohibited-root scans и cleanup audit |
| 6. Human evidence trust | ADR-015: Ed25519 envelope, offline trust manifest/revocation и isolated signer | Verification & Evidence Team | `REVIEW-02`; tamper/role/expiry/revocation/agent-key corpus |
| 7. Wall-dependent script outcomes | ADR-014: instruction/fuel/allocation authority, wall trip only `NonConforming` | RPG Framework Team | `SCRIPT-P5`; CPU/load/watchdog permutations |
| 8. Package/plugin signature ambiguity | ADR-014: hash for all, signed trusted distribution, consent-bound unsigned local ceiling | Security & Governance Team | `MOD-P2`; signed/unsigned/invalid-signature matrix |
| 9. External physical normative dependency | ADR-013: complete local ownership/LOD/contact/safety/fallback contract | Physical Embodiment Team | docs-check plus public-contract scan; annex only non-normative |
| 10. Non-compositional performance limits | ADR-016: one 8/12 ms matrix, deterministic 100-NPC cadence и integrated methodology | Release Engineering | `PERF-01`; 1 000 warm-up + 10 000 measured ticks |
| 11. Composite requirement ownership | Promotion manifest below assigns exactly one primary owner and separate contributors | Architecture Working Group | docs-check rejects new composite primary owners |

## Atomic promotion manifest

Promotion разрешён только одним reviewed changeset после recorded approvals Architecture Working Group, всех указанных primary owners и Security для ADR-014/015. Exact transaction:

1. ADR-004/006/007/010 получают `Superseded` и backlink; ADR-012…016 получают `Accepted`.
2. INDEX/SPEC/glossary/evidence/traceability переходят на packet 1.5; accepted dependency graph удаляет два grandfathered external links ADR-004/SPEC-05.
3. SPEC-02/03/10 используют только ADR-012 ordering/identity/encoding; SPEC-05 — ADR-013; SPEC-07/11/13 — ADR-014; SPEC-09/10/11/12/15 — ADR-015; SPEC-06/08/12/13 — ADR-016.
4. REQ-005/011/013/015/019–023/031/032/036/064/072/073/075 и соответствующие FAIL rows получают новые ADR/gate/evidence links без изменения total counts: 78 REQ, 24 FAIL, 15 VS gates, 23 technology rows.
5. Glossary получает `CommandStreamId`, `IssuerPrincipal`, `CanonicalBinaryV1`, `AttestationEnvelope`, `ReviewerTrustManifest`, `GameplayBudgetMatrix`. JCS/Ed25519 libraries не добавляются как technology rows.
6. Traceability заменяет `Owner` на `Primary owner` и добавляет `Contributors / required approvers`; каждая строка получает ровно одного primary owner.

| Row | Primary owner after approval | Contributors / required approvers |
|---|---|---|
| REQ-032, FAIL-003 | RPG Framework | Security & Governance |
| REQ-043 | Gameplay Extensibility | Runtime |
| REQ-045, FAIL-009 | Persistence | Gameplay Extensibility |
| REQ-048, FAIL-010 | Developer Experience | Security & Governance |
| REQ-049 | Gameplay Extensibility | Physical Embodiment |
| REQ-051 | Agent Intelligence | Gameplay Extensibility |
| FAIL-007 | Importer | Security & Governance |

Если любой approval отсутствует или отклонён, transaction не выполняется: packet 1.4 остаётся authoritative, ADR-012…016 остаются `Proposed`, lifecycle state — `AwaitingReview`. Review record после будущего promotion связывает exact commit/diff hashes; agent не создаёт approval и не подписывает его.

## Review и перенос в отдельный репозиторий

Architecture baseline принимается только единым review-пакетом. Частичное одобрение отдельных файлов не делает архитектуру принятой. Initial packet version 1.0 был расширен version 1.1 через ADR-008/SPEC-13, version 1.2 — через ADR-009/SPEC-14, version 1.3 — через ADR-010/SPEC-15, а version 1.4 — через ADR-011 и синхронное уточнение platform/training/importer/traceability contracts. Accepted packet 1.4 содержит 32 Markdown-документа, 16 subsystem SPEC, 15 vertical gates, 78 requirements, 24 failure paths и 23 technology rows. Packet 1.5 candidate добавляет ровно пять Proposed ADR и доводит свой remediation track до ADR-000…ADR-016 и 37 architecture documents. Отдельный post-1.5 dialogue/model track добавляет SPEC-16, ADR-017 и parsed RESEARCH-002, поэтому текущий индекс содержит 40 architecture documents; frozen RESEARCH-001 учитывается отдельно. Любое последующее изменение подчиняется ADR/supersession rules. Review MUST подтвердить отсутствие orphan-документов, нерешённых архитектурных вопросов, vendor-типов в публичных контрактах и нетрассируемых vertical-slice требований.

Candidate закрывает review findings contractually, но не заявляет `vertical-v1`, `LEGAL-IMPORT-01=PASS` или `PhysicalCertified`. При отсутствии хотя бы одного required approval его итоговый lifecycle state остаётся `AwaitingReview`, ADR-012…016 — `Proposed`, а packet 1.4 — authoritative.

При создании самостоятельного репозитория каталог MUST быть перенесён в `docs/architecture/` с сохранением ID, истории ADR и относительных ссылок. Изменение путей MAY быть отдельным механическим commit; изменение решений в том же commit запрещено. После успешного local migration новый repository становится source of truth, а эта копия остаётся frozen historical source до появления reviewed permanent remote URL.
