# World Dynamics Follow-up
## Unified Physical, Thermochemical, Biological, and Arcane World Architecture

**Document type:** umbrella architecture and research paper  
**Status:** follow-up and correction paper for specification generation  
**Context date:** August 2026  
**Target:** custom Rust game engine with a systemic simulated world  
**Primary consumers:** architecture agent, specification agent, engine developers, physics/ML research agents

**Related documents:**

- `physical_world_layer_architecture_for_specs.md`
- `continuum_physics_water_mud_offroad_research_brief.md`
- `vegetation_physics_destructible_trees_research.md`
- `vegetation_physics_followup_production_architecture.md`
- `arcane_world_layer_magic_architecture_paper.md`
- `neural_assisted_world_physics_architecture_paper.md`

---

# Abstract

This paper updates the world architecture after adding three major requirements:

1. water, mud, soil, trees, rigid bodies, atmosphere, and destruction must coexist in one coupled physical world;
2. heat, cold, ice, steam, fire, corrosion, dissolution, and simplified chemistry must be represented as general material processes rather than unrelated gameplay effects;
3. mana and magic must exist as first-class world laws that can exchange energy, momentum, matter, phase state, biological work, and information with the physical world.

The central correction is ontological:

> **Fire is not a fundamental substance or standalone physical domain. It is a family of exothermic reactions. Cold is not a positive substance; it is low thermal state or removal of enthalpy. Ice is a mechanically active solid phase of water. Chemistry transforms material composition while preserving explicitly defined mass and energy. Magic is a separate arcane substrate that may couple to these processes through constrained, auditable conversions.**

The proposed top-level architecture is named `WorldDynamics`. It orchestrates specialized domains rather than replacing them with a universal solver:

```text
WorldDynamics
│
├── Physical Domains
│   ├── RigidDomain
│   ├── ArticulatedDomain
│   ├── ContinuumDomain
│   ├── VegetationDomain
│   └── AtmosphereDomain
│
├── ThermochemicalLayer
│   ├── ThermalTransport
│   ├── PhaseTransitionRuntime
│   ├── ReactionRuntime
│   └── Process Packages
│
├── ArcaneDomain
├── VitalDomain
├── IdentityDomain            // optional, setting-dependent
│
├── CouplingGraph
├── RepresentationManager
├── SimulationScheduler
├── MaterialRegistry
├── ConservationLedger
├── Persistence / Streaming
├── Replication
├── WorldQueries
└── CausalTrace / Validation
```

The engine must remain pragmatic. It should not simulate molecular chemistry, atmospheric CFD, or full multiphysics at maximum resolution across the whole world. It should use sparse state, reduced-order representations, local activation, specialized solvers, and explicit couplers.

The project thesis is:

> **Build one causally connected world from multiple specialized solvers, where matter has composition, phase, temperature, mechanical state, biological state, and optional arcane state; where heat, reactions, magic, and physical motion exchange well-defined quantities; and where high-fidelity simulation is activated only where it can affect gameplay.**

---

# 0. Status of previous architectural decisions

This document does not replace all previous papers. It refines their common umbrella architecture.

The following previous decisions remain valid:

- one physical world, multiple specialized solvers;
- explicit state ownership;
- couplers instead of direct cross-domain mutation;
- representation promotion and adaptive fidelity;
- compact persistent state;
- deterministic classical solvers as source of truth;
- neural networks as accelerators, predictors, and LOD controllers rather than unvalidated authorities;
- semantic queries for AI and gameplay;
- topology-changing events must be authoritative and persistent.

The following decisions are revised:

## Previous simplification

```text
ThermalDomain
FireDomain
```

## Updated model

```text
ThermochemicalLayer
├── ThermalTransport
├── PhaseTransitions
├── ReactionRuntime
└── CombustionProcess
```

`FireSystem` may still exist as a scheduling, propagation, VFX, or gameplay-facing subsystem, but it is not the source of truth for fuel, heat, reaction extent, char, gas products, or structural weakening.

Similarly, `ColdSystem`, `IceDamage`, and `AcidDamage` should not be foundational engine concepts.

Their physical meanings are:

```text
cold effect
→ negative heat flux / enthalpy removal / phase stabilization

ice
→ solid water phase with mechanical state

acid
→ reactive mixture with concentration and material-specific reactions

poison
→ chemical species interpreted by VitalDomain

explosion
→ rapid energy release + product expansion + pressure impulse
```

---

# 1. Central architectural decision

The engine should represent the world as several interacting substrates and processes.

```text
Matter
+
Mechanical state
+
Thermal state
+
Chemical composition
+
Biological organization
+
Arcane state
+
Identity state
```

Not every entity has every category.

Examples:

```text
Stone
├── matter
├── mechanical
├── thermal
├── chemical
└── optional arcane

Tree
├── matter
├── structural mechanics
├── thermal
├── chemical
├── biological
├── optional arcane
└── persistent damage/topology

Mage
├── articulated body
├── thermal/chemical body state
├── biological state
├── arcane reservoirs/channels
└── optional identity/soul

Water parcel
├── mass
├── momentum
├── enthalpy
├── phase fractions
├── dissolved species
└── optional arcane state
```

The core principle is:

> **World universality belongs in shared state semantics, conservation rules, material capabilities, coupling protocols, representation transitions, and debugging—not in one universal numerical method.**

---

# 2. Ontology: substance, state, field, process, and representation

Several concepts must remain distinct.

## 2.1. Substance / species

A species describes a compositional component:

- water;
- oxygen-like oxidizer;
- fuel vapour;
- salt;
- mineral;
- acid component;
- toxin;
- dry wood constituents;
- ash;
- char;
- mana-bearing reagent if the setting allows it.

A species is not necessarily rendered or simulated independently.

---

## 2.2. Material

A material is a physical mixture and constitutive definition.

Examples:

```text
FreshWood
├── cellulose-like solid fraction
├── water content
├── resin fraction
├── pores
└── biological state

Mud
├── water
├── clay
├── sand
├── dissolved salts
└── organic content

Brine
├── water
└── dissolved salt
```

---

## 2.3. State

State is the current condition of a material parcel or body:

- position;
- velocity;
- stress;
- temperature/enthalpy;
- phase fractions;
- composition;
- porosity;
- saturation;
- damage;
- burn progress;
- arcane quantity/coherence/spectrum;
- biological viability.

---

## 2.4. Field

A field assigns a quantity over space or a graph:

- wind velocity;
- temperature approximation;
- pressure;
- ambient mana potential;
- ley-network flow;
- radiation exposure;
- humidity;
- contamination.

Fields may be represented by grids, particles, kernels, graphs, basis functions, or regional summaries.

---

## 2.5. Process

A process changes state:

- flow;
- heat conduction;
- melting;
- combustion;
- dissolution;
- corrosion;
- fracture;
- growth;
- healing;
- mana conversion;
- spell maintenance.

Fire belongs here.

---

## 2.6. Representation

Representation is the numerical or persistent form currently used:

```text
water:
spectral → shallow/reduced → particles → refined multiphase

ice:
phase mask → fracture graph → bonded solid → rigid fragments

tree:
shader → modal → rods → local fibers

chemistry:
aggregate progress → compact mixture → detailed local kinetics

mana:
ley graph → regional state → local field → spell field
```

A representation is not the world law itself.

---

# 3. Updated top-level architecture

