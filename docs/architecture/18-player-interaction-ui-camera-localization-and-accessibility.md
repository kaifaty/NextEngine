# SPEC-18: Player interaction, UI, camera, localization и accessibility

| Поле | Значение |
|---|---|
| ID | SPEC-18 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | Player Experience Team |
| Требуемые согласующие | Architecture Working Group, Rendering Team, Runtime Team, RPG Framework Team, Developer Experience Team, Verification & Evidence Team, Security & Governance Team |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md), [ADR-006](adr/006-scripting-and-plugin-model.md), [ADR-010](adr/010-artifact-first-headless-validation-and-review.md) |
| Заменяет | отсутствует |

## Статус предложения

Этот документ входит в отдельный post-1.5 foundation-completeness proposal track. Он не изменяет Accepted packet 1.4, remediation candidate 1.5 или SPEC-16/ADR-017. Термины MUST/MUST NOT являются future acceptance contract только после атомарного promotion SPEC-17…20 и ADR-018…021. До этого lifecycle state — `AwaitingReview`.

## Назначение и invariants

SPEC-18 определяет границу между raw platform input, canonical player actions, gameplay commands и presentation-only UI/camera state.

- Platform event или UI callback MUST NOT мутировать gameplay state напрямую.
- Action semantics MUST быть device-independent; bindings MAY различаться без изменения action IDs и domain outcomes.
- UI, camera, localized text и accessibility preferences MUST NOT быть gameplay authority.
- Aiming/targeting, dialogue choice, inventory action и interaction MUST входить в simulation как versioned action-derived command candidate and common validation.
- Headless scenario MUST уметь подавать тот же `PlayerActionFrame` без window/input device.
- Observable UI/camera changes MUST проходить SPEC-15 automatic evidence and authorized human review.
- Missing locale/device/optional panel MUST иметь bounded accessible fallback и не блокировать offline correctness.

## Source of truth и ownership

| State | Единственный owner/source of truth | Не является source |
|---|---|---|
| Raw normalized controls/focus/device lifecycle | SPEC-04 PlatformHost adapter | OS event objects after normalization |
| Action maps, context stack, binding resolution | Player Experience `ActionMapManifest` + session input service | UI widget callbacks, device driver profile |
| Canonical action input for one simulation sample | Immutable `PlayerActionFrame` | current renderer frame or wall clock |
| Gameplay result of an action | Owning domain after accepted WorldCommand | UI animation, camera ray, input binding |
| UI semantic state | Player Experience projection of immutable queries/snapshots | widget tree, GPU resources |
| Camera state | Player Experience presentation controller | character pose, projectile authority |
| Locale/accessibility preferences | Local `PlayerPreferenceProfile` | content IDs, quest/dialogue state |
| Localized resources | Cooked content registry by stable text ID + locale | rendered glyph string as identifier |

Player Experience Team владеет public action/UI/camera/localization/accessibility schemas. Rendering Team владеет raw platform adapter and visual backend. Runtime/RPG/Mechanics own validation and committed outcomes. Extension panel cannot acquire more authority than its package capabilities.

## Public contracts

### ActionMapManifest

`ActionMapManifest` содержит schema/version, stable action IDs, value kind, allowed phases, binding slots, supported device classes, context predicates, conflict policy, accessibility metadata and content hash.

Action IDs are locale- and device-independent namespaced values. Binding stores semantic controls, not SDL/Win32/XInput handles or scancodes in public contracts. Device adapter maps native inputs to engine-owned `NormalizedControlEvent` values containing device class/instance, control ID, phase/value, monotonic sample sequence and timestamp source.

### InputContext и resolution

`InputContext` is a versioned stack entry with context ID, revision, priority, capture/passthrough policy and allowed action set. Context transitions occur at declared input-sampling boundaries. Equal events, map, context revision and sampling window MUST produce the same action sequence.

Resolution order:

1. validate device/control/value bounds and monotonic sequence;
2. snapshot context stack revision for the sampling window;
3. resolve exact binding and conflict policy;
4. quantize analog values by declared integer/fixed-point profile;
5. order actions by sampling tick, context priority, action ID, device instance and sample sequence;
6. publish immutable `PlayerActionFrame` to production input gateway.

