# Continuum Physics / Realistic Water & Deformable Terrain
## Research and Design Brief for the Rust Game Engine

**Status:** working research brief, not a frozen specification
**Context date:** August 2026
**Primary goal:** provide an agent with enough context to derive architecture decisions, ADRs, milestones, experiments, and implementation specifications.

---

## 0. Executive summary

The original question was how to implement realistic water physics in a custom Rust game engine without reducing water to a heightfield or a fixed grid of "water cells".

The discussion evolved into a broader conclusion:

> The engine should probably not have a narrowly scoped `WaterSystem` as the fundamental abstraction.
> A more powerful long-term direction is a general **Continuum Physics / Material World** system capable of representing water, mud, wet soil, sand, snow, slurry, sediment, and related deformable materials.

The key architectural idea is:

- represent material primarily with **Lagrangian material samples / particles** rather than a permanent world-space voxel grid;
- use a temporary spatial acceleration structure (hash grid / Morton buckets / BVH-like structure) only for neighborhood queries;
- solve material behavior using physically motivated constitutive laws and constraints;
- support multiple solver backends:
  - a mature baseline such as DFSPH for incompressible liquid;
  - a research backend based on unified/nonlocal variational particle methods;
  - later experimental continuous velocity representations such as divergence-free kernel fields;
- support adaptive refinement via particle split/merge;
- couple fluid/material particles bidirectionally to rigid and articulated bodies;
- generalize the same infrastructure to soil, mud, snow, sand, and potentially other continuum materials.

This opens a path not only to realistic water but also to a highly realistic off-road simulation where:

- tires physically sink into soil;
- ruts persist and affect following vehicles;
- mud is displaced rather than painted as a decal;
- wheel slip is a consequence of soil shear failure, not a hardcoded friction multiplier;
- water saturation changes soil strength;
- tire pressure changes the contact patch and sinkage;
- different tread geometries interact differently with deformable terrain;
- rain can transform dry terrain into progressively saturated, weak, waterlogged ground;
- vehicles can dig themselves in, become high-centered, or reshape the route for other vehicles.

This document captures the architecture, research directions, implementation strategy, risks, and open questions discussed so far.

---

# 1. Design philosophy

## 1.1. What we are explicitly avoiding

We do **not** want the foundation of the simulation to be:

```rust
struct WaterCell {
    height: f32,
    velocity_x: f32,
    velocity_z: f32,
}
```

That representation is useful for shallow-water approximations and large-scale gameplay water, but it fundamentally restricts the topology of the liquid.

A heightfield cannot naturally represent:

- overturning waves;
- droplets;
- detached jets;
- water films;
- waterfalls;
- air pockets;
- overhangs;
- multiple water layers at the same horizontal coordinate;
- liquid inside arbitrary 3D containers.

The objection is not that all grids are "fake". Finite-volume and Eulerian grids are mathematically legitimate discretizations of continuum mechanics.

The actual design preference is:

> Avoid making the *physical degrees of freedom* of the material permanently tied to a fixed world-space grid.

A spatial hash may still be used for neighbor lookup. That is an acceleration structure, not the physical representation.

---

## 1.2. The unavoidable fact: discretization cannot disappear

A computer cannot simulate a truly continuous field with infinitely many degrees of freedom.

Even a "gridless" method still discretizes the continuum using one of:

- Lagrangian particles;
- scattered collocation points;
- dynamically generated control volumes;
- basis functions;
- kernels;
- Gaussian primitives;
- neural fields;
- Monte-Carlo samples.

Therefore the real problem is not:

> How do we remove discretization?

It is:

> Which discretization best preserves the physical structure we care about while remaining practical for a real-time engine?

---

# 2. Physical model of water

At game-relevant scales, water is usually modeled as an incompressible continuum.

The conceptual base is the incompressible Navier-Stokes system:

\[
\rho \left(
\frac{\partial \mathbf{u}}{\partial t}
+
\mathbf{u}\cdot\nabla\mathbf{u}
\right)
=
-\nabla p
+
\mu \nabla^2 \mathbf{u}
+
\rho \mathbf{g}
+
\mathbf{f}_{surface}
+
\mathbf{f}_{solid}
\]

with the incompressibility constraint:

\[
\nabla \cdot \mathbf{u} = 0
\]

Where:

- `u` = velocity field;
- `p` = pressure;
- `rho` = density;
- `mu` = dynamic viscosity;
- `g` = gravity;
- `f_surface` = surface-tension-related forces;
- `f_solid` = interaction with solid boundaries and objects.

For realistic free-surface water, this is not enough by itself.

The complete simulation problem also includes:

- free-surface tracking;
- surface tension;
- curvature;
- contact angle / wetting;
- moving solid boundaries;
- two-way rigid-body coupling;
- droplets;
- thin sheets;
- spray;
- bubbles;
- entrained air;
- topology change;
- very large scale differences.

---

# 3. Solver landscape

## 3.1. Shallow Water Equations

Useful for:

- rivers;
- flooding;
- large shallow water bodies;
- terrain-scale flow;
- low-cost wave propagation.

Advantages:

- very cheap relative to full 3D fluid;
- excellent for large world areas;
- easy to run on GPU;
- naturally works with terrain heightfields.

Limitations:

- effectively 2.5D;
- no overturned surface;
- no droplets;
- no real waterfall volume;
- no arbitrary free-surface topology.

Conclusion:

> Good optimization / LOD / world-scale layer, but not the desired physical foundation.

---

## 3.2. Spectral / FFT ocean

Useful for:

- ocean-scale waves;
- lakes;
- wind-driven wave spectra.

Advantages:

- extremely cheap compared with volumetric CFD;
- good visual quality at huge scale.

Limitations:

- not a material volume;
- cannot naturally flood terrain;
- cannot represent arbitrary local topology;
- local rigid interactions require an additional system.

Conclusion:

> Useful as a far-field ocean representation, not as the fundamental material solver.

---

## 3.3. PIC / FLIP / APIC

Hybrid particle-grid methods.

Conceptual flow:

```text
particles
   ↓
particle → temporary grid transfer
   ↓
forces / pressure projection
   ↓
grid → particle transfer
   ↓
particle advection
```