```text
                              WorldDynamics
                                    │
        ┌───────────────────────────┼───────────────────────────┐
        │                           │                           │
 Physical Domains          ThermochemicalLayer           ArcaneDomain
        │                           │                           │
 Rigid / Articulated       heat / phase / reactions     mana / ley / fields
 Continuum / Vegetation    combustion / corrosion       reservoirs / channels
 Atmosphere                dissolution / precipitation  spell substrate
        │                           │                           │
        └───────────────────────────┼───────────────────────────┘
                                    │
                          VitalDomain / IdentityDomain
                                    │
                             CouplingGraph
                                    │
      ┌─────────────────────────────┼─────────────────────────────┐
      │                             │                             │
RepresentationManager      SimulationScheduler           ConservationLedger
      │                             │                             │
Persistence / Streaming      Multi-rate solves       mass / momentum / energy
Replication                  coupling iterations      arcane / topology audits
WorldQueries                 event ordering           causal trace
```

`WorldDynamics` does not numerically solve every subsystem. It owns orchestration, contracts, ordering, diagnostics, and global policies.

---

# 4. Domain responsibilities

# 4.1. `RigidDomain`

Owns:

- transforms;
- linear/angular velocity;
- mass and inertia;
- rigid contacts;
- rigid constraints;
- sleeping state.

Receives:

- fluid/soil impulses;
- tree collision impulses;
- thermal expansion or damage requests;
- explosion pressure impulses;
- telekinetic forces.

It does not own temperature, chemical composition, or mana unless explicitly embedded as components.

---

# 4.2. `ArticulatedDomain`

Owns:

- articulated body pose;
- joint state;
- actuators;
- contacts;
- character/vehicle mechanisms.

Receives:

- continuum contact;
- vegetation contact;
- thermal injury/constraint changes;
- arcane forces;
- biological actuator limitations.

This domain remains the physical execution target of the Motor System.

---

# 4.3. `ContinuumDomain`

Owns active continuum matter:

- water;
- mud;
- soil;
- sand;
- snow;
- slurry;
- sediment;
- other local deformable materials.

Owns:

- mass;
- position/velocity of material samples;
- pressure and deformation state;
- porosity and saturation;
- phase-dependent constitutive state;
- transport of dissolved species when active.

It does not independently decide chemical reactions. It supplies local mixture and transport state to `ThermochemicalLayer` and applies returned composition/phase updates.

---

# 4.4. `VegetationDomain`

Owns:

- plant/tree topology;
- structural graph;
- rod/modal state;
- wood damage;
- roots;
- foliage;
- biological growth state;
- local material parcels associated with wood and tissue.

Receives:

- wind loads;
- soil support;
- moisture mass transfer;
- thermal/chemical updates;
- arcane growth or extraction effects;
- rigid collisions.

Combustion changes material and strength, but graph fracture remains owned by vegetation structural mechanics.

---

# 4.5. `AtmosphereDomain`

First production levels may own reduced fields:

- wind;
- gusts;
- humidity;
- ambient temperature;
- precipitation;
- smoke/fog transport approximations;
- gas mixture summaries.

Later local regions may use more detailed gas transport.

It supplies oxidizer availability, convective heat transport, vapour movement, embers, and weather inputs.

---

# 4.6. `ThermochemicalLayer`

Owns shared algorithms and process state for:

- energy transfer;
- thermal equilibrium approximations;
- phase transitions;
- reaction selection;
- reaction progress;
- stoichiometric conversion;
- heat of reaction;
- catalysts and inhibitors;
- reaction-produced pressure/volume changes;
- process-specific causal records.

It does not own world-space transforms of matter.

---

# 4.7. `ArcaneDomain`

Owns:

- mana quantity;
- potential;
- coherence;
- entropy;
- spectrum;
- ambient field;
- ley network;
- reservoirs;
- conduits/channels;
- arcane materials;
- spell-field state;
- arcane anomalies.

It must not directly mutate rigid transforms, material masses, temperatures, or biological health. Those changes occur through explicit couplers.

---

# 4.8. `VitalDomain`

Owns reduced biological truth:

- tissue state;
- metabolism;
- injury;
- toxicity response;
- disease;
- healing;
- growth;
- homeostasis;
- biological resource constraints.

Chemical toxins and heat become biological consequences here.

---

# 4.9. `IdentityDomain`

Optional and setting-dependent.

May own:

- soul/identity continuity;
- true names;
- bonds;
- ownership/authority;
- oath state;
- resurrection identity anchors.

It must not be silently reduced to mana or chemical state.

---

# 5. Source-of-truth ownership

Each state variable must have exactly one authoritative owner.

| State | Owner |
|---|---|
| rigid transform | `RigidDomain` |
| articulated pose | `ArticulatedDomain` |
| water particle position | `ContinuumDomain` |
| tree topology | `VegetationDomain` |
| wind field | `AtmosphereDomain` |
| material composition | owning matter domain, updated through thermochemical transactions |
| reaction progress | `ThermochemicalLayer` process state |
| specific enthalpy | owning material state, transferred by thermochemical algorithms |
| mana reservoir | `ArcaneDomain` / standardized bio-arcane component |
| tissue viability | `VitalDomain` |
| identity continuity | `IdentityDomain` |

A coupler may propose or transact changes, but it must not introduce a second source of truth.

---

# 6. Unified material architecture

The common material system should separate immutable definitions from runtime state.

```rust
pub struct MaterialDefinition {
    pub mechanical: Option<MechanicalDefinition>,
    pub continuum: Option<ContinuumDefinition>,
    pub porous: Option<PorousDefinition>,
    pub thermal: Option<ThermalDefinition>,
    pub phase: Option<PhaseDiagramDefinition>,
    pub reactive: Option<ReactiveDefinition>,
    pub fracture: Option<FractureDefinition>,
    pub combustion: Option<CombustionDefinition>,
    pub biological: Option<BiologicalMaterialDefinition>,
    pub arcane: Option<ArcaneMaterialDefinition>,
}
```

Runtime state must remain compact and specialized.

Do not create one giant structure containing all possible fields for every sample.

Use capability-specific SoA pools or sparse component blocks.

---

# 7. Species, mixtures, and material parcels

Chemistry requires distinguishing material identity from compositional species.

```rust
pub struct SpeciesFraction {
    pub species: SpeciesId,
    pub mass_fraction: f32,
}

pub struct CompactMixture {
    pub components: SmallVec<[SpeciesFraction; 4]>,
}
```

Most parcels should carry only a few active species.

Examples:

```text
Pure water parcel:
[H2O-like species: 1.0]

Brine parcel:
[water: 0.95, salt: 0.05]

Wet wood section:
[dry wood: 0.68, water: 0.24, resin: 0.08]

Smoke parcel:
[hot gas products, soot, vapour]
```

The engine is not required to expose real-world molecular chemistry. Species may be fictional macroscopic components.

---

# 8. Thermodynamic state: enthalpy first

Temperature alone is insufficient when phase transitions occur.

Recommended source variable:

```rust
pub struct ThermalPhaseState {
    pub specific_enthalpy_j_per_kg: f32,

    // cached/derived values
    pub temperature_k: f32,
    pub solid_fraction: f32,
    pub liquid_fraction: f32,
    pub gas_fraction: f32,
}
```

A simplified relation:

\[
h = h_{sensible}(T) + f_l L_f + f_g L_v
\]

Where:

