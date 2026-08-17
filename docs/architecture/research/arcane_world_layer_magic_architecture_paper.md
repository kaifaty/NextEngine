# Arcane World Layer
## Magic as a Simulated Substrate of the Physical World

**Document type:** architecture and research paper  
**Status:** working foundation for ADRs, specifications, research spikes, and gameplay-system design  
**Context date:** August 2026  
**Target:** custom Rust game engine with a unified `PhysicalWorld / WorldDynamics` layer  
**Related documents:**
- `physical_world_layer_architecture_for_specs.md`
- `continuum_physics_water_mud_offroad_research_brief.md`
- `vegetation_physics_destructible_trees_research.md`
- `vegetation_physics_followup_production_architecture.md`

---

# Abstract

This paper proposes a world-level magic architecture in which magic is not implemented as a disconnected list of abilities, scripted effects, or resource bars. Instead, the engine treats magic as an additional simulated substrate of the world: a field and transport system with sources, sinks, reservoirs, conduits, spectra, quality, dissipation, conversion rules, and coupling to matter, living organisms, atmosphere, heat, vegetation, fluids, rigid bodies, and identity.

The proposed design introduces an `ArcaneDomain`, a `VitalDomain`, an optional `IdentityDomain`, and a data-driven `SpellRuntime`. These systems integrate with the existing `PhysicalWorld` through a `CouplingGraph`. Mana can be stored by organisms, plants, artifacts, crystals, and locations; transferred through channels and ley networks; converted into physical work; degraded by entropy or incoherence; filtered by materials; and shaped into spells through executable effect graphs.

The goal is to make magic a systemic property of the world. A mage should be able to accumulate mana, overload internal channels, draw energy from a forest, heat actual wood, push real rigid bodies, redirect simulated water, strengthen roots, destabilize soil, or create a barrier that exchanges momentum with projectiles. Plants and creatures should participate in the same arcane ecology. The same laws should support different cultural interpretations such as mana, aether, prana, qi, spirit energy, divine power, ley lines, chakras, runes, and artifacts.

The core thesis is:

> **Magic should be represented as a world substrate with conserved or explicitly sourced quantities, while spells act as constrained programs that couple this substrate to the physical, biological, and informational domains of the world.**

---

# 1. Motivation

Most game magic systems are constructed around abstractions such as:

```rust
struct Spell {
    mana_cost: f32,
    damage: f32,
    cooldown: f32,
}
```

This is effective for conventional gameplay, but it does not produce a world in which magic is genuinely present.

Such systems generally cannot answer:

- Where does mana come from?
- Why can one object store it and another cannot?
- What does a ley line physically mean?
- Why does wet wood resist a fire spell?
- Can a mage extract mana from plants?
- Can a plant become ill when local mana is depleted?
- Why can one mage hold a large reserve but fail to cast quickly?
- Can a damaged magical channel cause overload?
- Can magic create unlimited mechanical work?
- What exactly does anti-magic do?
- Why are runes useful?
- Is a soul just another energy reserve?
- How does divine power differ from arcane power?
- Can NPCs reason about magic as part of the environment?

A world-level architecture should answer these questions through a consistent simulation model rather than special-case scripts.

---

# 2. Scope

This document covers:

- the ontology of mana and arcane fields;
- energy, potential, quality, coherence, and entropy;
- ambient mana and ley networks;
- biological accumulation and circulation;
- plant and creature interaction;
- spell execution;
- arcane-to-physical conversion;
- magic materials, crystals, runes, and artifacts;
- anti-magic and interference;
- arcane ecology and economy;
- performance and representation LOD;
- persistence and networking;
- AI queries and planning;
- integration with physical-world systems;
- implementation roadmap;
- required ADRs and specifications.

This document does **not** freeze:

- exact schools of magic;
- spell names;
- class systems;
- lore terminology;
- numerical balance;
- visual style;
- the metaphysical truth of the setting.

The engine should provide universal primitives. A particular game defines the interpretation.

---

# 3. Design principles

## 3.1. Magic is a world law, not an exception system

The engine should avoid:

```text
if spell == Fireball:
    spawn_fireball()
```

as the fundamental model.

Instead:

```text
acquire mana
→ shape a containment field
→ convert part into thermal energy
→ accelerate the structure
→ maintain coherence
→ release on contact
```

Named spells may compile into this graph, but the graph is the underlying mechanism.

---

## 3.2. One world, several specialized domains

Magic should not be merged into one universal monolithic solver.

The architecture should remain:

```text
WorldDynamics
│
├── PhysicalWorld
├── ArcaneDomain
├── VitalDomain
├── IdentityDomain        // optional
├── SpellRuntime
├── CouplingGraph
├── RepresentationManager
└── SimulationScheduler
```

Each domain owns its state and uses appropriate mathematics.

---

## 3.3. Conversion requires an explicit source

If magic produces:

- heat;
- motion;
- matter;
- light;
- pressure;
- biological repair;

the source and cost must be explicit.

Otherwise mana becomes an accidental infinite-energy exploit that invalidates the world economy.

Infinite or externally supplied energy may be a deliberate setting decision, but it must be encoded as a law.

---

## 3.4. Energy and control are separate

A spell may require little energy but extreme precision.

Examples:

- illusion;
- surgery-like healing;
- memory manipulation;
- fine telekinesis;
- identity binding.

Therefore a single scalar `mana_cost` is insufficient.

---

## 3.5. Magic should couple to existing simulated matter

A water spell should preferably act on actual `ContinuumDomain` water.

A fire spell should inject heat into `ThermalDomain`.

A force spell should apply impulses through a coupler.

A plant-growth spell should modify biological growth and structural vegetation state.

This keeps magic inside the same world rather than creating parallel fake effects.

---

## 3.6. Hard invariants belong to deterministic systems

ML may assist:

- spell planning;
- field approximation;
- relevance selection;
- preconditioning;
- effect synthesis.

But the engine should preserve core laws through deterministic projection and constraints.

---

## 3.7. Expensive simulation must be local and adaptive

The engine should not maintain a high-resolution mana field over the whole world.

Use:

```text
global ley graph
→ regional summaries
→ local active fields
→ organism reservoirs
→ temporary spell fields
```

---

# 4. Terminology and literary unification

Different traditions can describe the same engine substrate using different language.

| Cultural term | Engine interpretation |
|---|---|
| Mana | available arcane free energy |
| Aether / ether | ambient arcane field or carrier substrate |
| Prana | bio-arcane circulation in living organisms |
| Qi / chi | organism-level arcane flow and balance |
| Chakras | reservoirs, transformers, or control nodes |
| Meridians | internal arcane conduits |
| Ley lines | high-conductivity edges in the world arcane network |
| Ley nodes | high-capacity or high-potential network junctions |
| Aura | near-body arcane field and leakage |
| Rune | symbolic control element in an arcane circuit |
| Magic circle | spatial constraint graph and boundary condition |
| Crystal | reservoir, resonator, converter, or spectrum filter |
| Artifact | persistent compiled spell circuit |
| Corruption | harmful spectrum shift, entropy, or unstable state |
| Divine power | remote authorized source or external domain coupling |
| Spirit | persistent non-material agent or identity-linked process |
| Soul | continuity/identity structure, not necessarily energy |
| Curse | persistent constraint or identity-linked effect graph |
| Blessing | authorized persistent coupling or state modifier |

