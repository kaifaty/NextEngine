# SPEC-18: Player interaction, UI, camera, localization и accessibility

| Поле | Значение |
|---|---|
| ID | SPEC-18 |
| Статус | Accepted |
| Версия | 2.9 |
| Последняя проверка | 2026-08-17 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-29](29-platform-host-and-application-session.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-019](adr/019-canonical-player-actions-and-presentation-authority.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-034](adr/034-player-targeting-replay-v5-and-mapping-provenance.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md), [ADR-074](adr/074-systemic-strategic-agent-owner-vertical.md) |
| Дополнительные зависимости V2.9 | [SPEC-36](36-functional-tissue-condition-and-injury.md), [ADR-075](adr/075-product-grounded-functional-anatomy-and-character-embodiment.md) |
| Заменяет | SPEC-18 2.8; retains the current replay envelope and adds the future qualitative body-status UI boundary without changing current UI schemas |

## История принятия

SPEC-18 принят в architecture packet 1.7 как Player Experience foundation
contract. Версия 2.0 сохраняет public schemas, authority boundaries и failure
paths, но заменяет admission-oriented verification обычными product checks по
ADR-030.

## Назначение и invariants

SPEC-18 определяет границу между raw platform input, canonical player actions, gameplay commands и presentation-only UI/camera state.

- Platform event или UI callback MUST NOT мутировать gameplay state напрямую.
- Action semantics MUST быть device-independent; bindings MAY различаться без изменения action IDs и domain outcomes.
- UI, camera, localized text и accessibility preferences MUST NOT быть gameplay authority.
- Aiming/targeting, dialogue choice, inventory action и interaction MUST входить в simulation как versioned action-derived command candidate через common validation.
- `game` и headless scenario MUST подавать один canonical `PlayerActionFrame` через общий current/next ingress contract; frame не выбирает authoritative tick.
- Missing locale/device/optional panel MUST иметь bounded accessible fallback и не блокировать offline correctness.
- Public action/UI/camera/localization/accessibility schemas MUST содержать только engine-owned IDs, fixed-width values и immutable data; OS, vendor, ECS и backend types запрещены.
- Этот contract не выбирает input library, widget toolkit, UI framework или device backend.

## Technical authority boundary

| State | Authoritative representation | Не является source |
|---|---|---|
| Raw normalized controls/focus/device lifecycle | SPEC-29 `PlatformEventV1`/`NormalizedControlEventV1` from private platform adapter | native event objects after normalization |
| Action maps, context stack, binding resolution | `ActionMapManifest` + session input service | UI widget callbacks, device driver profile |
| Canonical action input for one ingress sample | Immutable `PlayerActionFrame` | physical device identity, current renderer frame or wall clock |
| Current/next tick assignment | Runtime `IngressAssignmentV1` + closed ingress batch | frame timestamp, renderer cadence, callback completion order |
| Gameplay result of an action | Owning domain state after accepted `WorldCommand` | UI animation, camera ray, input binding |
| UI semantic state | immutable projection of queries/snapshots | widget tree, GPU resources |
| Camera state | presentation controller | character pose, projectile authority |
| Locale/accessibility preferences | Local `PlayerPreferenceProfile` | content IDs, quest/dialogue state |
| Localized resources | Cooked content registry by stable text ID + locale | rendered glyph string as identifier |

Public action/UI/camera/localization/accessibility schemas остаются
engine-owned. Private platform/visual adapters нормализуют input and render
presentation, while Runtime/RPG/Mechanics keep ingress assignment, validation
and committed outcomes. Extension panel cannot acquire more authority than its
package capabilities.

## Public contracts

Все public contracts этого раздела находятся за engine-owned facade. Native event/widget/window/device objects, renderer resources, ECS storage/components и vendor/backend handles не входят в schema, save, replay, script/plugin API или process protocol. Конкретные input/UI implementations остаются private replaceable adapters.

### ActionMapManifest

`ActionMapManifest` содержит schema/version, stable action IDs, value kind, allowed phases, binding slots, supported device classes, context predicates, conflict policy, accessibility metadata and content hash.