- `h` = specific enthalpy;
- `L_f` = latent heat of fusion;
- `L_v` = latent heat of vaporization;
- `f_l`, `f_g` = phase fractions.

This allows energy to continue changing during melting/freezing without unstable temperature toggling.

---

# 9. Thermal transport

The first production implementation should support:

- conduction between contacting/neighboring parcels;
- convection through atmosphere/continuum approximations;
- radiation approximation;
- phase-change energy;
- heat generated by reactions;
- mechanical dissipation converted to heat where relevant;
- arcane heat transfer.

Conceptual exchange:

```rust
pub struct HeatTransaction {
    pub source: WorldStateRef,
    pub target: WorldStateRef,
    pub energy_joules: f64,
    pub mechanism: HeatTransferMechanism,
}
```

`energy_joules` should be signed by a clear convention.

---

# 10. Cold is an energy-flow condition

There should be no generic conserved `cold_energy` in the normal physical model.

A cold effect is one of:

```text
negative heat flux
heat transfer to a colder reservoir
endothermic reaction
phase-change cooling
expansion cooling
arcane extraction of enthalpy
ontological heat deletion          // only if the setting explicitly allows it
```

A spell should therefore declare the destination or conversion rule for removed energy.

Examples:

```text
Target heat
→ crystal thermal reservoir

Target heat
→ ambient atmosphere

Target heat
→ arcane energy with efficiency loss

Target heat
→ extradimensional sink
```

The final option is valid only as an explicit world law.

---

# 11. Phase transitions

The `PhaseTransitionRuntime` maps enthalpy, pressure, composition, and nucleation state into phase fractions.

Required initial transitions:

```text
water ↔ ice
water ↔ vapour
snow/slush ↔ water
melted material ↔ solid material       // later, selected materials
```

Important features:

- latent heat;
- hysteresis;
- nucleation thresholds;
- pressure/composition-dependent transition points;
- gradual mixed phases;
- volume/density changes;
- representation promotion.

---

# 12. Water, ice, slush, and steam

A useful physical ladder:

```text
water
↓ cooling
water + ice crystals
↓
slush / bonded slurry
↓
continuous ice sheet
↓ fracture
ice fragments
↓ heating
meltwater
```

Each region may select a mechanical model from phase state:

```rust
pub enum WaterMechanicalRegime {
    Liquid,
    Slush,
    BondedIce,
    BrittleIce,
    Vapour,
    Mixed,
}
```

The transition should normally blend properties rather than switch at a single threshold.

---

# 13. Ice mechanics

Ice is not only a visual material.

It may require:

- elastic response;
- bending of sheets;
- brittle fracture;
- crack persistence;
- load-rate sensitivity;
- temperature-dependent strength;
- melting and refreezing;
- rigid fragments.

Recommended representation ladder:

```text
far frozen surface
→ static phase/strength mask

relevant surface
→ structural plate/fracture graph

local interaction
→ bonded particles / XPBD solid

hero research mode
→ continuum/MPM solid

fragment detached
→ RigidDomain
```

---

# 14. Freezing expansion

Water density changes during freezing.

A local active model may produce:

```text
pore water freezes
→ expansion pressure
→ soil heave
→ rock crack growth
→ pipe/container damage
```

This effect should be representation-gated.

It is not necessary to resolve expansion pressure in every distant frozen puddle.

---

# 15. Thermal stress

Material strain may include:

\[
\epsilon_{thermal} = \alpha \Delta T
\]

Temperature gradients can therefore create stress and fracture.

Applications:

- rapid cooling of hot stone;
- heated metal deformation;
- cracked glass;
- ice fracture;
- fire-weakened structures;
- freeze-thaw rock damage.

Thermal strain is sent to the owning mechanical/structural domain.

---

# 16. ReactionRuntime

A generic reaction is defined by:

```rust
pub struct ReactionDefinition {
    pub reactants: SmallVec<[StoichiometricTerm; 4]>,
    pub products: SmallVec<[StoichiometricTerm; 4]>,

    pub allowed_phases: PhaseMask,
    pub temperature_window: Range<f32>,
    pub pressure_window: Range<f32>,

    pub activation: ActivationModel,
    pub rate: ReactionRateModel,

    pub heat_per_extent_j: f64,

    pub catalysts: SmallVec<[CatalystRule; 2]>,
    pub inhibitors: SmallVec<[InhibitorRule; 2]>,

    pub flags: ReactionFlags,
}
```

The runtime should operate on macroscopic reaction extent, not molecules.

---

# 17. Reaction step

For each active local mixture:

```text
1. Build compact mixture signature
2. Query compiled candidate-reaction index
3. Check phase/temperature/pressure/catalyst requirements
4. Compute limiting reagent
5. Compute bounded reaction extent for dt
6. Consume reactants
7. Produce products
8. Apply heat of reaction
9. Update pressure/volume implications
10. Emit causal record and semantic events
```

All concentrations/fractions must remain non-negative after projection.

---

# 18. Reaction indexing

Do not test every reaction against every pair of materials.

Compile at content-load time:

```text
SpeciesId
→ candidate reactions
→ required partner signatures
→ phase requirements
```

Runtime should usually evaluate only a small number of candidates per active parcel/region.

---

# 19. Initial process packages

The generic runtime should be proven through a small number of high-value process families.

## 19.1. Combustion

```text
fuel + oxidizer + activation
→ products + heat + expansion
```

Supports:

- wood;
- oil;
- grass;
- cloth;
- gas;
- coal-like fuel.

---

## 19.2. Evaporation and condensation

Primarily a phase process, but composition and humidity matter.

```text
liquid ↔ vapour
```

---

## 19.3. Dissolution

```text
solute + solvent
→ dissolved mixture
```

Uses:

- salt/brine;
- alchemy;
- pollution;
- nutrient transport;
- toxins.

---

## 19.4. Corrosion / reactive surface loss

```text
reactive fluid + solid surface
→ dissolved/solid products + surface damage
```

The reaction runtime computes material loss; fracture/structure systems interpret the weakening.

---

## 19.5. Neutralization and purification

Useful for:

- acids/bases-like systems;
- poison treatment;
- water purification;
- alchemy;
- magical contamination.

---

## 19.6. Precipitation and crystallization

```text
dissolved species
→ solid deposit / crystal
```

Applications:

- deposits;
- blocked pipes;
- cave growth;
- crafting;
- mana crystals.

---

## 19.7. Organic decomposition

```text
organic matter
→ gas + liquid + residual solids + biological/arcane products
```

Connects ecology, soil, fungi, chemistry, and mana cycles.

---

# 20. Combustion is a reaction package, not a world substance

Combustion requires:

- fuel;
- oxidizer;
- activation energy;
- reaction rate;
- heat release;
- products;
- transport.

For wood:

```text
heat
→ moisture evaporation
→ pyrolysis
→ volatile products
→ oxidation / flame
→ char
→ ash
→ cross-section and strength loss
```

Visible flame is a rendering/atmospheric phenomenon driven by reaction output.

---

# 21. Fire propagation

Fire spread may use several reduced channels:

- conductive contact;
- radiative exposure;
- convective hot gas;
- burning droplets;
- embers/firebrands;
- direct flame contact.

The propagation layer should transfer heat and reactive material, not set `burning = true` without cause.

A boolean may exist as a derived semantic state for gameplay and VFX.

---

# 22. Atmosphere and reacting gases