This allows multiple cultures to disagree in lore while the simulation remains coherent.

---

# 5. What mana is

Mana should be modeled as more than one number.

A useful base model includes:

```rust
struct ArcanePacket {
    quantity: f32,
    potential: f32,
    coherence: f32,
    entropy: f32,
    spectrum: ArcaneSpectrum,
}
```

These dimensions have distinct meaning.

---

## 5.1. Quantity

`quantity` describes the amount of arcane substance or energy-equivalent available.

It limits total work.

---

## 5.2. Potential

`potential` describes the capacity of mana to flow or drive an effect.

Two reservoirs with equal quantity but different potential may behave differently.

Potential differences cause flow.

---

## 5.3. Coherence

`coherence` describes how well the mana can be shaped into a precise organized effect.

High coherence supports:

- complex barriers;
- healing;
- stable portals;
- fine telekinesis;
- persistent enchantment.

Low coherence may still support:

- explosions;
- chaotic discharge;
- diffuse heat;
- corruption.

---

## 5.4. Entropy

`entropy` measures disorder, contamination, or irreversible degradation.

Possible effects:

- reduced conversion efficiency;
- instability;
- harmful side effects;
- spontaneous anomalies;
- poor compatibility with living systems.

Entropy may increase when:

- mana is repeatedly converted;
- channels overload;
- incompatible spectra mix;
- spells collapse;
- corruption spreads.

---

## 5.5. Spectrum

Mana should have a configurable multi-dimensional spectrum.

```rust
struct ArcaneSpectrum {
    components: [f32; ARCANE_BANDS],
}
```

Possible abstract bands may correspond to affinities with:

- heat;
- motion;
- structure;
- life;
- perception;
- identity;
- decay;
- space;
- order;
- chaos.

The engine should not hardcode their lore names.

A game can map bands to schools, elements, deities, colors, or philosophical concepts.

---

# 6. Arcane conservation models

The setting must choose how mana behaves globally.

Several models are possible.

---

## 6.1. Closed arcane cycle

```text
coherent mana
→ spell
→ physical effect
→ degraded mana
→ natural purification
→ coherent mana
```

Total arcane quantity is approximately conserved.

Quality changes.

This produces:

- exhausted regions;
- arcane pollution;
- purification ecosystems;
- strategic reservoirs.

---

## 6.2. Open cosmic inflow

Mana enters the world through:

- stars;
- sun;
- dimensional boundaries;
- planetary core;
- gods;
- ley anchors.

The world is an open system.

This can support civilization-scale magical energy, but inflow rates become important to economics.

---

## 6.3. Biological generation

Living systems transform:

```text
sunlight
chemical metabolism
growth
information
→ life-aspected arcane energy
```

Forests and ecosystems become real magical infrastructure.

---

## 6.4. External-domain access

Divine, infernal, ancestral, or extradimensional magic may be implemented as access to remote reservoirs.

The caster does not generate the energy.

The caster authenticates and channels it.

This enables:

- patron relationships;
- revocation;
- contracts;
- divine authority;
- bandwidth limits;
- remote risk.

---

## 6.5. Hybrid model

A world may combine:

- weak ambient cosmic inflow;
- biological refinement;
- geological ley concentration;
- artificial storage;
- external beings.

This is likely the richest model.

---

# 7. Ambient arcane field

At the continuum level, ambient mana can be described by a density and flux.

A conceptual transport equation:

\[
\frac{\partial \rho_m}{\partial t}
+
\nabla \cdot J_m
=
S_m - U_m
\]

Where:

- \(\rho_m\) = local mana density;
- \(J_m\) = mana flux;
- \(S_m\) = sources;
- \(U_m\) = sinks and consumption.

A possible flux relation:

\[
J_m =
-K \nabla \mu_m
+
J_{\text{ley}}
+
J_{\text{carrier}}
\]

Where:

- \(K\) = arcane conductivity;
- \(\mu_m\) = arcane potential;
- `J_ley` = transport along ley structures;
- `J_carrier` = transport by living beings, fluids, artifacts, or atmosphere.

This equation is a setting law, not a claim about real physics.

---

# 8. Ley network

World-scale mana should primarily use a sparse network rather than a dense 3D field.

```text
             ley node
                ●
               / \
              /   \
             ●─────●
            /       \
           ●         ●
```

A ley edge may have:

- conductivity;
- capacity;
- spectrum;
- directionality;
- stability;
- leakage;
- ownership/control;
- blockage.

Conceptual:

```rust
struct LeyEdge {
    from: LeyNodeId,
    to: LeyNodeId,

    conductance: f32,
    max_flow: f32,

    spectrum_filter: ArcaneSpectrum,
    leakage: f32,
    stability: f32,
}
```

Ley nodes may correspond to:

- geological structures;
- ancient artifacts;
- temples;
- forests;
- city infrastructure;
- dimensional anchors;
- world-tree roots.

---

# 9. Regional mana state

Each streamed region can maintain an aggregate state:

```rust
struct RegionalArcaneState {
    density: f32,
    potential: f32,
    coherence: f32,
    entropy: f32,
    spectrum: ArcaneSpectrum,

    inflow: f32,
    outflow: f32,
}
```

This is sufficient for:

- ecology;
- regeneration;
- long-term depletion;
- broad anomalies;
- NPC planning.

High-resolution fields activate only when required.

---

# 10. Local active arcane fields

A local field may be created around:

- active spells;
- artifacts;
- ley nodes;
- rituals;
- high-energy creatures;
- anomalies;
- magical battles.

Representation options:

- sparse samples;
- particles;
- kernels;
- adaptive grids;
- basis functions;
- graph fields.

The representation should be replaceable.

The `ArcaneDomain` API must not expose solver-specific storage to gameplay.

---

# 11. Sources and sinks

## 11.1. Sources

Potential sources:

- ley nodes;
- celestial cycles;
- living ecosystems;
- crystals;
- dimensional tears;
- gods;
- rituals;
- decomposition;
- emotional/cognitive activity;
- heat or motion conversion.

---

## 11.2. Sinks

Potential sinks:

- spell conversion;
- organisms;
- artifacts;
- anti-magic materials;
- corruption;
- dimensional leakage;
- purification processes;
- dead zones.

---

## 11.3. Storage

Potential reservoirs:

- organisms;
- plant tissues;
- crystals;
- enchanted metals;
- water bodies;
- soil;
- structures;
- artifacts;
- souls, if the setting permits.

---