Action IDs are locale- and device-independent namespaced values. Binding stores engine-owned semantic controls, never native handles, event objects or device scancodes. Private device adapter maps native input through SPEC-29 `PlatformEventV1` to bounded `NormalizedControlEventV1` values containing engine-owned device class, opaque session nonce, semantic control ID, phase/value, normalized platform sample and monotonic source sequence. Platform sample MAY remain diagnostic/order metadata, but cannot enter canonical action bytes, cutoff assignment or conflict resolution.

### InputContext и resolution

`InputContext` is a versioned stack entry with context ID, revision, priority, capture/passthrough policy and allowed action set. Context transitions occur only at declared action-frame boundaries. A frame captures one exact action-map revision and one exact ordered context-stack revision; late transition applies to the next frame. Equal semantic controls, map/context revisions and frame boundary MUST produce the same action values independent of physical device or adapter.

Resolution order:

1. validate device/control/value bounds and monotonic sequence;
2. snapshot action-map and context-stack revisions for the frame;
3. resolve exact binding and conflict policy;
4. quantize analog values by declared integer/fixed-point profile;
5. remove exact semantic duplicates according to the declared conflict policy and order remaining values by higher context priority first, then `(action ID, phase tag, canonical value bytes, semantic occurrence ordinal)` ascending in canonical byte order;
6. publish immutable canonical `PlayerActionFrame` bytes to the production input gateway as an `InputSampleV1` payload.

Device completion order, UI traversal order, native callback order and renderer frame cannot choose winner. Rebinding publishes a new map revision only at a frame boundary; an in-flight frame uses one revision. An unresolved collision rejects the frame rather than selecting a device by arrival order.

### PlayerActionFrame

`PlayerActionFrame` contains schema version, player/controller `PersistentId` when applicable, logical frame sequence, action-map content hash/revision, context-stack hash/revision and bounded canonical ordered actions. Each action contains namespaced action ID, phase (`Started`, `Performed`, `Completed`, `Cancelled`), quantized scalar/vector value and semantic occurrence ordinal assigned after canonical duplicate/conflict resolution.

Physical device/source identity, native sample reference, wall-clock timestamp, target tick, camera transform and backend object MUST NOT appear in canonical frame bytes. The surrounding `InputSampleV1` uses the logical player/controller source identity and monotonic source sequence. The frame MUST fit the admitted runtime input-payload bound.

Interactive input service and headless scenario producer enqueue the same `PlayerActionFrame` schema through the same gateway. The ADR-022 current/next close barrier creates persisted `IngressAssignmentV1`; enqueue before the barrier belongs to current tick, while enqueue linearized in or after the barrier belongs to next tick. Wall time, renderer cadence and worker completion do not decide the side. The production mapper derives command target tick only from the persisted assignment, never from a frame field or scenario override.

The frame is production input, not committed gameplay. Input gateway maps assigned actions to declared command candidates; common capability/schema/domain/physics validation still decides outcome. Current replay stores the `ClosedIngressBatchV1` frame and assignment together with the closed command-admission batches, feeds them through the same mapper/validator and compares exact commands, receipts, events and roots. `InputMappingReceiptV2` binds ordered per-action mapping results to contiguous canonical command references, so one composite frame can prove both movement and interaction provenance without a single-command shortcut. Neither stored action nor expected command is a direct mutable backdoor.

## Camera, aiming и targeting

`CameraIntent` contains presentation mode, focus subject/point, framing constraints, zoom/orbit deltas, collision policy ID and blend request. Camera controller reads immutable [SPEC-30](30-presentation-extraction-and-render-content.md) `PresentationSnapshotV3` and cooked camera profiles; it never writes character/physics transforms.

An assigned action requiring gameplay targeting creates a `TargetingIntent` with `IngressAssignmentV1` tick, player/ability/action IDs, quantized aim values, declared query kind, `TargetingQueryProfile` ID/hash and optional proposed target `PersistentId`. It MUST NOT contain a camera transform/profile, rendered depth, interpolated pose, display extent or post-processing result.