### PIC
Stable but dissipative.

### FLIP
Better kinetic-energy preservation but noisier.

### APIC
Adds local affine velocity information per particle and improves transfer quality, rotation preservation, and numerical behavior.

Modern descendants include very large-scale and two-phase FLIP systems.

Strengths:

- mature;
- excellent free surfaces;
- robust global pressure solve;
- high visual fidelity;
- very strong VFX baseline.

Weaknesses for this project:

- still depends on a temporary volumetric grid for core physics;
- not aligned with the "no permanent grid as material representation" research goal.

Conclusion:

> Essential reference and quality baseline; not the preferred final foundation if we remain committed to a particle-continuum architecture.

---

# 4. Particle continuum methods

## 4.1. SPH

Smoothed Particle Hydrodynamics represents the fluid with material particles.

A particle is **not a molecule**.

It represents a finite amount of continuum material.

Typical state:

```rust
struct FluidParticle {
    position: Vec3,
    velocity: Vec3,
    mass: f32,
    density: f32,
    rest_volume: f32,
}
```

A field is reconstructed using nearby samples and a smoothing kernel.

Benefits:

- no permanent volumetric physics grid;
- free surfaces appear naturally;
- topology changes are natural;
- well suited to GPU neighborhood computation;
- good match for arbitrary containers and moving materials.

Problems of naive SPH:

- compressibility artifacts;
- density drift;
- particle disorder;
- tensile instability;
- difficult boundary treatment;
- noisy curvature;
- expensive neighbor iterations;
- poor thin-sheet behavior in simple formulations.

Therefore naive WCSPH is not the intended target.

---

## 4.2. DFSPH as the baseline solver

**Divergence-Free SPH** is the recommended first serious implementation.

It separately controls:

- velocity divergence;
- density error.

Conceptually:

```text
predict velocity
    ↓
divergence solve
    ↓
density solve
    ↓
integrate
```

Reasons to choose it as baseline:

- mature enough to validate against existing implementations;
- physically meaningful;
- fully particle-based in terms of material state;
- GPU friendly;
- capable of handling realistic free-surface water;
- suitable as an oracle for newer research backends.

Recommended role:

```text
production / baseline solver = DFSPH
research solver             = variational particle method
experimental layer          = continuous divergence-free field
```

---

# 5. Position-based and variational liquid solvers

## 5.1. PBF / Position Based Fluids

PBF applies position corrections so that particle density approximately satisfies the incompressibility constraint.

Advantages:

- robust;
- large timesteps;
- easy integration with position-based frameworks.

Weaknesses:

- can be less physically faithful than more advanced pressure formulations;
- behavior depends on iterative corrections and tuning.

Still valuable as a conceptual stepping stone.

---

## 5.2. IPBF

Implicit Position-Based Fluids reformulates incompressibility using a variational / energy perspective.

Conceptually:

\[
x^{n+1}
=
\arg\min_x
[
E_{inertia}
+
E_{incompressibility}
+
E_{boundary}
]
\]

Why it matters:

- moves away from a sequence of loosely coupled corrections;
- makes the fluid step an optimization problem;
- potentially improves stability and consistency.

---

## 5.3. Unified / nonlocal variational free-surface methods

A particularly interesting 2026 research direction is to solve several physical effects in one coupled variational formulation:

```text
incompressibility
+
viscosity
+
surface tension
+
boundary terms
        ↓
single coupled objective
```

Instead of:

```text
pressure
→ viscosity
→ surface correction
→ collision correction
→ repair density
```

we want something closer to:

\[
E(x)
=
E_{inertia}
+
E_{volume}
+
E_{viscosity}
+
E_{surface}
+
E_{boundary}
+
E_{contact}
\]

and solve for the next particle configuration.

This is one of the strongest candidates for the long-term research core.

---

# 6. Continuous / grid-free velocity-field research

## 6.1. Divergence-Free Kernel Fields (DDFK)

The idea:

Represent velocity as a linear combination of analytically divergence-free basis functions:

\[
u(x)
=
\sum_i K_{df}(x, x_i) a_i
\]

where:

\[
\nabla \cdot K_{df} = 0
\]

therefore:

\[
\nabla \cdot u = 0
\]

everywhere in the represented field, not only at discrete grid samples.

Potential benefits:

- continuous velocity query at arbitrary coordinates;
- low numerical dissipation;
- no pressure projection grid;
- global coherent flow representation.

Current limitations:

- not a complete free-surface water solution by itself;
- material volume still needs to be tracked;
- boundaries remain difficult;
- published methods are still research-grade and can be slower than conventional solvers.

### Proposed experimental use

Do **not** immediately replace DFSPH with DDFK.

Instead:

```text
material particles
     ↓
local physical solve
     ↓
fit / reconstruct low-frequency divergence-free velocity field
     ↓
use field for global transport / vorticity preservation / correction
```

Possible decomposition:

\[
u = u_{local-particle} + u_{global-DDFK}
\]

This is a research hypothesis, not a ready-made production method.

---

## 6.2. Neural Monte Carlo fluid solvers

Another research direction is:

- continuous neural velocity field;
- Monte-Carlo solution of pressure / Poisson equations;
- no regular volumetric grid.

Potential advantages:

- continuous representation;
- geometrically flexible boundaries;
- sparse / query-driven solution.

Current problems:

- Monte-Carlo variance;
- cost for stable pressure gradients;
- limited demonstrated free-surface water capability;
- difficult material conservation;
- not ready as the foundation of a real-time engine.

Recommended role:

- research experiment;
- possible global pressure correction;
- possible offline teacher/reference;
- possible low-frequency field solver.

Not recommended as first implementation.

---

## 6.3. Learned mesh-free differential operators

Research such as learned mesh-free differential operators uses a GNN to estimate local gradient/divergence/Laplacian operator weights from irregular point neighborhoods.

Potential architecture:

```text
particle neighborhood
      ↓
GNN
      ↓
operator weights
      ↓
gradient / divergence / Laplacian
```

Important design rule:

> ML should improve the discretization or initialization, but should not own physical invariants.

Good ML roles:

