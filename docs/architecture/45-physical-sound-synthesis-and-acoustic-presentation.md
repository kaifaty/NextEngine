# SPEC-45: Proposed physical sound synthesis and acoustic presentation

| Field | Value |
|---|---|
| ID | SPEC-45 |
| Status | Proposed |
| Version | 0.1 |
| Last verified | 2026-08-26 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-071](adr/071-canonical-physics-material-lineage.md) |
| Related research | [Physical sound synthesis research, 2026-08-26](../development/physical-sound-synthesis-research-2026-08-26.md), [quality evaluation](../development/physical-sound-quality-evaluation-research-2026-08-26.md), [steel calibration](../development/physical-sound-steel-calibration-2026-08-26.md), [wood/glass calibration](../development/physical-sound-wood-glass-calibration-2026-08-26.md) |
| Replaces | None; first candidate revision |

## Status and decision boundary

This SPEC defines a candidate presentation layer that synthesizes source audio
from committed physical excitations. It does not add a current public schema,
runtime crate, save segment, `WorldDynamics` owner, roadmap work package or
shipping claim. The accepted clip-based `AudioSceneSnapshotV1` and
`AudioMixerV1` path remains the production baseline and the mandatory
fallback.

The candidate is intentionally narrower than a universal procedural-audio
system. Its first useful consumer is rigid-body contact sound: impact first,
then bounded rolling and scraping. Cloth, fluids, fire, fracture and biological
sound production are separate source-model tracks. They may reuse the same
bounded excitation and mixer interfaces after their own owners and evidence
exist, but they are not implied by the rigid modal vertical.

Non-normative implementation note (2026-08-26): an isolated P0/P0.5
laboratory now exists in `next_presentation::physical_sound_lab`. The `xtask
physical-sound-lab` command emits external 48 kHz audition WAVs, and the
reference demo can mix committed `Begin` contacts behind the explicit Cargo
feature `physical-sound-lab`. It uses separately screened 12-mode steel,
dry-hardwood-block and thick-glass-plate candidates, plus a provisional
adjacent-snapshot speed estimator because the current contact record lacks
impulse/effective-mass and material fields. The small heterogeneous CC0 screens
improved bounded descriptor sets but are not a controlled corpus, universal
material profiles or substitutes for human audition. This experiment does not
implement the candidate content records, does not satisfy a P1 ProductCheck and
does not relax the production block below. Disabling the feature preserves the
ordinary clip baseline.

The same isolated experiment now includes `xtask physical-sound-eval`. It
consumes only external hash-frozen WAV manifests, emits deterministic classical
signal/modal/spectrum/decay reports and can build a seeded blind A/B browser for
matched references. Its Q0 baseline and self/mismatch controls are evidence
tooling only: no reviewed real corpus or human calibration has run, every
matched candidate remains `NeedsHumanAudit`, and the command creates no public
content schema, runtime dependency or promotion evidence.

A production consumer requires a later Accepted ADR under ADR-046. That ADR
must freeze the exact engine-owned projection, content records, limits,
reference numeric profile and ProductChecks. Until then all record shapes and
check IDs below are illustrative candidate semantics.

## Product intent and non-goals

The product goal is more coherent reactive sound for physical interactions:
the same object should respond continuously to where and how it was struck,
rolled or scraped without selecting one event-specific recording from a large
sample table.

The candidate MUST NOT:

- make PCM, mixer state, audio device state or propagation output authoritative;
- derive gameplay hearing from a waveform or presentation voice admission;
- create a second writer for rigid transforms, contacts, materials or topology;
- solve an acoustic pressure field or finite-element eigenproblem at audio rate;
- promise that one small set of material constants synthesizes every acoustic
  behavior;
- require neural inference, network access or an optional propagation SDK;
- remove authored clips, speech, music, ambience or declared failure fallbacks;
- present cloth, liquids, fire, footsteps, vocal tracts or birds as automatic
  consequences of a rigid modal resonator.