# 12. Arcane material properties

Every physical material may optionally define arcane capabilities.

```rust
struct ArcaneMaterialProperties {
    conductivity: f32,
    capacity: f32,

    absorption: f32,
    leakage: f32,

    coherence_loss: f32,
    entropy_generation: f32,

    spectrum_response: ArcaneSpectrum,
    saturation_limit: f32,
}
```

Examples:

## Copper-like magical conductor

```text
high conductivity
low capacity
low coherence loss
```

## Crystal

```text
medium conductivity
high capacity
high coherence
strong resonance
```

## Anti-magic alloy

```text
high absorption
high coherence loss
rapid dissipation
```

## Living wood

```text
moderate storage
life-spectrum filtering
moisture-dependent behavior
```

## Water

Possible setting-dependent behavior:

- good carrier;
- poor storage;
- spectrum purifier;
- emotional memory medium;
- high leakage.

The engine should not hardcode one answer.

---

# 13. Resonance

Resonance allows efficient interaction between matching spectra.

A conceptual compatibility function:

\[
R(a,b) =
\frac{a \cdot b}
{\|a\|\|b\|}
\]

Higher resonance can provide:

- greater conductivity;
- lower conversion loss;
- easier control;
- stronger response.

Low or negative resonance may cause:

- resistance;
- reflection;
- entropy;
- instability;
- destructive interference.

---

# 14. Interference

Multiple spells and fields should be able to interfere.

Possible interactions:

```text
constructive resonance
destructive interference
phase cancellation
spectrum contamination
overload
field locking
```

This creates systemic counterplay without hardcoded spell counters.

---

# 15. Anti-magic

Anti-magic should not normally be a boolean zone.

Possible physical meanings:

- absorb mana;
- lower potential;
- break coherence;
- increase entropy;
- redirect flux;
- short-circuit channels;
- isolate a region from ley inflow;
- create incompatible spectrum noise.

An anti-magic field can therefore have parameters:

```rust
struct AntiArcaneField {
    absorption_rate: f32,
    coherence_decay: f32,
    entropy_injection: f32,
    spectrum_disruption: ArcaneSpectrum,
}
```

Different anti-magic technologies can behave differently.

---

# 16. Living organisms and mana

Living beings need more than a `mana` scalar.

Recommended organism model:

```rust
struct ArcaneOrganism {
    reservoir: ManaReservoir,
    channels: ArcaneChannelGraph,

    absorption_rate: f32,
    regeneration_rate: f32,

    max_throughput: f32,
    control_bandwidth: f32,

    coherence_tolerance: f32,
    entropy_tolerance: f32,

    affinities: ArcaneSpectrum,
}
```

---

# 17. Reservoirs

A reservoir has:

```rust
struct ManaReservoir {
    current_quantity: f32,
    capacity: f32,

    potential: f32,
    coherence: f32,
    entropy: f32,

    spectrum: ArcaneSpectrum,
}
```

Reservoirs may be:

- biological organs;
- chakras;
- cores;
- blood;
- bones;
- nervous tissue;
- souls;
- attached crystals;
- symbiotic organisms.

---

# 18. Throughput

A mage may have a large capacity but low safe output.

```text
capacity
≠
throughput
```

Example:

- ritualist: large reserve, slow output;
- battle mage: smaller reserve, high burst;
- healer: low energy output, high control;
- sorcerous beast: huge output, poor precision.

---

# 19. Control bandwidth

`control_bandwidth` limits the number and complexity of constraints the caster can maintain.

It may depend on:

- training;
- cognition;
- nervous system;
- external focus;
- runes;
- assistants;
- ritual geometry;
- fatigue.

A complex illusion may consume little energy but much control bandwidth.

---

# 20. Arcane channels

Model the internal body as a graph:

```text
                    crown
                      ●
                      │
             ●────────●────────●
           left      core     right
            arm       │        arm
                      ●
                  reservoir
                    /   \
                   ●     ●
                 left   right
                  leg    leg
```

A channel has:

```rust
struct ArcaneChannel {
    conductance: f32,
    max_flow: f32,
    damage: f32,

    spectrum_filter: ArcaneSpectrum,
    entropy_generation: f32,
}
```

Flow can use:

\[
Q = G(\phi_a - \phi_b)
\]

Where:

- `Q` = mana flow;
- `G` = conductance;
- `φ` = arcane potential.

---

# 21. Channel overload

If:

```text
flow > safe throughput
```

possible consequences:

- tissue heating;
- arcane burns;
- channel rupture;
- coherence loss;
- involuntary discharge;
- local corruption;
- nerve damage;
- reservoir explosion.

This allows meaningful magical injury.

---

# 22. Regeneration

Regeneration should be decomposed into mechanisms:

```text
ambient absorption
+
metabolic refinement
+
ley connection
+
rest
+
external reservoir
```

A creature may regenerate quickly in one region and slowly in another.

---

# 23. Plants

Plants can be first-class arcane organisms.

Potential mechanisms:

- absorb ambient mana through leaves;
- absorb ley energy through roots;
- store mana in sap, fruit, seeds, wood, or flowers;
- filter spectrum;
- lower entropy;
- distribute mana through root/fungal networks;
- use mana for growth, defense, communication, or reproduction.

---

# 24. Magical forest as infrastructure

A large forest may behave as:

```text
distributed reservoir
+
purification network
+
living ley stabilizer
+
biological sensor network
```

Draining it may produce:

- wilting;
- reduced growth;
- failed reproduction;
- animal migration;
- corruption;
- reduced root strength;
- altered fire susceptibility;
- collapse of local mana quality.

---

# 25. Plant channels and tree physics

The vegetation system can store arcane state at:

- root nodes;
- trunk segments;
- branch junctions;
- foliage clusters;
- fruit/seed organs.

Example coupling:

```text
ArcaneDomain
   ↓ mana inflow
VegetationDomain
   ↓ growth / repair / stiffness change
Structural tree
```

A magical tree may:

- strengthen fibers;
- rapidly heal cuts;
- move branches;
- resist fire;
- alter root-soil coupling;
- emit a protective field.

---

# 26. Animals and magical creatures

Creatures may obtain mana from:

- food;
- ambient field;
- sunlight;
- prey;
- symbionts;
- specialized organs;
- environmental elements.

Examples:

## Dragon

```text
chemical metabolism
+
fire-spectrum mana
→ breath organ
→ pressure + heat + combustion
```

## Mana predator

```text
detect aura
→ attach channel
→ drain reservoir
```

## Elemental creature

```text
ambient field
+
physical carrier material
→ body stability
```

## Spirit-linked animal

```text
biological body
↔
IdentityDomain agent
```

---

# 27. Arcane ecology

A full cycle may be:

```text
ley node
   ↓
plants
   ↓
herbivores
   ↓
predators
   ↓
death
   ↓
decomposition / fungi
   ↓
soil and field
   ↓
ley network
```

This supports:

