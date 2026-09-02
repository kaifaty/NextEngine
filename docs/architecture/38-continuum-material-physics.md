# SPEC-38: Proposed continuum material physics

| Field | Value |
|---|---|
| ID | SPEC-38 |
| Status | Proposed |
| Version | 2.2 |
| Last verified | 2026-09-02 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-23](23-jobs-memory-resource-residency-and-io-backpressure.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-076](adr/076-continuum-material-physics-track.md), [ADR-081](adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md), [ADR-100](adr/100-authoritative-water-volume-and-presentation-only-gpu-water.md), [ADR-103](adr/103-authoritative-water-flow-network.md), [ADR-104](adr/104-water-v1-authority-is-the-exact-table-and-flow-network.md) |
| Candidate revision note | Version 1.9 records the first R8d increment of [ADR-103](adr/103-authoritative-water-flow-network.md): `WaterFlowNetworkV1` inside the physics world checkpoint (schema version 3), the flow command kind and `CONTINUUM-WATER-FLOW-P1 = PASS`; version 1.8 recorded the first R8c increment of [ADR-100](adr/100-authoritative-water-volume-and-presentation-only-gpu-water.md): `WaterVolumeSetV1` inside the physics world checkpoint, the water level command and `CONTINUUM-WATER-VOLUME-P1 = PASS`; the authority split, density-only boundary support, presentation surface path, W0H CPU reference lane and ADR-081 guardrails are unchanged from 1.7 |
| Related Proposed tracks | [SPEC-43](43-thermochemical-material-processes.md), [SPEC-44](44-neural-assisted-world-simulation.md), [ADR-079](adr/079-thermochemical-material-process-track.md), [ADR-080](adr/080-neural-assistance-as-bounded-proposals.md) |

## Status and scope

This SPEC defines candidate semantics and promotion gates for bounded local
continuum materials. It does not authorize runtime schemas, alter the current
PhysX-only production implementation, or claim that water, sand, mud, soil or
snow is shipped. The umbrella work-package index is maintained in
[the continuum specification series](../plans/continuum-material-physics/README.md);
water execution is maintained independently in
[the water roadmap](../plans/continuum-water/README.md).

The first selected consumer is one sealed water basin with an authoritative
`WaterVolume`, player wading/swimming classification from its queries and
live presentation dynamics on its free surface (ADR-100). The movable PhysX
crate and any continuum reaction batch belong to a later consumer under its
own ADR. Dry terrain is a separate numerical lane. Global ocean/weather, cross-region particle transfer,
adaptive resolution, broad gameplay queries, generic solver/plugin ABI, full
vehicle simulation and production sleep conversion are outside the first
water consumer.

`CONTINUUM-WATER-VOLUME-P1` (R8c) and `CONTINUUM-WATER-FLOW-P1` (R8d)
pass through `xtask water-volume` and `xtask water-flow`; the remaining
product checks are `CONTINUUM-WATER-PRESENT-P1` and
`CONTINUUM-WATER-BUOYANCY-P1` (ADR-104). The research checks below are
reports, not promotion gates. The authoritative water table lives in the physics world
checkpoint under Proposed ADR-100 and SPEC-26 2.6; the particle lanes below
still change no production world, save/replay format or public contract.

## Candidate authority and canonical water state

Physical Embodiment owns the authoritative water. Under ADR-100, ADR-103 and
ADR-104 water V1 has two owners with one direction of flow, and the
authority ladder is closed: the exact integer table plus the exact flow
network are the only canonical water, particle water is presentation only
for the whole of V1, rigid coupling reads exact cell levels through the
one-pass reaction batch, and the particle lanes remain research oracles
and calibration sources. No particle state is canonical, saved, replayed,
queried by gameplay or reduced into a rigid reaction.

- **`WaterVolume` is the authoritative water.** A bounded set of sealed
  regions in the portable CPU core, each binding an exact integer extent, a
  still-water level, an optional authored level schedule and a profile
  revision. Submersion depth, wading/swimming classification and any later
  buoyancy read only this volume through SPEC-26 queries. It changes only
  through validated `WorldCommand` transactions, publishes exact roots,
  saves and replays with the world on `game` and `headless`, and needs no
  floating-point execution profile.
