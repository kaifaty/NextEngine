# ADR-016: Rust-first audited FFI boundary v2

| Поле | Значение |
|---|---|
| ID | ADR-016 |
| Статус | Proposed |
| Версия | 1.0 |
| Владелец | Core Architecture + Security & Governance |
| Дата решения | ожидает human approval |
| Последняя проверка evidence | 2026-07-22 |
| Нормативные зависимости | [ADR-001](001-product-repository-license-and-platforms.md), [ADR-012](012-standalone-authority-and-acyclic-dependencies.md) |
| Связанные документы | [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-11](../11-security-licensing-and-governance.md), [ADR-002](002-rust-first-ffi-and-ecs-facade.md) |
| Заменяет | ADR-002 после human approval exact candidate hash |
| Заменён | не заменён |

## Контекст

ADR-002 допускает narrow FFI crates, но workspace policy и текущие crates запрещают `unsafe` без формального механизма исключения. Снятие запрета для workspace целиком разрушило бы portable-core invariant.

## Решение

Rust-first core, ECS facade и vendor-boundary решения ADR-002 сохраняются; после принятия этот ADR полностью его заменяет.

- `[workspace.lints.rust] unsafe_code = "forbid"` остаётся default и наследуется contracts/core/runtime/tooling crates.
- Исключение возможно только для отдельного backend/FFI crate, который одновременно: назван Accepted ADR; перечислен в `[workspace.metadata.nextengine.ffi].allowed_crates`; не наследует workspace lint table; задаёт `unsafe_code = "warn"` и `unsafe_op_in_unsafe_fn = "deny"`; имеет safe engine-owned public facade.
- Начальный allowlist пуст. Этот ADR не разрешает unsafe ни одному существующему crate.
- Каждый unsafe block требует локального `SAFETY:` rationale; каждый `unsafe fn` — `# Safety`; ownership/threading/ABI/cleanup и panic crossing фиксируются в contract tests.
- FFI/vendor/raw pointer types MUST NOT пересекать `crates/contracts` или другие engine-owned публичные contracts.
- `boundary-scan` отклоняет unsafe tokens, lint opt-out или allowlist drift вне exact исключения.

## Gate для Proposed частей

| Поле | Требование |
|---|---|
| Владелец | Core Architecture + Security & Governance |
| Сценарий/команда | `next gate FFI-01 --workspace` |
| Threshold | allowlist exact; 0 unsafe outside allowlist; 0 vendor/FFI types in public contracts; all applicable wrapper tests, Miri pure-Rust paths and native ASan/UBSan harnesses PASS |
| Evidence | Cargo metadata, boundary/API reports, SAFETY audit, Miri/sanitizer manifests |
| Fallback | safe Rust adapter, external process boundary или отсутствие backend; никакого workspace-wide allow |
| Срок повторной проверки | при первом FFI crate и каждом allowlist/ABI change |

## Supersession

До human approval ADR-002 остаётся Accepted. После одобрения exact candidate hash ADR-002 получает `Superseded` и ссылку на ADR-016.
