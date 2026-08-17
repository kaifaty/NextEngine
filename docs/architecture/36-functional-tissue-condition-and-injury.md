# SPEC-36: Functional tissue condition, injury and structural body changes

| Поле | Значение |
|---|---|
| ID | SPEC-36 |
| Статус | Accepted |
| Scope status | Product, ownership, projection, treatment, parity and fallback semantics are Accepted; exact wire schemas and the production vertical remain Proposed |
| Версия | 1.1 |
| Последняя проверка | 2026-08-17 |
| Product decision | [PRODUCT-FA-001](../product/functional-anatomy-and-character-embodiment.md) |
| Нормативные зависимости | [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-066](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-075](adr/075-product-grounded-functional-anatomy-and-character-embodiment.md) |
| Заменяет | SPEC-36 1.0; grounds the technical boundary in the approved player experience, treatment loop, parity/LOD and acceptance matrix |

## Назначение и status boundary

SPEC-36 defines functional anatomy for gameplay. It lets one committed event
distinguish partial muscle weakness, lost tendon transmission, lost nerve
control, stable/unstable fracture, an attached but passive limb and complete
detachment. The observable chain is:

```text
impact or authorized effect
  → local tissue/structural condition
  → effective capability and physical topology
  → motor execution, gait, fall or crawl
  → presentation/UI projection
  → stabilization, repair and rehabilitation
```

This is not a medical simulation and not a second movement controller. The
current joint-target plus fixed safety/PD route remains the actuation baseline.
Functional muscle groups describe how much directional capability remains.

Accepted here are product semantics, ownership, deterministic transaction and
projection rules, player/NPC parity, treatment stages and fallbacks. Current
RPG operations, save/replay versions, `BodySchemaV1/V2`, Motor tensor layouts
and content catalogs do not change. Exact body-condition, treatment, anatomy
and topology contracts appear only with the first production consumer under
ADR-046.

## Scope

In scope:

- immutable functional anatomy bound to one exact `BodySchema`;
- durable local tissue/structural condition and simplified systemic condition;
- evidence-driven deterministic damage and treatment proposals;
- sparse functional-group projection into existing actuator safety;
- authored stable, unstable-retained and detached structural variants;
- preserved player/NPC intention with injury-compatible execution;
- staged medicine/magic recovery;
- durable condition across physical/render/population LOD;
- save/load/replay, qualitative UI and immutable presentation projections;
- identical public package paths for first-party and community mechanics.

Not required for the first vertical:

- per-fiber/fascicle muscle simulation or full muscle actuation;
- internal organ gameplay or detailed vascular/tendon networks beyond consumed
  lower-limb dependencies;
- medically precise diagnosis, pain behavior or healing times;
- arbitrary runtime mesh/body cutting;
- arms, face, whole-body injury breadth or nonhuman morphologies;
- renderer/model-owned damage or treatment;
- tactical retreat, surrender or social response to injury.

The schema may later add organs and finer anatomy through new consumed
revisions; their future possibility does not add hidden first-slice fields.

## Ownership

| Mutable fact | Единственный owner | Разрешённые consumers |
|---|---|---|
| Local tissue integrity, continuity, structural/attachment mode and recovery stage | Future RPG-owned body-condition aggregate | Mechanics, Physical Embodiment, AI, UI and presentation through immutable revision-bound views |
| Simplified systemic condition/consciousness/death transition | RPG domain | Motor/AI/UI through immutable views; proposals through normal command validation |
| Damage/treatment definitions, protection rules and reducers | Mechanics Runtime/content package | Bounded proposal into the common `WorldCommand` path |
| Active pose, velocities, contacts, constraints and body topology | Physical Embodiment physics world | Motor, outcome resolver and immutable presentation extraction |
| Effective actuator capability | Reconstructible Physical Embodiment projection of BodySchema + committed owner views | Safety/PD and Motor observation; never durable parallel state |
| Player action and NPC intent | Player input / Agent owner according to existing contracts | Motor/animation planner; injury does not take ownership of intent |
| Visual wound, exposed tissue, surface deformation and body UI | Presentation/Rendering | Read-only; zero write-back |

Mechanics may propose but cannot own accepted damage. Physics proves impact but
cannot reduce condition. Physical Embodiment derives capability but cannot copy
condition into a mutable muscle-health store. Presentation and UI never diagnose
or commit hidden injury.

## Immutable functional anatomy

The future `BodyTissueSchemaV1` is an immutable content-addressed member of one
`PhysicalArchetypeBundle`, bound to an exact BodySchema hash. It contains stable
schema-scoped IDs and canonical records for:

