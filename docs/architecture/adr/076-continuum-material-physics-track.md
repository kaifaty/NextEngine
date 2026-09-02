# ADR-076: Continuum material physics track

| Field | Value |
|---|---|
| ID | ADR-076 |
| Status | Proposed |
| Version | 1.4 |
| Decision date | 2026-08-16 |
| Last verified | 2026-09-02 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [SPEC-25](../25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-071](071-canonical-physics-material-lineage.md), [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md) |
| Candidate revision note | Version 1.4 binds the W0G reversible/static-impact energy semantics over unchanged W0F operations without promoting this Proposed track |
| Superseded by | Partially [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md): it supersedes the world-generation key, unprofiled private-float, unbounded continuation, implicit fault-domain and legacy-budget clauses. Partially [ADR-100](100-authoritative-water-volume-and-presentation-only-gpu-water.md) (Proposed): for water V1 it supersedes GPU correspondence-only, the crate-coupled first consumer and debug-points-only presentation. Partially [ADR-104](104-water-v1-authority-is-the-exact-table-and-flow-network.md): it supersedes the water clauses (CPU DFSPH as the canonical water candidate, the canonical particle water boundary, exact active-sample persistence and checkpoint epochs for water, the particle reaction batch as the water coupling source); the terrain clauses, the one-pass composite step and the rigid-writer authority remain. |

## Context

Next Engine needs bounded local water and, later, deformable terrain without
creating a second rigid-state writer, allowing renderer/GPU state to drive
gameplay or weakening exact save/replay. The current Accepted baseline uses
PhysX 5.9.0 as the only production rigid/articulation backend and admits public
contracts only for demonstrated production consumers.

The initial broad research identified different numerical needs for free
surface water and history-dependent terrain. It did not resolve the canonical
water state, substep composition, first persistence representation or product
scale. Those ambiguities would allow implementations with incompatible hidden
continuation state and partial failure semantics.

The selected first consumer is now concrete: one sealed `4 × 2 × 1 m` basin,
`0.75 m` water depth, at most `50,000` fixed-resolution samples, one movable
PhysX crate and read-only debug-particle presentation. This selection narrows
the Proposed program but does not promote it into the current runtime.

## Proposed decision

### One owner boundary, separate numerical lanes

Physical Embodiment SHOULD eventually own active continuum state alongside
the rigid/articulated world. CPU `f64` DFSPH is the sole canonical candidate
for water V1. APIC/MLS-MPM with one calibrated Drucker-Prager profile is the
separate first dry-sand lane. Wet/saturated terrain follows exact dry-terrain
persistence. A universal solver or universal particle record is rejected.

GPU DFSPH is optional correspondence-only work. It cannot write canonical
water, body impulses, saves, commands or events. Final quantization of a GPU
trajectory does not establish equivalent history and cannot promote it.

### Canonical water boundary

The private solver may use `f64` during a fixed substep, but the accepted state
contains only stable sample identity plus position in signed `i64`
micrometres and velocity in signed `i64` micrometres per second. Publication
uses one checked ties-to-even conversion, and the next substep reconstructs
private floats from that accepted state.

Before solver code, the authoritative path binds the exact ADR-081
`CanonicalFloatExecutionProfile`. Cross-target adversarial roots are a
production gate; two failed remediation cycles keep this path research-only or
require a separate fixed-point/soft-float authority decision.

Uniform sample mass is profile state. Density, pressure/divergence factors,
neighbor/grid structures, warm-start values and render buffers are rebuilt
caches. V1 disables warm start, adaptivity, surface tension, optional
viscosity/vorticity models and variable time step so that no unrecorded field
can affect continuation.

The exact successor baseline uses 240 Hz cadence, water density `1000 kg/m³`,
particle radius `0.025 m`, spacing `0.05 m`, cubic-spline support `0.1 m`,
sample mass `0.125 kg` and gravity `9.81 m/s²`. Density uses projected
diagonally preconditioned active-set PCG for `2..=50` iterations with mean
positive compression error `<= 0.01%`; divergence remains `1..=20` with mean
error `<= 0.1%`.