- learned differential operators;
- learned preconditioner;
- initial pressure guess;
- local error estimation;
- split/merge prediction;
- secondary spray;
- surface reconstruction;
- turbulence closure.

Physics should still project the result onto a valid conservative state.

---

# 7. Adaptivity

Uniform particles are not enough for world-scale real-time simulation.

Particle count scales roughly as:

\[
N \propto \frac{V}{d^3}
\]

Reducing particle spacing by 2 multiplies particle count by roughly 8.

Therefore we need adaptive resolution.

## 7.1. Desired behavior

```text
quiet interior water:

•          •          •

high curvature / collision / turbulence:

• • • • • • • • • • •

quiet again:

•          •          •
```

Refinement should be driven by **physical error**, not only camera distance.

Potential error estimator:

\[
e_i =
w_\kappa |\kappa_i|
+
w_\omega |\nabla \times u_i|
+
w_\rho |\rho_i - \rho_0|
+
w_s ||\nabla u_i||
+
w_b / d_{boundary}
\]

Possible refinement triggers:

- high curvature;
- high vorticity;
- high strain rate;
- density error;
- proximity to moving rigid bodies;
- impending topology change;
- free-surface region;
- thin sheet;
- splash region.

---

## 7.2. Split / merge conservation

Particle splitting and merging must preserve at least:

- total mass;
- center of mass;
- linear momentum;
- rest volume.

Preferably also:

- angular momentum;
- local affine velocity field;
- kinetic energy within controlled bounds;
- phase/material state;
- plastic history for solids/soil.

Example split:

```text
      ●
      ↓
   • • • •
```

Example merge:

```text
   • • • •
      ↓
      ●
```

Adaptivity is one of the highest-risk components.

---

# 8. Boundaries and solid coupling

The difficult part of a real game implementation is not merely solving water in a box.

The system must work with:

- arbitrary triangle meshes;
- thin objects;
- moving geometry;
- rotating geometry;
- doors;
- pumps;
- articulated bodies;
- wheels;
- tires;
- characters;
- deformable terrain interactions.

Simple post-penetration pushout is insufficient.

The physical condition should enforce approximately:

\[
(u_{fluid} - u_{solid}) \cdot n = 0
\]

for impermeable contact.

Momentum exchange should be symmetric:

\[
\Delta p_{solid}
=
-\Delta p_{fluid}
\]

Torque transfer:

\[
\Delta L
=
(x - x_{COM}) \times \Delta p
\]

This must feed back into the engine's rigid-body / articulated-body solver.

---

# 9. Two-way rigid coupling

A realistic object in water should:

- displace material volume;
- create waves;
- experience pressure;
- experience drag;
- receive torque;
- alter the surrounding flow.

A fake buoyancy sample system is insufficient for the final goal.

Important problem: **added-mass instability**.

Light objects coupled strongly to a fluid can oscillate numerically:

```text
fluid corrects rigid body
        ↓
rigid body moves a lot
        ↓
fluid state becomes inconsistent
        ↓
larger correction
```

Possible solutions:

- fluid-rigid subiterations;
- implicit coupling;
- monolithic constraints;
- conservative impulse exchange;
- special handling of extreme mass ratios.

---

# 10. Free-surface representation

The physics representation and render representation should be decoupled.

## 10.1. Physics needs

- surface vs interior classification;
- local normal;
- curvature;
- surface area contribution;
- topology;
- thin-sheet awareness.

## 10.2. Rendering needs

Potential reconstruction methods:

- isotropic density field;
- anisotropic kernels;
- moving least squares;
- local signed-distance reconstruction;
- alpha shapes;
- explicit tracked surface mesh;
- ray-marched implicit field;
- neural surface reconstruction.

A temporary rendering voxel grid is acceptable if it does not become the physical representation.

Recommended architecture:

```text
physics particles
      ↓
surface reconstruction
      ↓
render-only mesh / field
      ↓
refraction / reflection / absorption
      ↓
foam / spray / bubbles
```

---

# 11. Secondary effects

The primary continuum should not need to explicitly resolve every tiny droplet.

Use a secondary particle system for:

- spray;
- mist;
- foam;
- small bubbles;
- tiny detached droplets.

Spawn criteria can be physically motivated:

- high kinetic energy;
- high curvature;
- high vorticity;
- breaking surface;
- high divergence before projection;
- high pressure impulse;
- strong air entrainment.

These particles may be visual-only or weakly coupled back to the main fluid.

---

# 12. Air-water coupling

Most water simulation can treat air as atmospheric boundary pressure.

Full two-phase air-water simulation should be activated only where it matters:

- closed air pockets;
- breaking waves;
- violent splashes;
- trapped gas;
- fast impacts;
- cavitation-like phenomena;
- large bubbles.

Suggested adaptive strategy:

```text
normal free surface
→ atmospheric boundary

closed air pocket detected
→ local compressed-air state

violent multiphase region
→ local two-phase solver
```

Do not simulate expensive two-phase air everywhere by default.

---

# 13. From WaterSystem to ContinuumMaterialSystem

The off-road/mud discussion changes the preferred architecture.

Instead of:

```text
WaterSystem
```

consider:

```text
ContinuumMaterialSystem
```

or:

```text
MaterialWorld / ContinuumPhysics
```

with water being only one constitutive model.

Suggested hierarchy:

```text
Continuum Physics
│
├── Particle Infrastructure
│
├── Neighbor Search
│
├── Spatial Acceleration
│
├── Adaptive Resolution
│
├── Constraints / Variational Solver
│
├── Constitutive Models
│   ├── Incompressible Water
│   ├── Viscoplastic Mud
│   ├── Drucker-Prager Soil
│   ├── μ(I) Granular Material
│   ├── Cam-Clay / Modified Cam-Clay
│   ├── Snow
│   └── Custom materials
│
├── Multiphase Coupling
│   ├── water ↔ soil
│   ├── water ↔ sand
│   ├── water ↔ air
│   └── sediment transport
│
├── Boundary / Rigid Coupling
│
├── Articulated Body Coupling
│
└── Rendering / Surface Extraction
```

---

# 14. Why mud is not "water with higher viscosity"

Real mud may behave as:

- viscoplastic fluid;
- saturated granular material;
- cohesive soil;
- porous solid-fluid mixture.