- mana food chains;
- magical parasites;
- seasonal flows;
- ecosystem collapse;
- migration;
- rare-resource zones;
- arcane agriculture;
- ecological conflict.

---

# 28. Arcane economy

If mana can perform physical work, it affects:

- energy production;
- transportation;
- mining;
- construction;
- agriculture;
- medicine;
- warfare;
- communication;
- logistics.

A world with abundant mana should not retain a conventional economy unchanged.

Important design questions:

- Is mana scarce?
- Is it renewable?
- Can it be transported?
- Can it be stored efficiently?
- Does storage degrade?
- Can anyone use it?
- Does use damage the environment?
- Are ley nodes politically controlled?
- Can mana replace fuel?

These are world-building consequences of the simulation laws.

---

# 29. Spell model

A spell should be represented as an executable graph.

```text
Intent
  ↓
Acquire
  ↓
Filter
  ↓
Shape
  ↓
Convert
  ↓
Couple
  ↓
Maintain
  ↓
Release
  ↓
Feedback
```

Conceptual type:

```rust
struct SpellGraph {
    nodes: Vec<SpellNode>,
    edges: Vec<SpellEdge>,
    constraints: SpellConstraints,
}
```

---

# 30. Spell node families

Possible generic node types:

## Source nodes

- internal reservoir;
- ambient field;
- crystal;
- ley node;
- external entity;
- target reservoir.

## Transport nodes

- channel;
- conduit;
- beam;
- tether;
- area link.

## Filter nodes

- spectrum filter;
- entropy separator;
- coherence stabilizer;
- amplifier;
- limiter.

## Shape nodes

- sphere;
- plane;
- shell;
- beam;
- vortex;
- volume;
- target-bound field.

## Conversion nodes

- mana → force;
- mana → heat;
- mana → light;
- mana → fluid pressure;
- mana → biological work;
- mana → matter transformation.

## Constraint nodes

- maintain distance;
- preserve shape;
- follow target;
- block category;
- limit energy;
- authorize identity.

## Release nodes

- impact;
- timer;
- threshold;
- command;
- proximity;
- target death.

---

# 31. Spell compilation

Named spells should compile to optimized graphs.

```text
"Fireball"
→ template
→ parameter binding
→ validated graph
→ optimized execution plan
```

This allows:

- authored spells;
- procedural spellcraft;
- rune construction;
- AI spell planning;
- artifacts;
- modding.

---

# 32. Spell runtime

```text
Player / AI intent
       ↓
Spell Planner
       ↓
Spell Graph
       ↓
Constraint validation
       ↓
Resource allocation
       ↓
Arcane execution
       ↓
Physical/Vital couplers
       ↓
Feedback
```

The runtime should track:

- quantity use;
- throughput;
- control load;
- coherence;
- entropy;
- environmental resistance;
- conversion efficiency;
- ongoing maintenance;
- instability.

---

# 33. Spell cost dimensions

A more complete cost model:

\[
C =
C_E +
C_C +
C_D +
C_T +
C_X +
C_S
\]

Where:

- \(C_E\) = energy cost;
- \(C_C\) = control complexity;
- \(C_D\) = distance;
- \(C_T\) = duration;
- \(C_X\) = conversion loss;
- \(C_S\) = stabilization against natural collapse.

Examples:

| Effect | Energy | Control | Throughput | Duration |
|---|---:|---:|---:|---:|
| Light | low | low | low | continuous |
| Illusion | low | high | medium | continuous |
| Fire impulse | high | medium | high | short |
| Shield | high | high | continuous | continuous |
| Healing | medium | very high | medium | variable |
| Teleportation | extreme | extreme | burst | instant |
| Curse | low–medium | extreme | low | persistent |

---

# 34. Three classes of magic

## 34.1. Catalytic magic

Mana controls an already possible process.

Examples:

- ignite existing fuel;
- guide existing flame;
- redirect flowing water;
- accelerate natural healing;
- trigger a chemical reaction;
- induce failure in a stressed structure.

Advantages:

- low energy cost;
- strong environmental dependence;
- physically grounded gameplay.

---

## 34.2. Transductive magic

Mana converts directly into physical energy.

Examples:

- telekinetic force;
- direct heating;
- light;
- shield pressure;
- shockwave;
- acceleration.

Constraint:

\[
E_{\text{physical}}
\le
\eta E_{\text{arcane}}
+
E_{\text{external}}
\]

Where `η` is conversion efficiency.

---

## 34.3. Ontological magic

Changes identity, topology, space, causality, or existence.

Examples:

- teleportation;
- matter creation;
- resurrection;
- true transformation;
- time manipulation;
- soul binding;
- rewriting ownership or oaths.

This should not be implemented as an expensive transductive spell.

It requires additional domains and rules.

---

# 35. Arcane-to-rigid coupling

`ArcaneRigidCoupler` converts spell state into physical forces and constraints.

Examples:

## Telekinesis

```text
arcane field
→ force / torque
→ rigid body
```

Do not write directly to transforms.

## Barrier

```text
field surface
↔ contact impulses
↔ projectile / body
```

The barrier must exchange momentum.

## Levitation

```text
continuous force
→ counters gravity
→ ongoing mana throughput
```

---

# 36. Arcane-to-articulated coupling

Possible interactions:

- push limbs;
- assist movement;
- paralyze joints;
- increase actuator strength;
- alter balance;
- control ragdolls;
- animate constructs.

The coupler should use:

- joint torques;
- target constraints;
- external impulses.

It should not bypass articulated physics unless the spell explicitly changes physical laws.

---

# 37. Arcane-to-continuum coupling

## Water manipulation

```text
arcane field
→ pressure / momentum
→ ContinuumDomain
```

The spell acts on real water.

## Mud/soil manipulation

```text
arcane stress
→ yield / displacement
→ soil deformation
```

## Sand manipulation

```text
localized force
→ granular flow
```

## Ice

Possible mechanisms:

- remove heat;
- change phase;
- impose structure;
- create temporary arcane lattice.

Each choice has different cost and persistence.

---

# 38. Arcane-to-thermal coupling

Fire magic should normally inject:

- heat;
- ignition energy;
- plasma-like energy;
- radiation;
- catalytic lowering of ignition barrier.

Then existing thermal and combustion systems determine:

- whether wet wood ignites;
- whether water boils;
- whether metal melts;
- whether fire spreads.

---

# 39. Arcane-to-atmosphere coupling

Possible effects:

- pressure waves;
- wind;
- local vortices;
- fog condensation;
- lightning initiation;
- oxygen transport;
- smoke control.

A "wind spell" should alter an atmospheric field or apply aerodynamic forces, not just play an animation.

---

# 40. Arcane-to-vegetation coupling

Possible effects:

- accelerated growth;
- healing cuts;
- branch motion;
- increased fiber stiffness;
- root extension;
- moisture transport;
- mana extraction;
- fire resistance;
- forced fruiting.