## Ownership and data flow

The authoritative and presentation paths remain separate:

```text
committed semantic/world facts
    -> deterministic AcousticFactV1
    -> gameplay hearing / AI perception

committed physics contact batch + exact immutable content revisions
    -> PhysicalSoundExtraction (presentation only)
    -> bounded excitation batch
    -> source synthesis voices
    -> attenuation / optional propagation / spatialization
    -> AudioMixerV1-compatible PCM / device
```

Physics remains sole owner of body state and contact facts. Assets owns
immutable cooked acoustic content. Presentation owns only reconstructible
extractor, resonator, voice, propagation and mixer state. No arrow returns from
the presentation path to Physics, Runtime, RPG, AI state or `WorldCommand`.

| Layer | Input authority | Output | Persistence |
|---|---|---|---|
| Gameplay acoustics | committed semantic/world facts | deterministic `AcousticFactV1` | existing authoritative path only |
| Physical excitation extraction | committed engine-owned physics projection | bounded immutable presentation records | not saved |
| Source synthesis | excitation records plus exact cooked acoustic models | mono or small-channel source PCM | reconstructible voice state |
| Propagation/spatialization | source PCM plus listener/room/portal facts | listener PCM | reconstructible cache |
| Device/capture | mixed PCM | hardware output or canonical capture | never gameplay authority |

Steam Audio or a similar adapter belongs after source synthesis. It models
distance, directivity, occlusion, transmission and reflections; it does not
replace the physical source model. Adapter handles and vendor types remain
private under SPEC-08.

## Candidate content model

The first candidate uses three separate PresentationOnly content roles.
Their exact V1 wire schemas MUST NOT land before the first production consumer.

### `AcousticMaterialProfileV1`

An acoustic material profile describes calibration needed by a sound model,
such as density, elastic constants, frequency-dependent damping, roughness or
friction-exciter coefficients, radiation class and calibration provenance.
Every value has declared units, finite bounds, revision and content hash.

It is separate from `PhysicsMaterialDescriptorV2`. The current physics material
owns contact friction, restitution, rolling/spinning friction and surface
velocity; it does not own Young's modulus, Poisson ratio, modal damping or
acoustic radiation. An exact binding may reference both records without
copying either record into the other or making acoustic calibration affect
collision response.

### `ModalSoundModelV1`

A cooked modal model binds:

- exact source geometry or an explicitly declared acoustic proxy revision;
- exact acoustic material/profile revision;
- cooker and numeric-profile identity;
- sorted bounded modes with frequency, damping and gain;
- a bounded spatial impact-response or participation map;
- a declared radiation approximation and validity range;
- a fallback clip or engine-native fallback class;
- source provenance, license metadata and canonical content hash.

Finite-element analysis, eigensolving and large transfer computation happen in
the cooker or offline research tool. Runtime never solves the eigenproblem.
The render mesh, collision mesh and acoustic proxy may differ, but the binding
must name the exact revisions and cannot infer identity from filenames.

### `PhysicalSoundBindingV1`

The binding maps an exact body/shape/content revision and canonical physics
material tags to one acoustic body/model, priority class and fallback. Multiple
physics shapes may drive one acoustic body only through a declared stable
mapping. Missing, duplicate, stale or hash-mismatched mappings fail before the
model is activated; they do not partially bind voices.

The primary modal branch SHOULD avoid event-specific impact WAV selection.
This does not mean “no sound assets”: acoustic profiles, cooked modal data,
calibration recordings and fallback clips remain ordinary governed content.

## Candidate excitation projection

SPEC-26's normative `ContactEventV1` already names the useful physical facts:
stable contact identity, tick/substep, participants/features, point, normal,
relative velocity, impulse bounds, effective mass and material tags. The
production synthesizer MUST consume an engine-owned quantized projection of
those committed facts, never raw PhysX callbacks, native manifolds, pointer
identity or solver scratch.