The first version should not solve full reacting-flow CFD.

Use tiers:

```text
Tier 0:
ambient oxidizer + visual smoke

Tier 1:
regional gas composition and temperature

Tier 2:
local gas parcels / reduced flow

Tier 3:
hero reacting-flow region
```

This keeps forest fire, rooms, smoke, and steam affordable.

---

# 23. Simplified chemistry is worth engine-level support

The goal is not scientific chemistry completeness.

The goal is:

```text
small set of general local laws
+
shared material state
+
physical transport
+
large combination space
=
emergent gameplay
```

This is promising because the same infrastructure supports:

- fire;
- freezing;
- steam;
- cooking;
- corrosion;
- poison;
- alchemy;
- decomposition;
- pollution;
- explosions;
- crystal growth;
- ecology;
- magical catalysis.

---

# 24. Limits of the chemistry model

The initial system should not attempt:

- molecular dynamics;
- quantum chemistry;
- hundreds of species per parcel;
- arbitrary autogenerated reaction pathways;
- full stiff kinetics over the whole world;
- universal real-world laboratory accuracy.

The content model must remain curated, sparse, and testable.

---

# 25. Arcane substrate recap

The `ArcaneDomain` models:

```rust
pub struct ArcaneState {
    pub quantity: f32,
    pub potential: f32,
    pub coherence: f32,
    pub entropy: f32,
    pub spectrum: ArcaneSpectrum,
}
```

Important distinction:

```text
arcane energy/resource
≠
physical thermal energy
≠
chemical mass
≠
biological organization
≠
identity/soul
```

Conversions require explicit couplers and losses.

---

# 26. Arcane coupling classes

The engine should distinguish:

## 26.1. Catalytic magic

Changes reaction barriers, nucleation, direction, or control with relatively low energy.

Examples:

- ignite existing fuel;
- accelerate crystallization;
- guide a flame;
- trigger fracture in an already stressed structure;
- accelerate healing with available biomass.

---

## 26.2. Transductive magic

Converts mana into physical energy or work.

Examples:

- force;
- heat;
- light;
- pressure;
- barrier contact impulse.

---

## 26.3. Transport magic

Moves existing quantities:

- heat;
- water;
- species;
- mana;
- charge;
- momentum.

---

## 26.4. Phase-control magic

Changes nucleation or stabilizes a phase.

Examples:

- freeze water more efficiently by encouraging ice formation while still removing latent heat;
- keep a temporary ice structure above its normal melting point at continuous mana cost;
- prevent boiling.

---

## 26.5. Ontological magic

Changes rules that ordinary conservation cannot explain:

- create matter;
- delete entropy;
- teleport;
- rewrite identity;
- reverse time.

These effects require dedicated specifications and should not silently use ordinary thermochemical couplers.

---

# 27. Arcane thermal coupler

```rust
pub struct ArcaneThermalRequest {
    pub arcane_source: ArcaneSourceRef,
    pub thermal_target: ThermalStateRef,

    pub desired_energy_j: f64,
    pub direction: ThermalDirection,

    pub conversion_mode: ArcaneThermalMode,
}
```

Modes may include:

```text
mana → heat
heat → mana
heat transfer to remote reservoir
heat transfer to extradimensional sink
phase stabilization
reaction catalysis
```

The transaction result must expose:

- actual transferred energy;
- mana consumed/generated;
- efficiency loss;
- arcane entropy increase;
- thermal side effects;
- overload risk.

---

# 28. Cold magic

A physically integrated cold spell may be implemented as:

```text
Target enthalpy
      ↓ removal
ArcaneThermalCoupler
      ↓
remote sink / crystal / mana conversion
      ↓
PhaseTransitionRuntime
      ↓
water → slush → ice
      ↓
Mechanical representation promotion
```

A caster may fail because:

- insufficient mana;
- insufficient heat-transfer throughput;
- heat sink saturated;
- latent heat requirement underestimated;
- target is continuously reheated;
- ice cannot bear the requested load.

This creates meaningful physical gameplay.

---

# 29. Fire magic

Possible mechanisms:

```text
add heat
add reactive fuel
supply oxidizer
lower activation energy
focus radiation
accelerate reaction rate
```

A catalytic fire spell may be cheap in a dry forest and expensive on wet stone.

A direct plasma/heat spell may ignore fuel but pay a much larger transductive cost.

---

# 30. Hydromancy and cryomancy

Hydromancy:

```text
ArcaneDomain
→ force/pressure request
→ ContinuumDomain
→ real water motion
```

Cryomancy:

```text
ArcaneDomain
→ enthalpy removal / phase stabilization
→ ThermochemicalLayer
→ ice phase
→ Continuum/structural ice representation
```

The second must not merely spawn an ice mesh.

---

# 31. Geomancy

Geomancy may use:

- physical stress on soil/rock;
- pore-pressure manipulation;
- crystallization/precipitation;
- phase stabilization;
- fracture catalysis;
- direct transductive force.

A raised wall should ideally use existing matter and leave a corresponding excavation, compaction, or mass source.

---

# 32. Alchemy

Alchemy becomes a programmable interface over:

- species separation;
- dissolution;
- catalysis;
- reaction routing;
- precipitation;
- phase changes;
- arcane spectrum filtering;
- artifact construction.

Alchemy can expose a simplified player grammar while compiling to `ReactionRuntime` and `SpellRuntime` operations.

---

# 33. Transmutation

Separate two meanings:

## Chemical transmutation

```text
existing species
→ reaction
→ new material composition
```

Handled by `ReactionRuntime`.

## Fundamental/ontological transmutation

```text
one elemental/nuclear identity
→ another
```

Requires explicit world rules, extreme energy or arcane authority, and a separate specification.

---

# 34. Vital and chemical coupling

Toxicity should not be a generic damage type.

```text
chemical species enters organism
→ transport/concentration
→ VitalDomain receptor/tissue response
→ symptoms, injury, metabolism, elimination
```

Similarly:

```text
heat
→ tissue thermal damage

cold
→ phase/osmotic/tissue injury

corrosive material
→ surface reaction + tissue damage
```

A reduced biological model is sufficient initially.

---

# 35. Healing magic

Healing may provide:

- energy;
- control/information;
- reaction catalysis;
- material transport;
- stabilization;
- infection/toxin removal.

It should still respect:

- biomass availability;
- tissue architecture;
- blood/transport;
- heat;
- time;
- identity continuity for advanced restoration.

---

# 36. Arcane ecology and chemistry

Arcane and chemical ecology may interact:

```text
plants absorb mana
→ synthesize arcane compounds
→ herbivores consume them
→ decomposition returns species and degraded mana
→ fungi purify or corrupt local field
```

This enables:

- mana-bearing sap;
- magical toxins;
- crystals grown by organisms;
- corrupted soil;
- arcane pollution;
- ley-dependent agriculture.

---

# 37. CouplingGraph update

```text
                              CouplingGraph

         Rigid ─────────────── Continuum ───────────── Atmosphere
           │                       │                        │
           │                       │                        │
           ├──────── Vegetation ───┼──────── Thermochemical┤
           │                       │                        │
           └────────────── Arcane ─┼────────── Vital ──────┘
                                   │
                               Identity
```

Each edge is a versioned, testable coupler contract.

---

# 38. Standard exchange quantities

The common exchange vocabulary should include:

```text
momentum / impulse
torque
mass
species mass
enthalpy / heat
phase fraction change
pressure work
reaction extent
moisture
arcane quantity
arcane spectrum/coherence
biological work request
topology change
identity/authority operation
```

---

# 39. Transaction model

Cross-domain updates should be represented as transactions where practical.

```rust
pub enum WorldTransaction {
    Momentum(MomentumTransaction),
    Heat(HeatTransaction),
    Species(SpeciesTransaction),
    Phase(PhaseTransaction),
    Arcane(ArcaneTransaction),
    Biological(BiologicalTransaction),
    Topology(TopologyTransaction),
}
```

A transaction contains:

- source;
- target;
- requested quantity;
- accepted quantity;
- losses;
- causal source;
- tick/version;
- validation result.

---

# 40. ConservationLedger

A new first-class service should track conserved and accounted quantities.

```text
mass
species mass
linear momentum
angular momentum
physical energy
arcane quantity / declared source model
reaction heat
phase latent energy
topology and identity authority
```

It does not need to audit every distant visual effect at full precision.

It should audit active, topology-changing, economy-relevant, and research-validation regions.

---

# 41. Energy accounting

Physical energy may be stored as:

- kinetic;
- gravitational;
- elastic;
- thermal/enthalpy;
- chemical potential approximation;
- pressure work;
- electromagnetic energy later.

Arcane energy is separate unless a coupler converts it.

A conversion record should satisfy a declared relation:

\[
E_{out} \le \eta E_{in} + E_{external}
\]

Any exception must be classified as an ontological law.

---

# 42. Mass accounting

Reactions transform species but should preserve total mass within configured approximation.

For reaction extent \(d\xi\):

\[
dm_i = \nu_i M_i d\xi
\]

The runtime may use fictional species and simplified coefficients, but the bookkeeping must remain consistent.

---

# 43. Momentum accounting

Reaction expansion, steam generation, explosion, magic force, and fluid-solid interaction must exchange momentum through owning physical domains.

Do not directly teleport ordinary physical objects for convenience unless the effect is explicitly ontological.

---

# 44. CausalTrace

Systemic worlds become difficult to debug without causal explanation.

Every significant effect should be traceable.

Example:

```text
Tree branch detached
because:
  bending stress > effective strength
because:
  effective strength reduced by char depth
because:
  combustion reaction progressed
because:
  dry wood temperature exceeded activation threshold
because:
  arcane thermal spell transferred 1.8 MJ
because:
  caster spell graph converted mana with 63% efficiency
```

Recommended service:

```text
WorldCausalTrace
├── transaction chain
├── solver decisions
├── reaction decisions
├── representation promotions
├── neural proposals/fallbacks
└── topology events
```

---

# 45. Semantic events

Derived events may include:

```text
MaterialIgnited
MaterialExtinguished
PhaseTransitionStarted
IceFractured
ReactionStarted
ReactionCompleted
CorrosionThresholdReached
ToxinExposure
ManaDepleted
ArcaneOverload
StructureFailed
ExplosionStarted
RegionContaminated
```

Events do not replace underlying state.

---

# 46. Simulation scheduling

Different systems operate at different rates.

Example:

```text
Rigid/articulated contacts:       60–240 Hz
Active continuum:                 30–120 Hz + substeps
Structural trees/ice:             30–120 Hz
Thermal local active regions:     10–60 Hz
Reaction runtime:                 5–60 Hz depending on stiffness
Atmosphere regional:              1–20 Hz
Regional ecology/chemistry:       0.1–5 Hz
Ley network:                      0.1–5 Hz
Spell control:                    30–120 Hz
Identity rules:                   event-driven
```

The scheduler must allow subcycling and coupling iterations.

---

# 47. Suggested active-tick order

A possible staged tick:

```text
1. Stream/promote representations
2. Update slow atmosphere, weather, and ley inputs
3. Gather external commands and spell requests
4. Predict physical-domain states
5. Build contact and neighborhood structures
6. Evaluate heat-transfer transactions
7. Evaluate phase transitions
8. Evaluate chemical reaction candidates
9. Evaluate arcane conversions/couplers
10. Apply composition, enthalpy, and phase updates
11. Select/update mechanical constitutive regimes
12. Solve continuum, rigid, articulated, and structural mechanics
13. Run required cross-domain coupling iterations
14. Process fracture/topology changes
15. Validate conservation and invariants
16. Commit persistent/replicated events
17. Publish semantic queries and causal traces
18. Update rendering/VFX representations
```

Some stiff processes may require a different local order or implicit coupled solve.

---

# 48. Explicit versus iterative coupling

Explicit coupling is acceptable when feedback is weak:

```text
rain → soil moisture
regional mana → slow plant growth
```

Iterative coupling is needed when feedback is strong:

```text
light boat ↔ water
ice sheet ↔ vehicle load
root plate ↔ soft saturated soil
explosion gas ↔ rigid debris
barrier field ↔ projectile contact
rapid phase change ↔ pressure
```

The coupler contract should declare its stability class.

---

# 49. Monolithic local solves

The engine should not use one monolithic solver globally.

However, local high-energy cases may benefit from a temporarily coupled solve:

```text
water + ice + rigid body
fire + gas pressure + structure
root + soil + water
arcane barrier + rigid contacts
```

These should be specialized local solve packages, not a universal world matrix.

---

# 50. RepresentationManager update

The `RepresentationManager` now manages physical, thermochemical, and arcane fidelity.

Responsibilities:

- relevance scoring;
- predictive activation;
- promotion/demotion;
- state transfer;
- GPU pool allocation;
- process activation;
- sleeping;
- streaming;
- reconstruction from persistent state;
- error-based fallback.

---

# 51. Water/ice representation ladder

```text
far water/ocean
→ spectral/reduced

regional flow
→ shallow/reduced continuum

active liquid
→ particle continuum

freezing surface
→ phase front + ice mask

loaded ice
→ structural plate/fracture graph

local breaking ice
→ bonded particles / refined solid

fragments
→ rigid bodies

melting
→ continuum water
```

---

# 52. Fire/combustion representation ladder

```text
inactive fuel
→ compact material state

heated region
→ coarse temperature/moisture

ignited object
→ per-segment/per-region reaction progress

hero combustion front
→ local detailed thermal/reaction samples

visible flame/smoke
→ atmosphere/VFX representation
```

---

# 53. Chemistry representation ladder

```text
far inactive region
→ aggregate composition + slow process progress

active gameplay region
→ compact mixture of 2–4 species

laboratory/hero region
→ additional species + gradients + detailed kinetics

sleeping changed region
→ compressed persistent chemistry state
```

---

# 54. Arcane representation ladder

```text
world
→ sparse ley graph

region
→ aggregate density/potential/spectrum

active location
→ local field samples

organism/artifact
→ reservoir and channel graph

spell
→ temporary execution graph + local constraint field
```

---

# 55. Persistent state

Persist only meaningful deviations from procedural/default world state.

Examples:

```text
phase state:
ice thickness, cracks, residual heat

chemistry:
composition deltas, contamination, corrosion, reaction progress

vegetation:
cuts, damage, burn/char, moisture, missing branches

soil:
compaction, saturation, frozen fraction, displaced mass

arcane:
regional depletion, corruption, artifacts, wards, ley damage
```

Do not persist every active sample indefinitely.

---

# 56. Sleeping-region compression

When a region sleeps:

```text
active particles/fields
→ conservative aggregation
→ persistent regional/material state
```

