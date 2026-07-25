# ADR-019: Canonical player actions and presentation authority

| Поле | Значение |
|---|---|
| ID | ADR-019 |
| Статус | Accepted |
| Версия | 1.1 |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-04](../04-rendering-and-platform.md), [SPEC-07](../07-rpg-scripting-and-plugins.md), [ADR-002](002-rust-first-ffi-and-ecs-facade.md), [ADR-014](014-deterministic-extensions-and-package-trust.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | отсутствует |
| Заменён | частично [ADR-030](030-product-first-development-and-lightweight-validation.md) |

## Process baseline ADR-030

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежний общий admission lifecycle. Canonical player actions и
separation of presentation state from gameplay authority остаются техническим
решением.

## Контекст

SPEC-04 нормализует raw platform controls, но gameplay bindings находятся выше
platform layer. Без одного canonical action format, targeting boundary,
localization identity и accessibility fallback widget, camera или device state
может стать скрытым gameplay input и разойтись между `game`, headless и replay.

## Решение

1. `ActionMapManifest` и input-context stack определяют action maps и binding
   resolution. `UiSemanticSnapshot`, camera presentation config, localization
   catalogs и accessibility profiles определяют presentation state.
   `IngressAssignmentV1`, common command validator и committed domain state
   сохраняют gameplay authority.
2. Private PlatformHost adapter выдаёт engine-owned normalized semantic
   controls. Deterministic resolver создаёт canonical device-independent
   immutable `PlayerActionFrame` bytes на declared frame boundaries. Public
   schema исключает OS/vendor/ECS/backend objects и не фиксирует concrete
   input/UI implementation.
3. Interactive input и headless scenario enqueue один `PlayerActionFrame`
   schema через common `InputSampleV1` gateway. ADR-022 current/next close
   barrier создаёт persisted `IngressAssignmentV1`; frame, wall clock, renderer
   cadence и scenario не выбирают target tick. Replay подаёт recorded closed
   ingress batch через тот же mapper/validator.
4. Player action является input proposal, а не authority. Production mapper
   derives command target tick только из `IngressAssignmentV1`; domain state
   меняется после common `WorldCommand` validation и atomic commit.
5. Camera state presentation-only. `TargetingIntent` carries assigned tick,
   quantized aim, gameplay targeting-profile identity и optional proposed
   `PersistentId`, но не camera transform, GPU depth или rendered pose. Runtime
   reconstructs `AuthoritativeTargetingQueryV1` из authoritative
   snapshot/pose/ability data и validates it before commit.
6. UI читает immutable `UiSemanticSnapshot`. First-party и capability-scoped
   extension panels публикуют actions/proposals через ADR-014 boundary;
   callbacks не mutate domain state, а denied, trapped или over-budget optional
   panel discards uncommitted proposals.
7. Stable text/content IDs locale-independent. Localization, accessibility и
   preferences не меняют command ordering, simulation schedule или
   authoritative outcome для того же assigned action input. Alternative
   controls используют тот же action ID и validation path.
8. Action-to-command mapping и targeting validation используют существующую
   ADR-016 `core-command-rpg` budget row. Required assigned actions не
   отбрасываются и не переупорядочиваются по wall time.

## Рассмотренные варианты

- Widget callbacks call gameplay services directly — `Rejected`: обход
  validation создаёт UI-specific authority.
- Persist device scancodes/native events as gameplay ABI — `Rejected`:
  platform/vendor coupling и poor accessibility.
- Camera ray/GPU depth is authoritative targeting — `Rejected`: результат
  зависит от renderer backend и frame timing.
- Localized strings as quest/action IDs — `Rejected`: locale changes ломают
  saves, replays и packages.
- Separate accessibility command paths — `Rejected`: создают hidden semantics.
- Rendering subsystem определяет player-facing action semantics — `Rejected`:
  backend не должен владеть domain-facing action/UI policy.

## Product checks

| Сценарий | Ожидаемый результат | Fallback |
|---|---|---|
| Equivalent semantic controls проходят interactive и headless producers до/на/после ingress cutoff | Canonical `PlayerActionFrame`, assignment, action order и derived commands exact; target tick задаёт только close barrier | Reject affected frame либо disable optional adapter, сохранив accessible semantic-action path |
| Targeting повторяется при разных camera transforms, renderer cadence и GPU depth | Authoritative query даёт одинаковый result для одного snapshot; stale/invalid intent отклоняется до commit | No-target result либо новый input на будущем tick |
| Optional UI panel получает denied capability, trap или budget overrun | Ни одна domain mutation не публикуется; base UI и gameplay продолжают работать | Disable panel и использовать engine-owned semantic UI |
| Locale, rebinding или alternative control profile меняется для той же action sequence | Stable IDs, command order и authoritative outcome совпадают | Source locale, readable default theme и default accessible bindings |

## Последствия

- Input, UI и camera contracts проверяемы в CPU headless form.
- Rebinding и accessibility могут меняться, пока semantic action IDs стабильны.
- Targeting может отвергнуть visually plausible stale input вместо commit
  inconsistent state.
- Public contracts не содержат native device, widget, window, renderer или ECS
  type.
- Direct UI mutation, authoritative camera/GPU targeting, localized durable
  identity, device-specific action semantics или private headless input path
  требуют нового ADR и обновления affected SPECs и lightweight traceability.