Device completion order, UI traversal order and OS callback order outside recorded sample sequence cannot choose winner. Rebinding publishes a new map revision only at boundary; an in-flight frame uses one revision.

### PlayerActionFrame

`PlayerActionFrame` contains schema version, player/controller PersistentId when applicable, input sampling tick/window, action map/context revisions and bounded ordered actions. Each action contains action ID, phase (`Started`, `Performed`, `Completed`, `Cancelled`), quantized scalar/vector value, source class and causal sample references.

The frame is production input, not committed gameplay. Input gateway maps actions to declared command candidates; common capability/schema/domain/physics validation still decides outcome. Replay authority remains accepted WorldCommand stream per ADR-007. A replay MAY retain action frames as an input oracle, but cannot apply them as a second mutable source.

## Camera, aiming и targeting

`CameraIntent` contains presentation mode, focus subject/point, framing constraints, zoom/orbit deltas, collision policy ID and blend request. Camera controller reads immutable PresentationSnapshot and cooked camera profiles; it never writes character/physics transforms.

An action requiring gameplay targeting creates a `TargetingIntent` with source tick, player/ability ID, normalized screen/aim coordinates, declared query kind and referenced camera profile/revision. Runtime reconstructs the engine-owned query from authoritative tick snapshot and validates the result before command commit. GPU depth, current interpolated frame, camera shake and post-processing cannot choose authoritative hit/interaction target.

Camera collision/occlusion is presentation. Lock-on target or interaction selection becomes durable only through accepted command/event. CapturePlan may override camera presentation for evidence but cannot change `TargetingIntent` or gameplay hash.

## UI semantic boundary

`UiSemanticSnapshot` is immutable and contains stable screen/panel/control IDs, semantic roles, enabled/visible/value/selection state, text resource IDs with arguments, focus graph and related command affordances. It excludes mutable aggregate references, widget/backend objects and localized strings as identity.

UI flow:

`PresentationSnapshot/immutable query → UiSemanticSnapshot → rendered widgets → PlayerActionFrame → command candidate → WorldCommand validation → DomainEvent → new projection`.

Pause/menu policy MUST declare whether simulation continues, pauses at fixed boundary or runs a separate non-authoritative menu timeline. A UI animation or async asset load cannot delay authoritative command commit. Failed optional panel is removed with diagnostic; required project UI schema incompatibility fails before world activation.

Luau/Wasm `ui.panel.register` creates a capability-scoped declarative panel over semantic controls. Extension code reads allowed projections and publishes allowed actions/proposals only. Arbitrary HTML/native views, filesystem/network access and direct domain mutation are not implied.

## Localization

- Public and durable content references use stable text/voice/subtitle `AssetId` or namespaced text ID, never rendered text.
- Locale is a valid BCP-47 tag with deterministic fallback chain declared by project content.
- Formatting receives typed arguments, explicit number/date/unit rules and simulation calendar values; host locale/wall clock cannot affect gameplay or replay.
- Missing string/format/unsupported glyph emits stable diagnostic and uses declared source/default locale or readable placeholder.
- Localized dialogue presentation cannot alter Dialogue node, choice ID, commitment or command payload.
- Search/sort that affects gameplay uses declared locale-independent semantic key; presentation-only lists MAY use locale collation but cannot determine command ordering.

## Accessibility и preferences

`PlayerPreferenceProfile` is versioned local data for bindings, input sensitivity/deadzones, hold/toggle alternatives, subtitle/caption settings, text/UI scale, reduced motion, camera shake, color/contrast aids and presentation mix. Each field declares supported range and `PresentationOnly` class from SPEC-17 future contract.

Accessibility alternative MUST invoke the same action ID and validation path as default input. Preference corruption falls back to safe defaults without touching save/domain state. Settings import/export validates schema/bounds and never loads executable content.

## Failure semantics