When it wakes:

```text
persistent state
→ reconstruction
→ local relaxation/projection
→ active representation
```

State transfer must preserve the important invariants for that representation.

---

# 57. Networking

The server should be authoritative for:

- reaction outcomes that create/destroy persistent material state;
- phase transitions that alter collision/navigation;
- ignition/extinguishing decisions;
- explosions;
- fracture/topology;
- major heat/mass transfers;
- mana consumption;
- persistent spells;
- identity operations.

Clients may simulate:

- flame shape;
- smoke details;
- minor liquid turbulence;
- ice crack visuals after authoritative topology;
- arcane VFX;
- secondary debris.

---

# 58. AI and gameplay queries

AI should receive semantic summaries, not solver internals.

```rust
pub struct WorldMaterialSample {
    pub support_strength: f32,
    pub sinkage_risk: f32,
    pub slip_risk: f32,

    pub temperature_k: f32,
    pub heat_flux: f32,

    pub solid_fraction: f32,
    pub liquid_fraction: f32,

    pub toxicity: f32,
    pub corrosivity: f32,
    pub flammability_state: f32,

    pub arcane: ArcaneEnvironmentSample,
}
```

Queries may answer:

- can this ice support the vehicle?;
- can this material ignite?;
- is there enough oxidizer?;
- is this water toxic?;
- can a cold spell freeze it in time?;
- where can mana be drawn without killing plants?;
- can a tree be felled by heating/cooling or force?;
- will a reaction create dangerous pressure?;

---

# 59. Spell planning over world physics

The spell planner can compare causal alternatives.

Goal:

```text
block a road
```

Options:

```text
freeze water into an ice barrier
raise and compact soil
fell a damaged tree
precipitate/crystallize material
create a maintained force field
```

The planner compares:

- mana quantity;
- throughput;
- time;
- environmental materials;
- heat capacity/latent heat;
- reaction requirements;
- structural stability;
- ecological consequences;
- persistence;
- risk.

---

# 60. Neural-assisted integration

The deterministic thermochemical and arcane laws must be implemented first.

Neural systems may later assist:

- warm start of thermal/pressure/constraint solvers;
- subgrid mixing;
- reaction-candidate ranking;
- approximate kinetics;
- learned constitutive models;
- representation activation;
- field reconstruction;
- spell planning;
- anomaly/OOD detection.

They must not silently violate:

- non-negative species;
- mass accounting;
- energy accounting;
- phase validity;
- stoichiometric bounds;
- topology authority;
- identity rules.

---

# 61. Good first neural roles

Low-risk candidates:

```text
reaction candidate classifier
thermal solver warm start
ice-fracture relevance predictor
chemistry LOD predictor
spell graph optimizer
subgrid smoke/steam VFX
```

Higher-risk candidates:

```text
learned reaction rate surrogate
learned phase closure
reduced-order local gas simulation
```

All require deterministic validation and fallback.

---

# 62. Rust architecture proposal

```text
crates/
├── world-dynamics-core
│   ├── ids
│   ├── units
│   ├── state_refs
│   ├── transactions
│   └── causal_ids
│
├── world-materials
│   ├── definitions
│   ├── species
│   ├── mixtures
│   ├── capabilities
│   └── registry
│
├── thermochemical-core
│   ├── enthalpy
│   ├── heat_transfer
│   ├── phase_diagrams
│   ├── reaction_definitions
│   ├── reaction_index
│   ├── reaction_runtime
│   └── process_state
│
├── thermochemical-processes
│   ├── combustion
│   ├── evaporation
│   ├── dissolution
│   ├── corrosion
│   ├── neutralization
│   ├── precipitation
│   └── decomposition
│
├── physics-rigid
├── physics-articulated
├── physics-continuum
├── physics-vegetation
├── physics-atmosphere
│
├── arcane-domain
├── vital-domain
├── identity-domain
│
├── world-coupling
│   ├── momentum
│   ├── thermal
│   ├── species
│   ├── phase
│   ├── arcane
│   ├── vital
│   └── topology
│
├── world-representation
├── world-scheduler
├── world-conservation
├── world-persistence
├── world-replication
├── world-query
├── world-causal-trace
├── world-validation
└── world-debug
```

Initially, prefer modules in a smaller number of crates until boundaries stabilize.

---

# 63. Core data sketches

```rust
pub struct MaterialParcelState {
    pub mass_kg: f32,
    pub mixture: CompactMixture,
    pub thermal_phase: ThermalPhaseState,
    pub material_state: MaterialRuntimeHandle,
    pub arcane_state: Option<ArcaneStateHandle>,
}
```

The actual GPU implementation should use SoA and specialized arrays.

---

# 64. Reaction transaction

```rust
pub struct ReactionTransaction {
    pub region: ActiveRegionId,
    pub reaction: ReactionId,
    pub extent: f32,

    pub consumed: SmallVec<[SpeciesMass; 4]>,
    pub produced: SmallVec<[SpeciesMass; 4]>,

    pub heat_j: f64,
    pub pressure_work_j: f64,

    pub cause: CausalId,
}
```

---

# 65. Phase transaction

```rust
pub struct PhaseTransaction {
    pub target: MaterialStateRef,
    pub old_phase: PhaseFractions,
    pub new_phase: PhaseFractions,
    pub latent_energy_j: f64,
    pub representation_hint: PhaseRepresentationHint,
    pub cause: CausalId,
}
```

---

# 66. Arcane conversion transaction

```rust
pub struct ArcaneConversionTransaction {
    pub source: ArcaneStateRef,
    pub target: WorldStateRef,

    pub mana_consumed: f32,
    pub physical_energy_j: f64,
    pub efficiency: f32,

    pub entropy_generated: f32,
    pub spectrum_shift: ArcaneSpectrum,

    pub cause: CausalId,
}
```

---

# 67. Debug visualization

Required debug views:

```text
temperature / enthalpy
solid-liquid-gas fractions
heat flux
species composition
active reactions
reaction extent/rate
oxidizer/fuel availability
corrosion depth
ice strength/cracks
pressure from phase/reaction expansion
mana density/potential/coherence/entropy
arcane conversion flow
conservation residuals
causal chain
representation state
neural proposal/fallback
```

---

# 68. Validation strategy

The system must be validated in layers.

## Thermal tests

- conduction slab;
- thermal equilibrium;
- radiative approximation;
- heat capacity;
- conservation.

## Phase tests

- water freezing curve;
- melting curve;
- boiling/condensation;
- latent heat plateau;
- freeze-thaw loop;
- slush transition.

## Ice mechanics tests

- plate bending;
- crack under load;
- temperature-dependent strength;
- fragment detachment;
- melt/refreeze.

## Reaction tests

- limiting reagent;
- stoichiometric mass balance;
- heat of reaction;
- catalyst/inhibitor;
- bounded rate;
- non-negative composition.

## Combustion tests

- wet versus dry wood;
- oxygen limitation;
- char progression;
- extinguishing by cooling;
- ember transfer.

## Arcane tests

- mana-to-heat conversion;
- heat extraction;
- phase stabilization;
- catalytic reaction;
- overload;
- conservation/declared source model.

---

# 69. Integrated validation scenarios

## Scenario A — Freeze a river and drive across

```text
spell removes enthalpy
→ ice front grows
→ structural ice representation promotes
→ vehicle loads sheet
→ cracks develop
→ partial failure
→ vehicle and fragments enter water
→ refreezing may occur
```