The current Rust `ContactEventV1` implementation publishes only a subset of
that normative record: contact identity, participant/features, point, normal,
phase and source-snapshot hash. It does not yet serialize relative velocity,
impulse bounds, effective mass, contact kind or material tags. Therefore a
production physical-audio vertical is blocked until the existing physics
projection gap is closed and its `PHYS-COLLISION-P1` evidence passes. An
offline P0 harness may supply exact synthetic excitation fixtures, but that
does not authorize a raw-backend runtime shortcut.

An illustrative private extraction record is:

```text
PhysicalAcousticExcitationV1 {
  source_contact_id,
  source_snapshot_hash,
  gameplay_tick,
  physics_tick,
  substep,
  participant_low,
  participant_high,
  feature_low,
  feature_high,
  point,
  normal_low_to_high,
  relative_velocity_low_to_high,
  impulse_lower_bound,
  impulse_upper_bound,
  effective_mass,
  canonical_material_tags[],
  phase: Begin | Persist | End,
  excitation_kind: Impact | Friction | Rolling,
  canonical_hash,
}
```

This is not a current public contract. A promoted version must define exact
quantization, bounds, canonical order, duplicate/collision behavior, end-event
semantics and the derivation from the complete committed contact batch.

`Impact` derives from a `Begin` record and bounded contact-energy/impulse
mapping. `Friction` and `Rolling` require a continuity interval, tangential
velocity, normal load/effective mass and stable surface calibration. A
`Persist` event alone is not proof of a usable friction force. If the first
projection cannot distinguish resting, sliding and rolling contact without
backend-private information, those exciters remain out of scope rather than
guessing from callback frequency.

Fracture, cloth, fluid, combustion and vocal excitation are not encoded as
fake contacts. A future owning subsystem must publish its own committed,
bounded, typed excitation fact before presentation may consume it.

## Modal runtime and exciters

For one damped mode, the continuous reference is:

```text
q'' + 2 * damping * angular_frequency * q'
   + angular_frequency^2 * q
   = participation_gain * excitation
```

The cooker supplies stable bounded coefficients. Runtime advances a bank of
damped resonators at the pinned audio sample rate and sums their radiation
gains. An impact injects a finite excitation at the contact location. A
scraping or rolling model feeds a separate bounded stochastic or pulse-train
exciter into the same resonator bank; noise generation uses an explicit seeded
profile, never ambient thread RNG.

The first reference implementation SHOULD use a scalar fixed-point or otherwise
integer-exact recurrence compatible with the existing 48 kHz canonical PCM
check. Optimized floating-point/SIMD implementations MAY be evaluated only
against that reference with a declared perceptual/numeric metric. Device PCM
is presentation data, but an optimized implementation cannot redefine the
pinned reference check or feed gameplay.

One promoted profile must declare:

- sample rate and channel layout;
- maximum modes per model and active modes per voice;
- maximum active physical voices and excitation records per tick/window;
- coefficient and accumulator ranges, saturation and stability rules;
- exact event-to-sample mapping and presentation epoch/reset behavior;
- canonical priority and tie-break order;
- allocation, queue, callback and whole-mixer budgets.

The real-time audio path preallocates its bounded state. It performs no file
I/O, dynamic allocation, blocking lock, unbounded queue operation or content
decode in the callback. Cooking, activation and model validation complete
before a voice is admitted.

## Radiation, propagation and spatialization

Object vibration and environmental sound propagation are different problems.
The first vertical uses a declared point-source, monopole/dipole or similarly
bounded radiation approximation. A later cooked acoustic-transfer model may
improve directivity from a geometrically complex vibrating body, but it remains
part of source radiation.

Room/portal response, occlusion, transmission and reflections remain the
SPEC-08 propagation layer. The engine-native attenuation/panning/zone path is
mandatory. Optional Steam Audio may consume generated PCM under its own
version, license, resource and fallback checks; its absence or failure cannot
disable source synthesis or change gameplay facts.