Static analytical geometry has one exact integer source shared by neighbor
visibility, two-layer `REST_VOLUME` density support, validation and swept
particle-radius contact. Internal plane patches have stable feature IDs and
closed rectangular openings; their support samples are oriented to one fluid
side. Contact executes after pressure and before integration, reduces impulse
per feature, and permits no positional repair or retry. The successor admits
at most `32,768` static boundary samples; future dynamic rigid samples require
a separate W3 capacity.

### One-pass composite PhysicalStep

For every substep the runtime freezes the prior rigid projection, solves water,
reduces one reaction batch, lets PhysX apply it and integrate once, then
publishes the water and rigid result as one composite PhysicalStep
transaction. There is no V1 coupling iteration, next-step delayed reaction or
completion-order-selected result.

The future engine-owned reaction record binds full `PhysicsBodyIdV1`, expected
world/body revisions, tick/substep, region/profile/prior-state roots,
fixed-point linear/angular impulse and frozen centre of mass as torque
reference. One canonical record remains per body after stable reduction.
Exactly one ADR-081 exchange tuple is allowed, including namespace, both owner
IDs, applicable world ID, expected revisions/roots, tick/substep, edge profile,
participants and operation slot; any duplicate or conflicting result rejects
the complete step. World/save/checkpoint generation is not an exchange key.

PhysX remains the sole writer of rigid/articulation transforms and velocities.
Production requires a later Accepted ADR narrowly superseding the applicable
ADR-058 wording while retaining this authority split.

### Sealed region, persistence and fallback

Water V1 uses one sealed, pinned-active region. Particle transfer, halos,
cross-region coupling and streaming eviction are absent. First persistence
serializes exact active samples inside the same composite physics checkpoint;
an independently committed sidecar is forbidden.

The production profile uses fixed scheduled checkpoint epochs to reconstruct
and validate a fresh PhysX scene even when no save was requested. Save waits
for the same barrier, which uninterrupted comparison runs also execute.

Sleep conversion is a separate optional lossy transition and is not on the
water critical path. It requires versioned receipts and repeated-cycle bounds
before use. Cross-region transfer requires another specification and cannot be
inferred from this decision.

Worst-case capacities are admitted before freeze; an expressible denial is
ordinary and exhaustion beyond the admitted bound is an invariant fault.
Missing/rejected capability before activation loads an authored dry basin
variant. After activation, silent switch to dry/decorative water, GPU authority
or frozen water while PhysX advances is forbidden. A fatal active failure
retains the last complete checkpoint and stops the affected physical run.

### Materials, contracts and presentation

`PhysicsMaterialDescriptorV2` remains the exact solid-contact descriptor.
Continuum constitutive profiles and rigid-boundary coupling mappings are
separate future consumer-backed schemas. No public continuum type is added for
the lab.

Debug points/spheres and diagnostic overlays are sufficient for the first
water vertical. Surface reconstruction, foam, spray and wetness remain
reconstructible optional presentation and never feed collision or gameplay.

### Terrain dependency ladder

The first terrain consumer is one prescribed instrumented wheel over dry sand.
For a declared deformable pair, MPM owns contact exclusively and the matching
PhysX ground pair is disabled. Exact dry state and contact evidence precede
exact persistence; saturation/drainage follows; a closed free-water/terrain
flux batch follows only after both owner lanes pass independently. Full
vehicle, generic soil, snow, clay, lossy sleep and two-phase poromechanics are
not implied by the first profile.

## Failure semantics

Invalid profile/content, nonfinite value, conversion overflow, capacity
excess, non-convergence, boundary escape, stale/missing body revision, result
collision, conservation violation, PhysX rejection or corrupt checkpoint
publishes neither water nor rigid state. The previous complete generation
remains authoritative. Retry-to-green, partial result, mid-run fallback and
separate sidecar recovery are forbidden.