The vegetation domain remains source of truth for actual tree structure.

---

# 41. Arcane-to-vital coupling

`VitalDomain` should own:

- metabolism;
- tissue state;
- healing;
- growth;
- disease;
- biological resources.

Healing is not simply:

```text
hp += 50
```

A more physical healing process may require:

- energy;
- biomass;
- structural information;
- blood supply;
- removal of damaged tissue;
- time.

Mana can:

- accelerate repair;
- supply energy;
- stabilize structure;
- guide growth;
- suppress inflammation;
- reconstruct missing information.

---

# 42. VitalDomain

Suggested architecture:

```text
VitalDomain
│
├── Metabolism
├── TissueState
├── Damage
├── Healing
├── Growth
├── Disease
├── BioArcaneChannels
└── Homeostasis
```

The full biological simulation can be reduced-order.

The important point is ownership and consistent coupling.

---

# 43. IdentityDomain

Mana should not automatically equal soul.

A separate optional domain may own:

- identity continuity;
- soul anchors;
- true names;
- persistent bonds;
- memory continuity;
- ownership;
- oaths;
- authority.

```text
IdentityDomain
│
├── IdentityHandle
├── SoulAnchor
├── BondGraph
├── Authority
├── TrueName
└── ContinuityRules
```

---

# 44. Resurrection

A resurrection effect may require:

```text
mana
→ energy and repair

VitalDomain
→ rebuild body

IdentityDomain
→ restore the correct person
```

This avoids treating a person as interchangeable energy.

---

# 45. Divine magic

Divine power can be modeled as:

```text
caster
→ authorization
→ remote reservoir
→ constrained effect graph
```

Differences from ordinary mana:

- source is external;
- access can be revoked;
- permitted effects may be restricted;
- authority may matter more than personal capacity;
- the source may impose goals or costs.

---

# 46. Runes

Runes should function as persistent program elements.

Possible roles:

- source routing;
- spectrum filtering;
- field shaping;
- stabilization;
- timing;
- target selection;
- safety interlock;
- authorization;
- feedback.

A rune sequence is a spatially embedded spell graph.

---

# 47. Magic circles

A circle may define:

- a closed boundary;
- directional flow;
- containment;
- resonance;
- amplification;
- exclusion;
- identity permission;
- topology.

It can be represented as a graph plus geometric constraints.

---

# 48. Artifacts

An artifact is a persistent compiled arcane device.

```text
Artifact
│
├── reservoir
├── conduits
├── graph
├── materials
├── trigger
├── permissions
└── maintenance state
```

Examples:

## Wand

- focusing geometry;
- low capacity;
- high control;
- directional output.

## Staff

- high throughput;
- thermal dissipation;
- large conduits.

## Crystal battery

- high capacity;
- resonance;
- slow charge/discharge.

## Enchanted sword

- material reservoir;
- contact-triggered effect;
- identity lock.

---

# 49. Magitech

Because magic has circuit-like primitives, civilization can engineer:

- power grids;
- mana pumps;
- transport gates;
- magical factories;
- agricultural amplifiers;
- medical devices;
- communication networks;
- defensive wards.

The engine architecture should support devices built from the same primitives as spells.

---

# 50. Spell instability

Instability may arise from:

- insufficient control;
- low coherence;
- overload;
- incompatible spectrum;
- damaged channels;
- environmental interference;
- moving target;
- broken rune geometry;
- anti-magic.

Possible outcomes:

- collapse;
- premature release;
- backflow;
- altered target;
- entropy burst;
- physical explosion;
- corruption.

---

# 51. Backlash

Backlash should emerge from stored field energy and failed constraints.

Example:

```text
large maintained barrier
→ caster loses control
→ field collapses inward
→ residual energy returns through channels
→ overload injury
```

This is more systemic than a random critical failure.

---

# 52. Corruption

Corruption can be represented as:

- high entropy;
- spectrum distortion;
- persistent unstable attractor;
- parasitic effect graph;
- foreign identity influence.

Different settings may choose one or combine them.

Corruption may affect:

- organisms;
- land;
- mana fields;
- artifacts;
- ley nodes;
- spells.

---

# 53. Arcane weather and seasons

The world may have:

- mana tides;
- eclipses;
- ley storms;
- seasonal spectra;
- auroras;
- dimensional pressure changes.

These can alter:

- regeneration;
- spell stability;
- creature behavior;
- plant growth;
- anomaly probability.

---

# 54. Representation hierarchy

A full-resolution field everywhere is unnecessary.

Recommended hierarchy:

```text
World scale
→ sparse ley network

Region scale
→ aggregate arcane state

Local active region
→ sparse/adaptive field

Organism
→ reservoirs + channel graph

Artifact
→ circuit graph

Spell
→ temporary execution graph and local field
```

---

# 55. Arcane RepresentationManager

The existing world `RepresentationManager` should support arcane state.

Responsibilities:

- activate local fields;
- increase resolution near active spells;
- compress inactive regions;
- transfer state;
- allocate GPU resources;
- predict magical relevance;
- stream ley regions.

---

# 56. Update frequencies

Suggested ranges:

```text
global ley network:
0.1–1 Hz

regional ecology:
1–5 Hz

plant/creature reservoirs:
5–20 Hz

artifact circuits:
5–60 Hz

active local fields:
15–60 Hz

spell control:
60–120 Hz

VFX:
render frequency
```

The scheduler should be multi-rate.

---

# 57. World integration

Updated umbrella architecture:

```text
WorldDynamics
│
├── PhysicalWorld
│   ├── RigidDomain
│   ├── ArticulatedDomain
│   ├── ContinuumDomain
│   ├── VegetationDomain
│   ├── AtmosphereDomain
│   └── ThermalDomain
│
├── ArcaneDomain
│   ├── LeyNetwork
│   ├── RegionalField
│   ├── LocalField
│   ├── Reservoirs
│   ├── Conduits
│   ├── Spectrum
│   ├── Materials
│   └── Anomalies
│
├── VitalDomain
├── IdentityDomain
├── SpellRuntime
├── CouplingGraph
├── RepresentationManager
├── SimulationScheduler
├── Persistence
├── Replication
└── WorldQueries
```

---

# 58. Ownership rules

## Arcane field

Owner:

```text
ArcaneDomain
```

## Creature mana reserve

Owner:

```text
ArcaneDomain or BioArcane component
```

One choice must be standardized.

## Tissue repair

Owner:

```text
VitalDomain
```

## Rigid-body position

Owner:

```text
RigidDomain
```

## Tree structure

Owner:

```text
VegetationDomain
```

## Soul/identity

Owner:

```text
IdentityDomain
```

Couplers may request changes but should not duplicate source-of-truth state.

---

# 59. Coupling graph

```text
                         ArcaneDomain
                               │
       ┌───────────────┬───────┼────────┬──────────────┐
       │               │       │        │              │
      Rigid       Continuum  Thermal  Vegetation      Vital
       │               │       │        │              │
       └─────────────── CouplingGraph ─────────────────┘
                               │
                           Identity
```

