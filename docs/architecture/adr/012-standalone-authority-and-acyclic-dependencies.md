# ADR-012: Standalone authority и ациклические нормативные зависимости

| Поле | Значение |
|---|---|
| ID | ADR-012 |
| Статус | Proposed |
| Версия | 1.0 |
| Владелец | Architecture Working Group |
| Дата решения | ожидает human approval |
| Последняя проверка evidence | 2026-07-22 |
| Нормативные зависимости | [INDEX-001](../README.md), [GLOSSARY-001](../glossary.md) |
| Связанные документы | все SPEC, ADR, [TRACE-001](../traceability.md), [EVIDENCE-001](../evidence-register.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Контекст

Packet 1.4 содержит внешнюю нормативную research-ссылку, взаимные зависимости SPEC/ADR и обратную зависимость SPEC-12 → TRACE. Такой граф не задаёт воспроизводимый порядок authority и не является самодостаточным.

## Решение

- `docs/architecture/` MUST быть единственным локальным нормативным пакетом. Внешний источник MAY использоваться только как provenance или evidence; нормативный внешний материал MUST быть механически заморожен локально с source commit, blob, SHA-256 и лицензией.
- Активный нормативный граф MUST быть ациклическим: `INDEX/GLOSSARY → ADR → foundational SPEC → subsystem SPEC → SPEC-15 → SPEC-13/SPEC-14 → SPEC-12 → TRACE/EVIDENCE`.
- ADR MUST зависеть только от INDEX/GLOSSARY и более ранних ADR. SPEC, который применяет ADR, указывается как `Связанный документ`, не как обратная нормативная зависимость ADR.
- `TRACE-001` зависит от SPEC-12; SPEC-12 MUST NOT зависеть от TRACE. Evidence register фиксирует состояние технологий, но не создаёт решения.
- `adr/000-template.md` является ненормативным template и не учитывается как ADR-решение.
- Индекс MUST совпадать с metadata каждого документа. `Superseded` допускается только с двусторонними ссылками `Заменяет/Заменён` после одобрения нового ADR.
- Packet candidate MUST оставаться `Proposed` до human approval exact changeset hash; agent не может создать это approval.

## Последствия

`docs-check` строит граф по локальным Markdown-ссылкам поля `Нормативные зависимости`, отклоняет циклы, HTTP(S), отсутствующие узлы, metadata/index drift и некорректное supersession. Замороженный annex проверяется отдельно и не обязан принимать Next Engine metadata внутрь исходных байтов.

## Gate для Proposed частей

| Поле | Требование |
|---|---|
| Владелец | Architecture Working Group + Developer Experience |
| Сценарий/команда | `cargo run -p xtask -- docs-check` плюс negative fixture suite |
| Threshold | 100% индексированных документов разрешены локально; 0 циклов/внешних normative links/orphans; все deliberate negative fixtures отклонены стабильным кодом |
| Evidence | graph report, fixture report, annex provenance/hash report, candidate changeset hash |
| Fallback | Packet 1.4 остаётся Accepted baseline; Packet 1.5 остаётся Proposed/AwaitingCapability |
| Срок повторной проверки | перед каждым architecture packet promotion |

## Supersession

Не применяется до human approval. После принятия ADR становится authority для структуры последующих пакетов.