- **Presentation dynamics are non-authoritative.** A presentation stage may
  animate the free surface with any solver, including the Nonlocal GPU
  candidate, and publish a bounded height-field surface to the renderer
  through the ADR-101 dynamic surface path. No command, event, query, save,
  reaction or root reads it; renderer cadence, device loss, GPU vendor,
  missing capability or a stopped presentation process fall back to the
  still surface of the authoritative level. `headless` never executes it.

CPU DFSPH and the Nonlocal GPU lane remain the research oracles for
particle-water fidelity and the calibration source of the network's
profile constants (discharge coefficients, exit speeds, boundary support);
neither is a shipping promise, and an authoritative particle solver would
be a new ADR, not a continuation of ADR-076.
A GPU particle solver cannot emit authoritative commands, events,
checkpoints or body impulses; GPU authority would require a new ADR with a
GPU `CanonicalFloatExecutionProfile`.

Research form only (ADR-104; nothing below is canonical, saved or replayed):
a future active water research region owns a bounded material-specific SoA.
At every accepted 240 Hz substep its complete future-affecting sample state
is:

```text
WaterSampleCanonicalStateV1 {
  sample_id: SampleId,
  position_micrometres: [i64; 3],
  velocity_micrometres_per_second: [i64; 3],
}
```

`SampleId`, region/profile revisions, tick/substep, exact sample count and
canonical state root are exact. Uniform particle mass belongs to the immutable
water profile. Density, pressure/divergence factors, neighbor cells/pairs,
warm-start accumulators and reduction scratch are reconstructed every substep
and are neither owner state nor continuation state.

The solver may decode the prior canonical integers and use private `f64`
within one substep only under an exact ADR-081 `CanonicalFloatExecutionProfile`
that fixes target/toolchain features, FMA, rounding/subnormal behavior,
mathematical primitives, reduction/factorization/tie order and convergence
branches. Publication performs exactly one checked
round-to-nearest-ties-to-even conversion to the SPEC-21 fixed-point boundary.
The next substep starts only from that published canonical state. Nonfinite,
overflow or out-of-range conversion rejects the candidate; private float state
cannot survive the boundary. Same-target exactness on the Linux reference
host is the gate (ADR-090); no Windows requirement exists. Failure after two
remediation cycles leaves this research authority research-only or forces a
separate fixed-point/soft-float decision.

Material-specific active state remains separate: a future MPM solid needs
deformation gradient, affine velocity and plastic/internal variables in its
own SoA. A universal particle record is forbidden. Spatial hashes, temporary
MPM grids, matrices, render meshes, wetness textures and GPU buffers remain
reconstructible caches.

## Research appendix: fixed particle water profile

Research lane only (ADR-104): this profile calibrates the presentation
solvers and the network constants and is not a promotion gate. The first
profile is a clean, fixed-resolution baseline:

| Property | V1 value |
|---|---:|
| Coordinate system | SPEC-26 right-handed MKS, `+Y` up |
| Rest density | `1000 kg/m³` |
| Particle radius / spacing | `0.025 m` / `0.05 m` |
| Cubic-spline support radius | `0.1 m` |
| Uniform particle mass | `0.125 kg` |
| Fixed cadence | `240 Hz` (`1/240 s`) |
| Gravity magnitude | `9.81 m/s²` toward `-Y` |
| Density solve | cold-started, diagonally scaled accelerated projected gradient with fixed step `0.25`, momentum `(iteration-1)/(iteration+2)` and one pressure-operator application per iteration; minimum `2`, maximum `50`; mean positive compression and mean projected-KKT error each `<= 0.01%`; directional curvature `<= 4` |
| Divergence solve | minimum `1`, maximum `20`, mean error `<= 0.1%` |
| Hard active capacity | `50,000` samples |
| Static analytical-boundary capacity | `32,768` samples; dynamic rigid samples use a separate later capacity |

V1 enables no warm start, surface tension, viscosity/vorticity model,
adaptive split/merge or variable time step. Analytical plane and box
boundaries are part of the reference solver rather than deferred to rigid
coupling. A capability/profile mismatch fails before region activation.

Every particle-water candidate, reference or presentation, inherits one
boundary rule (ADR-100, evidence D-047/D-048): floor and wall
neighbourhoods carry boundary density support, and fixed boundary samples
contribute to density and the incompressibility term only, never to
viscosity or surface terms. Analytic contact without density support lets a
floor monolayer compress in plane until it carries no pressure and stalls.

