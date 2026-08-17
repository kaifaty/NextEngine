# Functional anatomy and character embodiment — product brief

| Field | Value |
|---|---|
| ID | PRODUCT-FA-001 |
| Status | Approved product direction; implementation not started |
| Decision date | 2026-08-17 |
| Product scope | Functional tissue injury, retained/detached limbs, treatment loop and final visible 3D characters |
| Architecture | [SPEC-36](../architecture/36-functional-tissue-condition-and-injury.md), [SPEC-37](../architecture/37-character-embodiment-and-surface-deformation.md), [ADR-075](../architecture/adr/075-product-grounded-functional-anatomy-and-character-embodiment.md) |

## Product promise

Next Engine should let the player damage a body as a functioning structure,
not only subtract health or remove a limb. A leg can remain physically present
while a fracture makes it unstable, a damaged nerve removes voluntary control,
or torn muscle groups can no longer transmit enough force. The character then
tries to continue the requested action with the function that remains.

The promise is **functional anatomy for systemic gameplay**, not a medical
simulator. The player should be able to understand the causal chain:

```text
impact
  → damaged structure or functional group
  → lost or reduced capability
  → changed load-bearing, gait, fall or crawl
  → readable 3D wound/body behavior and body-status UI
  → stabilization, repair and functional recovery
```

This is compatible with the current joint-target plus fixed safety/PD motor.
Muscle groups describe what force and control remain; they do not have to be
simulated as the force-producing actuator for every movement.

## Experience pillars

### One body, one cause

Gameplay state, physical behavior, AI decisions and presentation must describe
the same committed injury. A wound mesh cannot invent damage, and a health
number cannot claim that a visibly unstable leg is fully usable.

### The limb may remain, but its function may not

The supported outcome space includes:

- partial weakness with voluntary motion still available;
- complete directional function loss from muscle, tendon or nerve damage;
- a stable fracture that remains aligned but cannot safely bear normal load;
- an unstable fracture where the distal leg is passive but retained by tissue;
- complete detachment as a different structural state, not the only severe
  injury result.

### The player retains intention

An injury changes execution, not input ownership. The player continues to ask
to move, turn, rise or attack. The motor/animation route produces the best
supported result: limp, guarded step, load transfer, fall, crawl, drag or
refusal of a locally impossible action. Full ragdoll is reserved for loss of
balance, consciousness or another declared physical reason.

### Same rules for player and NPC

Player and NPC bodies use the same condition types, damage reducers, capability
rules and treatment transitions. Simulation and presentation fidelity may
change by tier, but an NPC does not recover a destroyed nerve or lose a fracture
because it moved off-camera.

### Readable without debug numbers

The third-person camera must communicate injury through pose, gait, guarding,
load transfer, falls, dragging, crawling, wound/material state and sound. A
body-status panel supplements those cues with qualitative region/function and
treatment-stage information. Exact internal values remain a developer view,
not the required player interface.

## Deliberate abstraction

### Functional groups, not every anatomical muscle

The first implementation models groups by gameplay function and force
direction. The unilateral lower-limb profile should be able to distinguish at
least:

- hip extensors/flexors and lateral stabilizers where required by the chosen
  gait;
- knee extensors and flexors;
- ankle plantar-flexors and dorsiflexors;
- the declared tendon and nerve paths that can remove transmission or voluntary
  control for those groups.

The exact count is selected by the lower-limb content profile. A group exists
only when two different states produce a meaningful difference in movement,
treatment, AI choice or presentation.

### Simplified whole-body condition

The first product profile does not expose separate player-facing pain, shock,
blood-volume and consciousness simulators. It uses one bounded systemic
condition ladder:

1. `Stable` — local injury without systemic impairment;
2. `Impaired` — reduced whole-body performance, but conscious agency remains;
3. `Critical` — severe systemic compromise and high risk of collapse;
4. `Unconscious` — no voluntary action until the owning RPG rules restore it;
5. `Dead` — terminal state according to the game rules.

Local bleeding/perfusion evidence may exist where required to drive this
summary and treatment, but it is not presented as a second set of player
resource bars. The exact authoritative schema remains consumer-driven.

### Deterministic damage with bounded authored variance

Damage resolution consumes committed evidence: target region, attack/damage
type, force or impulse, direction, load state and protection. The same canonical
inputs, content profile and seed stream produce the same result. Bounded
authored variance may prevent a fully mechanical threshold feel, but render
frames, wall clock and unrecorded randomness never choose the injury.

## Treatment fantasy

The setting supports ordinary medicine and magic through the same three
functional stages:

1. **Stabilize** — stop worsening, control bleeding, immobilize or safely bind
   the structure; this does not restore full force.
2. **Repair** — restore bone alignment, tissue continuity, nerve function or
   structural attachment through an authored medical or magical method.
3. **Rehabilitate** — recover usable capacity, coordination and load tolerance.