Each coupling should be explicit and separately testable.

---

# 60. Coupler contracts

A coupler should specify:

- source domain;
- target domain;
- transferred quantity;
- conservation rule;
- update frequency;
- feedback;
- failure behavior.

Example:

```text
ArcaneThermalCoupler

input:
arcane quantity, spectrum, efficiency

output:
heat flux

feedback:
entropy generation, residual mana

invariant:
physical energy ≤ converted arcane energy + external source
```

---

# 61. Semantic world queries

Gameplay and AI should not inspect low-level field samples directly.

Possible queries:

```rust
struct ArcaneEnvironmentSample {
    density: f32,
    potential: f32,
    coherence: f32,
    entropy: f32,
    spectrum: ArcaneSpectrum,

    ley_direction: Vec3,
    instability: f32,
}
```

Organism query:

```rust
struct ArcaneCapability {
    reserve: f32,
    capacity: f32,
    max_throughput: f32,
    control_available: f32,
    overload_risk: f32,
}
```

---

# 62. AI integration

An NPC can reason about:

- local ambient mana;
- personal reserve;
- regeneration;
- ley proximity;
- material availability;
- target resistance;
- environmental consequences;
- overload risk;
- collateral damage.

Architecture:

```text
Strategic Agent
       ↓
Tactical Controller
       ↓
Spell Planner
       ↓
Spell Graph
       ↓
Arcane Runtime
       ↓
World Domains
```

---

# 63. Spell planning example

Goal:

```text
block enemy path
```

Planner alternatives:

```text
raise soil wall
freeze nearby water
fell damaged tree
create force barrier
ignite vegetation
```

The planner can compare:

- mana cost;
- available materials;
- duration;
- control complexity;
- ecological damage;
- tactical value.

---

# 64. Magic perception

Creatures may sense:

- density;
- spectrum;
- flow;
- reservoirs;
- recent spell traces;
- identity signatures.

This becomes another perception channel for AI.

Detection may depend on:

- sensitivity;
- distance;
- concealment;
- coherence;
- spectrum match;
- environmental noise.

---

# 65. Persistence

Persistent arcane state may include:

- ley-node modifications;
- regional depletion;
- corruption;
- active artifacts;
- long-lived wards;
- organism reserves;
- channel damage;
- permanent enchantments;
- ritual structures.

Do not store high-resolution field samples for inactive regions.

Store reduced state and reconstruct.

---

# 66. Sparse persistence

Pristine regions can be generated from deterministic world data.

Only save deltas:

```text
ley edge damaged
node drained
artifact installed
corruption increased
ward created
regional spectrum changed
```

---

# 67. Persistent spells

Long-lived effects should be represented as persistent compiled graphs with state.

```rust
struct PersistentArcaneEffect {
    graph_template: SpellTemplateId,
    bound_parameters: PackedParameters,

    source_binding: SourceBinding,
    target_binding: TargetBinding,

    remaining_reserve: f32,
    stability: f32,
    entropy: f32,
}
```

---

# 68. Networking

Do not replicate the full local field every frame.

Server authoritative for:

- resource consumption;
- spell acceptance;
- topology-changing effects;
- major physical consequences;
- persistent enchantments;
- identity changes.

Clients may simulate:

- field animation;
- minor fluctuations;
- VFX;
- secondary particles.

---

# 69. Network spell event

Conceptual:

```rust
struct SpellExecutionEvent {
    caster: EntityId,
    graph_id: SpellGraphId,
    parameters: PackedSpellParameters,

    source_state: PackedArcaneSource,
    seed: u64,
    start_tick: u64,
}
```

Physical domain results remain authoritative.

---

# 70. Determinism

Bitwise deterministic GPU fields may be unnecessary.

The server should decide:

- target;
- mana cost;
- success/failure;
- major impulses;
- ignition;
- fracture;
- persistent changes.

Clients can differ slightly in visual field behavior.

---

# 71. Debugging tools

Visualize:

- mana density;
- potential;
- flux;
- coherence;
- entropy;
- spectrum;
- ley edges;
- organism reservoirs;
- channel load;
- overload;
- spell graph execution;
- conversion efficiency;
- anti-magic zones;
- coupler transfers.

Without this, systemic magic will be difficult to debug.

---

# 72. Validation

Magic requires tests just as physical solvers do.

## Conservation tests

- reservoir transfer;
- closed-loop flow;
- conversion loss;
- leakage;
- entropy increase;
- capacity saturation.

## Channel tests

- steady flow;
- overload;
- damaged channel;
- parallel paths;
- feedback.

## Field tests

- diffusion;
- ley transport;
- local depletion;
- source/sink equilibrium;
- anti-magic absorption.

## Coupler tests

- mana-to-force;
- mana-to-heat;
- mana-to-water pressure;
- mana-to-growth;
- barrier impulse exchange.

## Spell tests

- stable graph;
- insufficient reserve;
- insufficient throughput;
- control overload;
- interrupted maintenance;
- incompatible spectrum.

---

# 73. Integrated scenarios

## 73.1. Forest extraction

```text
mage draws ambient mana
→ local potential falls
→ plants use reserves
→ leaves wilt
→ animal behavior changes
→ recovery occurs over time
```

---

## 73.2. Physical fire spell

```text
caster reservoir
→ heat conversion
→ dry branch ignition
→ combustion
→ structural weakening
→ branch fracture
→ ember spread
```

---

## 73.3. Hydromancy

```text
nearby water body
→ arcane pressure
→ real water flow
→ object displacement
→ soil erosion
```

---

## 73.4. Geomancy

```text
mana
→ stress field
→ soil yields
→ wall rises
→ roots and water are displaced
```

---

## 73.5. Magical overload

```text
large reserve
→ excessive flow
→ channel damage
→ coherence collapse
→ backflow
→ organism injury
```

---

## 73.6. Ley-city economy

```text
city draws ley power
→ downstream regions lose potential
→ agriculture declines
→ political conflict emerges
```

---

# 74. Performance strategy

Use a budget per representation:

```text
global ley nodes:
thousands

regional summaries:
streamed regions

active local field regions:
dozens

active spell graphs:
hundreds

detailed organism channels:
nearby relevant entities
```

Exact targets depend on hardware.

---

# 75. GPU use

Good GPU candidates:

- local field sampling;
- flux evaluation;
- spell-shape evaluation;
- spectrum operations;
- many artifact circuits;
- visualization;
- relevance evaluation.

CPU candidates:

- sparse global ley graph;
- high-level spell planning;
- persistence;
- identity rules;
- authoritative topology events.

---

# 76. Rust architecture proposal