W0H changes only the algorithm used to solve W0F's existing non-negative
pressure quadratic program. A zero inverse-diagonal coordinate has multiplier
fixed to zero while its unscaled residual remains in both convergence metrics.
The step is fixed: backtracking, adaptive selection and hidden pressure
continuation are not part of the profile. W0F geometry/support/contact and W0G
energy semantics remain immutable parent inputs.

The W0F successor represents static density support as a two-layer
`REST_VOLUME` lattice complement. Internal axis-aligned plane patches use
stable feature IDs, closed rectangular openings and side-oriented support so
that one side cannot see the other side's ghost samples. The same exact
integer geometry filters fluid-neighbor visibility, generates density support,
classifies openings and supplies swept-sphere contact features. Contact runs
after pressure and before integration, resolves outer faces plus internal
faces and aperture edges in stable feature order, reports per-feature impulse,
and performs no post-integration clamp, retry or positional repair.

## One-pass rigid coupling

The one-pass composite step below is the coupling path of every later
water consumer. For water V1 (ADR-104) the reaction source is the exact
network: buoyancy and drag from the exact cell level and the body's exact
submerged geometry (`CONTINUUM-WATER-BUOYANCY-P1`). The particle reaction
reduction (steps 2-5 as written for samples) is the research form of the
same pass. Each physical substep has one staged coupling pass:

1. freeze the prior canonical water state and exact rigid projection;
2. reconstruct water caches in stable `(cell key, SampleId)` order;
3. solve water and analytical/moving-boundary constraints;
4. reduce sample reactions into one canonical body-reaction batch;
5. validate the complete water candidate and batch;
6. let PhysX apply the batch and integrate rigid bodies exactly once;
7. publish water and rigid state as one PhysicalStep transaction, or publish
   neither;
8. extract presentation only after the complete physical commit.

The future reaction record is engine-owned and includes the complete
`PhysicsBodyIdV1`, expected world/body revisions, physics tick/substep,
region/profile/prior-state roots, fixed-point linear and angular impulse, and
the frozen body centre of mass as the explicit torque reference. Records sort
by complete body identity and reduce to at most one record per body.

Exactly one batch may exist for the ADR-081 identity tuple: `world_namespace`,
source/destination owner IDs, applicable destination `world_id`, expected
source/destination revisions and roots, tick/substep, edge profile, region and
body participant IDs and operation slot. An exact
duplicate or different result under the same key is a result collision and
rejects the whole uncommitted step. No arrival-order winner, retry, coupling
iteration selected by wall time or delayed reaction on the next substep is
permitted in V1.

PhysX remains the only writer of rigid/articulation transforms and velocities.
The continuum solver writes only its region candidate and reaction proposal.
A production promotion requires a later Accepted ADR narrowing ADR-058 while
preserving that rigid-body authority.

## Region, persistence and fallback

Water V1 persistence is the exact table and network inside the physics
checkpoint (SPEC-26 2.6, `CONTINUUM-WATER-VOLUME-P1`,
`CONTINUUM-WATER-FLOW-P1`); the particle region, particle persistence and
checkpoint-epoch clauses below are the research form and bind no product
consumer (ADR-104). The first particle research vertical has exactly one
sealed, pinned-active region. Its
analytical boundary admits no particle crossing. Halo exchange, cross-region
neighbor pairs, ownership transfer, camera-selected activation and active
eviction are forbidden.

Water V1 persistence and fallback are the exact table and network inside
the physics checkpoint: a corrupt or unsupported segment rejects before
mutation, and a world without a network keeps its authored levels (R8c
behaviour); no capability is needed. The paragraphs that follow are the
research form of particle persistence.

Research form: first production persistence would store the exact active
water state in the same atomic physical owner checkpoint as the PhysX
canonical state. A sidecar
cannot commit, restore or fail independently. Save-at-N/resume-to-M must reach
the same canonical root as uninterrupted execution. Neighbor structures,
pressure scratch and render buffers are rebuilt.

Research form: that production profile would define a fixed positive
checkpoint epoch. At every
epoch, with or without a save request, it reconstructs and validates a fresh
PhysX scene from canonical state before atomic replacement. Saves publish only
after this barrier, and uninterrupted comparison runs execute the same barrier.

