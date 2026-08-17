# ADR-081: World-dynamics gap closure and promotion guardrails

| Field | Value |
|---|---|
| ID | ADR-081 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-17 |
| Last verified | 2026-08-17 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-38](../38-continuum-material-physics.md), [SPEC-39](../39-layered-physical-world.md), [SPEC-40](../40-structural-vegetation-physics.md), [SPEC-41](../41-world-substrate-composition.md), [SPEC-42](../42-arcane-substrate-and-physical-magic.md), [SPEC-43](../43-thermochemical-material-processes.md), [SPEC-44](../44-neural-assisted-world-simulation.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-059](059-event-sourced-physx-continuation-reconstruction.md), [ADR-076](076-continuum-material-physics-track.md), [ADR-077](077-layered-physical-world-and-living-structures-track.md), [ADR-078](078-world-substrate-and-arcane-physical-interaction-track.md), [ADR-079](079-thermochemical-material-process-track.md), [ADR-080](080-neural-assistance-as-bounded-proposals.md) |
| Related research | [Hash-bound source-paper manifest](../research/world-dynamics-source-papers.md) |
| Supersedes | Conflicting Proposed clauses in ADR-076 through ADR-080 concerning world-generation keys, same-step structural topology, unbounded exact PhysX continuation, runtime neural proposals and incompletely scoped failure/budget semantics |
| Superseded by | none |

## Context

SPEC-38 through SPEC-44 establish sound owner boundaries, typed exchange and
consumer-driven promotion, but their first composition pass left several
cross-document contracts open:

- exact PhysX continuation was required for an indefinitely running world even
  though ADR-059 reconstructs hidden TGS history only inside a bounded training
  episode;
- Arcane and Thermochemical owners were scheduled at the physical substep even
  though the current SPEC-02 profile permits that substep to write physical
  state only;
- pairwise couplers each required one PhysX integration without one exact
  composition DAG or stage-level transaction profile;
- structural fracture published durable PhysX topology in the solver step even
  though SPEC-26 requires a validated topology command;
- private `f64` trajectories claimed exact cross-target roots without an exact
  floating-execution profile;
- an admitted optional neural proposal could turn a default-path success into a
  fatal owner-step failure;
- proposed performance numbers did not yet own a row in the compositional
  gameplay budget;
- failure containment, capacity admission, exchange identity and several
  domain-specific conservation details were not closed.

The decisions below are promotion guardrails. They do not activate a world-
dynamics owner, change the current 12-stage profile, add a public schema or
alter current PhysX/save behavior. Exact contracts land only with a production
consumer under ADR-046.

## Decision

### A successor `WorldDynamicsStep` reuses stage 8

The first production profile containing a non-physical Arcane or
Thermochemical owner MUST introduce a successor schedule profile whose stage 8
is named `WorldDynamicsStep`. The number and relative order of the twelve
runtime stages remain unchanged. The current profile retains its physical-only
stage 8.

The successor manifest binds a closed owner set and exact read/write access
sets. It admits Physical Embodiment owners and only the explicitly activated
non-physical substrate owners. An owner not present in the manifest has no
empty/default segment and cannot execute at that stage.

One consumer-backed `WorldDynamicsCompositionProfile` MUST bind:

- exact owner/profile/prior-root closure;
- a complete acyclic edge dependency graph and stable topological order;
- one canonical inbound collection and reducer rule per destination field;
- fixed cadence and the relation between gameplay ticks and physical substeps;
- per-owner, per-edge and aggregate capacities reserved before freeze;
- candidate/result roots, exchange-receipt ordering and checkpoint segments;
- fault-domain and performance-profile identities.

Water reaction, Arcane wrench and any other rigid input are merged before the
one PhysX integration. PhysX contact/load output is then consumed by downstream
structural owners. No pairwise profile may integrate PhysX independently when
the composition manifest activates more than one edge.

### Stage-level publication is fail-stop, not rollback-to-continue

The stage protocol is:

```text
Freeze -> Prepare candidates -> Execute each owner once
       -> Validate all roots/receipts/capacities -> Publish all or fault
```

Runtime owns the coordinator and publication generation. Owners receive only
immutable prior projections and private candidate storage. A successful step
publishes all participating owner roots and receipts atomically.

