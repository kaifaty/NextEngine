# Physical embodiment and injury — current task state

| Field | Value |
|---|---|
| Status | `R8A_INTACT_TOPOLOGY_CONDITION_IMPLEMENTED / CROSS_CHECKS_PASS` |
| Updated | `2026-08-28` |
| Task key | `physical-embodiment-injury` |
| Scope | Current bounded intact-topology lower-limb condition/capability increment plus the remaining full injury/embodiment direction. |
| Definition of done | R8a has one BodySchema-bound unilateral profile, RPG/Mechanics condition and treatment path, derived Motor capability, identical player/NPC evidence and cross-check closure without claiming fracture or full embodiment. |
| Authority | Working context only; Accepted ADR/SPEC and `docs/roadmap.md` outrank this file. |

## Resume in 60 seconds

- **Current result:** ADR-098 promotes one exact unilateral left-knee profile,
  separate RPG `BodyCondition`, shared Mechanics compiler and derived Motor
  capability envelope through the existing fixed-PD path.
- **Observed matrix:** Player and NPC both produce intact `150000000`, partial
  `75001144`, tendon/nerve zero `0` and rehabilitated `150000000` µN·m positive
  knee effort; ten transitions repeat with digest `2ce8bed4…d65b5`.
- **Cross-checks:** workspace `host-check`, `play`, `persistence-replay`,
  `content-package` and the bounded `physical-character` extension pass on
  native Linux/Rust 1.97.1.
- **Approved product:** A limb can remain attached but be structurally or
  neurologically unusable; ordinary player/NPC intention degrades into limp,
  load transfer, fall, crawl or drag. Treatment is stabilization → repair →
  rehabilitation through medicine or magic.
- **Visible target:** Third-person anatomically plausible character with
  `Reduced`, `Realistic` and `Graphic` presentation, qualitative body UI and a
  complete authored skinning/corrective fallback.
- **Implemented boundary:** Intact/partial/tendon-loss/nerve-loss and staged
  medical/magical stabilization, repair and rehabilitation only; topology and
  tensor/action layout do not change.
- **Scale target:** 16 nearby detailed, 64 active simplified, distant durable
  state-only; exact performance budgets are still Proposed.
- **Next action:** Add stable lower-leg fracture as a separate increment with an
  explicit atomic RPG condition/Physics boundary and conservative procedural
  fallback. Do not fold in retained fracture, detachment, UI or learned routes.

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

## Current authority decision

ADR-075 establishes the product direction. ADR-098 now makes only the bounded
intact-topology R8a records and owner path current while preserving these
boundaries:

- RPG owns durable local/systemic condition and treatment stage;
- Mechanics owns damage/treatment definitions and submits `WorldCommand`
  proposals;
- Physical Embodiment owns active physics/topology and derives capability;
- Motor consumes the capability envelope through the existing safety/PD path;
- presentation and UI are immutable read-only consumers;
- the promoted R8a wire schemas are current-only under ADR-046; fracture,
  surface, UI, LOD and learned-route schemas remain Proposed.

## Current evidence

| Evidence | Result | Consequence |
|---|---|---|
| User product discovery, completed 2026-08-17 | `APPROVED` | Product promise, first vertical, treatment, parity, UI/visual and scale choices are closed. |
| `docs/product/functional-anatomy-and-character-embodiment.md` | `APPROVED_DIRECTION` | Provides player-facing brief and acceptance matrix. |
| `docs/architecture/adr/075-product-grounded-functional-anatomy-and-character-embodiment.md` | `ACCEPTED` | Establishes the product-grounded decision without changing ADR-074 R4d authority. |
| `docs/architecture/adr/098-bounded-intact-topology-functional-anatomy-condition-vertical.md` | `ACCEPTED / IMPLEMENTED` | Promotes only the unilateral intact-topology profile, condition/treatment operations and derived capability boundary. |
| `docs/architecture/36-functional-tissue-condition-and-injury.md` | `PARTIAL CURRENT` | R8a condition/capability subset is current; fracture, topology, locomotion adaptation and visible embodiment remain Proposed. |
| `docs/architecture/37-character-embodiment-and-surface-deformation.md` | `ACCEPTED_SEMANTICS / PROPOSED_CONTRACTS` | Defines realistic third-person surface, severity profiles, asset fallback and workload. |
| `physical-character` / `INJURY-CONDITION-P1` | `PASS` | Two subjects, ten transitions, exact intact/partial/zero/recovered effort matrix and repeated digest `2ce8bed4…d65b5`. |
| `play` | `PASS` | 32 ticks, 52 events, 23 RPG events; the existing offline loop remains complete. |
| `persistence-replay` | `PASS` | 20 ticks, two generations and exact current RPG owner replay closure. |
| `content-package` | `PASS` | Reference profile cooks/activates; creator-smoke omits it and stays valid. |
| `host-check` | `PASS` | Full workspace format, static analysis and tests pass on `x86_64-unknown-linux-gnu`, Rust 1.97.1. |
| Full `INJURY-EMBODIMENT-P1` | `NOT_RUN` | No fracture/topology, adapted locomotion, UI/surface or workload-LOD credit is claimed. |

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

- Stable PhysX retained-fracture representation.
- Authored/procedural/trained split for limp and crawl.
- Per-tier timing/memory budgets and visual technique on target hardware.
- Atomic stable-fracture condition/Physics transaction shape and conservative
  fallback.

## Smallest next action

Plan the stable lower-leg fracture increment. Freeze one authored break site,
one condition-to-Physics atomic transition and one no-partial-mutation failure
case; retain the current procedural controller as fallback. Do not expand into
retained fracture, detachment, generic cutting, UI or learned locomotion until
that boundary passes its own product check.