Runtime reconstructs `AuthoritativeTargetingQueryV1` only from the authoritative snapshot at the assigned tick, authoritative actor pose, ability/mechanic definition and hash-bound gameplay `TargetingQueryProfile`. The common validator resolves visibility/range/collision/affordance and accepts or rejects the resulting command candidate. Presentation camera consumes the immutable targeting projection/result to place reticle, lock-on and feedback; the data flow never reverses.

`ReplayManifestV10` stores the ordered targeting intents, reconstructed queries,
closed physics query batch/results and their compare hashes. Replay повторно
выполняет production mapper/query path и отклоняет первое отличие receipt,
intent, query, result, command, event или root. Retired Replay V9 and earlier are rejected
from its outer version and is not evidence for this targeting closure.

Camera collision/occlusion, smoothing, shake, FOV, aspect and framing are
presentation. Lock-on target or interaction selection becomes durable only
through accepted command/event. An optional developer capture may override
camera presentation for screenshots or video, but cannot change
`TargetingIntent`, `AuthoritativeTargetingQueryV1`, accepted command or
gameplay hash.

## UI semantic boundary

`UiSemanticSnapshot` is immutable and contains stable screen/panel/control IDs, semantic roles, enabled/visible/value/selection state, text resource IDs with typed arguments, focus graph and related action/command affordances. It excludes mutable aggregate references, widget/backend objects and localized strings as identity.

UI flow:

`PresentationSnapshotV3/immutable query → UiSemanticSnapshot → rendered widgets → PlayerActionFrame → command candidate → WorldCommand validation → DomainEvent → new projection`.

Pause/menu policy in the admitted project composition MUST declare whether simulation continues, requests SPEC-29 `Suspended` at a fixed lifecycle boundary or runs a separate non-authoritative menu timeline. UI emits the declared action/request; a widget or platform callback cannot pause/resume/close authoritative state directly. UI animation or async asset load cannot delay authoritative command commit. Failed optional panel is removed with diagnostic; required project UI schema incompatibility fails before world activation.

Luau/Wasm `ui.panel.register` creates a capability-scoped declarative panel over semantic controls under ADR-014. Extension code reads allowed projections and publishes allowed actions/proposals only. Registration grants no arbitrary native view, filesystem/network access or direct domain mutation. Trap, denial or deterministic budget overrun discards uncommitted panel proposals and disables only the optional panel; wall-clock completion never creates gameplay authority.

### Future qualitative body-status UI

The SPEC-36 lower-limb consumer adds one immutable qualitative body projection;
exact schema remains Proposed until that consumer exists. The player-facing
panel contains only:

- stable body-region identity and localized display text ID;
- qualitative functional severity such as normal, reduced or unavailable;
- structural/attachment category such as intact, stable fracture, retained
  unstable fracture or detached;
- the simplified systemic band;
- current treatment stage and the next valid stabilization/repair/
  rehabilitation affordance when known to the player.

The default player UI does not expose exact capacity, reducer coefficients,
hidden vascular state, RNG values or a medically certain diagnosis. Those
belong to capability-scoped developer diagnostics. The UI reads the same
committed condition revision as SPEC-37 presentation and cannot infer injury
from mesh visibility, blood decals, camera view or gait.

Clothing, armor, the `Reduced` visual profile or camera occlusion may hide a
wound without hiding the already known qualitative status. Conversely, the UI
must not reveal an unobserved NPC condition through a hidden authoritative
world read; NPC knowledge follows existing epistemic/perception boundaries.

Treatment buttons publish ordinary semantic actions/proposals through the same
Mechanics/RPG validation as any other source. They cannot set a stage, restore
capacity or skip rehabilitation directly. Missing body-panel assets fall back
to localized text/list presentation; gameplay and body condition remain intact.

## Localization

- Public and durable content references use stable text/voice/subtitle `AssetId` or namespaced text ID, never rendered text.
- Locale is a valid BCP-47 tag with deterministic fallback chain declared by project content.
- Formatting receives typed arguments, explicit number/date/unit rules and simulation calendar values; host locale/wall clock cannot affect gameplay or replay.
- Missing string/format/unsupported glyph emits stable diagnostic and uses declared source/default locale or readable placeholder.
- Localized dialogue presentation cannot alter Dialogue node, choice ID, commitment or command payload.
- Search/sort that affects gameplay uses declared locale-independent semantic key; presentation-only lists MAY use locale collation but cannot determine command ordering.

