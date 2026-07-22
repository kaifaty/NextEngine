# ADR-013: Canonical WorldCommand ordering

| Поле | Значение |
|---|---|
| ID | ADR-013 |
| Статус | Proposed |
| Версия | 1.0 |
| Владелец | Runtime Team |
| Дата решения | ожидает human approval |
| Последняя проверка evidence | 2026-07-22 |
| Нормативные зависимости | [ADR-007](007-identities-persistence-and-replay.md), [ADR-012](012-standalone-authority-and-acyclic-dependencies.md) |
| Связанные документы | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Контекст

SPEC-02 называет tuple сортировки, но текущий command envelope не определяет доверенный `priority_class`, стабильный issuer key и tie-break semantics. Это оставляет ordering зависимым от места доставки и позволяет недоверенному издателю влиять на приоритет.

## Решение

- Кандидат команды MUST содержать `target_tick`, стабильный `CommandIssuerId`, выданный издателем `issuer_sequence: u64`, `command_id` и payload/schema ID. Издатель MUST NOT задавать authoritative priority.
- Валидатор выводит `priority_class` из versioned engine-owned `CommandPriorityRegistry` по schema и capability издателя.
- Принятая `CanonicalWorldCommand` MUST хранить полный `CanonicalCommandOrderKey = (target_tick, priority_class, issuer_id, issuer_sequence, command_id)`.
- Сравнение ID выполняется по canonical big-endian bytes. `issuer_sequence` строго возрастает для каждого issuer; gaps разрешены, stale/duplicate sequence отклоняется до mutation.
- Повторный `command_id` имеет стабильную idempotency/rejection семантику и никогда не создаёт второй commit.
- Ledger последовательностей и hash priority registry MUST входить в save/replay/schema manifest, достаточный для продолжения и диагностики.

## Failure semantics

Подделанный priority игнорируется и диагностируется; stale/duplicate sequence получает `COMMAND_SEQUENCE_REJECTED`; конфликтующий повторный ID — `COMMAND_ID_CONFLICT`; несовпавший registry hash — `INCOMPATIBLE_COMMAND_ORDER_REGISTRY`. Частичный commit запрещён.

## Gate для Proposed частей

| Поле | Требование |
|---|---|
| Владелец | Runtime Team + Verification & Evidence Team |
| Сценарий/команда | `next gate ORDER-01 --scenario command-order-v1 --permutations all` |
| Threshold | все input/internal/async permutations дают exact command/event/state hashes; 100% forged priority и stale/duplicate cases отклонены до mutation |
| Evidence | accepted-command trace, order registry manifest, permutation hashes, rejection corpus |
| Fallback | serial deterministic intake за тем же canonical key; feature не принимается без exact parity |
| Срок повторной проверки | перед runtime command schema freeze |

## Supersession

Изменение tuple, sequence ownership или priority derivation требует нового ADR.