Important behavior:

- supports stress below a yield threshold;
- flows after yielding;
- changes strength with saturation;
- develops pore pressure;
- can compact;
- can shear and remold;
- can transition toward liquid-like behavior.

Possible constitutive models:

- Herschel-Bulkley for viscoplastic flow;
- Bingham-like models;
- Drucker-Prager for frictional granular soil;
- μ(I) rheology for dense granular flow;
- Modified Cam-Clay for pressure-dependent plastic soils;
- specialized snow plasticity;
- poromechanics / two-phase soil-water models.

---

# 15. Soil-water multiphase model

For the most realistic mud, represent soil and water as coupled phases rather than a single "mud material".

Conceptual structure:

```text
water phase
   ↓ pore pressure / seepage
solid granular phase
```

Relevant state variables:

```rust
struct ContinuumParticle {
    position: Vec3,
    velocity: Vec3,
    mass: f32,
    rest_volume: f32,

    material_id: MaterialId,

    stress: Mat3,
    deformation_gradient: Mat3,
    plastic_state: PlasticState,

    porosity: f32,
    saturation: f32,
    water_content: f32,
    pore_pressure: f32,
}
```

Not every material needs every field; production implementation should use compact material-specific SoA storage.

Physical chain:

```text
rain
 ↓
infiltration
 ↓
water saturation increases
 ↓
pore pressure changes
 ↓
effective soil strength changes
 ↓
tire sinkage / traction changes
```

This is preferable to:

```cpp
if (is_wet) {
    friction *= 0.5;
}
```

---

# 16. Off-road simulation opportunities

A continuum terrain system creates gameplay directly from physics.

## 16.1. Sinkage

Vehicle weight physically deforms the terrain.

```text
before:
────────O────────

after:
──────\___/──────
       O
```

Sinkage depends on:

- vehicle mass;
- contact patch;
- tire pressure;
- soil constitutive model;
- saturation;
- loading rate;
- previous compaction;
- rut history.

---

## 16.2. Rut formation

A wheel:

- compresses soil;
- shears it backwards;
- displaces it sideways;
- can pump water and slurry out of the contact patch.

The rut is persistent geometry/material state.

Following vehicles interact with the changed terrain.

This allows persistent route evolution.

---

## 16.3. Wheel slip

Wheel slip becomes a physical outcome.

```text
wheel angular speed ↑
vehicle forward speed ↓
      ↓
soil shear failure
      ↓
traction loss
      ↓
more excavation
```

A vehicle may dig itself in without a special "stuck in mud" rule.

---

## 16.4. High-centering / belly contact

Deep ruts can become deep enough for:

- axle contact;
- chassis contact;
- differential housing contact;
- underbody drag.

Then some vehicle weight is carried by the chassis rather than tires.

Traction changes naturally.

---

## 16.5. Tire tread becomes meaningful

With sufficiently detailed contact:

- highway tire;
- all-terrain;
- mud-terrain;
- tractor tread;
- tracks;

can produce different shear/displacement patterns.

Real tread geometry can interact with the deformable continuum rather than only selecting a friction coefficient.

---

## 16.6. Tire pressure becomes gameplay

Lower tire pressure:

- increases contact patch;
- decreases ground pressure;
- reduces sinkage on soft ground;
- changes carcass deformation;
- changes rolling resistance;
- can increase risk of bead loss / tire damage.

This means tire pressure can become a mechanically meaningful strategy variable.

A high-fidelity off-road simulator will eventually need a deformable tire model, not only rigid wheels.

---

# 17. Rain, drying, and terrain evolution

The terrain should evolve over time.

Potential state transitions:

```text
dry soil
   ↓ rain / infiltration
damp soil
   ↓
partially saturated
   ↓
fully saturated / weak
   ↓ wheel loading
remolded mud / rut
   ↓ drainage / evaporation
drying / reconsolidation
```

Interesting consequences:

- small rain may strengthen some granular materials via capillary cohesion;
- heavy saturation may reduce effective strength;
- puddles collect in ruts;
- subsequent vehicles remobilize wet sediment;
- routes become progressively worse;
- terrain can recover partially after drying.

This creates systemic gameplay rather than authored mud zones.

---

# 18. Other materials unlocked by the same architecture

The same continuum infrastructure can potentially support:

| Material | Candidate model |
|---|---|
| Water | incompressible fluid |
| Thick mud | viscoplastic |
| Wet clay | saturated elastoplastic |
| Dry sand | frictional granular |
| Wet sand | granular + pore fluid |
| Snow | compressible elastoplastic |
| Wet snow | multiphase snow + liquid |
| Slush | viscoplastic / multiphase |
| Silt | fine sediment + water |
| Swamp material | porous organic matrix + water |
| Gravel | granular |
| Avalanche | granular / snow flow |
| Sediment transport | water + granular phase |

Potential future expansion:

- lava;
- slurry;
- blood at macroscopic scale;
- soft biological tissue;
- destruction debris continuum;
- powders.

These require careful material-specific validation and should not be assumed to work automatically.

---

# 19. Candidate architecture for Rust

Suggested crates/modules:

