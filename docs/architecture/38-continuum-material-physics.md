# SPEC-38: Proposed continuum material physics

| Field | Value |
|---|---|
| ID | SPEC-38 |
| Status | Proposed |
| Version | 1.6 |
| Last verified | 2026-08-18 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-23](23-jobs-memory-resource-residency-and-io-backpressure.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-076](adr/076-continuum-material-physics-track.md), [ADR-081](adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md) |
| Candidate revision note | Version 1.6 binds the W0H fixed accelerated pressure algorithm and projected-KKT gate over unchanged W0F physical operations and W0G energy semantics while retaining ADR-081 promotion guardrails and Proposed status |
| Related Proposed tracks | [SPEC-43](43-thermochemical-material-processes.md), [SPEC-44](44-neural-assisted-world-simulation.md), [ADR-079](adr/079-thermochemical-material-process-track.md), [ADR-080](adr/080-neural-assistance-as-bounded-proposals.md) |

## Status and scope

This SPEC defines candidate semantics and promotion gates for bounded local
continuum materials. It does not authorize runtime schemas, alter the current
PhysX-only production implementation, or claim that water, sand, mud, soil or
snow is shipped. The umbrella work-package index is maintained in
[the continuum specification series](../plans/continuum-material-physics/README.md);
water execution is maintained independently in
[the water roadmap](../plans/continuum-water/README.md).

The first selected consumer is one sealed water basin containing one movable
PhysX crate and read-only debug-particle presentation. Dry terrain is a
separate numerical lane. Global ocean/weather, cross-region particle transfer,
adaptive resolution, broad gameplay queries, generic solver/plugin ABI, full
vehicle simulation and production sleep conversion are outside the first
water consumer.

All `CONTINUUM-*` checks remain `NOT_RUN`. Until a later consumer-backed
Accepted ADR narrows ADR-058 and promotes exact schemas, the production world,
save/replay formats and public contracts remain unchanged.

## Candidate authority and canonical water state

Physical Embodiment is the future owner of active continuum state. CPU DFSPH
is the sole candidate canonical water solver for V1. A GPU implementation is
an optional correspondence mirror and cannot emit authoritative commands,
events, checkpoints or body impulses.

One future active water region owns a bounded material-specific SoA. At every
accepted 240 Hz substep its complete future-affecting sample state is:

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
cannot survive the boundary. Production also requires exact Windows/Linux roots
on adversarial rounding and convergence cases. Failure after two remediation
cycles leaves this authority research-only or forces a separate fixed-point/
soft-float decision.

Material-specific active state remains separate: a future MPM solid needs
deformation gradient, affine velocity and plastic/internal variables in its
own SoA. A universal particle record is forbidden. Spatial hashes, temporary
MPM grids, matrices, render meshes, wetness textures and GPU buffers remain
reconstructible caches.

## Fixed water profile V1

The first profile is a clean, fixed-resolution baseline:

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

## One-pass rigid coupling candidate

Each physical substep has one staged coupling pass:

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

The first water vertical has exactly one sealed, pinned-active region. Its
analytical boundary admits no particle crossing. Halo exchange, cross-region
neighbor pairs, ownership transfer, camera-selected activation and active
eviction are forbidden.

First production persistence stores the exact active water state in the same
atomic physical owner checkpoint as the PhysX canonical state. A sidecar
cannot commit, restore or fail independently. Save-at-N/resume-to-M must reach
the same canonical root as uninterrupted execution. Neighbor structures,
pressure scratch and render buffers are rebuilt.

That production profile defines a fixed positive checkpoint epoch. At every
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

Before activation, missing or rejected continuum capability selects an
authored dry basin variant. After activation, the runtime cannot silently
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

The V1 water roadmap still forbids warm start. SPEC-44 may evaluate a learned
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

## Presentation

V1 extracts bounded immutable sample records after an accepted physical
commit. Debug points/spheres and diagnostic overlays are the mandatory path.
Surface reconstruction, screen-space fluid, foam, spray and wetness are
non-authoritative optional stages and do not block first promotion. Renderer
cadence, camera state and device/cache loss cannot change a water root,
reaction batch or gameplay result.

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

The production fixture is one `4 × 2 × 1 m` sealed basin filled to `0.75 m`,
nominally `48,000` samples under the V1 spacing, with one `0.5 m` cube of mass
`50 kg`. Expected equilibrium immersion is `0.20 ± 0.05 m`. Player control
uses the production command path; deterministic headless consumes the same
recorded command trace without test-only mutation.

| Check | Required result |
|---|---|
| `CONTINUUM-WATER-REF-P1` | Exact sample count/mass; mean positive compression and projected-KKT residual each `<= 0.01%` within `2..=50` pressure-operator applications; no nonfinite/non-convergence; boundary-centre penetration `<= 2.5 mm`; normalized impulse residual `<= 1%`; W0G absolute energy drift `<= 1%` for reversible/control scenarios and positive energy excess `<= 1%` for named static-impact scenarios with deficit/stage accounting; mandatory hydro/dam-break/orifice reference RMSE `<= 5%` and maximum error `<= 10%`; repeat/insertion permutations have the same target-local root. |
| `CONTINUUM-COUPLING-P1` | One-pass reaction closure, crate float/impact and failure cases publish one complete composite result or none; no second rigid writer. |
| `CONTINUUM-PERSISTENCE-P1` | Exact active save/restart continuation matches uninterrupted roots; corrupt/stale/capacity cases fail before mutation. |
| `CONTINUUM-MIRROR-P1` | Optional GPU aggregate correspondence passes its predeclared metrics without an authority claim. |
| `CONTINUUM-TERRAIN-P1` | One calibrated dry-sand profile and prescribed wheel/terrain contact pass declared conservation and reference curves. |
| conditional `performance` | `50k` water meets the standalone THOTH `4/6 ms` p95/p99 stop target; before integration, the full combined workload must pass the successor mutually exclusive `world-dynamics-step` row measured across every substep in one gameplay tick. `10k` and `100k` remain report profiles, with `100k` not a production promise. |

The CPU oracle also compares published curves with aggregate output from an
independently executed SPlisHSPlasH revision. External solver code, generated
trajectories and heavy reports remain outside Git. Same-target exactness is the
research gate; Windows/Linux canonical-root equality is additionally required
before production promotion.

## Promotion boundary

No public contract is added for the serial lab. When the basin becomes a real
runtime consumer, the smallest candidate public set is
`ContinuumRegionDefinitionV1`, `ContinuumWaterProfileV1`,
`ContinuumWaterCanonicalStateV1`, `ContinuumBodyReactionBatchV1`,
`ContinuumPresentationSnapshotV1` and a composite successor to
`PhysicsWorldCheckpointV2`. Exact schemas require the later Accepted
promotion ADR and synchronized SPEC-02/03/21/25/26/30, routing, traceability
and roadmap updates.

Generic solver interfaces, raw particle/grid access for plugins and new broad
gameplay queries remain out of scope until a separate demonstrated consumer
requires them under ADR-046.
