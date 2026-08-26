# Physical sound synthesis research — 2026-08-26

| Field | Value |
|---|---|
| Status | `REPORT_ONLY / DRAFT_SPEC_PRODUCED` |
| Question | Can Next Engine add a separate physics-driven sound-generation layer, and what is the smallest defensible architecture? |
| Result | Yes for a bounded presentation-only source-synthesis layer; start with cooked modal rigid impacts, keep clips as fallback, and treat other source classes as independent tracks |
| Architecture output | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md) |
| Evidence date | 2026-08-26 |
| Scope limit | No implementation, public contract, roadmap slot, runtime neural model or ProductCheck PASS |

## Executive conclusion

The core idea is technically credible and product-relevant, provided it is
framed as **physical source synthesis**, not as a universal acoustic simulator.
The strongest first route is established modal synthesis:

1. precompute a compact vibration model from exact geometry plus an acoustic
   material/calibration profile;
2. derive a bounded excitation from a committed contact fact;
3. run a bank of damped resonators at audio rate;
4. pass the generated source PCM to the ordinary propagation/spatialization
   and mixer path.

This is compatible with Next Engine because the entire layer can remain a
reconstructible presentation consumer. Physics continues to own contacts and
body state; deterministic `AcousticFactV1` continues to own gameplay-hearing
input; the waveform never feeds back into the world.

The smallest falsifiable product vertical is not “all material sound.” It is a
small engine-owned corpus of steel-, wood- and glass-like rigid objects under
hammer/drop impacts at several positions and energies. Rolling and scraping
come next because high-quality persistent contact needs more than a contact-
begin pulse. Cloth, liquids, fire, fracture, footsteps and birds each require a
specialized exciter/model or a hybrid residual and should not share one vague
milestone.

## Repository baseline and exact gap

Current Next Engine audio already has the right authority separation:

- `AudioSceneSnapshotV1` is an immutable presentation projection;
- `AudioMixerV1` renders bounded 48 kHz stereo PCM from cooked clips;
- `AcousticFactV1` is deterministic gameplay perception and is independent
  from device output;
- engine-native attenuation/panning/zone fallback remains available without
  Steam Audio;
- displayless `AUDIO-02` pins PCM and event-to-sample alignment while keeping
  gameplay roots independent from output.

Physics also has a suitable normative boundary. SPEC-26 defines stable contact
identity, point/normal, relative velocity, impulse bounds, effective mass,
material tags and canonical order. However, the current Rust
`ContactEventV1` record serializes only contact identity, participant/feature,
point/normal, phase and source-snapshot hash. Relative velocity, impulse
bounds, effective mass, kind and material tags are absent from the current
implementation. A runtime physical-audio consumer therefore cannot be built
correctly by simply wiring the existing struct to the mixer.

The correct blocker is narrow: close the already specified engine-owned
contact projection through the physics boundary and its existing collision
evidence. Consuming raw PhysX callbacks would hide, not solve, the gap.

Current `PhysicsMaterialDescriptorV2` is also deliberately insufficient as an
acoustic model. It owns contact friction/restitution, rolling/spinning friction
and surface velocity. It does not own density, elastic moduli, Poisson ratio,
frequency damping, radiation or calibration provenance. These belong in a
separate PresentationOnly acoustic profile and cooked model, linked by exact
revisions.

## Primary-source findings

### Modal rigid-body sound is a proven architecture

