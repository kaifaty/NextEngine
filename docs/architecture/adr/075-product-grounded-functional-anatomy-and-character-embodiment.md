# ADR-075: Product-grounded functional anatomy and character embodiment

| Поле | Значение |
|---|---|
| ID | ADR-075 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-17 |
| Последняя проверка | 2026-08-17 |
| Product decision | [PRODUCT-FA-001](../../product/functional-anatomy-and-character-embodiment.md) |
| Нормативные зависимости | [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-18](../18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-19](../19-rpg-domain-and-narrative-state.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-28](../28-skeletal-animation-retargeting-and-ik.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [SPEC-36](../36-functional-tissue-condition-and-injury.md), [SPEC-37](../37-character-embodiment-and-surface-deformation.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-066](066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md) |
| Заменяет | Первое Accepted решение по functional anatomy и character embodiment; существующий [ADR-074](074-systemic-strategic-agent-owner-vertical.md) остаётся authority для R4d Strategic Agent vertical |
| Заменён | не заменён |

## Контекст

Existing RPG, Physics, Motor and presentation boundaries separate durable
condition, physical capability and visible surface deformation, but no accepted
decision had connected them to an elicited player experience. Product discovery
has now closed the missing choices:

- this is functional anatomy for systemic gameplay, not medical simulation;
- player and NPC use the same rules with fidelity LOD;
- an attached but structurally or neurologically unusable limb is a primary
  outcome, not an intermediate form of amputation;
- the player retains ordinary movement intention while execution degrades into
  limping, guarding, falling, crawling or dragging;
- whole-body systemic condition is deliberately simplified;
- treatment is stabilization → repair → rehabilitation, using medicine or
  magic through the same owner boundary;
- the game is third-person and needs both physical/visual cues and a qualitative
  body-status UI;
- the first vertical is one unilateral lower limb; full organs, arms and full
  muscle actuation are not first-slice requirements;
- the target encounter scale is 16 detailed, 64 active-simplified and distant
  state-only characters, subject to a future measured production profile.

Without this product-grounded decision, the technical abstraction could be implemented
without delivering those observable product outcomes.

## Решение

### 1. One actuation law, one capability projection

The current motion path remains joint targets → fixed safety/PD → applied joint
effort → Physics. Functional muscle groups, tendons, nerves and structural
state reduce the capability envelope consumed by that path. They do not form a
second controller and do not require every movement to be muscle-actuated.

The chain is strictly one-way:

```text
committed body condition
  → derived capability envelope
  → existing motor/safety/PD path
  → committed physical outcome
  → read-only animation, 3D surface and UI
```

True muscle actuation remains a separate future profile with its own state,
safety, replay and correspondence contract. Estimated muscle recruitment from
joint effort is presentation-only.

### 2. Functional anatomy is a gameplay abstraction

The content profile models the smallest set of functional groups and
bone/joint/tendon/nerve/vascular dependencies needed to create distinct player
outcomes. Per-fiber, per-fascicle and complete organ anatomy are excluded from
the first vertical.

The first lower-limb profile distinguishes directional hip, knee and ankle
function as required by its gait and treatment scenarios. A group that changes
neither capability, treatment, AI behavior nor presentation is not admitted.

### 3. Durable condition and systemic summary remain RPG-owned

RPG owns the typed local body condition and the simplified systemic condition.
The first product profile exposes one bounded ladder equivalent to `Stable`,
`Impaired`, `Critical`, `Unconscious` and `Dead`, rather than independent
player-facing pain, shock and blood-volume simulators. Local perfusion or
bleeding facts exist only when a production reducer or treatment consumes them.

Mechanics owns damage/treatment definitions and submits revision-bound
proposals through `WorldCommand`. Physics contact, AI, Motor, renderer and UI
cannot commit injury, healing, consciousness or death directly.

### 4. Damage is evidence-driven and deterministic

Damage resolution consumes bounded committed evidence: region, damage type,
force/impulse, direction, load state and protection. Bounded authored variance
uses named canonical RNG streams and is replayed exactly. Backend callback
order, wall clock, renderer cadence, wound appearance and model output cannot
select the condition.

### 5. Attached, passive and detached are different states

Stable fractures retain topology. Unstable retained fractures use predeclared
conservative body/topology variants and a bounded retention constraint. The
distal segment may be passive, collidable and unusable while remaining
attached. Detachment is a different atomic owner + physics transition.

Missing or invalid retained topology falls back to a functionally disabled
intact topology with a stable diagnostic; it cannot silently amputate, invent a
body split or publish half a transition.

### 6. Intention is preserved; execution adapts

Player input and NPC intent continue to request ordinary movement/action
outcomes. Injury-conditioned authored, procedural or learned routes translate
those intentions into supported behavior. The first vertical must cover limp,
load transfer, fall, crawl and passive drag. It does not assume that clamping
PD effort alone produces natural compensation.

Full ragdoll is entered only for a declared physical or RPG cause such as loss
of balance or consciousness. Unsupported injury/topology masks use a compatible
passive/procedural fallback and never invoke the intact controller by
approximation.

### 7. Player/NPC rule parity with fidelity LOD

Player and NPC use the same condition types, damage/treatment reducers,
capability projection and topology transitions. LOD may select cadence,
conservative summary, physical representation and surface quality. It cannot
heal, restore, detach or discard condition because the character is distant.

The target workload is up to 16 nearby detailed actors, 64 active simplified
actors and distant durable-state-only actors. This is an approved product
workload target, not a current performance PASS or an accepted microsecond
budget.

### 8. Treatment is staged and mechanism-neutral

Treatment transitions are classified as stabilization, structural/tissue
repair and rehabilitation. Stabilization stops declared worsening but does not
restore full load or control. Repair restores declared continuity/alignment;
rehabilitation restores capacity. Authored medicine and magic use the same
validated transition path. A magical effect may combine stages only when its
definition explicitly declares that outcome.

### 9. Third-person presentation and UI are read-only

The default visible target is anatomically plausible `Realistic` presentation,
with `Reduced` and `Graphic` profiles over the same committed condition and
topology. Presentation uses render rig, base skinning, pose correctives and
authored injury variants as the complete baseline. Load-aware, secondary-tissue
and neural deformation are optional.

Third-person gait, guarding, load transfer, fall/crawl/drag, wound/material
state and sound are primary cues. A qualitative body-status projection shows
affected region, functional severity, structural attachment and next treatment
stage. Neither camera nor UI owns or derives hidden injury truth.

### 10. One bounded lower-limb vertical promotes the exact contracts

The first consumer covers intact, partial knee-extensor loss, complete tendon
or nerve loss, stable fracture, unstable retained fracture and detachment as a
comparison. It includes stabilization, medical and magical repair,
rehabilitation, player agency, first-slice NPC physical adaptation, save/replay,
three visual profiles and 16/64/distant LOD behavior.

This ADR accepts product/ownership/fallback semantics. Exact condition,
command, treatment, content, presentation and performance schemas remain
Proposed until that production consumer exists under ADR-046.

## Product impact

The decision makes the distinguishing feature clear: a character may still
have a leg while losing its useful structure or force path, and every gameplay
layer responds to that same fact. The system supports flesh damage and recovery
without requiring a full musculoskeletal simulator or making visual muscle
estimation authoritative.

It also prevents a common double-logic failure. PD does not compete with a
second muscle controller; it enforces the one capability envelope derived from
the committed body condition.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `INJURY-EMBODIMENT-P1` (future) | Lower-limb damage, treatment and player/NPC physical adaptation through production commands | Exact condition/capability/topology/treatment/event roots; retained segment remains attached/passive; intention yields declared limp/fall/crawl/drag behavior | Disabled intact topology plus compatible passive/procedural route |
| `CHARACTER-EMBODIMENT-P1` (future) | Same committed states under third-person `Reduced`/`Realistic`/`Graphic`, LOD, cadence and optional-deformer permutations | Complete readable surface and body UI; zero authoritative-root differences | Base authored skinning, pose correctives and static injury variant |
| `content-package` when admitted | Exact anatomy/group/break-site/rig/surface/treatment/fallback closure | Invalid or incomplete content rejects before activation | Retain prior complete profile |
| `play` + `persistence-replay` when admitted | Damage → behavior → treatment, save/load and headless/null-presentation parity | Player/NPC share rules and continuation is exact | Reject invalid proposal/profile; preserve prior state |
| conditional `performance` | 16 detailed + 64 active-simplified + distant state-only workload | Measured profile meets its future budgets or chooses deterministic LOD without outcome drift | Declared physical/presentation downgrade, never condition loss |

Until a production consumer exists, executable results are
`NotRun(NoProductionConsumer)` and this decision is not an implementation or
performance claim.

## Рассмотренные варианты

- **Full muscle actuation as the default movement system** — Rejected for the
  first product vertical. It is substantially more complex and not required to
  express the chosen injury gameplay. Reconsider only if the functional-group
  vertical cannot produce a stated behavior.
- **Independent PD and muscle damage controllers** — Rejected. Two mutable
  control laws would disagree about available force. Condition derives one
  envelope consumed by one safety/PD path.
- **One health value per limb** — Rejected. It cannot distinguish directional
  force, nerve control, fracture stability and attachment.
- **Separate medical simulation of pain, shock and blood volume now** —
  Rejected for the first profile. One systemic ladder keeps the player-facing
  model legible and bounded.
- **Renderer or neural model diagnoses damage** — Rejected. Visual state is a
  consumer only.
- **Detachment as the only severe injury** — Rejected. Attached-but-passive is
  a required first-class outcome.
- **Different player and NPC injury rules** — Rejected. Fidelity LOD is allowed;
  semantic rule divergence is not.
- **Arbitrary runtime cutting** — Rejected for the first consumer. Authored
  topology/mesh variants give bounded conservation, replay and fallback.

## Последствия

- SPEC-36 owns the detailed condition, capability, treatment, agency and LOD
  semantics; SPEC-37 owns the read-only 3D content and visual profile semantics.
- SPEC-18 gains a qualitative read-only body-status projection.
- A future implementation touches RPG, Mechanics, Physical Embodiment, Motor,
  AI, persistence, content and presentation and therefore requires one coherent
  production vertical, not isolated subsystem demos.
- Full organs, detailed vascular networks, arm/face injury, tactical NPC
  reactions, arbitrary cuts and true muscle actuation remain later separately
  gated breadth.
- The approved feature remains post-baseline/non-blocking in the current
  roadmap until an explicit release-scope decision promotes it.

## Supersession

This is the first Accepted decision for functional anatomy and character
embodiment; it does not supersede the unrelated ADR-074 R4d Strategic Agent
vertical. Moving durable condition outside RPG ownership, allowing
presentation/model output to commit injury, giving player and NPC different
semantic rules, removing the fixed-PD fallback or changing the approved product
promise requires a later ADR that explicitly supersedes ADR-075 and updates
SPEC-18/36/37, routing, traceability and roadmap.