Metrics:

- energy balance;
- ice thickness;
- load capacity;
- fracture timing;
- state-transfer stability.

---

## Scenario B — Burning tree in rain

```text
rain adds water mass
→ wood moisture rises
→ heating evaporates moisture
→ ignition delayed
→ local combustion begins
→ char weakens branch
→ wind and gravity fracture it
→ hot branch falls into wet soil
```

---

## Scenario C — Salt and ice

```text
salt dissolves
→ mixture changes
→ phase diagram changes
→ local melting
→ fluid flow
→ gate or mechanism releases
```

---

## Scenario D — Acid on stone structure

```text
reactive fluid contacts stone
→ surface reaction
→ solid mass loss
→ fracture capability updates
→ structural failure
```

---

## Scenario E — Magical cooling overload

```text
caster attempts rapid freeze
→ latent heat demand exceeds throughput
→ crystal heat sink saturates
→ channel overload
→ partial slush forms instead of full ice
```

---

## Scenario F — Arcane pollution ecology

```text
factory draws ley mana
→ emits high-entropy residue
→ soil/plants accumulate contamination
→ plant chemistry changes
→ creatures become toxic/corrupted
→ regional mana quality declines
```

---

# 70. Performance philosophy

The biggest cost is not reaction arithmetic. It is active degrees of freedom and transport.

Optimization priorities:

```text
1. avoid activating detailed regions
2. reduce active particles/elements
3. reduce coupling iterations
4. reduce species count
5. compile reaction lookup
6. lower update frequency
7. use compact persistent summaries
8. use neural assistance only after strong classical baselines
```

---

# 71. Chemistry content budget

The first game should define a small curated species set.

Example initial macro-species:

```text
water
vapour
ice phase
oxidizer
combustible gas
wood fuel
char
ash
oil fuel
salt
acidic reagent
alkaline reagent
mineral solid
dissolved mineral
organic matter
toxin
smoke/soot
```

This can already produce many systemic interactions.

---

# 72. Recommended development roadmap

## Phase A — Units, ownership, and transaction foundation

Implement:

- SI-like units policy;
- world IDs/state references;
- transaction types;
- causal IDs;
- conservation ledger skeleton;
- material capability registry.

Exit criteria:

- cross-domain updates cannot bypass ownership contracts.

---

## Phase B — Deterministic Thermal Core

Implement:

- enthalpy;
- temperature conversion;
- heat capacity;
- conduction;
- basic convection/radiation approximation;
- thermal validation.

Materials:

- water;
- wood;
- stone;
- metal;
- air.

---

## Phase C — Water / Ice / Steam

Implement:

- latent heat;
- phase fractions;
- freezing/melting;
- boiling/condensation;
- representation hooks.

Start with compact/non-mechanical ice.

---

## Phase D — Mechanical Ice

Implement:

- bonded/plate representation;
- load-bearing ice;
- brittle fracture;
- rigid fragments;
- meltwater return.

---

## Phase E — Combustion Package

Implement:

- fuel;
- oxidizer;
- ignition;
- reaction heat;
- moisture;
- pyrolysis/char summary;
- fire spread channels;
- structural weakening outputs.

---

## Phase F — Generic ReactionRuntime

Generalize:

- species;
- mixtures;
- reaction definitions;
- candidate index;
- stoichiometric projection;
- catalysts/inhibitors;
- causal traces.

---

## Phase G — High-value chemistry set

Add:

- salt/brine;
- dissolution;
- corrosion;
- neutralization;
- precipitation;
- decomposition;
- selected toxins.

---

## Phase H — Arcane physical couplers

Implement:

```text
mana → force
mana → heat
heat → mana/sink
mana → continuum pressure
mana → reaction catalysis
mana → phase stabilization
```

Keep SpellRuntime deterministic and constrained.

---

## Phase I — Arcane ecology and chemistry

Add:

- mana-bearing plants;
- arcane compounds;
- contamination;
- regional depletion/pollution;
- purification/decomposition loops.

---

## Phase J — Persistence, streaming, and replication

Prove:

- sleeping chemistry state;
- ice crack persistence;
- regional thermal summaries;
- persistent spell/reaction state;
- authoritative topology events.

---

## Phase K — Neural acceleration

Only after classical validation:

- thermal warm start;
- reaction candidate ranking;
- chemistry LOD;
- subgrid mixing;
- spell planning;
- fallback and telemetry.

---

## Phase L — Advanced metaphysics and high-fidelity local solves

Research:

- true matter creation;
- teleportation;
- entropy deletion;
- detailed reacting gases;
- local MPM multiphase ice;
- electrochemistry;
- identity/soul operations.

---

# 73. Recommended first vertical slice

## `Thermochemical Arcane Sandbox`

Scene:

- small water tank/stream;
- wood object/tree segment;
- metal/stone objects;
- simple atmosphere;
- one mage reservoir.

Capabilities:

- heat and cool materials;
- freeze/melt water;
- create load-bearing ice slab;
- ignite dry wood;
- wet wood resists ignition;
- extinguish through real cooling/water;
- generate steam;
- use salt to alter local freezing;
- trace energy and mana costs.

No:

- large forest;
- full chemistry catalog;
- open-world streaming;
- souls;
- full gas CFD;
- neural solver.

This slice proves the corrected ontology.

---

# 74. Candidate ADRs

1. `ADR-WorldDynamics-Ontology.md`
   - matter, state, field, process, representation.

2. `ADR-Thermochemical-Layer.md`
   - replacement of standalone FireDomain.

3. `ADR-Thermal-State-Enthalpy.md`
   - enthalpy as source variable.

4. `ADR-Phase-Transition-Model.md`
   - fractions, latent heat, hysteresis.

5. `ADR-Ice-Mechanical-Representation.md`
   - masks, plates, bonded particles, fragments.

6. `ADR-Species-And-Mixtures.md`
   - separation of material and species.

7. `ADR-Reaction-Runtime.md`
   - generic data-driven reactions.

8. `ADR-Combustion-As-Reaction-Package.md`

9. `ADR-World-Conservation-Ledger.md`

10. `ADR-Cross-Domain-Transactions.md`

11. `ADR-Arcane-Thermochemical-Coupling.md`

12. `ADR-Cold-Magic-Energy-Sink.md`

13. `ADR-Chemistry-LOD.md`

14. `ADR-Thermochemical-Persistence.md`

15. `ADR-Thermochemical-Replication.md`

16. `ADR-Causal-Trace.md`

17. `ADR-Ontological-Magic-Boundary.md`

18. `ADR-Neural-Thermochemical-Trust.md`

---

# 75. Candidate specifications