Active-to-sleep conversion is a later optional lossy model transition. It is
not required by the water roadmap and cannot claim owner-root equality with
the exact active representation. A future conversion profile must carry a
receipt for mass, momentum, volume/surface and material-history error and pass
repeated sleep/wake cycles before streaming can use it.

Worst-case sample, neighbor, pair, reaction-batch and rigid-participant
capacities are admitted before freeze. User-expressible denial is an ordinary
activation/action rejection; exhaustion after admission is an invariant fault.

Research form: before activation, missing or rejected continuum capability
would select an authored dry basin variant. After activation, the runtime cannot silently
substitute decorative water, freeze the water while PhysX advances, switch to
GPU authority or downgrade to the dry variant. An active fatal failure retains
the last complete checkpoint and stops the affected physical run with a typed
diagnostic.

`PhysicsMaterialDescriptorV2` remains the solid-contact descriptor. A future
consumer-backed `ContinuumMaterialProfile` is separate, and an explicit
coupling-material mapping binds an exact rigid surface/material revision to
the continuum boundary response. Neither schema is current in this revision.

Temperature, composition, water/ice phase and reaction progress are not added
to the base water particle state. SPEC-43's separately promoted
Thermochemical owner owns those fields on stable material-parcel attachments.
A later water/ice consumer must atomically bind parcel inventory/enthalpy to
the continuum region and pass `THERMOCHEM-CONTINUUM-P1`; neither owner may
derive an independently mutable copy.

Research form: the particle water roadmap still forbids warm start. SPEC-44
may evaluate a learned
warm-start proposal only as an independent post-promotion report/shadow branch
after the classical reference, production and exact-persistence gates pass. It
cannot change W1-W6, production initialization or work, CPU authority, GPU
correspondence status, stopping rules, failure classification or canonical
roots. Runtime advice requires the later certificate-backed Accepted decision
required by ADR-081.

## Other material lanes

Dry deformable terrain begins only with one calibrated Drucker-Prager sand
profile implemented through APIC/MLS-MPM. The MPM lane exclusively owns a
declared deformable wheel/terrain or foot/terrain contact pair; the matching
PhysX ground contact is disabled, and MPM emits the sole bounded reaction
batch. The first consumer is an instrumented prescribed single-wheel rig, not
a complete vehicle.

Exact active terrain persistence follows dry-sand/contact evidence and
precedes saturation. Wet material then advances through saturation/drainage,
mechanical response, and only later a closed atomic free-water/terrain flux
batch. Cross-region transfer, lossy sleep and two-phase poromechanics are
separately gated later work.

## Water tiers and practices (ADR-104)

Water scales by tiers, never by particle count. The survey of shipping
engines and games (`docs/development/water-engines-research-2026-09-02.md`)
shows the same split everywhere: a level or height/shallow-water field for
the body, a bounded particle or 2D system for active water, rendering for
the look. These practices bind every later water increment:

1. **Lattice cells are the large-body tier (planned, ADR-103 0.3).** A map-wide body is a
   `WaterFlowNetworkV1` whose cells form a regular lattice over the
   terrain and whose edges are `Open` sills to the four neighbours (the
   Timberborn column model and the virtual-pipes model, made exact).
   Floods, channels and terrain-following water cost one edge evaluation
   per wet neighbour pair, are saved with the world and need no float
   profile. The lattice is authored per region with a declared cell size
   and cell count bound; it is not global and it is not adaptive. The
   lattice increment needs its own bounds (cells and edges per region; the
   first increment admits `64` cells and `256` edges per world) and a rule
   for face-sharing cells, because `WaterVolumeDefinitionV1` extents are
   closed intervals and face-sharing volumes currently reject as
   overlapping.
2. **Only active water steps (planned).** A cell whose stored volume and every
   incident edge state are unchanged from the previous tick, and whose
   neighbours are likewise unchanged, is at rest and is skipped; a
   command or a neighbour change wakes it. Rest is exact (no flux), so
   skipping changes no root. The activity set is derived state, rebuilt
   on restore, never saved; a skipped edge records flux `0`.
3. **Presentation is spawned at edges (planned).** Jets, falls, splashes and foam
   are spawned by the presentation stage from the exact flux of a gate,
   sill, pipe mouth or waterfall edge and return visually to the
   destination cell's surface; the authoritative volume never leaves the
   network (the hybrid rule of Chentanez and Müller applied across the
   authority split).