```text
crates/
├── continuum-math
│   ├── kernels
│   ├── differential_operators
│   ├── matrix_small
│   └── numerical_checks
│
├── continuum-core
│   ├── particles
│   ├── materials
│   ├── phases
│   ├── timestep
│   ├── invariants
│   └── state
│
├── continuum-neighbors
│   ├── morton
│   ├── radix_sort
│   ├── spatial_hash
│   ├── bucket_ranges
│   └── neighbor_lists
│
├── continuum-solvers
│   ├── dfsph
│   ├── pbf
│   ├── ipbf
│   ├── variational
│   ├── viscosity
│   ├── surface_tension
│   ├── ddfk_experimental
│   └── monte_carlo_experimental
│
├── continuum-materials
│   ├── water
│   ├── herschel_bulkley
│   ├── drucker_prager
│   ├── mu_i
│   ├── cam_clay
│   ├── snow
│   └── porous_soil
│
├── continuum-multiphase
│   ├── soil_water
│   ├── sediment
│   ├── air_water
│   └── phase_transfer
│
├── continuum-boundaries
│   ├── analytical
│   ├── sdf
│   ├── triangles
│   ├── rigid_coupling
│   ├── articulated_coupling
│   └── deformable_tire_coupling
│
├── continuum-adaptivity
│   ├── error_estimator
│   ├── split
│   ├── merge
│   ├── conservation_projection
│   └── refinement_policy
│
├── continuum-surface
│   ├── classification
│   ├── anisotropic_field
│   ├── meshing
│   └── thin_sheet
│
├── continuum-secondary
│   ├── spray
│   ├── foam
│   ├── bubbles
│   └── droplets
│
├── continuum-gpu
│   ├── buffers
│   ├── pipelines
│   ├── scans
│   ├── sort
│   ├── reductions
│   ├── indirect_dispatch
│   └── profiler
│
├── continuum-validation
│   ├── water_tests
│   ├── soil_tests
│   ├── tire_soil_tests
│   ├── metrics
│   └── reference_data
│
└── continuum-render
    ├── water
    ├── mud
    ├── wetness
    ├── terrain_surface
    └── debug_visualization
```

This is a conceptual decomposition, not a requirement to create all crates immediately.

A simpler initial monorepo module structure is preferable until boundaries stabilize.

---

# 20. GPU data layout

Prefer Structure of Arrays.

Example:

```text
positions[]
velocities[]
masses[]
rest_volumes[]

material_ids[]

densities[]
pressure_multipliers[]

stress[]
deformation_gradients[]
plastic_state[]

porosity[]
saturation[]
pore_pressure[]

flags[]
resolution_levels[]
```

Reasons:

- compute passes access only relevant fields;
- better memory bandwidth;
- simpler compaction;
- easier split/merge;
- better GPU coalescing;
- easier double buffering.

Avoid large AoS particle structs in hot GPU paths.

---

# 21. Spatial hash

The particle system still needs fast neighbor lookup.

Possible pipeline:

```text
positions
   ↓
compute Morton/hash key
   ↓
radix sort (key, particle_id)
   ↓
build bucket ranges
   ↓
iterate neighboring buckets
```

Critical distinction:

> The hash cells do not contain the physical water/soil state.

They are temporary acceleration buckets.

Destroying/rebuilding the hash does not destroy the continuum state.

---

# 22. Frame / timestep pipeline

A future generalized timestep might look like:

```text
1. Apply external forces
2. Predict positions / velocities
3. Compute spatial keys
4. Radix sort particles
5. Build bucket ranges
6. Build/update neighbor relations
7. Update surface/interior classification
8. Evaluate material constitutive state
9. Assemble incompressibility / volume residuals
10. Solve fluid constraints
11. Solve solid/plastic material response
12. Solve viscosity / dissipation
13. Solve surface tension
14. Solve multiphase coupling
15. Resolve solid boundaries
16. Accumulate reaction impulses
17. Update rigid / articulated bodies
18. Commit velocities and positions
19. Estimate local error
20. Split/merge particles
21. Conservation projection
22. Reconstruct render surface
23. Spawn secondary spray/foam/sediment particles
24. GPU metrics / diagnostics
```

Not all passes should be enabled for all materials.

A production implementation should compile/build specialized pipelines per material combination.

---

# 23. Validation strategy

A visually convincing splash is not proof of a correct solver.

The engine needs automated physical benchmarks.

## 23.1. Water tests

- hydrostatic tank;
- dam break;
- oscillating droplet;
- Taylor-Green vortex;
- Poiseuille flow;
- sloshing tank;
- capillary rise;
- water wheel;
- floating rigid body;
- falling object splash;
- rotating container;
- thin-sheet breakup;
- closed air pocket.

## 23.2. Terrain/material tests

- angle of repose;
- column collapse;
- granular pile formation;
- soil compaction;
- shear box;
- triaxial compression;
- penetration test;
- wet/dry strength comparison;
- seepage through porous soil;
- saturated slope failure;
- sediment entrainment.

## 23.3. Off-road tests

- single driven wheel in dry sand;
- single driven wheel in saturated soil;
- wheel sinkage under static load;
- slip ratio vs traction curve;
- repeated wheel passes;
- rut depth evolution;
- tire pressure sweep;
- tread-pattern comparison;
- vehicle high-centering;
- water-filled rut;
- slope ascent on deformable terrain.

---

# 24. Metrics

Track at least:

## Fluid

\[
\epsilon_M =
\frac{|M_t - M_0|}{M_0}
\]

\[
\epsilon_V =
\frac{|V_t - V_0|}{V_0}
\]

\[
\epsilon_\rho =
\max_i \frac{|\rho_i - \rho_0|}{\rho_0}
\]

\[
\epsilon_{div}
=
\max_i |\nabla \cdot u_i|
\]

Also:

- total momentum;
- angular momentum;
- kinetic + potential energy;
- artificial dissipation;
- pressure iteration count;
- maximum penetration depth;
- surface-volume consistency.

## Granular/soil

- mass;
- momentum;
- plastic work;
- compaction;
- porosity;
- saturation;
- pore-pressure drift;
- yield-condition residual;
- angle of repose;
- rut depth;
- soil displacement volume.

## Performance

- total particle count;
- active particle count;
- average neighbors;
- max neighbors;
- sort time;
- neighbor time;
- solver time;
- surface reconstruction time;
- rigid coupling time;
- split/merge count;
- GPU memory;
- number of global synchronizations;
- timestep;
- substep count.

---

# 25. Implementation difficulty

Subjective difficulty scale:

| Target | Difficulty |
|---|---:|
| Simple SPH/PBF box demo | 4/10 |
| Correct CPU DFSPH | 6/10 |
| 3D GPU DFSPH | 8/10 |
| Robust arbitrary boundaries | 9/10 |
| Strong two-way rigid coupling | 9/10 |
| Gameplay-ready particle water | 9/10 |
| Adaptive particle continuum | 9.5/10 |
| Unified variational solver | 9.5/10 |
| Soil + water multiphase | 10/10 |
| Realistic tire-soil coupling | 10/10 |
| DDFK/Monte-Carlo hybrid free-surface water | research / uncertain |

The full system is effectively:

> a second physics engine specialized in continuum mechanics.