A powerful magical effect may explicitly combine or skip stages, but only when
its authored effect says so. Medicine and magic submit the same validated
condition transitions; neither gets a private mutation path.

## Visual target

The engine must support anatomically plausible, sufficiently realistic
characters and injury presentation from a third-person gameplay camera. The
same committed injury can be rendered through three content profiles:

- `Reduced` — closed/covered wounds, restrained blood and no exposed-tissue
  detail;
- `Realistic` — the recommended default, with plausible deformation, fracture
  alignment, wounds and blood without gratuitous emphasis;
- `Graphic` — exposed tissue and stronger wound detail where the project and
  platform permit it.

All profiles preserve the same gameplay state and physical topology. Clothing
or armor may occlude a wound visually while its protection and body condition
remain authoritative.

The production baseline is authored render rig + skinning + pose correctives +
authored injury variants. Effort-aware deformation, secondary tissue and
neural deformers are optional refinements with a complete base fallback.

## First vertical slice

The first slice is one neutral humanoid, one unilateral lower limb and a small
test encounter viewed from the third-person camera. It includes this state
matrix:

| State | Voluntary function | Physical behavior | Player-readable result |
|---|---|---|---|
| Intact control | Full declared capacity | Normal stand/walk/turn | Baseline gait and body panel |
| Partial knee-extensor damage | Reduced knee extension | Limp and load transfer; fall under excessive demand | Weakness visible in gait and body panel |
| Complete tendon or nerve loss | Zero declared direction/control | No active extension; leg guards or drags | Action remains requested, impossible local motion is not faked |
| Stable lower-leg fracture | Reduced safe loading; topology intact | Guarded stance/limp, failure under unsafe loading | Leg present, fracture and reduced support readable |
| Unstable retained fracture | Distal segment passive | Retention constraint, instability, fall/crawl/drag | Leg remains attached; it is visibly and mechanically unusable |
| Detached comparison | Missing chain | Separate committed topology/object behavior | Clearly distinct from retained flesh injury |

Treatment coverage for that matrix includes stabilization, one medical repair,
one magical repair and staged functional recovery. First-slice NPC behavior
needs physical adaptation, fall and crawl only. Tactical retreat, surrender and
social reactions are later breadth.

## Scale and LOD target

The intended representative encounter contains:

- up to 16 nearby detailed characters with full selected physical injury and
  visible embodiment paths;
- up to 64 active characters with the same durable rules but simplified
  physical/presentation evaluation;
- more distant characters with durable condition and conservative capability
  summaries, without per-frame tissue or deformation work.

These are workload targets, not a current performance claim or timing budget.
The exact profile and hardware gate are defined only with the production
consumer. LOD may reduce how a state is evaluated or drawn; it may not change
the state itself.

## Product acceptance

The concept is accepted as working only when a production-path scenario proves
all of the following:

- identical impact evidence produces identical condition, capability and
  topology results in `game`, headless, save/load and replay;
- partial weakness, zero function, stable fracture, retained passive fracture
  and detachment remain mechanically distinct;
- the player keeps issuing ordinary intentions and receives a compatible limp,
  fall, crawl, drag or local inability result without hidden pose teleport;
- first-slice NPCs use the same rules and show the same physical adaptation;
- stabilization prevents declared worsening but does not restore structure;
  repair and rehabilitation change only their declared stages;
- the third-person view and qualitative body UI let a tester identify the
  affected region, lost function and next treatment stage without debug tools;
- `Reduced`, `Realistic` and `Graphic` presentation, render LOD, cadence and
  optional deformer changes alter zero authoritative roots;
- 16/64/distant workload tiers preserve condition and gameplay outcomes and
  report honest measured performance or deterministic downgrade.

## Explicit non-goals for the first vertical

- per-fiber muscle dynamics or full musculoskeletal actuation;
- medical diagnosis accuracy or separate simulation of every physiological
  variable;
- internal organ gameplay, detailed vascular networks or individual tendons
  beyond the lower-limb dependencies required by the slice;
- arms, hands, face injury, whole-body injury breadth or arbitrary creatures;
- arbitrary runtime cuts/remeshing;
- neural deformation as a dependency of correctness;
- tactical surrender/retreat logic, social reactions or a hospital economy.

The anatomy/content model should remain extensible to organs, additional tendon
paths and finer vascular structure, but those are later product decisions, not
hidden first-slice obligations.

## Open implementation questions

Product intent is closed; these engineering choices remain intentionally open
until the bounded slice is scheduled:

- exact current-only condition/command/content schemas;
- exact lower-limb group count and fixed-point reducer coefficients;
- which authored unstable-fracture representation is most stable in PhysX;
- procedural, authored-animation or trained controller split for limp/crawl;
- deformation technique and per-tier timing/memory budgets on target hardware;
- whether the approved feature becomes a v1 gate or a post-v1 signature
  increment. The current roadmap keeps it post-baseline and non-blocking.