## Accessibility и preferences

`PlayerPreferenceProfile` is versioned local data for bindings, input sensitivity/deadzones, hold/toggle alternatives, subtitle/caption settings, text/UI scale, reduced motion, camera shake, color/contrast aids and presentation mix. Each field declares supported range and the SPEC-17 `PresentationOnly` configuration class. The profile itself is not save/domain state or authoritative configuration; only its resulting canonical action frame is production input.

Accessibility alternative MUST invoke the same action ID and validation path as default input. Preference corruption falls back to safe defaults without touching save/domain state. Settings import/export validates schema/bounds and never loads executable content.

## Scheduling и budgets

- Action-frame enqueue uses the ADR-022 current/next ingress cutoff exactly once; Player Experience cannot add a private wall-time cutoff or extra command-admission barrier.
- Action-to-command mapping and targeting validation executed inside the gameplay tick are charged to the existing ADR-016 `core-command-rpg` exclusive span. This specification creates no additional budget category.
- Required assigned actions MUST NOT be silently dropped, reordered or routed differently to meet a wall-time budget. Declared deterministic bounds reject an oversized/invalid frame before command mutation.
- UI projection, camera smoothing and localized rendering remain presentation work. Their delay or failure cannot change ingress assignment, command order or authoritative state.

## Failure semantics

| Code / failure | Required outcome |
|---|---|
| `INPUT_EVENT_INVALID` | Drop/reject malformed event; no action/command; device remains isolated |
| `INPUT_SEQUENCE_NON_MONOTONIC` | Reject affected frame and diagnose source; no reordered best-effort action |
| `INPUT_CONTEXT_STALE` | Reject frame built from stale revision; next boundary uses current context |
| `INPUT_BINDING_CONFLICT` | Apply declared deterministic policy or reject map before activation |
| `INPUT_DEVICE_LOST` | Cancel active actions once, select declared alternative device/UI path, no duplicate command |
| `INGRESS_ASSIGNMENT_CORRUPT` | Fail closed before action mapping/command mutation; preserve previous valid replay/save generation |
| `UI_SCHEMA_INCOMPATIBLE` | Optional panel disabled; required UI rejects project pre-world |
| `CAMERA_TARGET_STALE` | Reject targeting candidate; presentation may continue without commit |
| `LOCALIZATION_RESOURCE_MISSING` | Declared locale fallback/placeholder; command/content IDs unchanged |
| `PLAYER_PREFERENCE_INVALID` | Quarantine profile and use bounded defaults; save/gameplay untouched |
| `NONDETERMINISTIC_RESULT` | Report the first divergent input/action/command and fail the affected product check; no retry-to-green |

## Product checks

| ID | Scenario | Expected behavior | Fallback |
|---|---|---|---|
| `INPUT-P1` | Feed equivalent semantic controls through interactive and headless producers, including before/at/after ingress cutoff and invalid/conflicting/stale inputs. | Canonical `PlayerActionFrame`, closed ingress assignment, action and command order are exact for equivalent inputs; no OS/vendor/ECS/backend type enters public contracts. | Reject the affected frame or disable the failing optional adapter while keeping a declared accessible semantic-action path. |
| `UI-P1` | Exercise inventory, dialogue, quest and menu flows through semantic projections and production actions/commands, including optional-panel faults. | No direct gameplay mutation, lost/duplicate commit or capability escape occurs; widget timing cannot select an authoritative outcome. | Disable the optional panel; reject an incompatible required UI schema before world activation. |
| `CAMERA-P1` | Replay targeting while varying presentation FPS, interpolation, shake, FOV and aspect; optional developer captures may be enabled. | `TargetingIntent`, authoritative query, command outcome and gameplay hash remain independent of camera/render state; stale targets are rejected. | Use a stable authored presentation profile and never accept rendered targeting data. |
| `ACCESS-P1` | Run default, remapped, reduced-motion and subtitle profiles across source, missing and pseudo locales. | Required semantic actions use the same IDs/validation and equivalent inputs produce the same accepted commands/gameplay hashes; invalid resources use bounded defaults. | Use safe preferences, the project source locale and a readable placeholder. |
| `BODY-UI-P1` (future) | Identify and treat every lower-limb matrix state with wounds visible, armored/covered and under all severity profiles. | Region/function/attachment/systemic/treatment stage remains qualitatively readable; UI actions use the common command path; UI/visibility permutations change zero authoritative roots. | Localized text/list body status and ordinary semantic treatment actions. |