The first production profile does not promise that a mutated private PhysX
adapter can be rolled back and gameplay can continue. A post-execution
validation/backend/invariant failure publishes no candidate root, retains the
last complete checkpoint as authority and transitions the declared fault domain
to `Faulted`. Full rollback-to-running requires a later exact clone/restore
proof and separate decision.

### Exact PhysX persistence uses deterministic checkpoint epochs

ADR-059 remains the bounded humanoid-training continuation solution and is not
generalized to an unbounded gameplay world. A production world-dynamics profile
that requires exact PhysX restart MUST define a positive fixed checkpoint-epoch
length in simulation ticks.

At every epoch boundary, independent of whether a user requested a save, the
live run reconstructs a fresh PhysX scene from the canonical owner closure and
atomically replaces the prior scene only after the declared continuation
witness passes. Save publication is legal only after that same scheduled
rehydration barrier; an earlier save request waits for the next boundary.

The epoch number is schedule state, not exchange identity. Epoch length,
rehydration inputs, resource bound, discontinuity metrics and worst-case cost
are hash-bound. The uninterrupted comparison run executes the same barriers.
If periodic rehydration cannot preserve required roots and physical-quality
bounds, the affected PhysX-coupled track cannot promote without a new backend
or an explicit change to exact-continuation semantics.

### Fault containment and capacity admission are explicit

Every composition profile binds one `WorldDynamicsFaultDomainProfile` with the
state machine:

```text
Running -> Faulted -> DiagnosticSaved -> Closed | ExplicitRestore
```

The first primary-gameplay-world profile maps a fatal world-dynamics failure to
the whole application session; Runtime/RPG cannot continue while required
physical or substrate owners are frozen. Independently provisioned training or
test scenes may isolate one declared scene slot. Terms such as "affected run",
"affected world" and "instance" cannot choose containment implicitly.

Worst-case content, topology and batch capacities validate before owner freeze.
User-expressible capacity denial is an ordinary action/activation rejection.
Post-freeze exhaustion is fatal only when it proves that the admitted profile
or implementation violated its reserved bound. Deterministic truncation is not
an implicit fallback and requires its own lossy profile.

### Exchange identity uses one exact tuple

Every world-dynamics record binds at least:

```text
world_namespace
source_owner_id
destination_owner_id
destination_world_id when applicable
expected_source_revision and expected_source_root
expected_destination_revision and expected_destination_root
tick and substep
edge_profile_id
stable source and destination participant IDs
operation_slot
```

`world generation`, checkpoint generation, save generation and an undefined
owner generation are forbidden exchange-key fields. A profile may add fields
but cannot replace any applicable identity/revision/root component above.

### Authoritative private `f64` requires an exact execution profile

Final fixed-point quantization alone does not establish deterministic float
history. Before a water or structural `f64` solver can become production
authority, its consumer MUST bind one `CanonicalFloatExecutionProfile` that
fixes:

- toolchain, target feature baseline and operation precision;
- FMA contraction, rounding mode and subnormal behavior;
- deterministic square-root and every other mathematical primitive;
- canonical neighbor, reduction, factorization and pivot/tie order;
- fixed iteration schedule or exact quantized convergence branches;
- worker-independent logical partitions and merge order.

Same-target serial evidence remains the research entry gate. Production also
requires exact Windows/Linux canonical roots over normal, boundary and
adversarial rounding/convergence cases. After two coherent remediation cycles,
failure to satisfy this contract keeps the solver research-only or requires a
fixed-point/soft-float authority decision.

### Structural fracture commits through Outcome

An endogenous structural solver failure at stage 8 publishes a bounded
`PendingFracture` fact in the structural candidate and proposes one stage-9
internal Outcome command. It does not create a durable PhysX body in the same
solver step.

The validated Outcome command owns the `PhysicsTopologyTransaction` and stable
derived-body identity. Its atomic result replaces the pending structural
component with the anchored graph plus one staged rigid body. The body becomes
active at the next physical substep. While pending, the component cannot split
again, downgrade, transfer or be simulated by both owners. A same-step topology
exception requires a later Accepted successor decision.

### Performance uses a successor compositional budget

PHYS-P4's `4/6 ms` training workload is evidence, not budget authority for
world dynamics. Before the first integrated consumer, a successor
`GameplayBudgetMatrix` MUST add one mutually exclusive `world-dynamics-step`
row and define the exact measurement window across all substeps in one gameplay
tick.