- tissue/anatomical regions and physical body-node mapping;
- admitted damage channels and bounded condition dimensions;
- functional muscle groups with sparse actuator-axis/direction contributions;
- consumed tendon, nerve and vascular dependencies;
- authored break sites and legal topology variants;
- contact-feature, UI and presentation bindings;
- capability, systemic, structural, damage and recovery reducer hashes;
- complete fallback for every optional structural or visual feature.

Conceptual IDs include `TissueRegionId`, `FunctionalMuscleGroupId`,
`BreakSiteId` and `TopologyVariantId`; exact names remain Proposed. Runtime
string matching, nearest-bone inference, mesh vertex indices or backend handles
cannot become identity.

The schema stores no current condition, pose, fatigue, systemic value, render
vertices or backend constraints. A changed map/reducer creates a new content
identity and cannot reinterpret an old save or policy.

### First lower-limb grouping rule

The unilateral first profile models the smallest functional set able to
distinguish its outcomes. It SHOULD include directional hip stabilization/
flexion/extension as required by the chosen gait, knee extension/flexion and
ankle plantar/dorsiflexion, plus the tendon/nerve paths consumed by the test
states. The exact number is content/profile-owned.

A group is justified only when separating it changes at least one of:
capability, supported action, gait/fall/crawl behavior, treatment, AI choice or
visible result. Anatomical detail with no product difference is out of scope.

## Typed local and systemic condition

One limb `hit_points` scalar is insufficient. The future condition aggregate
uses closed typed local facts:

| Tissue kind | Semantic dimensions when admitted |
|---|---|
| Skin/fascia | integrity and closed/open continuity; graphic severity remains presentation-only |
| Bone | integrity plus `Intact`, `StableFracture`, `UnstableFracture` or `Detached` |
| Joint/ligament | stability and allowed-motion/load restriction; never active pose |
| Functional muscle group | contractile capacity and myotendinous continuity |
| Tendon | force-transmission continuity for declared dependent groups |
| Nerve path | voluntary-control capacity for declared dependent groups |
| Vascular path | local perfusion/bleeding only when consumed by damage or treatment |

The first product profile also derives/owns one bounded systemic ladder:

1. `Stable`;
2. `Impaired`;
3. `Critical`;
4. `Unconscious`;
5. `Dead`.

Exact tags/schema remain Proposed. Separate player-facing pain, shock and blood
bars are not first-profile authorities. A local bleed may advance the systemic
reducer, but does not create an independent presentation-owned resource.

All authoritative values use bounded integer/fixed-point fields with
profile-owned units, saturation/rejection and canonical ordering. Exertion
fatigue remains with its declared RPG/Mechanics owner and is a separate input
to capability. Wall clock, renderer frames, async completion and inference
cannot heal, worsen or advance systemic condition.

## Damage evidence and transaction

The production path is:

```text
committed contact / authorized gameplay effect
  → normalized bounded evidence
  → Mechanics reducer and named RNG stream
  → revision/hash-bound WorldCommand proposal
  → staged RPG condition + optional physical topology transition
  → one atomic fixed-stage commit or no commit
  → ordered DomainEvents and immutable projections
```

Damage evidence may include target region, type, point/direction, impulse or
force, effective mass, current load, source/weapon/material and protection.
The same canonical evidence, content profile and RNG state produces the same
result. Bounded authored variance is allowed only through a named replayed
stream. Raw callback order, penetration spikes, renderer state and free-form
model output are invalid evidence.

Stale revision, invalid region/channel, mismatched content hash, invalid
protection, non-finite/out-of-range input or incompatible topology rejects
before mutation. Duplicate/conflicting identity follows ADR-022.

When condition and topology change together, both owner replacement and the
complete SPEC-26 transaction are validated against the same preconditions and
publish together. No observer may see a committed retained fracture whose
physical variant failed to activate.

## Capability projection and PD

A functional muscle group is a gameplay/control abstraction, not necessarily
one anatomical belly and not a physics actuator. Its effective capacity is a
deterministic bounded function of:

- exact BodySchema base capability;
- committed group/tendon/nerve continuity;
- structural availability and safe load;
- permitted stats/equipment/protection overlays;
- current fatigue/power projection from its owner.

The hash-bound reducer intersects the BodySchema effort/rate/power/work/ROM/
velocity safety envelope with these facts. Injury can preserve or reduce the
base envelope, never increase it. Complete loss gives exact zero/neutral
applied capacity for the declared direction according to the profile.

