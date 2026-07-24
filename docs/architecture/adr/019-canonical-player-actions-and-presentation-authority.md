# ADR-019: Canonical player actions и presentation authority

| Поле | Значение |
|---|---|
| ID | ADR-019 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Player Experience Team |
| Требуемые согласующие | Architecture Working Group, Rendering Team, Runtime Team, RPG Framework Team, Developer Experience Team, Verification & Evidence Team, Security & Governance Team |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-04](../04-rendering-and-platform.md), [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [ADR-002](002-rust-first-ffi-and-ecs-facade.md), [ADR-014](014-deterministic-extensions-and-package-trust.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## История принятия

ADR принят в architecture packet 1.7 как authority для canonical player actions и presentation-only UI/camera/localization/accessibility state. Downstream Player Experience specification depends on this ADR; reverse dependency отсутствует. Принятие contract не объявляет implementation gates или `vertical-v1` conformance пройденными и не выбирает input library, widget toolkit, UI framework или device backend.

## Контекст

SPEC-04 нормализует raw platform controls, но оставляет gameplay bindings выше platform layer. Existing contracts требуют UI/camera evidence и capability-scoped panels, однако без этого решения не задают одного owner, canonical action format, targeting boundary, localization identity или accessibility fallback. Widget/camera/device state иначе может стать скрытым gameplay input и разойтись между `game`, headless и replay.

## Решение

1. Player Experience Team владеет action maps, input contexts, semantic UI, camera presentation, localization и accessibility contracts внутри existing Presentation bounded context. Runtime/RPG/Mechanics сохраняют ownership ingress assignment, command validation and committed outcomes.
2. Private PlatformHost adapter выдаёт engine-owned normalized semantic controls. Deterministic resolver produces canonical device-independent immutable `PlayerActionFrame` bytes at declared frame boundaries. Public schema excludes OS/vendor/ECS/backend objects and не фиксирует concrete input/UI implementation.
3. Interactive input and headless scenario enqueue the same `PlayerActionFrame` schema through the common `InputSampleV1` gateway. ADR-022 current/next close barrier alone creates persisted `IngressAssignmentV1`; frame, wall clock, renderer cadence and scenario cannot choose target tick. Capture replay feeds the recorded closed ingress batch through the same mapper/validator.
4. Player action is an input proposal, not authority. Production mapper derives command target tick only from `IngressAssignmentV1`; domain state changes only after the action-derived candidate passes common `WorldCommand` validation and atomic commit.
5. Camera state is presentation-only. `TargetingIntent` carries assigned tick, quantized aim, gameplay targeting-profile identity and optional proposed `PersistentId`, never camera transform, GPU depth or rendered pose. Runtime reconstructs `AuthoritativeTargetingQueryV1` from authoritative snapshot/pose/ability data and validates it before commit.
6. UI reads immutable `UiSemanticSnapshot` projections. First-party and capability-scoped extension panels publish actions/proposals through the same ADR-014 boundary; callbacks cannot mutate domain state, and denied/trapped/over-budget optional panels discard uncommitted proposals.
7. Stable text/content IDs are locale-independent. Localization/accessibility/preferences cannot change command ordering, simulation schedule or authoritative outcome for the same assigned action input. Alternative controls invoke the same action ID and validation path.
8. Action-to-command mapping and targeting validation inside the gameplay tick use the existing ADR-016 `core-command-rpg` budget row. This decision creates no new budget owner and never permits wall-time dropping/reordering of required assigned actions.
9. Observable UI/camera changes retain SPEC-15 automatic evidence and authorized ADR-023 V2 human-review admission. Human review cannot waive semantic/replay failure.

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
- Public contracts remain implementation-neutral and expose no native device, widget, window, renderer or ECS type.

## Gates и fallback

Implementation verification requires `INPUT-P1`, `UI-P1`, `CAMERA-P1` and `ACCESS-P1`. A failing optional adapter/panel is disabled behind the same semantic contract; invalid required map/schema/profile fails before world mutation; declared bounded preference/source-locale/readable fallbacks remain available. Automatic semantic/replay failures cannot be waived by visual review.

## Supersession

ADR-019 does not supersede an earlier decision. Direct UI mutation, authoritative camera/GPU targeting, localized durable identity, device-specific action semantics, a private headless input path or a concrete public input/UI backend choice requires a new superseding ADR and synchronized downstream SPEC/trace/evidence updates.