The mathematics is only one part of the difficulty.

The engineering challenge combines:

- numerical methods;
- GPU compute;
- dynamic data structures;
- computational geometry;
- rigid-body coupling;
- constitutive modeling;
- validation;
- rendering;
- profiling.

---

# 26. Recommended implementation roadmap

## Phase A — CPU reference water solver

Goal:

- Rust;
- `f64`;
- deterministic;
- fixed resolution;
- simple neighbor lookup;
- DFSPH;
- analytical box boundaries;
- no rendering dependency.

Purpose:

- numerical oracle;
- debugging baseline;
- validation harness.

Exit criteria:

- hydrostatic stability;
- dam-break behavior;
- bounded density error;
- acceptable mass/volume conservation.

---

## Phase B — GPU fixed-resolution DFSPH

Implement:

- GPU particle buffers;
- spatial hash;
- radix sort;
- bucket ranges;
- density computation;
- divergence iterations;
- density iterations;
- integration;
- GPU diagnostics.

Do not yet add:

- adaptivity;
- air;
- multiphase mud;
- DDFK;
- complex surface tension;
- deformable tire.

Exit criteria:

- CPU/GPU agreement within expected numerical tolerance;
- stable local water scenes.

---

## Phase C — Robust boundaries

Add progressively:

```text
plane
→ sphere
→ box
→ convex
→ SDF
→ triangle mesh
→ moving mesh
```

Then add conservative rigid reaction impulses.

Exit criteria:

- no leakage through tested boundaries;
- stable moving container;
- physically plausible floating objects;
- momentum transfer.

---

## Phase D — Surface reconstruction and rendering

Implement separately from the physics core:

- surface classification;
- anisotropic reconstruction;
- render mesh or implicit surface;
- optical shading;
- spray/foam.

Exit criteria:

- render system never modifies physical water state.

---

## Phase E — Variational water backend

Keep DFSPH as baseline.

Add:

```rust
enum FluidSolver {
    Dfsph,
    Variational,
}
```

Compare:

- stability;
- incompressibility;
- energy behavior;
- iteration count;
- cost.

---

## Phase F — Adaptive particles

Start conservatively:

- refine near free surface;
- refine near moving bodies;
- coarsen quiet deep interior.

Initially forbid split/merge in:

- thin sheets;
- high-energy impacts;
- complex boundary contacts;
- multiphase interfaces.

Exit criteria:

- demonstrable particle-count reduction for same error;
- conservation under repeated split/merge.

---

## Phase G — Generalize to continuum materials

Refactor common infrastructure.

Add first non-water material:

- dry sand using a known granular constitutive model.

Then:

- viscoplastic mud;
- snow;
- cohesive soil.

This validates that the architecture is truly material-generic.

---

## Phase H — Tire / wheel coupling

Start with:

- rigid wheel;
- simple tread;
- dry sand.

Then:

- saturated soil;
- wet mud;
- wheel slip;
- repeated passes;
- ruts.

Only later:

- deformable tire;
- detailed tread deformation;
- sidewall behavior.

---

## Phase I — Soil-water multiphase

Add:

- saturation;
- porosity;
- permeability;
- seepage;
- pore pressure;
- effective stress.

Target scenarios:

- rainfall;
- puddle formation;
- saturated rut;
- vehicle entering wet soil;
- drainage.

---

## Phase J — DDFK research branch

Experiment without replacing production water:

```text
DFSPH particles
    ↓
fit continuous divergence-free field
    ↓
measure:
- divergence
- vorticity preservation
- energy dissipation
- cost
```

Only allow feedback into particles after standalone validation.

---

## Phase K — Local air-water two-phase regions

Activate only around:

- trapped air;
- strong splashes;
- breaking wave;
- high-energy impact.

This should be optional and local.

---

# 27. Production path vs research path

Recommended organizational split:

## Production path

```text
GPU DFSPH
+
robust boundaries
+
rigid coupling
+
surface reconstruction
+
fixed / limited adaptive resolution
```

Goal:

- shippable material water;
- predictable performance;
- strong debugability.

## Research path

```text
unified variational particles
→ adaptive particle continuum
→ multiphase soil-water
→ DDFK continuous field
→ Monte-Carlo / learned operators
```

Goal:

- new capabilities;
- possible novel research result;
- not allowed to block production baseline.

---

# 28. Important architectural principle: physics invariants beat ML

If ML is introduced:

```text
ML proposes
     ↓
physics solver projects
     ↓
valid conservative state
```

Never make the base rule:

```text
neural network predicts next frame
```

for the core material simulation.

Hard invariants to preserve:

- mass;
- volume where applicable;
- linear momentum;
- angular momentum where applicable;
- non-penetration;
- constitutive yield constraints;
- phase/material conservation.

ML may accelerate or approximate secondary parts.

---

# 29. World-scale strategy

A fully resolved particle continuum for an entire open world is not realistic.

Use multiple levels of representation.

Possible hierarchy:

```text
far ocean
→ spectral FFT

large shallow terrain flow
→ SWE / reduced model

active gameplay fluid volume
→ particle continuum

violent local event
→ refined particles / local multiphase

tiny spray
→ secondary particles
```

For terrain:

```text
inactive terrain
→ compact constitutive state / baked representation

near vehicle / active deformation region
→ continuum particles

recently modified terrain
→ persisted compressed state

far old terrain
→ re-meshed / re-coarsened state
```

The important principle:

> Reduced models are acceptable as LODs.
> They should not dictate the fundamental representation of local high-fidelity gameplay material.

---

# 30. Persistence and streaming

For an off-road game, terrain deformation is gameplay state.

Need to persist:

- rut geometry;
- compaction;
- saturation;
- residual water;
- material displacement;
- possibly plastic strain history.

Storing every particle forever is undesirable.

Future research/engineering problem:

```text
active particles
     ↓ when region sleeps
compress / homogenize
     ↓
persistent terrain state
     ↓ when region wakes
reconstruct particles
```

Potential representations for sleeping regions:

- adaptive surface mesh + depth/material layers;
- voxelized material summary;
- sparse signed distance + constitutive fields;
- clustered particles;
- reduced-order local material state.