```text
crates/
├── arcane-core
│   ├── quantity
│   ├── potential
│   ├── spectrum
│   ├── coherence
│   ├── entropy
│   └── units
│
├── arcane-field
│   ├── regional
│   ├── local
│   ├── sources
│   ├── sinks
│   ├── flux
│   └── anomalies
│
├── arcane-ley
│   ├── graph
│   ├── nodes
│   ├── edges
│   ├── flow
│   └── streaming
│
├── arcane-reservoir
│   ├── storage
│   ├── transfer
│   ├── leakage
│   └── saturation
│
├── arcane-organism
│   ├── channels
│   ├── throughput
│   ├── overload
│   ├── affinity
│   └── regeneration
│
├── arcane-materials
│   ├── conductivity
│   ├── resonance
│   ├── absorption
│   └── anti_magic
│
├── arcane-spell
│   ├── graph
│   ├── nodes
│   ├── compiler
│   ├── runtime
│   ├── stability
│   └── planner
│
├── arcane-artifacts
│   ├── circuits
│   ├── runes
│   ├── circles
│   └── triggers
│
├── arcane-coupling
│   ├── rigid
│   ├── articulated
│   ├── continuum
│   ├── thermal
│   ├── atmosphere
│   ├── vegetation
│   ├── vital
│   └── identity
│
├── vital-domain
├── identity-domain
│
├── arcane-persistence
├── arcane-replication
├── arcane-query
├── arcane-debug
└── arcane-validation
```

Initially, use modules inside one or two crates rather than creating all crates immediately.

---

# 77. Minimal engine interfaces

Conceptual API:

```rust
trait ArcaneFieldQuery {
    fn sample(&self, position: Vec3) -> ArcaneEnvironmentSample;
}

trait ArcaneReservoirAccess {
    fn available(&self, entity: EntityId) -> ArcaneCapability;
    fn transfer(&mut self, request: ArcaneTransferRequest)
        -> ArcaneTransferResult;
}

trait SpellExecutor {
    fn begin(&mut self, request: SpellExecutionRequest)
        -> Result<SpellInstanceId, SpellError>;

    fn stop(&mut self, instance: SpellInstanceId);
}
```

Gameplay code should not directly modify internal field buffers.

---

# 78. Material capability integration

Extend the common material registry.

```rust
struct MaterialDefinition {
    mechanical: Option<MechanicalMaterial>,
    fluid: Option<FluidMaterial>,
    thermal: Option<ThermalMaterial>,
    porous: Option<PorousMaterial>,
    fracture: Option<FractureMaterial>,
    combustion: Option<CombustionMaterial>,
    arcane: Option<ArcaneMaterialProperties>,
}
```

This allows physical and magical interaction to use the same material identity.

---

# 79. Security and authorization model

Persistent magical systems may need:

- ownership;
- permissions;
- identity locks;
- faction access;
- true-name binding;
- revocation.

This belongs at the border of `ArcaneDomain` and `IdentityDomain`.

Artifacts should not rely only on gameplay tags if identity is part of the setting's metaphysics.

---

# 80. Modding and data-driven content

The engine should expose:

- spectrum definitions;
- material responses;
- spell graph nodes;
- templates;
- artifact circuits;
- organism channel layouts;
- source/sink definitions.

The engine core should not hardcode lore schools.

---

# 81. Major architectural risks

## 81.1. Infinite energy

Mana regeneration plus efficient physical conversion may trivialize the economy.

Mitigation:

- explicit inflow;
- conversion efficiency;
- entropy;
- transport constraints;
- storage limits;
- ecological cost.

---

## 81.2. Matter creation

Creating matter affects:

- mass conservation;
- economy;
- terrain;
- climate;
- logistics.

Prefer:

- transfer;
- transformation;
- condensation;
- temporary stabilized constructs.

True creation belongs to ontological magic.

---

## 81.3. Universal scalar mana

A single scalar cannot model:

- quality;
- control;
- spectrum;
- stability;
- throughput.

Use multiple state dimensions.

---

## 81.4. Too much field simulation

A dense world field is expensive and often invisible.

Use sparse hierarchy and local activation.

---

## 81.5. Hardcoded spell subsystems

Avoid separate core systems such as:

```text
FireballSystem
HealSystem
TeleportSystem
```

Build them from graph nodes and couplers where possible.

---

## 81.6. Magical bypass of physics

Direct transform changes, instant material deletion, and fake damage will break systemic interaction.

Use couplers unless the spell explicitly belongs to ontological magic.

---

## 81.7. Overly general metaphysics too early

Do not begin with:

- souls;
- fate;
- timelines;
- resurrection;
- causality rewriting.

Start with reservoirs, fields, and physical couplers.

---

# 82. Recommended implementation roadmap

## Phase A — Arcane quantities and reservoirs

Implement:

- quantity;
- capacity;
- transfer;
- leakage;
- coherence;
- entropy;
- spectrum.

Tests:

- reservoir transfer;
- saturation;
- loss.

---

## Phase B — Organism channels

Implement:

- channel graph;
- conductance;
- throughput;
- overload;
- recovery.

Demo:

```text
mage charges
→ casts burst
→ channel overload
→ temporary impairment
```

---

## Phase C — Three physical couplers

Implement:

```text
mana → force
mana → heat
mana → continuum pressure
```

This unlocks:

- telekinesis;
- physical fire;
- water manipulation;
- simple barriers.

---

## Phase D — Spell graph runtime

Implement:

- graph format;
- validation;
- source nodes;
- conversion nodes;
- shape nodes;
- maintenance;
- release;
- feedback.

---

## Phase E — Arcane materials and artifacts

Implement:

- conductivity;
- capacity;
- resonance;
- crystals;
- runes;
- simple devices.

---

## Phase F — Regional field and ley graph

Implement:

- sparse ley network;
- regional state;
- source/sink flow;
- depletion;
- regeneration.

---

## Phase G — Plants and arcane ecology

Implement:

- plant reservoirs;
- root/leaf absorption;
- environmental depletion;
- growth coupling;
- forest recovery.

---

## Phase H — VitalDomain

Implement reduced-order:

- tissue damage;
- healing;
- biomass requirement;
- mana-assisted repair.

---

## Phase I — Anti-magic and interference

Implement:

- absorption;
- coherence decay;
- spectrum cancellation;
- entropy injection.

---

## Phase J — Persistence and networking

Implement:

- regional field deltas;
- persistent artifacts;
- spell events;
- authoritative resource use.

---

## Phase K — IdentityDomain

Only after physical magic works.

Implement:

- identity handles;
- bonds;
- authorization;
- soul anchors if required.

---

## Phase L — Ontological magic

Research:

- teleportation;
- transformation;
- matter creation;
- resurrection;
- causality.

Each should receive an individual specification.

---

# 83. Recommended first milestone

## Milestone: `Arcane Physical Interaction`

Requirements:

- one mage reservoir;
- channel throughput;
- spell graph runtime;
- mana-to-force coupler;
- mana-to-heat coupler;
- mana-to-water-pressure coupler;
- debug visualization.

Demo:

```text
mage accumulates mana
→ lifts rigid object
→ heats wet and dry wood
→ moves real water
→ overloads channel if output is too high
```

No:

- ley network;
- ecology;
- souls;
- artifacts;
- teleportation.

This proves the foundation.

---

# 84. Recommended second milestone

## Milestone: `Arcane Ecology`

Requirements:

- regional mana state;
- plants absorb/store mana;
- mage can draw from environment;
- depletion affects plants;
- recovery over time;
- simple ley source.

Demo:

```text
healthy forest
→ mage extracts mana
→ vegetation weakens
→ ambient field drops
→ forest slowly recovers
```

---

# 85. Recommended third milestone

## Milestone: `Arcane Engineering`

Requirements:

- crystals;
- runes;
- persistent spell circuits;
- artifact reservoirs;
- material resonance;
- anti-magic material.

Demo:

```text
build mana battery
→ route through rune circuit
→ maintain barrier
→ anti-magic alloy disrupts field
```

---

# 86. Candidate ADRs

1. `ADR-Arcane-World-Ontology.md`
   - what mana is;
   - quantity/potential/coherence/entropy/spectrum.

2. `ADR-Arcane-Conservation-Model.md`
   - closed/open/hybrid world.

3. `ADR-Arcane-Field-Representation.md`
   - ley graph;
   - regional state;
   - local fields.

4. `ADR-Arcane-Material-Capabilities.md`
   - conductivity;
   - resonance;
   - absorption.

5. `ADR-BioArcane-Organism.md`
   - reservoirs;
   - channels;
   - throughput.

6. `ADR-Spell-Graph-Runtime.md`
   - nodes;
   - validation;
   - execution.

7. `ADR-Arcane-Physical-Coupling.md`
   - force;
   - heat;
   - continuum.

8. `ADR-Arcane-Ecology.md`
   - plants;
   - creatures;
   - depletion.

9. `ADR-Anti-Magic.md`
   - physical interpretation.

10. `ADR-Identity-Separation.md`
    - why soul is not mana.

11. `ADR-Arcane-Persistence.md`

12. `ADR-Arcane-Network-Replication.md`

13. `ADR-Ontological-Magic-Boundary.md`
    - which effects need separate metaphysics.

---

# 87. Candidate specifications

- `SPEC-Arcane-Units.md`
- `SPEC-Arcane-Spectrum.md`
- `SPEC-Mana-Reservoir.md`
- `SPEC-Arcane-Transfer.md`
- `SPEC-Arcane-Channel-Graph.md`
- `SPEC-Arcane-Overload.md`
- `SPEC-Ley-Network.md`
- `SPEC-Regional-Arcane-State.md`
- `SPEC-Local-Arcane-Field.md`
- `SPEC-Arcane-Material-Properties.md`
- `SPEC-Spell-Graph-Format.md`
- `SPEC-Spell-Graph-Compiler.md`
- `SPEC-Spell-Runtime.md`
- `SPEC-Arcane-Rigid-Coupler.md`
- `SPEC-Arcane-Thermal-Coupler.md`
- `SPEC-Arcane-Continuum-Coupler.md`
- `SPEC-Arcane-Vegetation-Coupler.md`
- `SPEC-BioArcane-Organism.md`
- `SPEC-Plant-Mana-Ecology.md`
- `SPEC-Anti-Magic-Field.md`
- `SPEC-Arcane-Artifact.md`
- `SPEC-Arcane-Persistence.md`
- `SPEC-Arcane-Replication.md`
- `SPEC-Arcane-Debug-Tools.md`
- `SPEC-Arcane-Validation.md`

---

# 88. Candidate research spikes

- `SPIKE-Arcane-Field-Representation.md`
- `SPIKE-Spectrum-Dimensionality.md`
- `SPIKE-Spell-Graph-Optimization.md`
- `SPIKE-Field-Interference.md`
- `SPIKE-Arcane-Ecology-Balance.md`
- `SPIKE-Plant-Reservoir-Coupling.md`
- `SPIKE-Mana-To-Physical-Energy.md`
- `SPIKE-Barrier-Contact-Solver.md`
- `SPIKE-Distributed-Rituals.md`
- `SPIKE-Identity-Domain.md`
- `SPIKE-Teleportation-Topology.md`
- `SPIKE-Resurrection-Continuity.md`

---

# 89. Normative requirements

## MUST

- The engine MUST separate mana quantity from control complexity.
- The engine MUST define explicit sources and sinks.
- The engine MUST prevent silent creation of physical energy.
- The `ArcaneDomain` MUST couple to physical domains through explicit couplers.
- Physical state MUST remain owned by its physical domain.
- Spell execution MUST report resource use, instability, and failure.
- High-resolution arcane fields MUST be local/adaptive.
- Persistent effects MUST have a compact serializable representation.
- Soul/identity MUST NOT be implicitly reduced to mana quantity.

## SHOULD

- Mana SHOULD include quality/coherence and spectrum.
- Organisms SHOULD distinguish capacity and throughput.
- Spells SHOULD be represented as data-driven graphs.
- Anti-magic SHOULD be modeled as field interaction rather than a boolean.
- Plants SHOULD participate in arcane ecology.
- Named spells SHOULD compile into reusable graph templates.
- Network replication SHOULD send authoritative events rather than full field state.

## MAY

- The world MAY conserve total mana.
- Mana MAY be generated biologically.
- Divine power MAY use remote reservoirs.
- Matter creation MAY be supported as ontological magic.
- IdentityDomain MAY be omitted in games without souls or true-name mechanics.

---

# 90. Central architectural statement

> **ArcaneDomain is a simulated world substrate with quantity, potential, coherence, entropy, spectrum, sources, sinks, reservoirs, conduits, and fields. SpellRuntime executes constrained effect graphs that transform and route this substrate through explicit couplers into physical, biological, and informational consequences.**

---

# 91. Project thesis

> **Build magic into the engine as a first-class world law: creatures, plants, artifacts, locations, and civilizations can accumulate, conduct, transform, sense, and consume mana, while spells operate as data-driven physical programs that interact with the same water, soil, trees, fire, bodies, and energy systems as the rest of the simulated world.**

---

# 92. Final recommendation

The first implementation should not attempt to solve all metaphysics.

Begin with:

```text
reservoirs
+
channels
+
throughput
+
spell graph
+
mana-to-force
+
mana-to-heat
+
mana-to-continuum pressure
```

This already creates a fundamentally different magic system:

- telekinesis pushes actual bodies;
- fire heats actual materials;
- water magic moves actual water;
- channel overload injures the caster;
- mana has a source and a cost.

Then add:

```text
regional field
→ ley network
→ plants
→ ecology
→ artifacts
→ anti-magic
→ healing
→ identity
→ ontological magic
```

The result is not merely a spell system.

It is a coherent magical layer of the world that can generate ecology, technology, religion, economics, combat, exploration, and emergent interactions from the same engine-level laws.
