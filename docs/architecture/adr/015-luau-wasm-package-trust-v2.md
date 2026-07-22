# ADR-015: Luau/Wasm package trust v2

| Поле | Значение |
|---|---|
| ID | ADR-015 |
| Статус | Proposed |
| Версия | 1.0 |
| Владелец | RPG Framework Team + Security & Governance |
| Дата решения | ожидает human approval |
| Последняя проверка evidence | 2026-07-22 |
| Нормативные зависимости | [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-012](012-standalone-authority-and-acyclic-dependencies.md) |
| Связанные документы | [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-006](006-scripting-and-plugin-model.md) |
| Заменяет | ADR-006 после human approval exact candidate hash |
| Заменён | не заменён |

## Контекст

ADR-006 требует signed/hashed manifest, тогда как package workflow допускает explicit local unsigned install. Требуется единая trust ladder без превращения подписи в sandbox bypass.

## Решение

Распределение ролей Luau/Wasm, capability isolation, budgets и failure semantics ADR-006 сохраняются; после принятия этот ADR полностью его заменяет.

- Каждый package/plugin artifact MUST иметь verified content hash и immutable manifest до загрузки.
- `UnsignedLocal` разрешён только по явному local user consent, не экспортируется как доверие и всегда ограничен untrusted capability ceiling.
- `UnsignedLocal` MUST NOT быть official, required distributed package, dependency release manifest или источником privileged capability.
- `SignedCommunity` и `OfficialRequired` проверяются по отдельному `PublisherTrustManifest`; reviewer/baseline keys не принимаются как publisher keys.
- `OfficialRequired` MUST иметь trusted non-revoked signature exact artifact+manifest hash. Signature не отменяет WIT negotiation, sandbox, budgets, capabilities или command validation.
- Unknown/revoked key, tamper, consent absence или попытка elevation отклоняются до world mutation; required-package failure закрывает world load, optional package изолированно отключается.

## Gate для Proposed частей

| Поле | Требование |
|---|---|
| Владелец | RPG Framework + Security & Governance |
| Сценарий/команда | `next gate TRUST-P1 --corpus package-trust-v1` |
| Threshold | 100% hash/tamper/unknown/revoked checks; unsigned local never exceeds ceiling; unsigned official/required always rejected; signature never alters sandbox results |
| Evidence | package/trust manifests, consent audit, capability traces, tamper/revocation corpus |
| Fallback | disable optional package; required package failure blocks world load; pin prior trusted package |
| Срок повторной проверки | перед distributed package support и при trust-root change |

## Supersession

До human approval ADR-006 остаётся Accepted. После одобрения exact candidate hash ADR-006 получает `Superseded` и ссылку на ADR-015.