## Voice admission and presentation LOD

Physical audio work is admitted in canonical priority order using only
presentation facts declared by the profile. One candidate ladder is:

1. `FullModal` — complete admitted mode set and spatial excitation map;
2. `ReducedModal` — deterministic subset/grouping of the same model;
3. `HeuristicProcedural` — bounded material-class resonator/noise fallback;
4. `AuthoredSampleFallback` — exact declared clip;
5. `Muted` — diagnostic-only last resort.

Camera visibility, frame time, callback completion order and vendor voice
order cannot choose a tier when canonical displayless PCM is being checked.
Interactive presentation MAY use listener distance and declared presentation
priority for quality scaling because it is non-authoritative, but replay and
gameplay roots must remain unchanged for every tier choice.

## Determinism, replay and restart

Synthesizer phase, filter history, random-exciter state, propagation cache and
device buffers are reconstructible presentation state and are not saved. Load,
restart or presentation recovery begins a fresh presentation epoch, clears
physical voices and resumes only from newly committed inputs. A displayless
capture profile MAY declare bounded preroll from an existing replay, but the
preroll is evidence tooling rather than a new save owner.

The following roots MUST be identical with physical synthesis enabled,
disabled, voice-limited, missing its content, faulted or replaced by the
authored fallback:

- gameplay and RPG state;
- command ledger and committed events;
- physics snapshot/contact roots;
- deterministic `AcousticFactV1` gameplay-hearing facts;
- save/load/replay owner roots.

Canonical PCM evidence fixes exact content, excitations, sample rate, window,
numeric profile, source admission order and sink. Hardware output, optimized
float PCM or a third-party propagation adapter cannot claim the exact pinned
PCM result unless it proves the same declared comparison.

## Failure semantics

| Failure | Required behavior |
|---|---|
| Missing, corrupt or incompatible acoustic model | Reject that model before voice activation; use its exact declared authored/heuristic fallback or silence; do not mutate world state. |
| Missing physical binding | Emit one bounded diagnostic and use the existing clip path or silence; do not infer a binding from renderer material/name. |
| Stale contact or model revision | Reject the excitation/model pair; never apply it to a different body generation. |
| Excitation queue overflow | Apply declared canonical presentation priority/drop policy, count the loss and preserve gameplay/physics roots. |
| Nonfinite/unstable/overflowing resonator | Terminate the affected voice, record a stable diagnostic and select fallback; never retry with looser coefficients. |
| Late async content or propagation result | Use the already selected fallback/window; never insert audio retroactively into an elapsed canonical capture window. |
| Device or optional propagation failure | Continue engine-native mixing or discard device output while gameplay and acoustic facts continue. |
| PCM mismatch | Fail the physical-audio evidence only; do not reinterpret or repair authoritative simulation. |

## Bounded evaluation and promotion sequence

### P0 — offline reference

Cook or import a modal model for a tiny engine-owned primitive corpus. Drive it
with exact synthetic impulses, render canonical 48 kHz PCM and compare modal
frequencies, decay and bounded perceptual descriptors against reference
recordings or a high-quality offline solver. P0 changes no runtime contract.

### P1 — first product vertical: rigid impact

Use three independently licensed/engine-owned object profiles representing
steel, wood and glass-like behavior and a small bounded geometry set. Exercise
hammer/drop impacts at multiple positions, energies and orientations through
the production contact/extraction path. The primary modal branch uses no
event-specific impact WAV; the declared sample/heuristic fallback remains
available for faults and A/B comparison.

P1 is intentionally impact-only. It must close the current contact-projection
implementation gap, content cooking/activation, canonical PCM and enabled/
disabled/fault root non-regression before rolling or scraping work begins.

### P2 — persistent contact

Add rolling and scraping only after P1. The corpus varies tangential speed,
normal load, surface roughness and contact continuity, and includes resting
and separating controls. If ordinary rigid contact facts cannot reproduce
stick-slip or micro-collision structure, evaluate one bounded flexible-contact
or micro-collision exciter rather than tuning arbitrary noise to the same
symptom.

