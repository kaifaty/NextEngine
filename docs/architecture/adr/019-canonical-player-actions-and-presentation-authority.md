# ADR-019: Canonical player actions и presentation authority

| Поле | Значение |
|---|---|
| ID | ADR-019 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | Player Experience Team |
| Требуемые согласующие | Architecture Working Group, Rendering Team, Runtime Team, RPG Framework Team, Developer Experience Team, Verification & Evidence Team, Security & Governance Team |
| Дата предложения | 2026-07-22 |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-18](../18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-04](../04-rendering-and-platform.md), [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [ADR-002](002-rust-first-ffi-and-ecs-facade.md), [ADR-006](006-scripting-and-plugin-model.md), [ADR-010](010-artifact-first-headless-validation-and-review.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Статус предложения

ADR входит в foundation-completeness proposal и остаётся `AwaitingReview` до atomic promotion всего track. Он не принимает UI framework/input library и не изменяет platform backend решения SPEC-04.

## Контекст

SPEC-04 нормализует raw platform controls, но оставляет gameplay bindings выше platform layer. Accepted пакет упоминает UI/camera evidence и `ui.panel.register`, однако не задаёт owner, canonical action format, targeting boundary, localization identity или accessibility fallback. Без решения widget/camera/device state может стать скрытым gameplay input и разойтись между game/headless/replay.

## Решение

При принятии:

1. Новый Player Experience Team владеет action maps, input contexts, semantic UI, camera presentation, localization и accessibility contracts внутри existing Presentation bounded context.
2. PlatformHost выдаёт engine-owned normalized controls. Deterministic resolver produces immutable `PlayerActionFrame` values at declared sampling boundaries.
3. Player action is a proposal source, not authority. Domain state changes only after action-derived command candidate passes common WorldCommand validation.
4. Camera state is presentation-only. Gameplay targeting carries bounded source-tick intent and is reconstructed/validated from authoritative snapshot; GPU/interpolated camera output cannot select hit authority.
5. UI reads immutable semantic projections. First-party and plugin panels publish actions/proposals through the same capabilities and cannot mutate domain state.
6. Stable text/content IDs are locale-independent. Localization/accessibility/user preferences cannot change command ordering, simulation schedule or gameplay hashes.
7. Headless scenarios inject the same action contract without PlatformHost; replay retains accepted command stream as authority and may store actions as oracle.

## Рассмотренные варианты

- Widget callbacks call gameplay services directly — `Rejected`: bypasses validation and creates UI-specific authority.
- Persist device scancodes/native events as gameplay ABI — `Rejected`: platform/vendor coupling and poor accessibility.
- Camera ray/GPU depth is authoritative targeting — `Rejected`: frame-rate/render/backend dependent.
- Localized strings as quest/action IDs — `Rejected`: locale changes break saves/replays/packages.
- Separate accessibility command paths — `Rejected`: hidden semantics and untested divergence.
- Rendering Team owns all player semantics — `Rejected`: backend ownership would absorb domain-facing action/UI policy; dedicated Player Experience owner is explicit.

## Последствия

- Input/UI/camera contracts become testable in CPU headless form.
- Rebinding and accessibility can vary freely while semantic action IDs remain stable.
- Targeting requires a versioned intent/query boundary and may reject visually plausible stale input rather than commit inconsistent state.
- Observable presentation changes retain SPEC-15 human evidence requirements.

## Gates и fallback

Acceptance requires `INPUT-P1`, `UI-P1`, `CAMERA-P1` and `ACCESS-P1`. Failing adapter/map/panel/camera profile remains Proposed or disabled; accessible keyboard/text/source-locale defaults remain available where project requirements permit. Automatic semantic/replay failures cannot be waived by visual review.

## Promotion и supersession

ADR does not supersede ADR-002/006/010. It is accepted only with SPEC-17…20 and ADR-018/020/021. Direct UI mutation, authoritative camera/GPU targeting, localized durable identity or device-specific action semantics require a new superseding ADR.