This is separate from the active simulation and may legitimately use cells/voxels as storage/LOD.

---

# 31. Interaction with the rest of the engine

The continuum system should expose queries to:

- rigid-body physics;
- vehicle system;
- character motor;
- AI navigation;
- rendering;
- audio;
- gameplay.

Potential API concepts:

```rust
struct MaterialSample {
    phase: Phase,
    density: f32,
    velocity: Vec3,
    pressure: f32,
    saturation: f32,
    yield_state: f32,
    surface_normal: Vec3,
}
```

Queries:

```rust
fn sample_material(position: Vec3) -> MaterialSample;
fn sample_region(aabb: Aabb) -> RegionMaterialStats;
fn apply_impulse(position: Vec3, impulse: Vec3);
fn inject_material(source: MaterialSource);
fn remove_material(sink: MaterialSink);
```

Vehicle-specific queries may include:

- local sinkage;
- ground reaction;
- shear resistance;
- pore pressure;
- terrain velocity;
- contact material history.

Avoid exposing solver-specific concepts such as "SPH density multiplier" to gameplay code.

---

# 32. Debugging tools that should exist from the beginning

Visualize:

- particle positions;
- particle level;
- material id;
- density error;
- divergence error;
- pressure;
- vorticity;
- surface classification;
- neighbor count;
- boundary distance;
- reaction impulses;
- saturation;
- porosity;
- pore pressure;
- plastic strain;
- yield state;
- split/merge events.

Debug modes should be GPU-friendly and switchable at runtime.

Without these, solver debugging becomes extremely difficult.

---

# 33. Reference projects / research directions discussed

These should be re-checked by the agent for latest versions and exact implementation details.

## Particle / fluid

- **SPlisHSPlasH**
  Reference SPH implementation with multiple pressure solvers, viscosity, surface tension, boundary handling, rigid coupling.
  https://github.com/InteractiveComputerGraphics/SPlisHSPlasH

- **Salva**
  Rust particle-fluid project and useful Rust API reference.
  https://github.com/dimforge/salva

- **DualSPHysics**
  Engineering-oriented SPH reference.
  https://dual.sphysics.org/

- **blub**
  Rust/wgpu PIC/FLIP/APIC experimentation.
  https://github.com/wumpf/blub

## Variational / particle research

- **PeriDyno**
  Research framework with recent particle/variational methods.
  https://peridynamics.com/

- **Implicit Position-Based Fluids (IPBF)**
  https://graphics.cs.utah.edu/research/projects/ipbf/

- **Nonlocal Unified Variational Framework for Free Surface Flows**
  SIGGRAPH 2026 research direction; implementation associated with PeriDyno.

## FLIP / high-end reference

- **APIC**
  Disney Animation research on Affine Particle-In-Cell.

- **PF-FLIP / very large-scale two-phase FLIP**
  https://ge.in.tum.de/publications/very-large-scale-two-phase-flip/

- **ST-FLIP / Spatiotemporal FLIP**
  https://ge.in.tum.de/publications/spatiotemporal-flip/

## Continuous / grid-free field research

- **Divergence-Free Kernel Fields (DDFK)**
  2026 research on continuous analytically divergence-free velocity representations.

- **Neural Monte Carlo Fluid Simulation**
  Continuous neural velocity field + Monte-Carlo PDE solving.

- **Walk-on-Spheres / Walk-on-Stars**
  Grid-free Monte-Carlo elliptic PDE methods useful conceptually for pressure/Poisson solving.

- **Learned Mesh-Free Differential Operators / NeMDO**
  GNN-based local differential operator approximation on irregular point sets.

## Vehicle / deformable terrain

- **Project Chrono Vehicle / CRM terrain**
  High-fidelity continuum deformable terrain with SPH-based soil models and vehicle coupling.
  https://api.projectchrono.org/vehicle_terrain.html
  https://api.projectchrono.org/vehicle_terrain_crm_api_.html

- Research on tire–mud coupling using SPH + finite-element tire models.

---

# 34. Key unresolved research questions

These should become explicit investigation tasks.

## Water

1. Which DFSPH formulation is the best practical GPU baseline?
2. What boundary representation is best for moving arbitrary meshes?
3. How much of surface tension should be in the main variational solve?
4. How to preserve thin sheets without exploding particle count?
5. Can adaptive split/merge be conservative enough for long gameplay sessions?
6. Can DDFK improve large-scale flow coherence without creating conflicting velocity states?
7. Is Monte-Carlo pressure useful as a low-frequency/global correction?
8. How to detect when local two-phase air simulation is required?

## Soil / mud

9. Which minimal constitutive model set covers:
   - sand;
   - clay;
   - mud;
   - snow?
10. How should saturation affect yield behavior?
11. Is a true two-phase poromechanics model required for believable gameplay mud?
12. How to persist plastic history when regions stream out?
13. How to reconstruct a sleeping deformable terrain region back into particles?

## Vehicle

14. How accurate must tread geometry be?
15. At what point does a deformable tire become necessary?
16. How to couple wheel/tire FEM or reduced deformable tire models to particle terrain?
17. What terrain resolution is required for meaningful tread interaction?
18. How to stabilize very stiff tire/terrain coupling?
19. How should chassis-ground contact interact with deformable terrain?
20. How much terrain history must be retained for believable repeated passes?

## Engine

21. Which GPU backend:
   - wgpu;
   - Vulkan directly;
   - CUDA interop;
   - mixed backend?
22. How deterministic must the solver be?
23. What networking strategy is possible for continuum state?
24. How to make the system streamable in an open world?
25. Which parts should run at lower frequency than the main physics step?

---

# 35. Suggested specification decomposition for the agent

The next agent should probably create separate specs/ADRs rather than one huge specification.

Recommended outputs:

## ADRs

1. `ADR-Continuum-Representation.md`
   - why particle continuum;
   - what "gridless" means;
   - permitted use of temporary grids.

2. `ADR-Water-Baseline-Solver.md`
   - choose DFSPH;
   - alternatives;
   - validation requirements.

3. `ADR-GPU-Neighbor-Search.md`
   - Morton/hash/radix sort architecture.