4. **Rotational flow is presentation (planned).** Whirlpools, eddies and flow
   lines come from either a presentation-only shallow-water grid (height
   plus 2D velocity) fed by the network's levels and edge fluxes, or an
   authored vortex field around a `Sink` edge for the particle pass.
   Gameplay effects of a whirlpool (pull, damage, transport) read the
   exact edge flux and the cell geometry, never the visual field.
5. **Waves are a layer (planned).** Ripples, wakes and wind waves are a
   presentation layer on the cell surfaces (wave packets or surface
   wavelets over the height field), artist-controlled, with no gameplay
   reading.
6. **Exact queries, no readback.** Gameplay, AI and audio read levels,
   volumes and fluxes through the SPEC-26 queries on the committed
   network; presentation stages read `effective_level` of the committed
   physics checkpoint at the published tick (and later `edge_flux`),
   nothing else. Nothing reads a presentation texture, particle set or grid
   back into gameplay; a presentation stage that cannot run degrades the
   look only.
7. **One writer per substance.** The network is the only writer of
   water volume; presentation stages are pure functions of the committed
   network plus their own reconstructible caches.

## Presentation

Debug points/spheres and diagnostic overlays remain the mandatory research
presentation of any particle lane. The candidate free-surface presentation
(ADR-100/ADR-101) is a top-down height field over a fixed pixel grid of one
quarter particle spacing: sphere-cap projection, every 8-connected
component of at least one particle footprint, one 3x3 close, local fill, a
5x5 grayscale closing of the height and one bilateral pass with range sigma
equal to the particle radius. It
publishes one bounded vertex/index update per frame into a declared dynamic
surface ring whose catalog mesh keeps identity, material and bounds; the
render-content catalog, snapshot and frame plan are built once per run. GPU
and CPU implementations are equivalent when masks and mesh counts are
identical and depths agree within `1 um`. The height field cannot represent
overhangs or spray; screen-space fluid, foam, spray and wetness remain
non-authoritative optional stages. Renderer cadence, camera state and
device/cache loss cannot change a water root or gameplay result.

## Failure semantics

Invalid content/profile, nonfinite value, fixed-point overflow, capacity
excess beyond a reserved bound, non-convergence, boundary escape, stale revision, missing body,
reaction collision, conservation violation, PhysX rejection or corrupt owner
segment rejects the complete candidate. The prior physical generation remains
the only published state. There is no retry-to-green, partial sample set,
water-only commit, rigid-only commit or mid-run fallback.

The first primary-gameplay profile faults the whole application session through
`Running -> Faulted -> DiagnosticSaved -> Closed | ExplicitRestore`. Only a
separately provisioned training/test scene may declare narrower isolation.

A serial research tool stops at the first such failure and emits a bounded
typed report. It does not keep the prior frame and continue, because that
would convert a failed trajectory into false evidence.

## Product scenario and checks before promotion

The product water scene is the reference basin (`WaterVolume`) and the two
flow vessels (ADR-103); the first coupling consumer adds one `0.5 m`
cube of mass `50 kg` whose expected equilibrium immersion is
`0.20 ± 0.05 m` from the exact level (`CONTINUUM-WATER-BUOYANCY-P1`).
Player control uses the production command path; deterministic headless
consumes the same recorded command trace without test-only mutation. The
`4 × 2 × 1 m` basin with `48,000` samples is the research scenario of
the particle lanes.

Product checks (ADR-104): water is promoted when the four
`CONTINUUM-WATER-*` rows pass on the reference host with pinned roots.

| Check | Required result |
|---|---|
| `CONTINUUM-WATER-VOLUME-P1` | Activate one sealed basin `WaterVolume`, query submersion at authored points, save/load and replay; exact roots on `game` and `headless`, queries never read presentation, level changes only through commands. |
| `CONTINUUM-WATER-FLOW-P1` | Step the two reference vessels through a gated pipe with a source and a sink; exact volume conservation every tick, drain within twice the analytic time, gate response within one tick, identical live and restored roots, stable rejections, no presentation read. |
| `CONTINUUM-WATER-PRESENT-P1` | Drive the basin surface from the presentation solver for a bounded window with capture; one catalog/snapshot/frame plan per run, declared ring capacity respected, gameplay roots identical with and without presentation, capture diagnostic only. |
| `CONTINUUM-WATER-BUOYANCY-P1` | One reference body over the exact cell level: buoyancy and drag delivered as one reaction batch through the one-pass composite step; identical roots on `game` and `headless`; the batch reads no presentation state; a body outside every cell receives no reaction. |

