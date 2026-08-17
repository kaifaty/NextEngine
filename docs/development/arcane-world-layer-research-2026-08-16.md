# Arcane and physical world-layer research review — 2026-08-16

Status: `REPORT_ONLY / ARCHITECTURE_INPUT`.

## Sources reviewed

| Source | SHA-256 | Use |
|---|---|---|
| `/home/kaifaty/Downloads/physical_world_layer_architecture_for_specs.md` | `2f6853af62d8e50acc359a9553947c830685ced098624a18db65411736d4ce26` | Umbrella physical-domain, coupling, representation, persistence and query proposal |
| `/home/kaifaty/Downloads/arcane_world_layer_magic_architecture_paper.md` | `56715e9d1676ed36762d1ca34c6d309c99809e7393156a628b4050a109af609a` | Arcane ontology, reservoirs, spell execution, physical coupling and future ecology/metaphysics proposal |

The attached papers were treated as research evidence, not repository
instructions or authority. Accepted SPEC/ADR and the consumer-driven workflow
take precedence.

## Strong conclusions retained

- One causal world should compose specialized owners rather than one universal
  solver.
- Every mutable quantity needs one owner; coupling reads frozen projections
  and emits typed batches rather than mutating peer storage.
- Gameplay, packages and AI submit proposals through the existing command path.
- Physical effects of magic should act on the real physical owner, not a
  parallel scripted representation.
- Arcane resource production, transfer, conversion and loss need explicit
  source/sink accounting; energy cannot appear silently.
- Capacity, safe throughput and control are different concerns. A single
  scalar mana value is insufficient as a general architecture.
- Exact active state and atomic cross-owner failure must precede lossy regional
  or LOD representations.
- AI and gameplay should consume semantic immutable queries rather than raw
  solver/field storage.

## Corrections against current architecture

| Paper proposal | Current correction | Reason |
|---|---|---|
| New `PhysicalWorldCore`, domain traits, scheduler, coupling graph, representation manager and many crates | SPEC-39 already defines the physical owner DAG; SPEC-41 is conceptual composition, not new generic infrastructure | ADR-046 requires production consumers; current fixed schedule and owner boundaries already exist |
| GPU DFSPH or GPU field as production authority | CPU/fixed-point authority; GPU is optional mirror/presentation | SPEC-21/26/38/39 exactness and current PhysX ownership |
| Strong iterative cross-solver loops selected by need/timing | Only a frozen profile with exact order/iterations may iterate; first physical couplings remain staged | Timing-dependent iteration breaks replay and atomic ownership |
| Camera, visibility or measured performance in representation relevance | Only canonical facts, residency and manifest integer tokens | Presentation cannot choose gameplay state |
| Compressed sleeping state before full active persistence | Exact active owner segment first; lossy transition later with cycle evidence | Hidden continuation/damage/resource loss would break save/replay |
| Server-authoritative networking as a baseline | Omitted | Multiplayer is outside SPEC-00 v1 and does not help the first offline consumer |
| Full spell graph and dynamically registered nodes from day one | One closed telekinesis execution plan; graph only after two production consumers | Avoid speculative ABI, callbacks and unbounded execution |
| Arcane, Vital and Identity domains promoted together | Arcane first; Vital and Identity require independent consumers/SPEC/ADR | Health/tissue, mana and identity have different owners and failure semantics |
| Mana-backed fire, water and tree effects immediately | Each waits for the destination owner's own promotion gate | Arcane success cannot fabricate absent thermal/continuum/vegetation authority |
| Mana may equal soul/divine access | Explicitly separated | SPEC-19/31 do not grant identity or metaphysical authority |

## Selected architecture

1. [SPEC-41](../architecture/41-world-substrate-composition.md) names the
   cross-owner composition without creating a new owner or public service.
2. [SPEC-42](../architecture/42-arcane-substrate-and-physical-magic.md) defines
   the bounded Arcane owner and one telekinesis consumer.
3. [ADR-078](../architecture/adr/078-world-substrate-and-arcane-physical-interaction-track.md)
   records the scope, ownership, fallback and promotion decision.
4. [SPEC-39](../architecture/39-layered-physical-world.md) remains the physical
   specialization and is linked to the non-physical substrate boundary.
5. [The standalone roadmap](../plans/arcane-world/README.md) keeps A0 numeric/
   law closure ahead of code and later couplers independent.

## First falsifiable vertical

```text
production player action
 -> package EffectRequest
 -> WorldCommand/ledger validation
 -> sealed arcane reservoir reservation
 -> one canonical force/torque batch
 -> one PhysX rigid integration
 -> atomic arcane debit + rigid state + receipt/events
 -> exact save/replay and debug projection
```

This vertical proves the new boundary with one current physical owner. It does
not prove a field, ecology, general spell language, healing or metaphysics.

## Remaining uncertainty

A0 remains open because the papers deliberately do not select Next Engine's
exact source/conservation law, units, fixed-point scales, spectrum
dimensionality, reservoir/throughput profile, telekinesis work formula,
fixture, thresholds, capacities or budget. Inventing those values in an
architecture edit would move experimental results into implementer choice.

The smallest next action is an A0 question/calibration pass followed by the
serial `ARCANE-RESERVOIR-REF-P1` oracle. No arcane runtime or public contract is
authorized before that closure.