4. `ADR-Boundary-Representation.md`
   - analytical / SDF / triangle / semi-analytical.

5. `ADR-Rigid-Coupling.md`
   - impulse conservation;
   - subiterations;
   - body interfaces.

6. `ADR-Continuum-Material-Model.md`
   - water;
   - granular;
   - viscoplastic;
   - elastoplastic.

7. `ADR-Adaptive-Particles.md`
   - split/merge rules;
   - conservation.

8. `ADR-Surface-Reconstruction.md`
   - physics/render separation.

9. `ADR-Offroad-Terrain.md`
   - active continuum region;
   - sleeping persistent terrain.

10. `ADR-Research-Backends.md`
    - variational solver;
    - DDFK;
    - Monte Carlo;
    - learned operators.

## Implementation specs

- `SPEC-CPU-DFSPH-Reference.md`
- `SPEC-GPU-Particle-Storage.md`
- `SPEC-GPU-Spatial-Hash.md`
- `SPEC-GPU-DFSPH.md`
- `SPEC-Fluid-Boundaries.md`
- `SPEC-Rigid-Fluid-Coupling.md`
- `SPEC-Water-Surface.md`
- `SPEC-Continuum-Validation.md`
- `SPEC-Adaptive-Particles.md`
- `SPEC-Granular-Soil.md`
- `SPEC-Soil-Water-Multiphase.md`
- `SPEC-Wheel-Terrain-Coupling.md`
- `SPEC-Terrain-Persistence.md`

## Research spikes

- `SPIKE-Variational-Fluid.md`
- `SPIKE-DDFK-Hybrid.md`
- `SPIKE-Monte-Carlo-Pressure.md`
- `SPIKE-Learned-Operators.md`
- `SPIKE-Deformable-Tire.md`
- `SPIKE-Two-Phase-Air-Water.md`

---

# 36. Recommended first milestone

The first milestone should intentionally be much smaller than the final vision.

## Milestone: "Material Water Prototype"

Requirements:

- Rust;
- deterministic CPU reference;
- fixed particle radius;
- single water material;
- DFSPH;
- gravity;
- static analytical boundaries;
- mass/volume/density diagnostics;
- basic visualization/debug draw.

Scenarios:

1. hydrostatic box;
2. dam break;
3. pouring between containers;
4. falling rigid object;
5. floating rigid object.

No:

- adaptivity;
- DDFK;
- two-phase air;
- mud;
- sand;
- deformable tire;
- open-world streaming.

Purpose:

> Establish a physically trustworthy numerical core before scaling the architecture.

---

# 37. Recommended second milestone

## Milestone: "GPU Interactive Water"

Requirements:

- GPU particle storage;
- GPU spatial hash;
- radix sort;
- fixed-resolution DFSPH;
- moving SDF/triangle boundaries;
- two-way rigid coupling;
- basic surface reconstruction;
- profiler.

Demonstration:

- character-sized local pool;
- several floating rigid bodies;
- object thrown into water;
- gate opens and water escapes;
- stable real-time interaction.

This is the first milestone that can become an engine feature.

---

# 38. Recommended third milestone

## Milestone: "Deformable Terrain Prototype"

Requirements:

- reuse particle infrastructure;
- implement one granular/elastoplastic material;
- rigid wheel coupling;
- persistent deformation within a bounded test area;
- diagnostics for stress/plasticity.

Demonstration:

- wheel sinks;
- wheel spins;
- rut forms;
- second wheel follows first rut;
- traction changes due to physically modified terrain.

This validates the broader `ContinuumPhysics` architecture.

---

# 39. Recommended fourth milestone

## Milestone: "Wet Mud / Off-road Prototype"

Requirements:

- water content / saturation;
- pore pressure or simplified coupled fluid state;
- viscoplastic or saturated-soil material;
- wheel slip;
- water displacement;
- persistent ruts;
- simple tire pressure parameter.

Demonstration:

```text
dry track
→ rain
→ saturation
→ first vehicle creates ruts
→ water accumulates
→ second vehicle has different route difficulty
→ low tire pressure improves soft-ground performance
```

This would be a strong technology showcase.

---

# 40. Long-term vision

The long-term system is not "better water".

It is:

> A physically simulated material layer for the game world.

Conceptually:

```text
                         WORLD
                           │
                Continuum Material Layer
                           │
       ┌───────────────────┼───────────────────┐
       │                   │                   │
     Water                Soil                Snow
       │                   │                   │
       ├───────┐           │           ┌───────┤
       │       │           │           │       │
      Mud   Sediment      Sand       Slush    Ice*
       │                   │
       └──────────────┬────┘
                      │
              Multiphase Coupling
                      │
            Rigid / Articulated Bodies
                      │
             Vehicles / NPC / World
```

`*` Ice would likely require a different fracture/solidification model and should not be assumed to fall out automatically from the initial solver.

This layer could eventually influence:

- vehicles;
- characters;
- AI navigation;
- combat;
- construction;
- weather;
- world persistence;
- destruction;
- environmental simulation.

---

# 41. Final recommendation

Do not begin by implementing the most exotic research idea.

Build in this order:

```text
1. CPU DFSPH reference
2. GPU DFSPH
3. robust boundaries
4. two-way rigid coupling
5. surface reconstruction
6. adaptive particles
7. generic constitutive material interface
8. sand / soil
9. mud + saturation
10. tire / terrain coupling
11. unified variational backend
12. DDFK hybrid experiments
13. local two-phase air/water
14. learned acceleration where useful
```

The key architectural commitment should be made early:

> The active high-fidelity material state is represented as a Lagrangian continuum, not as a permanent heightfield or fixed voxel water grid.

But the implementation should remain pragmatic:

- use grids for temporary neighbor acceleration;
- allow reduced-order/grid LODs far from gameplay;
- keep mature baseline solvers beside research solvers;
- validate physics continuously;
- do not allow novel research components to block a usable engine feature.

---

# 42. One-sentence project thesis

> **Build a GPU-accelerated adaptive Lagrangian continuum physics layer for a Rust game engine, starting with incompressible water and expanding to multiphase deformable terrain, so that fluids, mud, soil, sand, snow, vehicles, and characters interact through shared material physics rather than scripted surface effects.**