| Code / failure | Required outcome |
|---|---|
| `INPUT_EVENT_INVALID` | Drop/reject malformed event; no action/command; device remains isolated |
| `INPUT_SEQUENCE_NON_MONOTONIC` | Reject affected frame and diagnose source; no reordered best-effort action |
| `INPUT_CONTEXT_STALE` | Reject frame built from stale revision; next boundary uses current context |
| `INPUT_BINDING_CONFLICT` | Apply declared deterministic policy or reject map before activation |
| `INPUT_DEVICE_LOST` | Cancel active actions once, select declared alternative device/UI path, no duplicate command |
| `UI_SCHEMA_INCOMPATIBLE` | Optional panel disabled; required UI rejects project pre-world |
| `CAMERA_TARGET_STALE` | Reject targeting candidate; presentation may continue without commit |
| `LOCALIZATION_RESOURCE_MISSING` | Declared locale fallback/placeholder; command/content IDs unchanged |
| `PLAYER_PREFERENCE_INVALID` | Quarantine profile and use bounded defaults; save/gameplay untouched |
| `NONDETERMINISTIC_RESULT` | Gate fails with first divergent input/action/command; no retry-to-green |

## Proposed gates

| Gate | Owner | Reproducible command/scenario | Pass threshold | Required evidence | Fallback |
|---|---|---|---|---|---|
| `INPUT-P1` | Player Experience + Rendering | `next gate INPUT-P1 --scenario player-input-parity --events 10000 --targets windows-x86_64,linux-x86_64` | 10,000 keyboard/mouse/gamepad neutral fixtures produce exact PlayerActionFrame bytes/order; 100% invalid/conflicting/stale cases classified; no OS/vendor type in contracts | event/action traces, map/context hashes, public API scan, diagnostics | native adapter fix; keyboard-only accessible baseline for optional device |
| `UI-P1` | Player Experience + RPG Framework | `next gate UI-P1 --scenario semantic-ui-command-loop --runs 1000` | 1,000 inventory/dialogue/quest/menu flows use immutable projections and production commands; 0 direct mutation/lost/duplicate commits; panel faults isolated | semantic snapshots, action/command/event traces, capability audit | disable optional panel; required schema blocks startup |
| `CAMERA-P1` | Player Experience + Runtime | `next gate CAMERA-P1 --scenario camera-targeting-replay --runs 1000` | camera FPS/interpolation/shake/capture variants produce exact targeting command outcomes and gameplay hashes; stale targets 100% rejected; required media complete | intent/query/command traces, replay hashes, CapturePlans, comparison media | stable authored camera profile; reject stale targeting |
| `ACCESS-P1` | Player Experience + Verification & Evidence | `next gate ACCESS-P1 --scenario locale-accessibility-matrix --profiles default,remapped,reduced-motion,subtitles --locales source,missing,pseudo` | all profiles complete mandatory actions; exact accepted commands/gameplay hashes; 100% corrupt/missing preference/locale fixtures use declared fallback; semantic snapshot closure complete | profile/locale manifests, action/replay hashes, UI captures, diagnostics | safe defaults + source locale + readable placeholder |

## Foundation requirement aliases

| Alias | Primary owner | Future acceptance contract |
|---|---|---|
| `FND-PLAYER-R1` | Player Experience | Device-independent action maps deterministically produce PlayerActionFrame values |
| `FND-PLAYER-R2` | Runtime Team | Gameplay targeting/action outcomes pass common production command validation |
| `FND-PLAYER-R3` | Player Experience | UI/camera/localized/accessibility state remains presentation-only and replay-safe |
| `FND-PLAYER-R4` | Player Experience | Plugin panels and fallback paths use the same semantic UI/action contracts |
| `FND-PLAYER-F1` | Player Experience | Invalid/stale/conflicting input produces no partial or duplicate gameplay command |
| `FND-PLAYER-F2` | Player Experience | Missing device/UI/locale/preference uses bounded accessible fallback without state mutation |

## Promotion contract

Promotion occurs only with SPEC-17/19/20 and ADR-018…021 in one reviewed transaction. It updates SPEC-00/01/02/04/07/09/11/12/15, glossary and traceability; maps INPUT/UI/CAMERA/ACCESS gates into VS-02/07/11/15; and preserves exactly fifteen VS gates. Observable UI/camera changes remain `HumanReviewRequired` under SPEC-15.

Foundation alias allocation follows SPEC-17: REQ-087…102 and FAIL-031…038 after accepted SPEC-16, or REQ-079…094 and FAIL-025…032 after its formal rejection/withdrawal. Pending SPEC-16 blocks promotion but not review. No external technology row is added.
