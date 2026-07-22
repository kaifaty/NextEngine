# ADR-014: Artifact-first review, baselines и attestation v2

| Поле | Значение |
|---|---|
| ID | ADR-014 |
| Статус | Proposed |
| Версия | 1.0 |
| Владелец | Verification & Evidence Team + Security & Governance |
| Дата решения | ожидает human approval |
| Последняя проверка evidence | 2026-07-22 |
| Нормативные зависимости | [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-012](012-standalone-authority-and-acyclic-dependencies.md) |
| Связанные документы | [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [ADR-010](010-artifact-first-headless-validation-and-review.md) |
| Заменяет | ADR-010 после human approval exact candidate hash |
| Заменён | не заменён |

## Контекст

ADR-010 определяет artifact-first review, но не задаёт первый baseline, atomic replacement, canonical signed bytes, trust roots или revocation semantics.

## Решение

ADR-010 сохраняется целиком, кроме расширений ниже; после принятия этот ADR полностью его заменяет.

- `BaselinePromotionDecisionV1` содержит schema/version, `mode: Bootstrap|Replace`, baseline key, candidate `EvidenceBaselineManifest` hash, previous baseline hash, EvidenceBundle/automatic gate root, VerificationPolicy hash, reviewer identity, `baseline.promote` capability, trust-manifest hash, rationale и `AttestationEnvelopeV1`.
- `Bootstrap` разрешён только когда baseline key отсутствует; previous hash MUST быть `null`. `Replace` требует exact current previous hash. Публикация нового immutable `BaselineIndexManifest` выполняется atomic compare-and-swap; mismatch ничего не меняет.
- `baseline.promote` проверяется независимо от capability review changeset. Один человек MAY иметь обе capabilities, но одна не подразумевает другую.
- `AttestationEnvelopeV1` использует algorithm registry; `ed25519-rfc8032` обязателен в v1. Signed bytes: ASCII domain `nextengine.review.v1\0` + RFC 8785 JCS decision object без поля `attestation`.
- JCS input MUST быть I-JSON. Все hashes/IDs и integer values вне безопасного IEEE-754 диапазона кодируются canonical strings.
- `ReviewerTrustManifest` является immutable versioned project trust root: key ID/public key, capabilities, validity, revocation и predecessor hash. Unknown algorithm/key/role или revoked key отклоняются fail-closed.
- Текущая revocation инвалидирует все решения соответствующего key и требует нового review; недоверенный timestamp не сохраняет старое approval.
- Reviewer credentials и private keys MUST находиться вне Git, agent context, evidence media и capture worker.

## Normative vectors

ATTEST-01 MUST фиксировать UTF-8/JCS bytes, SHA-256, Ed25519 public key/signature и expected decision hash для success case, а также tamper, field-order, unsafe-number, wrong-domain, wrong-role, unknown/revoked-key cases.

## Gate для Proposed частей

| Поле | Требование |
|---|---|
| Владелец | Verification & Evidence + Security & Governance |
| Сценарий/команда | `next gate BASELINE-01 --corpus baseline-lifecycle-v1` и `next gate ATTEST-01 --vectors review-v1` |
| Threshold | Bootstrap/Replace/CAS/role cases exact; cross-platform canonical bytes/signature verification 100%; все tamper/revocation cases fail closed |
| Evidence | baseline index/decision manifests, canonical vectors, role/revocation audit, atomic-publication fault report |
| Fallback | candidate остаётся неповышенным; действующий baseline неизменен; missing reviewer даёт AwaitingCapability |
| Срок повторной проверки | перед первым baseline и при algorithm/trust-manifest change |

## Supersession

До human approval ADR-010 остаётся Accepted. После одобрения exact candidate hash ADR-010 получает `Superseded` и ссылку на ADR-014.