O'Brien, Shen and Gatchalian describe synthesizing sound from rigid-body
simulation by precomputing deformation modes and frequencies from object
geometry/material properties, then using rigid-body force data to excite them
at runtime. This directly supports the candidate `cook -> excitation -> modal
bank` split. It does not establish universal material parameters or a game-
ready content format. Source: [Synthesizing Sounds from Rigid-Body Simulations,
SIGGRAPH 2002](https://graphics.berkeley.edu/papers/Obrien-SSR-2002-07/).

Raghuvanshi and Lin demonstrate scalable modal contact sound with priority-
based quality scaling for many moving objects and impact/rolling examples.
This supports deterministic mode/voice admission and reduced-modal LOD. The
paper's historical hardware result is evidence of feasibility, not a current
Next Engine budget. Source: [Interactive Sound Synthesis for Large Scale
Environments, I3D 2006](https://www.microsoft.com/en-us/research/publication/interactive-sound-synthesis-large-scale-environments/).

Langlois, An, Jin and James show large compression of modal models without
perceptible error for their evaluated objects. The often repeated “100x”
number is a result of this specific compression work, not a blanket runtime
speedup or a universal mode-count guarantee. Source: [Eigenmode Compression
for Modal Sound Models, SIGGRAPH 2014](https://www.cs.cornell.edu/projects/Sound/modec/modec.pdf).

### Impacts are the right first vertical; scraping is materially harder

Zheng and James show why simple rigid contact impulses can produce inadequate
humming or buzzing: high-quality contact sound may require micro-collisions,
chattering and stick-slip/frictional contact over a richer temporal contact
model. This is decisive evidence against promising scraping from only
`Begin/Persist/End` callbacks. It supports impact-first sequencing and a
separate persistent-contact gate. Source: [Toward High-Quality Modal Contact
Sound, SIGGRAPH 2011](https://www.cs.cornell.edu/projects/Sound/mc/ModalContactSound2011.pdf).

The implication for Next Engine is concrete: `Impact` may use a stable contact
begin plus quantized velocity/impulse/effective mass. `Rolling` and `Friction`
need continuity, tangential motion, load and calibrated roughness, and may
eventually need a bounded flexible-contact or micro-collision exciter. Backend
callback frequency is not a physical model.

### Source radiation and environmental propagation are separate

Precomputed Acoustic Transfer models radiation from geometrically complex
vibrating sources. Steam Audio's official simulation API models distance,
directivity, air absorption, occlusion, transmission, reflections and pathing
for a supplied source. Together they support two distinct stages: source
generation/radiation first, room/world propagation second. Sources:
[Precomputed Acoustic Transfer](https://graphics.stanford.edu/~djames/publication/precomputed-acoustic-transfer-output-sensitive-accurate-sound-generation-for-geometrically-complex-vibration-sources/)
and [Steam Audio simulation API](https://valvesoftware.github.io/steam-audio/doc/capi/simulation.html).

Steam Audio is therefore a plausible optional propagation adapter, not the
implementation of the proposed modal source layer. The engine-native SPEC-08
fallback remains required.

### Specialized source classes do not collapse into one modal body model

The Stanford physical sound synthesis course separates rigid objects,
fracture, thin shells, cloth, liquids and fire into different source-model
families. This is useful architectural evidence: sharing a mixer and bounded
excitation interface does not imply one canonical solver or content record.
Source: [Stanford SIGGRAPH 2016 course on physically based sound](https://graphics.stanford.edu/courses/sound/).

Specific examples reinforce the split:

- Fracture work uses precomputed soundbanks and time-varying rigid models; it
  is a topology-aware specialized pipeline, not “ordinary modal contact plus
  more events.” Source: [Precomputed Soundbank Synthesis for Natural Sounding
  Fracture](https://www.cs.cornell.edu/projects/FractureSound/).
- Cloth sound uses motion-driven concatenative synthesis from micro-samples,
  demonstrating a practical hybrid rather than a generic rigid modal model.
  Source: [Example-Based Wrinkle Sound Synthesis for Clothing Animation](https://www.cs.cornell.edu/projects/Sound/cloth/).
- Physically based footsteps explicitly model shoe/surface/anthropometric
  factors and use perceptual evaluation, supporting a specialized footstep
  branch. Source: [Physically based synthesis of footsteps sounds](https://www.sciencedirect.com/science/article/pii/S0003682X15001747).

The first rigid-impact vertical should not claim credit for these modalities.

### Hybrid and offline inverse methods are useful, but do not justify a runtime neural base

Raghuvanshi, Narain and Lin extract modal models from recordings and add a
residual component to improve perceptual similarity, including a video-game
deployment. This supports calibration and an authored residual/fallback when
pure modal output is insufficient. It does not require a neural runtime.
Source: [Sound Synthesis for Impact Sounds in Video Games, I3D 2011](https://www.microsoft.com/en-us/research/publication/sound-synthesis-impact-sounds-video-games/).

DiffSound demonstrates differentiable modal rendering and inverse estimation
of physical parameters, shape and impact location. This is strong evidence for
an offline cooker/calibration tool. It is not evidence that runtime neural
audio should become mandatory or authoritative. Source: [DiffSound, SIGGRAPH
2024](https://hellojxt.github.io/DiffSound/).

Recommended order:

1. analytic/cooked modal reference;
2. measured perceptual error on the exact corpus;
3. offline parameter fitting or an authored residual if needed;
4. only then consider a runtime learned residual with immutable lineage,
   bounded inference and a complete non-neural fallback.

### Biological synthesis is plausible but is a different instrument

The electronic-syrinx work models bird sound using air pressure and muscle
tension controls and reproduces important qualitative behaviors. This
supports the long-term idea of controllable biological instruments. It does
not show that a rigid-contact acoustic body can generate arbitrary bird calls
or that species-level game content becomes asset-free. Source: [An electronic
bird vocal organ, Physical Review E 72, 2005](https://journals.aps.org/pre/abstract/10.1103/PhysRevE.72.051926).

The attached discussion cited Lyrebird as evidence for thousands of fully
synthesized bird calls. The current repository does not support that claim:
its README describes a much smaller species/syllable inventory and states that
most of its corpus is real field audio. It is a useful soundscape project, not
evidence for a universal syrinx synthesizer. Source: [Lyrebird repository](https://github.com/sha5b/Lyrebird).

## Claims from the prior discussion: retained, narrowed or rejected

| Prior idea | Assessment | Next Engine interpretation |
|---|---|---|
| “Do not brute-force air pressure everywhere” | Retained | Use modal/source models and ordinary propagation; no global pressure-field solver in the first track. |
| “Material + shape + excitation can generate object sound” | Retained with calibration | Geometry, boundary conditions, damping, radiation and acoustic calibration are exact content inputs; physics material alone is insufficient. |
| “About 40–200 modes per body” | Not accepted as a contract | Plausible example range only; P0 must measure mode count against error and callback budget. |
| “100x acceleration” | Narrowed | Specific papers report compression/acceleration on their cases; no Next Engine budget follows without measurement. |
| “No impact audio assets are needed” | Narrowed | The primary P1 branch can avoid event-specific impact WAVs, but still needs cooked modal assets, profiles, calibration and fallbacks. |
| “Rolling and scraping naturally follow contact” | Narrowed | Rolling is demonstrated, but high-quality scraping can require micro-collision and stick-slip models beyond ordinary rigid contacts. |
| “Cloth, liquids, fire and voice can use the same layer” | Rejected as one solver | They may share bounded presentation plumbing but require separate source models/exciters and independent gates. |
| “A neural residual can finish realism” | Deferred | First measure the classical residual; prefer offline fitting or authored residual before bounded runtime inference. |
| “Bird calls are already fully synthesized at large scale” | Rejected from cited evidence | Electronic-syrinx research is real, but the cited Lyrebird project is mostly field-audio based. |

## Candidate architecture options

### A. Keep authored clips only

This is the current reliable baseline. It is cheap, controllable and already
supports deterministic PCM, but it scales poorly across continuous impact
position/energy/object variation and tends to select rather than generate.

**Decision:** retain as mandatory fallback and A/B comparator, not as the only
future path.

### B. Cooked modal source synthesis as a presentation extension

Offline preprocessing produces a compact model; committed engine-owned
contacts excite it; the result enters the existing mixer and propagation
path. Complexity and failure are bounded per voice, and gameplay stays
independent.

**Decision:** recommended first implementation and basis of SPEC-45.

### C. High-quality flexible/micro-contact solver first

This can improve scraping and contact texture but expands solver, contact and
performance uncertainty before the basic product value is measured.

**Decision:** reserve for P2 if the P1/P2 control corpus falsifies ordinary
contact excitation.

### D. End-to-end or residual neural runtime first

This adds dataset/model lineage, runtime provider, bounded inference,
determinism and fallback work before there is a measured classical residual.

**Decision:** reject as the base path; allow offline inverse calibration now and
reconsider a runtime residual only after a named perceptual failure remains.

## Smallest evidence-backed experiment

### P0 reference corpus

Use engine-owned or CC0 content only:

- three acoustic material profiles: steel-like, wood-like and glass-like;
- three small geometries or acoustic proxies with distinct aspect ratios;
- impulse positions covering centre, edge and off-axis points;
- at least three energy levels plus silence/invalid controls;
- exact 48 kHz mono source PCM before propagation;
- reference recordings or a higher-quality offline solver with declared
  acquisition/provenance.

Measure:

- modal peak frequency error and decay-time error;
- bounded spectral/envelope descriptors selected before comparison;
- event-to-sample alignment and byte-exact repeat PCM for the reference path;
- maximum mode count, model bytes, active-voice cost and whole-mixer p95/p99;
- blinded or otherwise predeclared perceptual preference/indistinguishability
  evidence for the tiny corpus.

P0 succeeds only if at least one bounded model/profile produces a useful,
repeatable quality/cost point. It does not require a runtime contact schema.

### P1 production path

After P0, close the SPEC-26 contact-projection implementation gap, cook/activate
exact models, and run hammer/drop impacts through production physics,
presentation extraction and displayless mixing. Prove identical gameplay,
ledger, physics, gameplay-acoustic and save/replay roots with the feature on,
off and faulted.

### Stop conditions

Pause the track rather than expanding it if:

- no P0 model reaches the predeclared perceptual/physical error at a bounded
  cost;
- the production contact projection cannot expose stable backend-independent
  excitation facts;
- two coherent calibration cycles only move audible artifacts without meeting
  the criterion;
- whole-mixer cost cannot fit a measured product budget after explicit mode/
  voice reduction.

The next response to a surviving blocker is a small counterfactual or bounded
research cycle, not an unrestricted new source model.

## Licensing and implementation references

The papers and project pages above are research evidence, not code licenses.
`openpbso` is an MIT-licensed historical runtime modal-synthesis reference, but
its repository notes old dependencies and does not include its preprocessing
toolchain. It should not be selected as a production dependency without a
fresh API, maintenance, license and cooker evaluation. Source:
[openpbso](https://github.com/jhwang7628/openpbso).

Any promoted implementation must use engine-owned contracts and a separately
reviewed dependency/license decision. Research recordings, generated PCM,
large modal models and captures remain external artifacts unless an exact
small neutral fixture is explicitly admitted through normal content policy.

## Decision

Create SPEC-45 as `Proposed`, with no roadmap activation. Preserve the current
clip path and gameplay acoustic facts. If the product owner schedules the
track, perform only P0 first; promote no public contract until the bounded
impact consumer, exact content closure, physics projection and ProductChecks
are concrete.