Physical Embodiment compiles one immutable `BodyCapabilityEnvelope`-equivalent
per committed revision. The current safety/PD path consumes that envelope.
There is no independent “muscle controller”, so the system contains no double
movement logic.

Policy target, PD error, requested effort and applied effort are not actual
muscle activation. In joint-actuated profiles, any
`EstimatedMuscleRecruitment` is presentation-only and cannot update condition,
fatigue, treatment or policy state.

## Structural states

`StableFracture` retains topology. It may restrict load, ROM and capability and
produce guarding, but does not invent a joint or body split.

`UnstableFracture` requires a declared break site and prevalidated topology
variant. The variant defines proximal/distal nodes, one bounded retention
constraint, mass/inertia conservation, collision/filter roles, stable-ID remap,
relative-motion/load limits and presentation binding.

An attached but unusable limb requires all of:

1. durable attachment remains retained, not detached;
2. affected voluntary capability is zero/reduced;
3. a declared constraint keeps the passive distal segment attached;
4. compatible movement falls back to guard/drag/fall/crawl behavior;
5. presentation consumes the same condition/topology revision.

`Detached` is a separate atomic transition. A removed part gets a new durable
`PersistentId` only when it is admitted as a gameplay object. Render hide/show
cannot create or remove that identity.

If no valid retained variant exists, the fallback is functionally disabled
intact topology plus a stable diagnostic. Silent amputation, guessed splitting,
arbitrary cutting and partial publication are forbidden.

## Agency and damaged-body behavior

Injury changes execution, not intention ownership. Player and NPC may continue
to request stand, move, turn, rise, attack or interact. The selected compatible
controller returns the best supported physical result.

The route compatibility key declares condition/capability schema, supported
group/structural/topology masks, handoff/reset behavior and fallback. Known
condition enters observation explicitly; it cannot hide only in recurrent
state. Unsupported masks never route through the intact controller by
approximation.

The first vertical requires authored, procedural or eventually learned support
for:

- limp and asymmetric load transfer under partial capacity;
- guarded loading under stable fracture;
- loss of local voluntary motion under tendon/nerve failure;
- fall when support/balance cannot be maintained;
- crawl and passive drag for severe attached lower-limb failure;
- ragdoll only after declared loss of balance/consciousness or another physical
  cause.

Capability clamping proves impairment but not natural compensation. The
behavior above needs focused logic/data and its own acceptance evidence.

## Player/NPC parity and simulation tiers

The same content definitions, reducers, local/systemic condition, capability
projection, structural transitions and treatment rules apply to player and
NPC. AI may choose different intentions; it cannot receive easier anatomy.

The target workload has three fidelity tiers:

- **near detailed:** up to 16 actors with the selected full physical injury and
  presentation paths;
- **active simplified:** up to 64 actors with durable condition and the same
  rules but declared simplified physics/controller/presentation evaluation;
- **distant state-only:** durable condition and conservative capability summary
  without per-frame tissue/deformer work.

These counts are product workload targets, not current performance evidence.
Tier changes preserve condition and structural identity. A pending transition,
unstable constraint, fall or external contact may forbid downgrade. Unsupported
precise outcomes upgrade, defer or take the authored conservative fallback;
they are never fabricated abstractly.

The first NPC slice covers physical adaptation, fall and crawl. Tactical
retreat, surrender and social response are later Agent/gameplay consumers.

## Treatment and recovery

Treatment uses three semantic stages:

1. `Stabilize` — prevent declared worsening, control local bleeding and/or
   immobilize; full capability is not restored;
2. `Repair` — restore declared bone alignment, tissue/tendon continuity, nerve
   control or attachment;
3. `Rehabilitate` — restore usable capacity, coordination and load tolerance.

Medicine and magic are authored effect channels over the same reducers and RPG
operation boundary. A magical effect may combine stages only by explicitly
declaring all resulting condition transitions and preconditions. No treatment
may restore a detached/absent structure or nerve path implicitly.

Recovery advances from canonical simulation time and committed treatment/rest
facts. Interruption, invalid resource, stale condition, incompatible structural
state or owner rejection produces no partial repair.

## Persistence and replay

The first consumer adds the complete durable local/systemic condition owner and
exact anatomy/content hashes to the current save/replay closure. Derived
capability, AI summaries and presentation projections are recomputed and may be
root-compared; they do not become mutable owners.

