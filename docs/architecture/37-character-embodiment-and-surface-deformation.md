# SPEC-37: Character embodiment, surface deformation and injury presentation

| Поле | Значение |
|---|---|
| ID | SPEC-37 |
| Статус | Accepted |
| Scope status | Third-person visual target and the R5g exact base-rig/skinning/pose-corrective plus bounded deformation-LOD route are Accepted/current; load/injury deformation, severity matrix and advanced deformers remain Proposed |
| Версия | 1.3 |
| Последняя проверка | 2026-08-18 |
| Product decision | [PRODUCT-FA-001](../product/functional-anatomy-and-character-embodiment.md) |
| Нормативные зависимости | [SPEC-04](04-rendering-and-platform.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-28](28-skeletal-animation-retargeting-and-ik.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [SPEC-36](36-functional-tissue-condition-and-injury.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md), [ADR-028](adr/028-platform-session-and-presentation-authority.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-075](adr/075-product-grounded-functional-anatomy-and-character-embodiment.md) |
| Заменяет | SPEC-37 1.2; records the production R5g authored pose-corrective and cadence/deformation-LOD authority-isolation route |

## Назначение и status boundary

SPEC-37 defines how committed physical pose and SPEC-36 body condition become
a final visible 3D character: render rig, skinning, pose correctives, load-aware
shape, wounds, retained tissue, clothing/armor occlusion and bounded secondary
motion. It delivers a readable third-person character without creating another
movement or injury authority.

Accepted here are the one-way data flow, anatomically plausible visual target,
`Reduced`/`Realistic`/`Graphic` severity semantics, mandatory base fallback,
third-person readability, LOD behavior and first vertical. The current
`NeutralBaseSkinningProfileV1`, exact presentation subprojection
and B0 LBS consumer are admitted because R5f provides a production consumer;
R5g additionally admits one bounded translation-driven sparse pose-corrective
set and its four-level presentation work selector. The broader
`CharacterEmbodimentManifestV1`, load/injury correctives, severity matrix,
wound catalog, general animation LOD and advanced deformer models remain
Proposed.

## Product visual target

The engine must be capable of sufficiently realistic human embodiment. The
recommended default is `Realistic`: plausible body volume, pose/load cues,
fracture alignment, wounds and blood without gratuitous emphasis. Projects also
declare:

- `Reduced` — covered/closed wounds, restrained blood and no exposed-tissue
  detail;
- `Graphic` — exposed tissue and stronger injury detail where project/platform
  policy permits it.

All three profiles render the same committed condition/topology. A severity
preference cannot change damage, capability, AI behavior, treatment or replay.

The primary camera is third-person. The character must remain readable at the
declared gameplay distance through silhouette, stance, asymmetric gait,
guarding, load transfer, fall, crawl, drag, wound/material state and sound.
Camera distance or visibility never becomes damage evidence.

## Invariants

- `RenderPose` comes only from the declared SPEC-28 physics/animation source.
- Presentation reads immutable committed pose, topology, condition and applied
  effort; it writes zero RPG, Motor, AI or Physics fields.
- Skin weights, vertex deltas, wound meshes, secondary motion, visibility and
  severity preference are presentation-only.
- A complete authored skinned character remains visible when every optional
  deformer/tissue feature fails.
- A physically retained leg cannot be hidden as an amputation by presentation;
  detachment is rendered only from the committed topology transition.
- Headless/null presentation executes the complete injury/gameplay path and
  yields the same authoritative roots.
- Runtime learning, external inference, network and LLM are forbidden from the
  deformation path.

## Ownership and inputs

| Fact | Owner | Presentation input |
|---|---|---|
| Physical pose, contacts and topology | Physical Embodiment physics world | committed `RenderPose` + normalized physical projection |
| Animation/retarget/IK presentation pose | Physical Embodiment animation bridge | committed render pose for the selected physical LOD |
| Local/systemic body condition and treatment stage | RPG owner under SPEC-36 | immutable revision-bound visual/UI condition view |
| Base safety and post-safety applied effort | Physical Embodiment safety/physics path | immutable exertion projection; requested target alone is insufficient |
| Rig, meshes, skinning, materials, correctives, clothing and wound assets | Assets/content | exact content-addressed embodiment profile |
| Vertex deformation and secondary tissue/cloth motion | Presentation/Rendering | reconstructible presentation state only |

The renderer does not read mutable RPG/ECS storage or vendor physics handles.
Presentation extraction publishes one complete bounded snapshot with exact
source/content revisions. Stale or mismatched data rejects the sample and uses
the prior/declared fallback.

## Embodiment content closure

The future `CharacterEmbodimentManifestV1` binds one exact physical archetype
and declares:

- render-skeleton signature and stable rig-node IDs;
- explicit BodySchema/animation-to-render mapping, including helper/twist bones;
- one or more skinned surface meshes, bounds and material slots;
- profile-selected LBS/DQS/equivalent bounded base skinning;
- authored pose-corrective data for the target gameplay range;
- functional-group influence regions when load deformation is present;
- tissue-region/break-site/topology visual bindings;
- intact, weakened, stable-fracture, retained-fracture and detached variants;
- wound opening/cap/exposed/retained tissue assets for each admitted severity;
- clothing/armor occlusion and split/hide rules without gameplay authority;
- secondary-motion attachment chains and bounded constraint profiles;
- deformation/render LODs, per-profile resource bounds and full fallback chain;
- source/provenance/license and canonical content hashes.

Physical and render skeletons need not have identical topology. The explicit
bridge may distribute twist, drive scapula/helper bones and map one physical
segment to several render nodes. It cannot modify physical pose.

Cross-domain bindings use stable IDs and exact signatures. Runtime nearest-bone
matching, filenames, vertex order or raw backend indices are invalid. Cooker
rejects missing nodes, cycles, invalid weights/deltas, incompatible topology,
unbounded buffers, illegal severity closure or absent base fallback before
activation.

The manifest contains no current pose, condition, effort, temporal deformer
state or renderer device object.

### Current R5f/R5g base profile

R5f does not introduce the future all-features manifest. Its exact current
`NeutralBaseSkinningProfileV1` binds one mesh, source skeleton and
`BodySchemaAssetV1` revision; stable render-joint IDs with explicit animation-
joint/body-semantic mappings; bind transforms; one-to-four positive LBS
influences per vertex summing exactly to `u16::MAX`; a positive instance bound;
and mandatory `BindPose` fallback. Cook and activation validate the complete
closure before atomic publication.

R5g extends that exact current-only profile with at most 64 stable corrective
IDs and 1,048,576 total sparse vertex deltas. Each corrective names one render
joint translation axis, signed start/full displacement from bind pose,
`Essential | Detail` class and canonical unique in-range mesh vertex deltas.
Checked fixed-point evaluation applies the selected deltas before LBS. This is
an authored normal-locomotion surface subset, not a general blend-shape graph,
load estimate, injury condition or topology selector.

The reference player and NPC share this profile and mesh while retaining their
distinct material and committed body-transform projections. Runtime name,
nearest-bone or raw-index matching is absent.

## Required baseline and optional refinement

Every admitted visible character has this complete baseline:

```text
committed pose/topology
  → explicit render-rig bridge
  → base skinning
  → small authored pose-corrective set
  → authored injury/topology variant
  → material and final render submission
```

The optional refinement stack is:

```text
post-safety effort / estimated recruitment
  → load-aware group deformation
  → bounded secondary tissue/cloth response
  → optional neural residual or local contact deformation
```

Optional stages may be fused by a backend only when ordering, bounds and
fallback results remain equivalent. No stage can move root intent, damage or
topology into presentation or feed output back.

The first product vertical does not require a neural deformer or live soft-body
solver. The production priority is a correct authored character and injury
matrix. Advanced deformation must improve measured appearance while retaining
the baseline on all target hardware.

R5g implements the prefix through one small authored pose-corrective set. The
broader limp/guard/fall/crawl/drag corpus and the injury/topology stages remain
mandatory for the complete future vertical and begin in later bounded cuts;
R5g does not claim them.

## Pose, effort and muscle semantics

Pose correctives depend on declared pose/morphology and repair joint collapse,
volume loss and silhouette. They do not claim exertion.

Load-aware deformation may consume pose/velocity/contact and actual post-safety
joint effort. In a joint-actuated profile it may also consume deterministic
`EstimatedMuscleRecruitment`, whose visual objective, style/co-contraction
inputs, filter and reset are profile-owned.

Estimated recruitment is underdetermined and presentation-only. It cannot drive
strength, fatigue, damage, treatment, policy selection or save authority. It is
not labeled as measured physiology in player/debug UI.

Actual activation/force is available only from an explicitly selected future
muscle-actuated SPEC-36 profile. The input is discriminated conceptually:

```text
JointActuated { applied_joint_effort, estimated_recruitment? }
MuscleActuated { actual_activation, muscle_force, tendon_state? }
```

No joint-actuated profile fabricates the second variant.

## Injury and retained-tissue presentation

Visual injury is selected from committed condition/topology, never direct
contact heuristics. Presentation techniques are bounded authored assets:

- material/decal/normal/roughness changes for bruising and superficial damage;
- corrective/morph shapes for swelling, deformation or volume loss;
- preauthored wound openings, caps and exposed-tissue meshes at declared sites;
- secondary rig/constraint chains for hanging retained tissue;
- topology-aware intact/retained/detached hide/show sets;
- clothing/armor coverage that may hide detail without hiding body-status truth.

For an attached unstable leg, Physics owns the passive distal movement and
retention constraint. Presentation maps that exact topology to the matching
variant and may add bounded hanging-tissue motion. If that optional simulation
fails, the limb remains physically attached and uses a static wound fallback.

Detached geometry follows the committed topology transaction. It does not
create the detached gameplay object or identity.

## Severity profiles

Each injury-capable embodiment declares a complete mapping for:

| Profile | Required visual behavior |
|---|---|
| `Reduced` | readable region/pose/material change; no exposed tissue required |
| `Realistic` | anatomically plausible wound/fracture/tissue state suitable for normal third-person play |
| `Graphic` | stronger blood/exposed-tissue detail while preserving the same topology and silhouette truth |

If `Graphic` assets are not legal/available on a target, the profile resolves
to its declared `Realistic` or `Reduced` fallback before play. It never removes
the mechanical injury. Accessibility preference is local presentation state.

## Optional neural/local deformers

A neural or local flesh deformer is a private replaceable presentation adapter.
Its immutable bundle declares exact input feature order, normalization, mesh/
topology signature, output bounds, model hash, runtime/operator capability,
temporal state/reset and fallback.

It may output only bounded local vertex/corrective deltas. It cannot infer or
commit damage, activation, force, contact or pose; train/update at runtime; call
an external service; or block the authoritative tick. Invalid, non-finite,
out-of-range or mismatched output is discarded atomically and the base result
is rendered with a stable diagnostic.

Training data should include rollouts from the actual motor/animation routes so
unusual limp, crawl, fall and recovery poses are covered. That is an offline
quality rule, not permission for a learned model to own gameplay.

## Snapshot and temporal state

The current base projection is
`CharacterSkinningPresentationRecordV1` inside `PresentationSnapshotV3`. It
binds stable object key, exact mesh/profile/skeleton/body-schema revisions,
source animation-profile hash, `Sampled | HeldPresentationPose |
BindPoseFallback` mode, `FullCorrectives | ReducedCorrectives |
BaseSkinningOnly | Culled` deformation LOD and one complete sorted local
render-joint pose. Scene and skinning records form an exact one-to-one closure
for every skinned object; Culled corresponds exactly to an invisible scene
record.

The future extended embodiment projection additionally contains:

- subject `PersistentId`, presentation epoch and sequence;
- embodiment/content profile hash;
- source physics/animation ticks and `RenderPose` revision;
- body/tissue schema and committed condition/topology revisions;
- stable rig palette, variant and severity selectors;
- applied-actuation source and optional visual recruitment;
- presentation-only temporal state or explicit reset marker.

Its extended wire shape remains Proposed. Vendor tensors, GPU handles,
descriptor objects and mutable ECS references are private.

Temporal filters reset on subject/profile/topology/severity change,
authoritative restart/cut, missing sequence or declared LOD transition. They use
presentation sequence/logical delta, not wall clock. They may be discarded and
rebuilt without gameplay change.

## Third-person readability and body UI

The visual profile is accepted only as part of the combined player experience:

- partial weakness is visible as asymmetry/limp/load transfer;
- stable fracture is guarded and underloaded;
- complete tendon/nerve loss lacks the impossible active motion;
- retained unstable fracture is visibly attached and passively unstable;
- fall/crawl/drag retains action readability and camera framing;
- detachment remains visually distinct from retained injury.

SPEC-18 qualitative body UI reads the same condition revision and supplies
region, function, attachment/systemic band and next treatment stage when the
surface is covered or visually ambiguous. Exact diagnosis numbers are not
required in player UI.

## LOD and workload

Physical and visual LOD are independent choices over the same committed state.
Presentation may select:

1. current `FullCorrectives`: base skinning plus Essential and Detail pose data;
2. current `ReducedCorrectives`: base skinning plus Essential pose data only;
3. current `BaseSkinningOnly`: no pose-corrective evaluation;
4. current `Culled`: an invisible skinned scene emits no renderer work while
   UI/game state remains available.

Future injury/load/secondary and impostor tiers refine these four bounded
presentation-work choices; they do not retroactively become current contracts.

The product workload target is:

- up to 16 nearby detailed characters;
- up to 64 active simplified characters;
- distant durable-state-only characters with no per-frame deformation.

Camera distance, visibility and measured GPU/resource pressure may choose only
presentation LOD. They cannot choose physical LOD, suppress retained topology,
restore capability or change condition. Manifest-bounded resource exhaustion
chooses the next fallback and never stalls/mutates the authoritative tick.

These actor counts do not assert that the current renderer meets a budget. A
future performance profile must measure the combined workload and report PASS,
FAIL or deterministic downgrade honestly.

## Headless, persistence and replay

Headless/null presentation instantiates no mesh/deformer but still executes all
SPEC-36 state, physics, behavior and save/replay logic.

Embodiment content identity joins project/content compatibility when admitted.
Vertex buffers, visual recruitment, deformer caches, secondary particles and
wound targets are reconstructible and not authoritative save state. Optional
presentation continuity may live in its separate segment and reset without
world change.

Replay/capture may store optional presentation diagnostics, but gameplay
acceptance uses condition/command/physics roots. Severity, cadence, shader,
deformer and capture availability change zero authoritative roots.

## First bounded product vertical

The bounded R5g prefix now includes:

- authored render skeleton, one skinned surface and explicit physical/animation
  mapping;
- one Essential and two Detail normal-locomotion pose correctives shared by the
  player and NPC;
- exact Full/Reduced/Base/Held/Culled B0 permutations and 30/60/144 Hz
  latest-complete-snapshot repetition with zero authoritative-root change.

The remaining first-vertical work includes:

- pose-corrective breadth sufficient for limp, guarded, fall, crawl and drag
  poses;
- intact, partial-damage, stable-fracture, retained-fracture and detached
  authored variants;
- complete `Reduced`, `Realistic` and `Graphic` mappings with legal fallback;
- armor/clothing occlusion test;
- player and one NPC under the same committed states;
- 16 detailed, 64 simplified and distant-state workload configuration;
- null-presentation/headless equivalence.

The future `CHARACTER-EMBODIMENT-P1` requires:

- deterministic cook/activation of every rig, mapping, severity, variant and
  fallback;
- a complete visible character under base skinning/correctives;
- correct third-person silhouette/behavior for every injury matrix state;
- retained remains attached and visually distinct from detachment;
- UI and covered-surface cases remain readable;
- invalid/missing optional deformer retains the base surface;
- 30/60/144 Hz, severity, LOD and deformer permutations preserve authoritative
  roots;
- 16/64/distant workload reports measured result or deterministic fallback;
- optional captures support human review but are not gameplay oracles.

R5g combines focused contracts/recovery/render/reference tests with
`content-package`, `play`, `persistence-replay` and conditional `platform`;
those checks admit
only the base plus small pose-corrective/cadence-LOD route. The broader
`CHARACTER-EMBODIMENT-P1`, injury matrix, general animation-LOD corpus and
16/64/distant performance claim remain `NotRun(NoProductionConsumer)` until
their own consumers exist.

## Failure semantics and fallback chain

| Failure | Required behavior |
|---|---|
| Invalid rig/body/tissue/topology/severity mapping | Reject the complete profile before activation |
| Missing base mesh/skinning/corrective fallback | Profile is not activatable; use another complete character profile |
| Missing optional load/wound/secondary/neural asset | Use the next declared base/static fallback; authority unchanged |
| Illegal/unavailable graphic content | Resolve to declared realistic/reduced profile before play |
| Invalid pose-corrective evaluation | Discard the corrected sample and retry exact sampled base skinning; invalid base output uses bind mesh |
| Invalid optional deformer output/runtime | Discard optional sample and render base skinning/correctives |
| Stale source sequence/revision | Reject sample, reset history and use prior/held/bind fallback |
| Presentation write-back attempt | Deny/quarantine and preserve authoritative roots |

## Implementation order

1. **Implemented by R5f:** author the complete base rig, physical/render
   mapping, skinned surface and normal third-person sampled/bind pose range.
2. **Implemented by R5g:** add the bounded normal-locomotion pose correctives
   and prove Full/Reduced/Base/Held/Culled plus 30/60/144 Hz authority
   isolation.
3. **Next embodiment vertical cut:** create all three severity profiles and the
   intact/partial/stable/retained/detached asset matrix with static fallbacks.
4. Bind committed SPEC-36 state and SPEC-18 qualitative body UI.
5. Add bounded retained-tissue secondary motion with static fallback.
6. Measure 16/64/distant workload and tune only declared visual/physical LOD.
7. Evaluate effort-aware, neural or local flesh refinement last against the
   accepted authored baseline.