Research reports (frozen plans and evidence; calibration sources, not
promotion gates):

| Check | Required result |
|---|---|
| `CONTINUUM-WATER-REF-P1` | Exact sample count/mass; mean positive compression and projected-KKT residual each `<= 0.01%` within `2..=50` pressure-operator applications; no nonfinite/non-convergence; boundary-centre penetration `<= 2.5 mm`; normalized impulse residual `<= 1%`; W0G absolute energy drift `<= 1%` for reversible/control scenarios and positive energy excess `<= 1%` for named static-impact scenarios with deficit/stage accounting; mandatory hydro/dam-break/orifice reference RMSE `<= 5%` and maximum error `<= 10%`; repeat/insertion permutations have the same target-local root. |
| `CONTINUUM-PARTICLE-COUPLING-R1` (research, formerly `CONTINUUM-COUPLING-P1`) | One-pass reaction closure from a particle set, crate float/impact and failure cases publish one complete composite result or none; no second rigid writer. |
| `CONTINUUM-PARTICLE-PERSISTENCE-R1` (research, formerly `CONTINUUM-PERSISTENCE-P1`) | Exact active particle save/restart continuation matches uninterrupted roots; corrupt/stale/capacity cases fail before mutation. |
| `CONTINUUM-MIRROR-P1` | Optional GPU aggregate correspondence passes its predeclared metrics without an authority claim. |
| `CONTINUUM-TERRAIN-P1` (separate lane) | One calibrated dry-sand profile and prescribed wheel/terrain contact pass declared conservation and reference curves. |
| conditional `performance` (network) | The exact flow step at the record bounds (`64` cells, `256` edges) on the reference host in a release build: currently `183 us` per step (plan 07, G6 `50 us` not met); report-only until a bound is frozen with a consumer. |
| conditional `performance` | The presentation solver owns its own budget row: `48k` presentation water at the THOTH `4/6 ms` p95/p99 stop target on the reference host; a missed budget lowers surface cadence or sample count, never gameplay. An authoritative particle solver, if ever proposed, must additionally pass the successor mutually exclusive `world-dynamics-step` row across every substep in one gameplay tick. `10k` and `100k` remain report profiles. |

The CPU oracle also compares published curves with aggregate output from an
independently executed SPlisHSPlasH revision. External solver code, generated
trajectories and heavy reports remain outside Git. Same-target exactness is the
research gate; same-target exactness on the Linux reference host is the
only root gate (ADR-090).

## Promotion boundary

No public contract is added for the serial lab or the developer water
bridge. The runtime basin consumer now owns `WaterVolumeDefinitionV1`,
`WaterVolumeStateV1`, `WaterVolumeSetV1` with its `submersion_at` query,
`WaterVolumeCommandV1` and `WaterVolumeChangedV1` inside
`PhysicsWorldCheckpointV1` schema version 3 (SPEC-26 2.6), next to the
ADR-103 `WaterFlowNetworkV1`, `WaterFlowCommandV1` and
`WaterFlowChangedV1` of the flow network.
The public water contracts are these types with their record, query,
rejection and helper types (`WaterLevelRampV1`, `WaterSubmersionV1`,
`WaterFlowEdgeV1`, `WaterFlowEdgeKindV1`, `WaterFlowEdgeStateV1`,
`WaterFlowCellStateV1`, `WaterFlowStepV1`, both rejection enums and the
exact helpers). `ContinuumWaterProfileV1` for the presentation solver,
`ContinuumBodyReactionBatchV1` and `ContinuumPresentationSnapshotV1` follow
only the later presentation and buoyancy consumers;
`ContinuumWaterCanonicalStateV1` is a research tool type and never a
contract (ADR-104). Exact schemas require the later Accepted
promotion ADR and synchronized SPEC-02/03/21/25/26/30, routing, traceability
and roadmap updates.

Generic solver interfaces, raw particle/grid access for plugins and new broad
gameplay queries remain out of scope until a separate demonstrated consumer
requires them under ADR-046.