## Requirements

| ID | Нормативное требование |
|---|---|
| `REQ-091` | Engine-owned device-independent action maps MUST deterministically produce canonical `PlayerActionFrame` bytes; interactive and headless producers MUST use the same schema, current/next cutoff, persisted assignment and production mapper. |
| `REQ-092` | Assigned player actions and gameplay targeting MUST derive authoritative tick/query inputs from `IngressAssignmentV1` and authoritative snapshots, then pass common production command validation; camera/GPU/widget state MUST NOT select the outcome. |
| `REQ-093` | Semantic UI, camera, localized resources and accessibility preferences MUST remain immutable presentation/local state, replay-safe and unable to change authoritative ordering or outcome for the same assigned action input. |
| `REQ-094` | First-party and capability-scoped extension panels, alternative controls and missing-resource fallbacks MUST use the same semantic UI/action contracts and MUST NOT gain direct domain mutation authority. |

## Failure paths

| ID | Trigger | Required result |
|---|---|---|
| `FAIL-033` | Invalid, non-monotonic, stale, conflicting, oversized or cutoff-corrupt control/action/targeting input | Reject the affected event, frame or candidate before partial/duplicate gameplay command; preserve exact assignment/diagnostic and never reorder by arrival or retry to green. |
| `FAIL-034` | Missing/lost optional device, failed optional UI panel, missing locale/glyph or invalid preference profile | Use only the declared bounded accessible semantic-action/source-locale/default-profile fallback; required schema failure remains pre-world; save/domain state and committed commands remain untouched. |

## Current quest journal and future narrative intent

The current quest UI reads only immutable SPEC-19 `Quest` aggregate
projections. A journal row binds the quest `PersistentId`, exact committed
revision/state ID and a stable text-catalog display-name ID. Localized text,
widget order and whether the originating NPC is loaded never become quest
identity or authority.

Reference-alpha renders its current states as `Frontier Relay - Available`,
`Frontier Relay - Active` and `Frontier Relay - Completed`. Those labels are
derived from the committed `available/active/completed` state ID after the
ordinary `TransitionQuest`; the journal cannot create, accept, complete or
hide a quest through presentation state. This is the player-visible projection
used by the current SPEC-20/ADR-052 R4a ProductCheck; the authored keeper
routine permits acceptance in `Duty` and exposes unavailable/unchanged state in
`Rest` without granting UI authority.

Autonomous opportunities, disclosure channels, offers/declines, deadlines,
divine offers, covenants, judgments and their commands/events remain Proposed
intent in SPEC-31 after ADR-046. `DiscloseQuestOpportunity`, `AcceptQuest`,
`DeclineQuestOffer`, `DivineOfferV1`, `DivineStandingProjectionV1`,
`DivineOfferTransitioned` and `DivineCovenantChanged` are not current APIs or
ProductCheck obligations. The inert serialized `DivineStandingPayloadV1`
listed by SPEC-19 does not create those presentation features.

## Consequences

- `REQ-091`…`REQ-094` and `FAIL-033`…`FAIL-034` remain the technical
  contract of this specification.
- `INPUT-P1`, `UI-P1`, `CAMERA-P1` and `ACCESS-P1` are ordinary executable
  product checks.
- Screenshots and camera/UI captures are optional developer aids.
- No external technology, input backend, widget toolkit or UI framework is accepted by this specification.
