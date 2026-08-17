# Physical embodiment and injury — current task state

| Field | Value |
|---|---|
| Status | `PRODUCT_SPEC_COMPLETE / IMPLEMENTATION_NOT_STARTED` |
| Updated | `2026-08-17` |
| Task key | `physical-embodiment-injury` |
| Scope | Approved functional-anatomy product direction, lower-limb damage/treatment vertical and final visible 3D embodiment architecture. |
| Definition of done | Product choices and acceptance examples are explicit; SPEC/ADR, routing/index/traceability/roadmap and validation encode them without claiming implementation. |
| Authority | Working context only; Accepted ADR/SPEC and `docs/roadmap.md` outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** Functional muscle groups are condition/capability
  abstractions over the existing joint-target fixed-PD path. They provide
  tissue-specific damage without a second movement controller or mandatory
  full musculoskeletal simulation.
- **Approved product:** A limb can remain attached but be structurally or
  neurologically unusable; ordinary player/NPC intention degrades into limp,
  load transfer, fall, crawl or drag. Treatment is stabilization → repair →
  rehabilitation through medicine or magic.
- **Visible target:** Third-person anatomically plausible character with
  `Reduced`, `Realistic` and `Graphic` presentation, qualitative body UI and a
  complete authored skinning/corrective fallback.
- **First vertical:** One neutral humanoid, one unilateral lower limb, intact/
  partial/tendon-or-nerve/stable-fracture/retained-fracture/detached states,
  staged treatment and one NPC using the same physical rules.
- **Scale target:** 16 nearby detailed, 64 active simplified, distant durable
  state-only; exact performance budgets are still Proposed.
- **Next action:** When scheduled, create an implementation plan for the exact
  lower-limb anatomy/content profile and consumer-driven schemas. Do not begin
  with neural deformation, organs or true muscle actuation.

## Product decisions

| Area | Decision |
|---|---|
| Fantasy | Functional anatomy for systemic gameplay, not medical simulation |
| Actuation | Existing joint-target + fixed safety/PD; tissue derives one capability envelope |
| Anatomy detail | Functional lower-limb groups plus consumed bone/tendon/nerve/vascular facts |
| Systemic condition | One bounded stable/impaired/critical/unconscious/dead ladder |
| Agency | Player retains intention; execution adapts; ragdoll only for declared physical/RPG cause |
| Parity | Same player/NPC rules, different fidelity tiers only |
| Treatment | Stabilize → repair → rehabilitate; medicine/magic share the owner path |
| Camera/UI | Third-person physical/visual cues plus qualitative body-status UI |
| Visual severity | Reduced/Realistic(default)/Graphic over identical gameplay state |
| First content | One unilateral lower limb; arms/organs/full-body breadth later |
| NPC slice | Physical adaptation, fall and crawl; tactical/social response later |
| Release | Approved post-baseline direction; not a current v1 gate |

## Authority decision

ADR-075 establishes the first Accepted functional-anatomy decision and preserves
these boundaries without changing the unrelated ADR-074 R4d authority:

- RPG owns durable local/systemic condition and treatment stage;
- Mechanics owns damage/treatment definitions and submits `WorldCommand`
  proposals;
- Physical Embodiment owns active physics/topology and derives capability;
- Motor consumes the capability envelope through the existing safety/PD path;
- presentation and UI are immutable read-only consumers;
- exact wire schemas remain Proposed until the production consumer exists.

## Current evidence

| Evidence | Result | Consequence |
|---|---|---|
| User product discovery, completed 2026-08-17 | `APPROVED` | Product promise, first vertical, treatment, parity, UI/visual and scale choices are closed. |
| `docs/product/functional-anatomy-and-character-embodiment.md` | `APPROVED_DIRECTION` | Provides player-facing brief and acceptance matrix. |
| `docs/architecture/adr/075-product-grounded-functional-anatomy-and-character-embodiment.md` | `ACCEPTED` | Establishes the product-grounded decision without changing ADR-074 R4d authority. |
| `docs/architecture/36-functional-tissue-condition-and-injury.md` | `ACCEPTED_SEMANTICS / PROPOSED_CONTRACTS` | Defines condition, deterministic damage, capability, agency, treatment and LOD. |
| `docs/architecture/37-character-embodiment-and-surface-deformation.md` | `ACCEPTED_SEMANTICS / PROPOSED_CONTRACTS` | Defines realistic third-person surface, severity profiles, asset fallback and workload. |
| Documentation-only validation | `PASS` | `git diff --check` and direct local-link/path/ID validation pass; ADR-074 and ADR-075 each retain one distinct Accepted decision. |

## Rejected approaches

### Full muscle actuation as the first implementation

- **Why rejected:** It multiplies actuator dynamics, calibration, observation,
  training, safety and replay cost without being necessary for the approved
  injury gameplay.
- **Reconsider when:** The bounded functional-group lower-limb vertical cannot
  produce one of its explicit behavior/readability requirements.

### Independent PD and muscle movement logic

- **Why rejected:** Two control authorities could disagree about available
  force. Condition must derive one capability envelope consumed by one motor
  safety path.
- **Reconsider when:** Only through a later Accepted ADR replacing ADR-075.

### Presentation-owned injury or recruitment

- **Why rejected:** Surface/camera cadence would become gameplay authority;
  net joint effort does not uniquely determine actual muscle activation.
- **Reconsider when:** Only through a later Accepted authority change.

### Arbitrary runtime cutting in the first vertical

- **Why rejected:** Authored break sites and topology/mesh variants make mass,
  constraints, replay and fallbacks bounded and testable.
- **Reconsider when:** The retained/detached authored matrix ships and a new
  player-visible consumer proves a need for arbitrary cuts.

## Remaining uncertainty

- Exact current-only condition/command/treatment/content schemas.
- Exact functional group count and reducer coefficients.
- Stable PhysX retained-fracture representation.
- Authored/procedural/trained split for limp and crawl.
- Per-tier timing/memory budgets and visual technique on target hardware.
- Whether a later release-scope decision promotes the vertical into v1.

## Smallest next action

Write a bounded implementation plan only when the work enters the roadmap WIP
slot. Start by freezing one BodySchema-bound unilateral anatomy profile and its
deterministic scenario corpus; then promote the smallest consumer-driven RPG/
Mechanics schemas before topology or advanced rendering work.