- `SPEC-World-Units.md`
- `SPEC-World-State-Ownership.md`
- `SPEC-World-Transaction-Protocol.md`
- `SPEC-Conservation-Ledger.md`
- `SPEC-Material-Capability-Registry.md`
- `SPEC-Species-Registry.md`
- `SPEC-Compact-Mixture.md`
- `SPEC-Thermal-Phase-State.md`
- `SPEC-Heat-Transfer.md`
- `SPEC-Phase-Diagram.md`
- `SPEC-Water-Ice-Steam.md`
- `SPEC-Ice-Structural-Plate.md`
- `SPEC-Ice-Fracture.md`
- `SPEC-Reaction-Definition.md`
- `SPEC-Reaction-Candidate-Index.md`
- `SPEC-Reaction-Runtime.md`
- `SPEC-Reaction-Stoichiometric-Projection.md`
- `SPEC-Combustion-Process.md`
- `SPEC-Wood-Pyrolysis-Char.md`
- `SPEC-Fire-Spread-Channels.md`
- `SPEC-Dissolution.md`
- `SPEC-Corrosion.md`
- `SPEC-Neutralization.md`
- `SPEC-Precipitation.md`
- `SPEC-Decomposition.md`
- `SPEC-Arcane-Thermal-Coupler.md`
- `SPEC-Arcane-Cold-Spell.md`
- `SPEC-Arcane-Reaction-Catalysis.md`
- `SPEC-Arcane-Phase-Stabilization.md`
- `SPEC-Thermochemical-Causal-Trace.md`
- `SPEC-Thermochemical-Debug-Views.md`
- `SPEC-Thermochemical-Validation.md`
- `SPEC-Thermochemical-Persistence.md`
- `SPEC-Thermochemical-Replication.md`

---

# 76. Candidate research spikes

- `SPIKE-Enthalpy-Phase-Solver.md`
- `SPIKE-Bonded-Particle-Ice.md`
- `SPIKE-Ice-Plate-Fracture.md`
- `SPIKE-Freezing-Expansion.md`
- `SPIKE-Phase-Dependent-Constitutive-Switching.md`
- `SPIKE-Compact-Mixture-GPU.md`
- `SPIKE-Reaction-Runtime-GPU.md`
- `SPIKE-Local-Reacting-Gas.md`
- `SPIKE-Thermal-Structural-Coupling.md`
- `SPIKE-Arcane-Heat-Removal.md`
- `SPIKE-Arcane-Phase-Stabilization.md`
- `SPIKE-Alchemy-Spell-IR.md`
- `SPIKE-Chemistry-Persistence-Compression.md`
- `SPIKE-Neural-Reaction-Ranking.md`
- `SPIKE-Neural-Subgrid-Mixing.md`
- `SPIKE-World-Causal-Trace-Storage.md`

---

# 77. Normative requirements

## MUST

- The engine MUST treat fire as a process, not a fundamental material entity.
- The engine MUST treat ordinary cold as enthalpy state/transfer, not a separate conserved substance.
- Ice MUST be represented as a phase of water with mechanical consequences when relevant.
- Phase transitions MUST account for latent energy.
- Chemical reactions MUST use explicit reactants, products, bounded extent, and mass accounting.
- Arcane-to-physical conversion MUST use explicit couplers and declared source/loss rules.
- State ownership MUST remain unique.
- Cross-domain topology changes MUST be authoritative and persistent.
- Detailed simulation MUST be local/adaptive.
- Neural components MUST be validated and have deterministic fallback.

## SHOULD

- Thermal state SHOULD use enthalpy as the authoritative variable where phase change matters.
- Materials SHOULD separate immutable definitions from runtime state.
- Chemistry SHOULD use sparse compact mixtures.
- Reaction lookup SHOULD be precompiled/indexed.
- Significant effects SHOULD produce causal traces.
- Fire VFX SHOULD derive from thermal/reaction state.
- Cold magic SHOULD expose where removed heat goes.
- Chemistry and arcane ecology SHOULD share material/species infrastructure where appropriate.
- AI SHOULD use semantic world queries rather than solver internals.

## MAY

- Distant regions MAY use aggregate reaction progress.
- Ice MAY demote to a static collision representation when safe.
- The setting MAY allow an extradimensional heat sink.
- The setting MAY allow non-conservative ontological magic, but it MUST be explicit.
- Local hero regions MAY use high-fidelity coupled solvers.
- Neural surrogates MAY handle quiet regions after validation.

---

# 78. Anti-patterns

Do not build the world around:

```text
FireDamage
IceDamage
AcidDamage
PoisonDamage
MagicDamage
```

These may exist as UI/combat summaries, but not as the foundational cause model.

Avoid:

- direct temperature assignment without energy accounting;
- instant water-to-ice mesh replacement;
- arbitrary reaction pair tables evaluated quadratically;
- full species arrays on every particle;
- one global thermochemical solver at maximum resolution;
- direct spell mutation of physical transforms;
- neural chemistry without stoichiometric projection;
- separate contradictory burn state in vegetation and fire systems;
- persisting every active sample forever.

---

# 79. Updated architectural diagram

```text
                               WORLD DYNAMICS
                                     │
        ┌────────────────────────────┼────────────────────────────┐
        │                            │                            │
  PHYSICAL DOMAINS          THERMOCHEMICAL LAYER           ARCANE DOMAIN
        │                            │                            │
 rigid / articulated         enthalpy / heat transfer      mana / potential
 continuum / vegetation      phases / reactions            coherence / entropy
 atmosphere                  combustion / corrosion        spectrum / ley graph
        │                            │                            │
        └────────────────────────────┼────────────────────────────┘
                                     │
                         VITAL / IDENTITY DOMAINS
                                     │
                               COUPLING GRAPH
                                     │
       ┌─────────────────────────────┼─────────────────────────────┐
       │                             │                             │
 REPRESENTATION MANAGER      SIMULATION SCHEDULER         CONSERVATION LEDGER
       │                             │                             │
 streaming / promotion       multi-rate / substeps       mass / momentum / heat
 persistent summaries        iterative coupling          species / mana / topology
       │                             │                             │
       └─────────────────────────────┼─────────────────────────────┘
                                     │
                       QUERIES / EVENTS / CAUSAL TRACE
                                     │
                            GAMEPLAY / AI / RENDERING
```

---

# 80. Final architectural statement

> **`WorldDynamics` is an orchestration layer for specialized physical, thermochemical, biological, and arcane domains. Matter-owning domains control motion and structure; `ThermochemicalLayer` controls heat transfer, phase changes, and reaction transactions; `ArcaneDomain` supplies a separately accounted substrate that can couple to matter through explicit conversions; `VitalDomain` interprets consequences for life; and `IdentityDomain` handles metaphysical continuity where required.**

---

# 81. Final project thesis

> **Create a game world in which water can freeze into load-bearing and breakable ice, fire emerges from fuel and heat, chemicals dissolve and react, trees burn and weaken structurally, soil freezes or corrodes, living organisms metabolize toxins and mana, and spells manipulate these same shared quantities through constrained physical and metaphysical couplers—while adaptive representations keep the system computationally practical.**

---

# 82. Recommended next documents

The specification agent should derive, in this order:

```text
1. ADR-WorldDynamics-Ontology
2. SPEC-World-State-Ownership
3. SPEC-World-Transaction-Protocol
4. SPEC-Conservation-Ledger
5. SPEC-Material-Capability-Registry
6. SPEC-Thermal-Phase-State
7. SPEC-Water-Ice-Steam
8. SPEC-Reaction-Runtime
9. SPEC-Combustion-Process
10. SPEC-Arcane-Thermal-Coupler
11. SPEC-Thermochemical-Validation
12. SPEC-Thermochemical-Causal-Trace
```

The first implementation milestone should remain narrowly scoped:

```text
water + wood + stone + metal
+
enthalpy and phase transitions
+
load-bearing ice
+
wood combustion
+
mana-to-heat / heat-removal couplers
+
conservation and causal debugging
```

This vertical slice is large enough to validate the unified architecture but small enough to implement and measure.
