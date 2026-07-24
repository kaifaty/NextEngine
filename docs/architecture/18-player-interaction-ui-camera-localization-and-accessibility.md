# SPEC-18: Player interaction, UI, camera, localization и accessibility

| Поле | Значение |
|---|---|
| ID | SPEC-18 |
| Статус | Accepted |
| Версия | 1.1 |
| Владелец | Repository Owner |
| Требуемые согласующие | Architecture Working Group, Rendering Team, Runtime Team, RPG Framework Team, Developer Experience Team, Verification & Evidence Team, Security & Governance Team |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-019](adr/019-canonical-player-actions-and-presentation-authority.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |

## История принятия

SPEC-18 принят в architecture packet 1.7 как Player Experience foundation contract. Принятие фиксирует public schemas, ownership, failure paths и verification gates, но не объявляет runtime implementation, gate `PASS`, human approval или `vertical-v1` conformance.

## Назначение и invariants

SPEC-18 определяет границу между raw platform input, canonical player actions, gameplay commands и presentation-only UI/camera state.

- Platform event или UI callback MUST NOT мутировать gameplay state напрямую.
- Action semantics MUST быть device-independent; bindings MAY различаться без изменения action IDs и domain outcomes.
- UI, camera, localized text и accessibility preferences MUST NOT быть gameplay authority.
- Aiming/targeting, dialogue choice, inventory action и interaction MUST входить в simulation как versioned action-derived command candidate через common validation.
- `game` и headless scenario MUST подавать один canonical `PlayerActionFrame` через общий current/next ingress contract; frame не выбирает authoritative tick.
- Observable UI/camera changes MUST проходить SPEC-15 automatic evidence and authorized human review.
- Missing locale/device/optional panel MUST иметь bounded accessible fallback и не блокировать offline correctness.
- Public action/UI/camera/localization/accessibility schemas MUST содержать только engine-owned IDs, fixed-width values и immutable data; OS, vendor, ECS и backend types запрещены.
- Этот contract не выбирает input library, widget toolkit, UI framework или device backend.

## Source of truth и ownership

| State | Единственный owner/source of truth | Не является source |
|---|---|---|
| Raw normalized controls/focus/device lifecycle | SPEC-29 `PlatformEventV1`/`NormalizedControlEventV1` from private platform adapter | native event objects after normalization |
| Action maps, context stack, binding resolution | Player Experience `ActionMapManifest` + session input service | UI widget callbacks, device driver profile |
| Canonical action input for one ingress sample | Immutable `PlayerActionFrame` | physical device identity, current renderer frame or wall clock |
| Current/next tick assignment | Runtime `IngressAssignmentV1` + closed ingress batch | frame timestamp, renderer cadence, callback completion order |
| Gameplay result of an action | Owning domain after accepted WorldCommand | UI animation, camera ray, input binding |
| UI semantic state | Player Experience projection of immutable queries/snapshots | widget tree, GPU resources |
| Camera state | Player Experience presentation controller | character pose, projectile authority |
| Locale/accessibility preferences | Local `PlayerPreferenceProfile` | content IDs, quest/dialogue state |
| Localized resources | Cooked content registry by stable text ID + locale | rendered glyph string as identifier |

Player Experience Team владеет public action/UI/camera/localization/accessibility schemas. Rendering Team владеет private raw platform adapter и visual backend. Runtime/RPG/Mechanics own ingress assignment, validation and committed outcomes. Extension panel cannot acquire more authority than its package capabilities.

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

The frame is production input, not committed gameplay. Input gateway maps assigned actions to declared command candidates; common capability/schema/domain/physics validation still decides outcome. Replay stores the `ClosedIngressBatchV1` frame and assignment together with the closed command-admission batches, feeds them through the same mapper/validator and compares exact commands, receipts, events and roots. Neither stored action nor expected command is a direct mutable backdoor.

## Camera, aiming и targeting

`CameraIntent` contains presentation mode, focus subject/point, framing constraints, zoom/orbit deltas, collision policy ID and blend request. Camera controller reads immutable [SPEC-30](30-presentation-extraction-and-render-content.md) `PresentationSnapshotV2` and cooked camera profiles; it never writes character/physics transforms.

An assigned action requiring gameplay targeting creates a `TargetingIntent` with `IngressAssignmentV1` tick, player/ability/action IDs, quantized aim values, declared query kind, `TargetingQueryProfile` ID/hash and optional proposed target `PersistentId`. It MUST NOT contain a camera transform/profile, rendered depth, interpolated pose, display extent or post-processing result.

Runtime reconstructs `AuthoritativeTargetingQueryV1` only from the authoritative snapshot at the assigned tick, authoritative actor pose, ability/mechanic definition and hash-bound gameplay `TargetingQueryProfile`. The common validator resolves visibility/range/collision/affordance and accepts or rejects the resulting command candidate. Presentation camera consumes the immutable targeting projection/result to place reticle, lock-on and feedback; the data flow never reverses.

Camera collision/occlusion, smoothing, shake, FOV, aspect and framing are presentation. Lock-on target or interaction selection becomes durable only through accepted command/event. CapturePlan may override camera presentation for evidence but cannot change `TargetingIntent`, `AuthoritativeTargetingQueryV1`, accepted command or gameplay hash.

## UI semantic boundary

`UiSemanticSnapshot` is immutable and contains stable screen/panel/control IDs, semantic roles, enabled/visible/value/selection state, text resource IDs with typed arguments, focus graph and related action/command affordances. It excludes mutable aggregate references, widget/backend objects and localized strings as identity.

UI flow:

`PresentationSnapshotV2/immutable query → UiSemanticSnapshot → rendered widgets → PlayerActionFrame → command candidate → WorldCommand validation → DomainEvent → new projection`.

Pause/menu policy in the admitted project composition MUST declare whether simulation continues, requests SPEC-29 `Suspended` at a fixed lifecycle boundary or runs a separate non-authoritative menu timeline. UI emits the declared action/request; a widget or platform callback cannot pause/resume/close authoritative state directly. UI animation or async asset load cannot delay authoritative command commit. Failed optional panel is removed with diagnostic; required project UI schema incompatibility fails before world activation.

Luau/Wasm `ui.panel.register` creates a capability-scoped declarative panel over semantic controls under ADR-014. Extension code reads allowed projections and publishes allowed actions/proposals only. Registration grants no arbitrary native view, filesystem/network access or direct domain mutation. Trap, denial or deterministic budget overrun discards uncommitted panel proposals and disables only the optional panel; wall-clock completion never creates gameplay authority.

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
- Action-to-command mapping and targeting validation executed inside the gameplay tick are charged to the existing ADR-016 `core-command-rpg` exclusive span. This specification creates no new budget owner row.
- Required assigned actions MUST NOT be silently dropped, reordered or routed differently to meet a wall-time budget. Declared deterministic bounds reject an oversized/invalid frame before command mutation.
- UI projection, camera smoothing and localized rendering remain presentation work. Their delay or failure cannot change ingress assignment, command order, authoritative state or gate result.

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
| `NONDETERMINISTIC_RESULT` | Gate fails with first divergent input/action/command; no retry-to-green |

## Verification gates

| Gate | Primary owner | Contributors | Reproducible command/scenario | Pass threshold | Required evidence | Fallback | VS / profile closure |
|---|---|---|---|---|---|---|---|
| `INPUT-P1` | Player Experience Team | Rendering Team, Runtime Team | `next gate INPUT-P1 --scenario player-input-parity --events 10000 --roots game,headless,capture-worker --targets windows-x86_64,linux-x86_64` | 10,000 equivalent semantic-control fixtures produce exact canonical `PlayerActionFrame`, `ClosedIngressBatchV1`, assignment, action and command order across roots/targets; before/at/after cutoff permutations are exact; 100% invalid/conflicting/stale cases classified; 0 OS/vendor/ECS/backend type in public contracts | event/frame/assignment/action/command traces, map/context/profile hashes, public API scan, diagnostics | fix or disable failing optional adapter; retain project-declared accessible semantic-action path | VS-02, VS-11, VS-15; player-input profile |
| `UI-P1` | Player Experience Team | RPG Framework Team, Security & Governance Team | `next gate UI-P1 --scenario semantic-ui-command-loop --runs 1000` | 1,000 inventory/dialogue/quest/menu flows use immutable semantic projections and production actions/commands; 0 direct mutation, lost/duplicate commit or capability escape; all injected panel faults isolated | semantic snapshots, action/command/receipt/event traces, capability/budget audit, UI captures | disable optional panel; incompatible required schema blocks startup before world activation | VS-02, VS-07, VS-15; semantic-ui profile |
| `CAMERA-P1` | Player Experience Team | Runtime Team, Rendering Team | `next gate CAMERA-P1 --scenario camera-targeting-replay --runs 1000` | Camera FPS/interpolation/shake/FOV/aspect/capture variants produce exact `TargetingIntent`, `AuthoritativeTargetingQueryV1`, accepted/rejected command outcomes and gameplay hashes; stale targets 100% rejected; required media complete | intent/query/command/receipt traces, replay hashes, CapturePlans, comparison media | use stable authored presentation profile; reject stale targeting; never accept rendered targeting data | VS-07, VS-11, VS-15; camera-targeting profile |
| `ACCESS-P1` | Player Experience Team | Verification & Evidence Team, RPG Framework Team | `next gate ACCESS-P1 --scenario locale-accessibility-matrix --profiles default,remapped,reduced-motion,subtitles --locales source,missing,pseudo` | Every profile completes mandatory semantic actions through the same IDs/validation and matches its exact expected assigned frame; equivalent scenario actions produce identical accepted commands/gameplay hashes across profiles; 100% corrupt/missing preference/locale fixtures use declared fallback; semantic snapshot closure complete | profile/locale manifests, frame/assignment/replay hashes, semantic snapshots, UI captures, diagnostics | safe bounded preferences, project source locale and readable placeholder | VS-02, VS-15; accessibility-localization profile |

## Requirements

| ID | Нормативное требование | Primary owner | Contributors | Blocking gates |
|---|---|---|---|---|
| `REQ-091` | Engine-owned device-independent action maps MUST deterministically produce canonical `PlayerActionFrame` bytes; interactive and headless producers MUST use the same schema, current/next cutoff, persisted assignment and production mapper. | Player Experience Team | Rendering Team, Runtime Team | INPUT-P1, CLOCK-P1 |
| `REQ-092` | Assigned player actions and gameplay targeting MUST derive authoritative tick/query inputs from `IngressAssignmentV1` and authoritative snapshots, then pass common production command validation; camera/GPU/widget state MUST NOT select the outcome. | Runtime Team | Player Experience Team, RPG Framework Team | INPUT-P1, CAMERA-P1 |
| `REQ-093` | Semantic UI, camera, localized resources and accessibility preferences MUST remain immutable presentation/local state, replay-safe and unable to change authoritative ordering or outcome for the same assigned action input. | Player Experience Team | Rendering Team, Verification & Evidence Team | UI-P1, CAMERA-P1, ACCESS-P1 |
| `REQ-094` | First-party and capability-scoped extension panels, alternative controls and missing-resource fallbacks MUST use the same semantic UI/action contracts and MUST NOT gain direct domain mutation authority. | Player Experience Team | RPG Framework Team, Security & Governance Team | UI-P1, ACCESS-P1 |

## Failure paths

| ID | Trigger | Required result | Primary owner | Contributors | Blocking gates |
|---|---|---|---|---|---|
| `FAIL-033` | Invalid, non-monotonic, stale, conflicting, oversized or cutoff-corrupt control/action/targeting input | Reject the affected event, frame or candidate before partial/duplicate gameplay command; preserve exact assignment/diagnostic and never reorder by arrival or retry to green. | Player Experience Team | Runtime Team, Rendering Team | INPUT-P1, CAMERA-P1, CLOCK-P1 |
| `FAIL-034` | Missing/lost optional device, failed optional UI panel, missing locale/glyph or invalid preference profile | Use only the declared bounded accessible semantic-action/source-locale/default-profile fallback; required schema failure remains pre-world; save/domain state and committed commands remain untouched. | Player Experience Team | RPG Framework Team, Verification & Evidence Team | INPUT-P1, UI-P1, ACCESS-P1 |

## Consequences и conformance boundary

- `REQ-091`…`REQ-094` and `FAIL-033`…`FAIL-034` are Accepted normative rows owned by this specification.
- `INPUT-P1`, `UI-P1`, `CAMERA-P1` and `ACCESS-P1` close through existing VS-02/VS-07/VS-11/VS-15 descriptors; this contract creates no additional vertical gate.
- Observable `ui` and `camera` changes remain `HumanReviewRequired` under SPEC-15/ADR-023 after all resolved automatic gates pass.
- No external technology, input backend, widget toolkit or UI framework is accepted by this specification.