### Later independent tracks

- fracture uses precomputed fragments/soundbanks or another bounded topology-
  aware source model after a committed fracture owner exists;
- footsteps combine a dedicated foot/shoe/surface contact model with authored
  or synthesized residuals;
- cloth, liquids and fire use their own motion-driven, particle/turbulence or
  combustion exciters;
- voice and bird syrinx models remain specialized biological instruments with
  separate controllability and quality criteria;
- differentiable or learned methods may fit acoustic parameters offline, but
  runtime neural residuals require a measured classical deficiency, immutable
  local artifact, bounded inference and a complete non-neural fallback.

## Candidate ProductChecks

These names reserve no current global gate. They become normative only with a
production consumer and promoting ADR.

| Check | Candidate observable result |
|---|---|
| `AUDIO-PHYS-SOURCE-P1` | Exact impact identity/point/energy drives the expected steel/wood/glass modal model and bounded PCM window; input/callback/worker permutations cannot change admitted excitation order. |
| `AUDIO-PHYS-CONTACT-P1` | Rolling/scraping fixtures distinguish resting, sliding, rolling and separation across declared speed/load/roughness controls without callback-frequency artifacts. |
| `AUDIO-PHYS-CONTENT-P1` | Acoustic profile/model/binding cook is repeatable and hash-closed; malformed, stale, oversized or incompatible content rejects atomically and selects the declared fallback. |
| `AUDIO-PHYS-PCM-P1` | Pinned 48 kHz displayless PCM and event-to-sample mapping are exact for the reference profile; enabled/disabled/faulted synthesis produces identical gameplay, ledger, physics, acoustic-fact and save/replay roots. |

The promoted implementation additionally maps to focused `fast`, `play` and
`content-package`; `platform` applies when host/device/propagation code changes,
and `performance` applies when the audio callback, mixer or cooker hot path is
materially affected. `persistence-replay` is required for proving the root
non-regression and fresh presentation epoch, not because synth state is saved.

No numeric quality or performance threshold is invented in this draft. P0
must measure the fixed corpus and choose an audible-error metric, maximum mode
count, voice count, queue capacity, memory budget and callback p95/p99 before
promotion. Complexity is approximately `O(active_modes * rendered_samples)`;
quality scaling must therefore reduce admitted modes/voices explicitly rather
than rely on an unbounded solver.

## Rejected alternatives

| Alternative | Reason |
|---|---|
| Replace all authored audio with one universal procedural layer | Different source classes need different models and authored fallback remains necessary for product quality and failure closure. |
| Feed raw PhysX callbacks directly to the mixer | Violates engine-owned contracts, stable identity, canonical ordering and backend isolation. |
| Put acoustic constants into `PhysicsMaterialDescriptorV2` | Creates parallel semantics in the physics authority and still lacks geometry, damping, radiation and calibration identity. |
| Runtime FEM/eigenmode solve per object | Unbounded and unnecessary; preprocessing can cook a compact resonator model. |
| Make waveform propagation determine NPC hearing | Couples gameplay to voice admission, device state and optional presentation adapters. |
| Start with scraping, fracture, liquids or a bird synthesizer | Expands the unknown source/owner surface before the smallest rigid-impact consumer is proven. |
| Runtime neural residual as the base path | Adds artifact/provider/resource/fallback complexity before a measured classical quality gap exists. |

## Research basis and limits

The candidate follows the established modal-sound pattern: offline geometry/
material analysis plus real-time excitation from rigid contact. The research
report records the primary sources and their limits. In particular, evidence
for impact/rolling, contact-rich scraping, fracture acceleration, parameter
fitting and source/propagation separation does not prove that one model covers
every material, object or biological sound. Product promotion depends on the
bounded Next Engine P0/P1 evidence above, not on a paper's demo or another
engine's marketing claim.