The gate includes PhysX, active continuum/structure/substrate owners,
canonical merges, validation and root publication in one combined workload.
Standalone solver measurements remain diagnostic or earlier-stage stop gates;
their PASS values are not summed into an integrated PASS.

### Neural world assistance remains shadow-only

N0 telemetry and N1 report/shadow proposals are allowed after the classical
owner's promotion. Model output cannot alter a production solver iterate,
preconditioner, active set, iteration count, failure class or root under
SPEC-44/ADR-080 as narrowed here.

Runtime N1 requires a later consumer-backed Accepted ADR and a mechanically
checkable admissibility certificate proving that every admitted proposal lies
inside a domain with the same canonical result and failure classification as
the default path. Corpus agreement alone is insufficient. Until that decision,
missing/model failure simply removes shadow telemetry and never faults gameplay.

### Domain-specific closure requirements

Before their first corresponding code stage:

- Arcane telekinesis freezes an analytical maximum-debit proof including body
  linear/angular speed bounds, application-point-to-CoM lever arm, force/free
  torque, cadence/duration, maintenance, conversion loss and every rounding
  bound. Insufficient reservation rejects before PhysX; `debit > slice` after
  freeze is an unreachable internal invariant.
- Thermochemical heat transfer stores one signed mutable sub-LSB residual per
  active interface. The residual is owner state, is included in roots and
  checkpoints and participates in the next canonical transfer. A deadband is
  allowed only through a separate explicit material law.
- The first standing structural tree maps every active collision proxy to one
  stable PhysX kinematic body/shape pair. A consumer-specific
  `StructuralContactLoadBatch` carries exact quantized impulse/moment and an
  equal-and-opposite receipt; current generic contact-event bounds are not
  silently interpreted as an exact load.
- One structural contact/load identity can contribute cut work at most once.
  The cut receipt allocates input work among rigid reaction, structural
  elastic/kinetic work, cell damage/fracture and declared dissipation.
- Thermochemical parcel split/merge/handoff uses one atomic attachment-topology
  transaction. Child IDs derive from the causal transaction plus canonical
  slots; the parent becomes a tombstone; species mass, enthalpy, reaction
  progress and mechanical mass close in one conservation receipt.

These are Proposed semantic shapes, not current public schema names. ADR-046
still requires the smallest exact consumer-backed contracts at promotion.

### Research provenance is closed or explicitly missing

Every source paper used to derive SPEC-38 through SPEC-44 is copied into the
research directory, hash-bound in its manifest and labelled non-normative.
An unavailable cited source is recorded as `MISSING_SOURCE` with the unresolved
filename and affected decision areas. Bare references must not imply that the
repository possesses or verified the missing document.

## Promotion consequences

- SPEC-38 through SPEC-44 and ADR-076 through ADR-080 remain Proposed unless a
  separate consumer-backed decision promotes them.
- No current crate, public schema, schedule, checkpoint or model lane changes
  merely because this guardrail ADR is Accepted.
- Serial fixed-point research oracles may proceed after their numeric/corpus
  blockers. Authoritative `f64` research additionally binds the candidate float
  profile before code.
- Coupled runtime work cannot pass its integration gate until schedule,
  composition, epoch, fault-domain and budget profiles are exact.
- Neural runtime advice remains prohibited; report-only research may continue.

## Alternatives rejected

- Replay PhysX from the beginning of an unbounded world: unbounded storage and
  restore cost.
- Save vendor-native PhysX state as portable authority: vendor/build coupling
  and no engine-owned migration semantics.
- Let a save request itself reset hidden physics state: save timing would alter
  gameplay; scheduled epochs avoid that authority leak.
- Pairwise owner commits or best-effort rollback: can expose partial roots or
  depend on mutation order.
- Continue Runtime/RPG while the required primary physics world is faulted:
  creates a knowingly inconsistent world.
- Treat capacity overflow from an admitted action as routine internal fatality:
  permits a deterministic content/gameplay denial of service.
- Same-step solver-created rigid topology: conflicts with the current validated
  topology-command boundary.
- Final quantization as the only float-determinism rule: cannot constrain
  upstream branches or iteration history.
- Corpus-only production neural warm starts: do not prove safety over all
  admitted reachable states.
- Reuse PHYS-P4 or add independent subsystem budgets: does not close the
  integrated gameplay budget.