The physical segment stores canonical body/topology/constraint state under
SPEC-26/27. Load validates schema identity, bounds, region references,
attachment variant, treatment/recovery state, owner revision and content hashes
before replacing the world. Corruption rejects the complete generation and
preserves source bytes/current world.

Replay records damage/treatment command, named RNG advancement, condition
events, topology transition and post-safety effort chain required by the
profile. `game` with null presentation and headless must yield identical
condition, command-ledger, physics and gameplay roots.

## UI and presentation projection

The immutable player-facing body view contains qualitative region, functional
severity, structural/attachment state, systemic band and next treatment stage.
Exact internal capacity, RNG and reducer values remain developer diagnostics.
SPEC-18 owns the UI semantics; SPEC-37 owns visual embodiment. Both consume the
same committed revision and neither writes back.

## Content and extension boundary

First-party damage/treatment uses the same definitions, capability checks,
proposals and validators as data/Luau/Wasm packages. Packages cannot add
unbounded fields, mutate Physical Embodiment, build trusted topology plans or
use renderer geometry as evidence.

Cook/activation validates region/group/dependency/break-site references,
ordering, bounds, acyclic dependency, topology conservation, treatment
preconditions, UI/presentation bindings and every fallback before publication.
Exact alpha schemas remain current-only when introduced.

## First bounded product vertical

One neutral humanoid, one unilateral lower limb and one third-person test
encounter cover:

1. intact control;
2. partial knee-extensor loss;
3. complete declared tendon or nerve loss with directional zero function;
4. stable lower-leg fracture;
5. unstable retained lower-leg fracture with passive distal segment;
6. detachment as a distinct comparator;
7. stabilization, one medical repair, one magical repair and rehabilitation;
8. player agency plus one NPC using the same physical adaptation rules;
9. save/load/replay at every state and malformed/stale failures;
10. 16/64/distant tier permutations under a future measured profile.

The future `INJURY-EMBODIMENT-P1` passes only when:

- canonical evidence deterministically produces exact condition/capability/
  topology/treatment/event roots;
- post-safety effort respects directional loss;
- retained remains attached, passive and stable without implicit amputation;
- ordinary intention yields the declared limp/fall/crawl/drag result;
- stabilization does not fake repair, and recovery follows authored stages;
- player/NPC and tier permutations preserve semantic outcomes;
- save/load/replay and worker/order permutations preserve roots;
- presentation/UI/severity/cadence/deformer permutations change zero
  authoritative roots.

`play`, `persistence-replay` and `content-package` are required when the
consumer exists. `platform`/`performance` are conditional. Until then all
executable checks are `NotRun(NoProductionConsumer)`.

## Full muscle-actuated future profile

A true profile changes the actuator chain to excitation → activation dynamics
→ muscle/tendon force → moment arm → joint effort → Physics. It needs a new
BodySchema/anatomy generation, observation/action/state, safety, replay,
runtime-training correspondence and ProductCheck. Actual activation is
authoritative only inside that explicit profile.

It preserves the joint-actuated fixed-PD route as shipping fallback unless a
later Accepted ADR changes the baseline. Visual recruitment or injury state
cannot silently promote muscle actuation.

## Failure semantics

| Failure | Required behavior |
|---|---|
| Invalid/stale region, revision, evidence, hash or damage/treatment channel | Reject before mutation; retain prior owner/topology/ledger roots |
| Invalid dependency/capability projection | Reject the complete projection/profile; retain the last valid safe envelope |
| Required retained variant fails validation/staging | Publish neither condition nor topology; permit only an authored conservative retry/fallback command |
| Unsupported damaged-body route | Select compatible procedural/passive route; never approximate with intact behavior |
| Unsupported precise outcome at lower tier | Upgrade, defer or use conservative authored result; never fabricate |
| Presentation/UI tries write-back | Quarantine the path; authoritative state unchanged |

## Implementation order

1. Freeze the unilateral lower-limb anatomy, product matrix and treatment
   definitions against one existing BodySchema.
2. Add the typed RPG condition/systemic owner and deterministic damage/treatment
   reducer without learned control.
3. Prove partial/zero capability, ordinary intention and save/replay on intact
   topology.
4. Add stable fracture, then one retained topology and passive/fall/crawl
   fallback.
5. Bind the same state to SPEC-18 UI and SPEC-37 base visual variants.
6. Prove player/NPC parity and 16/64/distant LOD correctness before hard budget
   claims.
7. Evaluate injury-conditioned learned control, advanced deformation, organs or
   true muscle actuation only as later separately gated increments.