The first primary gameplay profile faults the whole application session via
the ADR-081 state machine; only independently provisioned test/training scenes
may isolate a narrower slot.

The offline serial oracle stops at its first failed substep and reports a typed
bounded failure. It cannot continue a trajectory from a retained prior frame
and present it as successful evidence.

## Product impact and promotion

The water program has an independent roadmap and remains
`PLANNED / NOT_ACTIVE` in the main R8 roadmap. `CONTINUUM-WATER-REF-P1` first
proves the serial oracle; only a PASS permits the main roadmap to activate the
integration track. A later production proposal must additionally prove:

- exact same-target repeat and insertion-order roots;
- published/reference and independent-solver aggregate error bounds;
- `50k` standalone performance at the THOTH `4/6 ms` p95/p99 stop target and
  the full combined successor `world-dynamics-step` row; `100k` is
  stress/report-only;
- one-pass crate float/impact and failure atomicity;
- exact active save/restart continuation;
- (retired by ADR-090/ADR-104) Windows/Linux canonical-root equality before
  production promotion;
- identical `game`/`headless` semantics and the authored dry pre-activation
  fallback.

If `50k` misses its standalone stop target or the successor combined budget
after two evidence-backed optimization cycles, the track remains research-only. GPU authority, a smaller production
sample gate or a larger total budget requires an explicit new decision.

## Considered alternatives

- One SPH formulation for every material — rejected because elastoplastic
  terrain requires distinct history and transfer semantics.
- GPU-first authority — rejected because order-dependent execution is not a
  cross-target replay contract.
- Persist density, pressure or warm-start scratch — rejected for V1 because
  the bounded clean-water profile can reconstruct them and hidden continuation
  would enlarge determinism and migration scope.
- Iterative or delayed rigid coupling — rejected for V1 because the selected
  basin/crate consumer does not yet justify another fixed iteration profile.
- Decorative water after active failure — rejected because presentation would
  conceal a missing physical fact.
- Lossy sleep before exact active persistence — rejected because uninterrupted
  and restored representations would no longer have an exact compare point.
- Extend `PhysicsMaterialDescriptorV2` — rejected because solid contact and
  continuum constitutive state have different owners and lifecycles.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `CONTINUUM-WATER-REF-P1` | Hydrostatics, dam break, free fall, still tank, drain and analytical boundaries | Fixed W0F solver plus W0G scenario-class energy/reference bounds and exact same-target roots pass | Keep the program offline; load authored dry basin |
| `CONTINUUM-COUPLING-P1` | One-pass water reaction against the crate plus stale/capacity/failure cases | One composite result or none; PhysX remains the only rigid writer | Do not activate the water region |
| `CONTINUUM-PERSISTENCE-P1` | Exact active save/restart and corrupt/incompatible inputs | Uninterrupted and restored roots match exactly | Retain prior save; do not activate the water region |
| `CONTINUUM-MIRROR-P1` | Optional CPU/GPU corpus | Declared aggregate correspondence without authority claim | CPU remains the only candidate authority |
| `CONTINUUM-TERRAIN-P1` | Calibrated dry-sand collapse/shear/sinkage/single-wheel corpus | Conservation, curves and exclusive contact pass | Use accepted rigid terrain |
| conditional `performance` | `10k/50k/100k` on exact THOTH profile | `50k` passes current ceilings; `100k` reports only | Stop promotion or revise scope through a later decision |

## Consequences

- SPEC-38 and this ADR remain Proposed; no current crate, schema, save format,
  ProductCheck result or PhysX contract changes.
- The first implementation is a private serial Rust oracle with external
  evidence, not a runtime subsystem.
- The water roadmap and the umbrella terrain series can progress independently.
- Public contracts are introduced only with the basin consumer under ADR-046.
- Production promotion must update SPEC-02/03/21/25/26/30, routing,
  traceability and the main roadmap in one coherent Accepted change.
