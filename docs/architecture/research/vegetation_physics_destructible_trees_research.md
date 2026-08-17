# Vegetation Physics / Destructible Trees
## Research and Design Brief for the Rust Game Engine

**Status:** working research brief, not a frozen specification
**Context date:** August 2026
**Primary goal:** provide an agent with enough context to derive architecture decisions, ADRs, milestones, experiments, validation scenarios, and implementation specifications for physically simulated vegetation.

---

# 0. Executive summary

The goal is not to build a collection of scripted tree effects such as:

- canned wind animation;
- fixed "tree HP";
- predefined falling animations;
- one-shot break states;
- fire timers;
- decals for axe cuts.

The target is a **hierarchical physically simulated plant structure** where the most important interactions emerge from mechanics.

The desired system should eventually support:

- trunks and branches that bend and twist under wind and collisions;
- realistic progressive damage rather than binary intact/broken states;
- arbitrary cutting and chopping;
- saw cuts;
- physically plausible felling direction from notch geometry, gravity, wind, and remaining hinge wood;
- branch fracture;
- tree-to-tree collision;
- vehicles hitting and breaking vegetation;
- trees falling onto terrain and deformable ground;
- realistic wind response;
- leaf and foliage flutter;
- moisture;
- combustion;
- charring and loss of structural strength;
- embers and fire spread;
- root anchoring;
- interaction between roots and deformable/wet soil;
- physics LOD so that forests remain affordable.

The main architectural conclusion is:

> Do not simulate the visual tree mesh directly.

Instead represent a tree with multiple layers:

```text
visual geometry
      ↑
foliage clusters / bark / leaves
      ↑
structural branch graph
      ↑
beam / rod mechanics
      ↑
material state
      ↑
damage / thermal / moisture / decay
```

The most promising structural foundation is a **hierarchical Cosserat-rod / beam skeleton** for the trunk and important branches, with a more detailed **fiber/strand or cross-section damage representation** activated only near cuts and fracture zones.

A full tree therefore does not need millions of mechanical degrees of freedom.

Most of its visual complexity can follow a much smaller physical skeleton.

The most important production principle is:

> Tree physics must be adaptive both in space and in behavior.

A distant tree can be shader-driven.
A nearby calm tree can use modal response.
A tree touched by a vehicle can activate a coarse beam solver.
A branch being chopped can locally refine into detailed fracture mechanics.

This makes highly realistic trees significantly more practical than a uniformly high-resolution continuum simulation.

---

# 1. System philosophy

## 1.1. Avoid "tree HP"

A conventional game implementation often reduces a tree to:

```text
Tree
├── HP
├── wind shader
├── intact mesh
└── broken mesh
```

Example:

```cpp
tree.hp -= axe_damage;

if (tree.hp <= 0) {
    play_falling_animation();
}
```

This does not allow physical causality to emerge.

A more systemic implementation should instead track:

- structural load;
- bending;
- torsion;
- compression;
- tension;
- shear;
- local cross-section loss;
- accumulated material damage;
- moisture;
- heat;
- decay;
- root anchoring.

The tree falls when its remaining structure can no longer support applied loads.

---

# 2. Core representation

## 2.1. Structural graph

Represent the tree as a rooted graph:

```text
                 branch tip
                     ●
                    /
                   ●
                  / \
                 ●   ●
                /
               ●
               │
               ●
               │
               ●
             root/trunk
```

Nodes may represent:

- branch junctions;
- discretization points;
- mass points;
- fracture boundaries;
- root junctions.

Edges represent structural segments.

Conceptual data:

```rust
struct TreeStructure {
    root_node: NodeId,
    nodes: NodeStorage,
    segments: SegmentStorage,
    foliage_clusters: FoliageStorage,
    root_system: RootSystem,
}
```

The visual mesh is attached to this structure rather than serving as the physical model itself.

---

# 3. Branch and trunk mechanics

## 3.1. Why not a rigid-body chain?

A simple chain of rigid capsules with rotational joints can create basic bending.

However it tends to suffer from:

- visible joint behavior;
- poor continuum bending;
- awkward torsion;
- difficult stiffness tuning;
- unstable very-stiff joints;
- excessive constraints;
- unrealistic deformation localization.

For high-quality structural vegetation, a beam/rod model is preferable.

---

# 4. Cosserat rods as the structural core

Cosserat rods are a natural mathematical representation for long slender structures.

A rod can represent:

- stretching;
- compression;
- shear;
- bending;
- twisting.

Conceptually:

```text
rest:
──────────────

bend:
──────╮
      ╰────

twist:
────↻────

stretch:
←────────→
```

For each rod element we need geometric and material properties such as:

```rust
struct RodSegment {
    rest_length: f32,

    radius_start: f32,
    radius_end: f32,

    density: f32,

    young_modulus_longitudinal: f32,
    young_modulus_transverse: f32,
    shear_modulus: f32,

    damping: f32,

    damage: DamageState,
    thermal: ThermalState,
}
```

The exact field layout should be specialized and stored in SoA form on the GPU.

---

# 5. Wood is anisotropic

Wood should not be modeled as isotropic plastic.

Its mechanical properties strongly depend on fiber direction.

Conceptually:

```text
fiber direction:

|||||||||||||||||||||
|||||||||||||||||||||
|||||||||||||||||||||
```

Strength differs for:

- tension parallel to grain;
- tension perpendicular to grain;
- compression parallel to grain;
- compression perpendicular to grain;
- shear along fibers;
- transverse shear.

This matters greatly for:

- axe cutting;
- branch splitting;
- longitudinal cracks;
- hinge wood during felling;
- splinter formation;
- partial branch failure.

A practical material model may begin with a reduced orthotropic model rather than full detailed wood mechanics.

---

# 6. Section mechanics

For a branch segment, important section properties include:

- cross-sectional area;
- second moment of area;
- torsional constant;
- remaining intact area after damage.

For an approximately circular cross section:

\[
A = \pi r^2
\]

and bending resistance depends strongly on radius:

\[
I = \frac{\pi r^4}{4}
\]

Therefore a seemingly small reduction in trunk radius can strongly reduce bending capacity.

This is extremely useful for physical chopping.

---

# 7. Stress and failure

A branch segment may track generalized internal loads:

```text
axial force
shear force
bending moment
torsional moment
```

These can be mapped to material stress measures.

A simplified bending relation:

\[
\sigma_b \sim \frac{M c}{I}
\]

where:

- `M` = bending moment;
- `c` = distance from neutral axis;
- `I` = second moment of area.

Failure should not be a single scalar HP threshold.

Potential failure channels:

```text
tension failure
compression crushing
shear failure
torsional failure
fiber splitting
fatigue
thermal weakening
biological decay
```

---

# 8. Damage model

Damage should be persistent and local.

Conceptually:

```rust
struct DamageState {
    tensile: f32,
    compression: f32,
    shear: f32,
    torsion: f32,

    fiber_severing: f32,
    crushing: f32,

    fatigue: f32,
}
```

For a first implementation these may be reduced.

The important principle:

> Damage modifies mechanical properties rather than merely counting toward destruction.

For example:

```text
effective_strength =
base_strength
× moisture_modifier
× temperature_modifier
× decay_modifier
× damage_modifier
```

And similarly:

```text
effective_stiffness =
base_stiffness
× damage_modifier
× thermal_modifier
```

---

# 9. Hidden damage

A powerful systemic behavior:

1. a vehicle hits a tree;
2. trunk fibers are damaged;
3. the tree remains standing;
4. later strong wind increases bending;
5. the damaged section fails.

This gives persistent structural history.

Likewise:

```text
previous axe strikes
+
rot
+
snow load
+
wind gust
→ delayed branch failure
```

No special scripted event is required.

---

# 10. Cutting and chopping

## 10.1. Desired behavior

An axe should not subtract generic tree health.

It should alter a local cross section.

Conceptually:

```text
before:

   _________
 / ||||||||| \
|  |||||||||  |
 \_|||||||||_/

after several strikes:

   _________
 / |||    ||| \
|  ||      ||  |
 \_|||____|||_/
```

The remaining structural material determines:

- bending stiffness;
- axial load capacity;
- torsional strength;
- direction of final fracture.

---

# 11. Local fiber / strand representation

A full tree does not need detailed fibers everywhere.

Instead use:

```text
coarse rod everywhere
      ↓
interaction detected
      ↓
local high-detail cross-section / strand region
```

Near a cut, represent the section as:

- fiber bundles;
- radial sectors;
- anisotropic material cells;
- procedural strands.

This region can accumulate severing.

Conceptually:

```text
coarse trunk rod
      │
      │
      ▼
┌───────────────┐
│ local section │
│ ||||||||||||| │
│ ||||||||||||| │
│ ||||||||||||| │
└───────────────┘
      │
coarse trunk rod
```

This is the tree equivalent of adaptive particle refinement in the continuum-material system.

---

# 12. Axe impact model

An axe strike has:

- blade geometry;
- blade velocity;
- mass;
- impact angle;
- sharpness;
- penetration depth;
- local grain direction.

A first model may estimate impact energy:

\[
E_k = \frac12 m v^2
\]

and use:

- local normal;
- cutting direction;
- material toughness;
- blade sharpness;

to determine how much local fiber area is severed.

Do not require physically simulating micron-scale cutting.

The target is structurally plausible macroscopic behavior.

---

# 13. Notch-based felling

Real tree felling is an excellent test of the structural model.

The player can make a face notch:

```text
      trunk
       │
       │
      /│
     / │
____/  │
```

and a back cut on the opposite side.

The local cross section becomes asymmetric.

The remaining uncut region acts as hinge wood.

Then:

```text
gravity
+
tree lean
+
wind
+
remaining hinge stiffness
→ falling direction
```

This behavior should emerge from the structural model.

No preselected falling direction is required.

---

# 14. Saw cutting

A saw can be modeled as a moving cutting manifold.

Conceptual process:

```text
saw blade
    ↓
find intersection with local cross section
    ↓
accumulate material removal
    ↓
sever fibers
    ↓
update section properties
```

For chainsaws:

- chain velocity;
- motor power;
- wood hardness;
- kerf width;
- cutting depth;

may determine cut rate.

The gameplay model can still be simplified while preserving physically meaningful geometry.

---

# 15. Fracture

When a section becomes unstable:

```text
structural graph
      ↓
fracture event
      ↓
split graph
```

Example:

```text
TreeStructure
    ↓
break
    ↓
RootedStructure
+
DetachedStructure
```

The detached part remains deformable initially.

After motion becomes small, it may transition to a cheaper representation.

---

# 16. Dynamic-to-sleep conversion

A freshly broken branch may require:

```text
rod dynamics
+
collision
+
secondary fracture
```

Once resting:

```text
deformable branch structure
      ↓
sleep threshold
      ↓
compound rigid body / static object
```

This is important for performance.

Possible later reactivation triggers:

- player grabs it;
- vehicle hits it;
- fire weakens it;
- terrain moves beneath it.

---

# 17. Tree-to-tree interaction

A falling tree can impact another tree.

The collision impulse should enter the branch structural solver.

Possible outcomes:

- elastic bending;
- branch fracture;
- trunk fracture;
- foliage stripping;
- no failure.

Conceptually:

```text
Tree A falling
       ↓
collision contact
       ↓
impulse
       ↓
Tree B rod stress
       ↓
damage / fracture
```

This allows domino-like environmental interactions without authoring them.

---

# 18. Vehicle-tree interaction

The continuum/off-road vehicle system makes trees especially interesting.

Example outcomes:

## Sapling

```text
vehicle →
sapling bends
vehicle passes
sapling recovers
```

## Medium tree

```text
vehicle impact
→ permanent bend
→ fiber damage
→ partial trunk fracture
```

## Large healthy tree

```text
vehicle impact
→ vehicle receives strong reaction impulse
→ vehicle body damage
→ tree only slightly damaged
```

## Rotten tree

```text
modest collision
→ trunk failure
→ collapse
```

The result depends on material state rather than an arbitrary tree class.

---

# 19. Wind architecture

Do not run full atmospheric CFD through every tree.

Use a layered wind field.

Conceptually:

```text
WorldWind(x,t)
=
weather mean wind
+
large gusts
+
turbulent field
+
local obstacle correction
```

Possible representations:

- procedural curl noise;
- spectral turbulence;
- precomputed flow volumes around static architecture;
- local wake approximation;
- terrain-aware wind modifiers.

The vegetation system queries wind velocity at representative points.

---

# 20. Aerodynamic loading

A basic drag model:

\[
F_d =
\frac12 \rho C_d A |v_{rel}| v_{rel}
\]

where:

- `rho` = air density;
- `Cd` = drag coefficient;
- `A` = projected area;
- `v_rel` = air velocity relative to moving plant segment.

This load is applied to:

- trunk segments;
- major branches;
- foliage clusters.

Do not apply separate full CFD forces to every polygon.

---

# 21. Crown reconfiguration

Real trees reduce aerodynamic load by reorienting branches and foliage.

As wind increases:

```text
wind speed ↑
      ↓
leaves align
branches bend
      ↓
projected area ↓
      ↓
effective drag coefficient changes
```

Therefore use effective area / drag models that depend on deformation.

A simple production approximation:

```text
A_effective =
A_rest × reconfiguration_factor(wind_speed, bend_state)
```

This improves both realism and numerical stability.

---

# 22. Modal tree dynamics

A key optimization for medium-distance vegetation is modal deformation.

Instead of integrating every rod element, approximate tree displacement as:

\[
x(t) \approx x_0 + \sum_k a_k(t)\phi_k
\]

where:

- `phi_k` = deformation mode;
- `a_k` = time-varying modal amplitude.

Typical modes may capture:

- whole-trunk sway;
- lateral crown sway;
- torsional crown response;
- major branch motion.

Benefits:

- very few dynamic degrees of freedom;
- coherent natural oscillation;
- suitable for thousands of trees;
- compatible with gust-driven forcing.

---

# 23. Physics LOD

This is the central optimization strategy.

Suggested levels:

| Physics LOD | Representation |
|---|---|
| LOD 4 | static / impostor |
| LOD 3 | shader wind |
| LOD 2 | modal tree response |
| LOD 1 | coarse Cosserat skeleton |
| LOD 0 | adaptive structural + local fracture |

LOD selection should not be based only on camera distance.

Use relevance signals:

```text
distance
+
visibility
+
wind severity
+
current damage
+
fire state
+
player interaction
+
vehicle proximity
+
potential falling hazard
```

---

# 24. Adaptive structural refinement

Even LOD 0 should not run maximum resolution over the whole tree.

Example:

```text
crown
→ modal/coarse

major branches
→ coarse rods

trunk
→ medium rods

axe impact area
→ high-resolution rod + fiber section
```

Refinement triggers:

- cutting;
- high stress gradient;
- crack initiation;
- contact;
- branch junction;
- fire front;
- major local curvature;
- vehicle impact.

Coarsening can occur after:

- interaction stops;
- stresses decrease;
- fracture region becomes stable.

---

# 25. Leaves and foliage

The visual tree may contain hundreds of thousands or millions of leaves.

Do not mechanically simulate them all.

Use foliage hierarchy.

---

# 26. Foliage clusters

Represent a set of leaves as one aerodynamic and animation unit:

```rust
struct FoliageCluster {
    anchor_segment: SegmentId,

    rest_area: f32,
    mass: f32,

    drag_coefficient: f32,

    flutter_frequency: f32,
    damping: f32,

    orientation: Quat,
}
```

A cluster controls many visual leaves.

---

# 27. Leaf flutter

A cheap physically motivated model is a driven damped oscillator:

\[
I\ddot{\theta}
+
c\dot{\theta}
+
k\theta
=
\tau_{wind}
\]

Add high-frequency turbulent forcing.

The result can be converted into:

- card rotation;
- vertex bending;
- small normal perturbation;
- clustered flutter.

This gives physical coherence without simulating air flow around each leaf.

---

# 28. Leaf LOD

## Far

```text
impostor / cards
+
shader motion
```

## Medium

```text
cluster oscillator
+
branch modal motion
```

## Near

```text
individual leaf instances
+
GPU procedural flutter
```

## Contacted leaf

Only if required:

```text
temporary hinge / PBD particle / tiny rigid element
```

Most leaves should never become individual physics bodies.

---

# 29. Leaf detachment

Leaves may detach due to:

- high wind;
- branch fracture;
- fire;
- seasonal state;
- collision.

When detached:

```text
attached leaf
   ↓
cheap particle / billboard
   ↓
ballistic + wind advection
   ↓
sleep / disappear / ground accumulation
```

No need for full rod physics.

---

# 30. Fire and wood combustion

Combustion should be separated into:

1. material thermochemistry;
2. structural weakening;
3. visible flames/smoke;
4. fire spread.

Do not use:

```text
burn_timer
→ delete tree
```

---

# 31. Wood thermal state

Conceptual per-segment state:

```rust
struct ThermalState {
    temperature: f32,

    moisture: f32,

    virgin_wood_fraction: f32,
    pyrolysis_fraction: f32,
    char_fraction: f32,
    ash_fraction: f32,
}
```

Not all of these need to exist in the first version.

---

# 32. Simplified combustion progression

Conceptual progression:

```text
wet wood
   ↓ heating
drying
   ↓
dry wood
   ↓ heating
pyrolysis
   ↓
combustible gases
   ↓
char formation
   ↓
ash / mass loss
```

Moisture delays ignition because energy is consumed by heating and evaporation.

---

# 33. Structural effects of fire

Combustion changes:

- cross-sectional area;
- mass;
- stiffness;
- fracture strength;
- surface properties.

Conceptually:

```text
temperature ↑
char depth ↑
intact section ↓
strength ↓
```

A branch can therefore fail under its own weight.

Example:

```text
burning branch
     ↓
strength decreases
     ↓
wind load unchanged
     ↓
fracture
```

No explicit "fire destroys branch" event is required.

---

# 34. Radial thermal approximation

Full volumetric combustion for every branch is too expensive.

A strong game-oriented approximation:

For each trunk/branch segment, track radial layers:

```text
outside
  ↓
surface
char
pyrolysis zone
heated wood
cold interior
  ↓
center
```

This reduces local thermal conduction to a near-1D radial problem.

Possible state:

```rust
struct RadialBurnState {
    surface_temperature: f32,
    char_depth: f32,
    pyrolysis_depth: f32,
    moisture_front_depth: f32,
}
```

This can update structural section properties cheaply.

---

# 35. Local volumetric combustion LOD

For hero interactions, activate a more detailed local model only where necessary.

Example:

```text
tree branch
───────────────
        ▲
    active flame
        │
   local burn volume
```

Outside that local region, use branch-level thermal state.

This mirrors the general adaptive philosophy used for water and fracture.

---

# 36. Visual fire should be separate

The structural solver should output physical quantities such as:

```text
heat release rate
surface temperature
fuel vapor generation
char state
ember generation rate
```

The rendering/fire VFX system then generates:

- flames;
- smoke;
- glow;
- sparks;
- embers;
- lighting;
- heat distortion.

A full Navier-Stokes combustion solver is not required for every burning tree.

---

# 37. Fire spread

Fire spread can have multiple channels.

## Radiation

Hot surfaces heat visible nearby fuel.

## Convection

Hot gas rises and is transported by wind.

## Embers / firebrands

Burning fragments travel with airflow and can ignite distant vegetation.

Conceptual:

```text
burning tree
   ↓
ember generation
   ↓
wind advection
   ↓
landing
   ↓
local moisture / temperature test
   ↓
ignition
```

---

# 38. Moisture and rain

Vegetation should interact with the world water/weather system.

Possible moisture hierarchy:

```text
rain
 ↓
leaf wetness
 ↓
bark wetness
 ↓
wood moisture
```

Effects:

- ignition delay;
- reduced flame spread;
- increased branch mass;
- altered stiffness;
- altered fracture behavior;
- different decay conditions.

This is an important connection between vegetation, weather, water, and fire.

---

# 39. Roots

Root mechanics are essential for realistic uprooting.

Do not initially simulate every fine root.

Use a hierarchical root graph:

```text
             trunk
               │
          ┌────┴────┐
          │         │
        root       root
       /   \         \
     root  root      root
```

Important variables:

- root radius;
- length;
- orientation;
- tensile capacity;
- bending stiffness;
- root-soil bond;
- soil confinement.

---

# 40. Root-soil interaction

This becomes particularly powerful with the planned deformable continuum terrain.

Conceptually:

```text
root graph
    ↕
ContinuumSoil
```

Soil properties may include:

- compaction;
- cohesion;
- friction;
- saturation;
- pore pressure;
- plastic deformation.

Then root anchoring emerges from actual soil state.

---

# 41. Uprooting vs trunk fracture

Under wind load, two main failure modes compete:

```text
tree load
   ↓
┌───────────────┬────────────────┐
│               │                │
wood failure    root failure     soil failure
│               │                │
trunk snaps     roots break      root plate rotates
```

Wet saturated soil may reduce anchoring.

Dry strong soil may make trunk fracture more likely.

This is a major opportunity for cross-system emergent behavior.

---

# 42. Root plate and soil lifting

A detailed model could allow the root system to pull up a chunk of ground:

```text
tree falls
    ↓
roots rotate
    ↓
soil yields
    ↓
root plate lifts
```

This would connect directly to:

- deformable soil;
- mud;
- water-filled root cavity;
- later terrain persistence.

This is high-cost and should be an advanced feature.

---

# 43. Decay

Wood decay should affect mechanical properties.

Possible reduced state:

```rust
struct DecayState {
    decay_fraction: f32,
    moisture_sensitivity: f32,
    strength_multiplier: f32,
}
```

Effects:

- lower tensile strength;
- lower shear strength;
- lower stiffness;
- increased brittleness;
- hollow trunks;
- localized weak zones.

Decay allows interesting delayed failure.

---

# 44. Hollow trees

Do not require fully volumetric FEM.

A trunk cross section can have:

```text
outer radius
inner hollow radius
damaged sectors
char depth
```

Then:

\[
I_{hollow}
=
\frac{\pi}{4}
(R^4-r^4)
\]

for a simple hollow circular section.

This gives large mechanical effects at low computational cost.

---

# 45. Snow and ice loading

Future integration can add environmental loading:

```text
snow accumulation
     ↓
branch mass ↑
     ↓
bending moment ↑
     ↓
fracture probability ↑
```

Ice accumulation can similarly:

- increase mass;
- alter aerodynamic area;
- stiffen foliage;
- increase brittleness.

This creates weather-dependent structural behavior.

---

# 46. Suggested global architecture

```text
                        VegetationSystem
                              │
                 ┌────────────┴────────────┐
                 │                         │
          Structural Layer             Foliage Layer
                 │                         │
        Hierarchical Graph            Foliage Clusters
                 │                         │
       Cosserat / Beam Solver        Leaf Animation
                 │                         │
       ┌─────────┼─────────┐               │
       │         │         │               │
   Elastic     Damage    Fracture          │
       │         │         │               │
       └─────────┼─────────┘               │
                 │                         │
            Material State                │
                 │                         │
       ┌─────────┼───────────┐             │
       │         │           │             │
   Moisture   Thermal      Decay           │
       │         │           │             │
       └─────────┼───────────┘             │
                 │                         │
              Fire / Burn                  │
                 │                         │
                 └───────────┬─────────────┘
                             │
                     Rendering / VFX
```

External coupling:

```text
WorldWind ─────────────────────► Vegetation

Weather / Rain ────────────────► Moisture

ContinuumSoil ◄────────────────► Roots

Rigid Physics ◄────────────────► Tree Structure

Vehicle Physics ◄──────────────► Trunk / Branches

Character Motor ◄──────────────► Branch Contact

FireSystem ◄───────────────────► Combustion State
```

---

# 47. Integration with the engine's physical-world philosophy

The broader engine direction is moving toward a small number of general physical subsystems rather than large numbers of scripted gameplay reactions.

A possible top-level model:

```text
                        Physical World
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
 Continuum Matter      Living Structures       Atmosphere
        │                     │                     │
 water / soil            trees / plants            wind
 mud / sand                    │                    │
 snow                          │                    │
        └───────────────┬──────┴────────────────────┘
                        │
                   Energy / Fire
                        │
                  Rigid Physics
                        │
               Characters / Vehicles
                        │
                      Gameplay
```

The systems should exchange real physical state rather than event-specific booleans wherever practical.

---

# 48. Rust module proposal

Conceptual structure:

```text
crates/
├── vegetation-core
│   ├── tree
│   ├── graph
│   ├── segments
│   ├── species
│   ├── state
│   └── lifecycle
│
├── vegetation-rods
│   ├── cosserat
│   ├── beam_elements
│   ├── constraints
│   ├── integration
│   └── damping
│
├── vegetation-materials
│   ├── wood
│   ├── anisotropy
│   ├── damage
│   ├── plasticity
│   ├── decay
│   └── thermal
│
├── vegetation-fracture
│   ├── section_state
│   ├── fiber_bundles
│   ├── cutting
│   ├── crack_growth
│   └── graph_split
│
├── vegetation-wind
│   ├── sampling
│   ├── aerodynamic_load
│   ├── crown_reconfiguration
│   ├── modal_response
│   └── gusts
│
├── vegetation-foliage
│   ├── clusters
│   ├── flutter
│   ├── detachment
│   └── lod
│
├── vegetation-roots
│   ├── root_graph
│   ├── anchoring
│   ├── root_soil
│   └── uprooting
│
├── vegetation-fire
│   ├── thermal
│   ├── moisture
│   ├── pyrolysis
│   ├── char
│   ├── embers
│   └── spread
│
├── vegetation-collision
│   ├── broadphase
│   ├── branch_shapes
│   ├── tree_tree
│   ├── vehicle_tree
│   └── character_tree
│
├── vegetation-lod
│   ├── policy
│   ├── modal
│   ├── structural_activation
│   ├── refinement
│   └── sleeping
│
├── vegetation-gpu
│   ├── buffers
│   ├── solve
│   ├── reductions
│   ├── indirect_dispatch
│   └── profiler
│
├── vegetation-validation
│   ├── beam_tests
│   ├── fracture_tests
│   ├── wind_tests
│   ├── cutting_tests
│   ├── fire_tests
│   └── root_tests
│
└── vegetation-render
    ├── bark
    ├── cut_surfaces
    ├── foliage
    ├── burn_visuals
    ├── char
    └── debug
```

This is conceptual.

Do not create all crates immediately.

Start with a smaller module structure and split when subsystem boundaries become stable.

---

# 49. GPU storage

Prefer SoA.

Example:

```text
segment_position[]
segment_orientation[]
segment_linear_velocity[]
segment_angular_velocity[]

rest_length[]
radius_start[]
radius_end[]

young_modulus[]
shear_modulus[]

damage_tension[]
damage_shear[]
damage_torsion[]

temperature[]
moisture[]
char_depth[]

parent_index[]
child_range[]
lod_state[]
flags[]
```

Separate hot simulation data from cold botanical metadata.

---

# 50. Collision representation

Do not collide against full bark render mesh by default.

Use:

- tapered capsules;
- swept spheres;
- oriented cylinders;
- low-resolution branch hulls.

Near active cutting/fracture regions, higher-detail contact geometry may be generated temporarily.

This keeps tree-tree and vehicle-tree collisions affordable.

---

# 51. Tree generation

Physics should be derived from procedural/botanical structure.

Input tree generation can provide:

```text
branch topology
branch radius
branch length
branch age
wood density
species material profile
foliage distribution
root approximation
```

The physical representation can then be generated automatically.

Avoid manually authoring physics joints for every tree asset.

---

# 52. Species profiles

Different species may define:

```rust
struct TreeSpeciesPhysics {
    wood_density: f32,

    longitudinal_modulus: f32,
    transverse_modulus: f32,
    shear_modulus: f32,

    tensile_strength: f32,
    compression_strength: f32,
    shear_strength: f32,

    fracture_toughness: f32,

    moisture_response: MoistureCurve,
    thermal_response: ThermalCurve,

    crown_drag_profile: DragProfile,
}
```

This allows materially meaningful differences between:

- pine;
- birch;
- oak;
- dead wood;
- young flexible saplings.

---

# 53. Botanical age

Age/growth may influence:

- radius;
- mass;
- stiffness;
- branch taper;
- decay risk;
- root anchoring;
- bark thickness.

A later ecosystem simulation can update physical structure as trees grow.

This should not be required for the first implementation.

---

# 54. Wind LOD policy example

```text
far + calm
→ shader

far + storm
→ stronger shader/modal hybrid

medium + calm
→ modal

medium + high wind
→ modal + coarse structural correction

near
→ coarse rods

near + collision/cutting
→ rods + local refinement

near + imminent fracture
→ full active structural region
```

This policy is more efficient than a distance-only LOD.

---

# 55. Fracture LOD

A tree that has never been damaged does not need detailed fracture state everywhere.

Store compact section health.

When stress or cutting activates a region:

```text
section summary
    ↓
expand
    ↓
detailed local fiber state
```

After stable fracture:

```text
detailed state
    ↓
collapse to detached rigid/deformable chunk
```

---

# 56. Fire LOD

Suggested levels:

## Fire LOD 4

No thermal simulation.

## Fire LOD 3

Simple wet/dry state.

## Fire LOD 2

Per-branch temperature and ignition.

## Fire LOD 1

Radial burn layers and structural weakening.

## Fire LOD 0

Local adaptive volumetric combustion / detailed heat transfer.

This allows forest fires without volumetric combustion everywhere.

---

# 57. Forest-scale simulation

A forest cannot run active structural physics on every tree.

Suggested distribution:

```text
most trees:
shader / modal

dozens near player:
modal / coarse rods

few actively interacting:
full rods

one or several cutting/breaking:
local fracture refinement
```

During a storm, the system can prioritize trees based on:

- proximity;
- visibility;
- damage;
- predicted failure probability;
- gameplay relevance.

---

# 58. Predictive activation

Interesting optimization:

Estimate whether a tree is approaching failure without running the highest-detail model.

For example:

```text
coarse modal / rod state
     ↓
estimated bending moment
     ↓
estimated safety factor
```

If:

```text
safety_factor < threshold
```

activate higher-detail structural simulation before fracture.

This avoids abrupt LOD artifacts at the moment of breakage.

---

# 59. Sleeping structural state

A calm tree does not need high-frequency integration indefinitely.

Possible sleep condition:

```text
low velocity
+
low wind variation
+
no nearby interaction
+
safety factor high
```

Then transition to:

```text
modal equilibrium / shader state
```

Wake on:

- gust;
- contact;
- cut;
- fire;
- soil movement;
- nearby explosion.

---

# 60. Cross-system examples

## Example A — Vehicle in wet forest

```text
heavy rain
  ↓
soil saturation
  ↓
root anchoring decreases
  ↓
storm wind
  ↓
tree safety factor falls
  ↓
vehicle hits weakened trunk
  ↓
tree uproots
  ↓
root plate deforms mud
  ↓
tree blocks trail
```

No single script needs to know this entire chain.

---

## Example B — Player fells a tree

```text
axe strikes
  ↓
local fiber damage
  ↓
face notch
  ↓
back cut
  ↓
remaining hinge wood
  ↓
gravity + wind
  ↓
tree falls
  ↓
branch collision
  ↓
secondary fracture
  ↓
ground impact
```

---

## Example C — Burning dead tree

```text
dry dead wood
  ↓
low moisture
  ↓
fast ignition
  ↓
char depth increases
  ↓
trunk section weakens
  ↓
wind gust
  ↓
fracture
  ↓
burning branch falls
  ↓
ember generation
  ↓
grass ignition
```

---

# 61. Validation

The system needs numerical/physical tests, not only visual judgment.

## Structural tests

- cantilever beam bending;
- beam natural frequency;
- torsion test;
- axial compression;
- buckling;
- branch junction test;
- tapered beam.

## Fracture tests

- notched beam;
- progressive cross-section cut;
- tension parallel to grain;
- splitting;
- branch bending until failure;
- repeated fatigue loading.

## Wind tests

- single flexible branch;
- whole-tree sway;
- gust response;
- crown drag;
- damping;
- modal-vs-rod comparison.

## Cutting tests

- axe strike;
- repeated chop;
- asymmetric notch;
- back cut;
- saw cut;
- arbitrary-height cut.

## Fire tests

- wet vs dry ignition;
- char depth over time;
- thermal weakening;
- burning branch failure;
- ember generation.

## Root tests

- trunk pull-over;
- root anchoring in dry soil;
- anchoring in saturated soil;
- root plate rotation.

---

# 62. Metrics

Track:

## Structural

- total strain energy;
- kinetic energy;
- damping loss;
- maximum axial strain;
- maximum curvature;
- maximum torsion;
- safety factor;
- fracture count;
- active rod element count.

## Damage

- damaged cross-sectional area;
- severed fiber fraction;
- crack propagation;
- permanent deformation.

## Fire

- total intact wood mass;
- moisture mass;
- char mass;
- ash mass;
- heat release estimate;
- burned section depth.

## Performance

- active trees;
- modal trees;
- rod trees;
- active rod elements;
- refined fracture regions;
- collision pairs;
- GPU solve time;
- tree update frequency;
- fire update time;
- foliage update time.

---

# 63. Difficulty

Subjective scale:

| Feature | Difficulty |
|---|---:|
| Shader wind | 2/10 |
| Foliage cluster flutter | 3/10 |
| Modal tree wind | 4–5/10 |
| Coarse beam/tree physics | 6/10 |
| GPU Cosserat tree skeleton | 7/10 |
| Vehicle-tree interaction | 7–8/10 |
| Progressive branch damage | 8/10 |
| Physical arbitrary chopping | 8/10 |
| Convincing anisotropic fracture | 8–9/10 |
| Structural combustion | 8–9/10 |
| Forest fire spread | 9/10 |
| Root ↔ deformable soil coupling | 10/10 |
| Fully unified physical ecosystem | research-scale |

Trees are generally more tractable than high-fidelity volumetric fluids because their important structural degrees of freedom are sparse.

A visually complex tree may require only hundreds or thousands of structural DOFs at high fidelity.

---

# 64. Recommended implementation roadmap

## Phase A — Beam reference

CPU, `f64`.

Implement:

- single cantilever beam;
- tapered beam;
- bending;
- torsion;
- damping.

No tree assets yet.

Exit criteria:

- matches analytical beam cases within expected error.

---

## Phase B — Structural tree graph

Implement:

- branch graph;
- trunk;
- major branches;
- mass distribution;
- static gravity sag;
- wind loads.

Exit criteria:

- stable tree sway;
- natural frequencies plausible;
- no visible joint behavior.

---

## Phase C — Modal LOD

Compute or approximate low-order modes.

Implement:

- modal projection;
- modal wind forcing;
- transition between modal and full rod state.

Exit criteria:

- visually continuous transition;
- large forest can animate cheaply.

---

## Phase D — Collision

Add:

- tapered capsule branch shapes;
- rigid body interaction;
- vehicle collisions;
- branch-ground collision.

Exit criteria:

- stable impacts;
- correct impulse transfer;
- no explosive solver behavior.

---

## Phase E — Damage and fracture

Implement:

- per-section stress;
- accumulated damage;
- material weakening;
- graph splitting.

Start with simple scalar section damage.

Exit criteria:

- branches break based on physical loading.

---

## Phase F — Cutting

Add:

- arbitrary cut location;
- local cross-section damage;
- axe;
- saw;
- directional notch.

Exit criteria:

- tree falls in physically plausible direction from cut geometry.

---

## Phase G — Local fiber refinement

Replace scalar cut damage near interaction with:

- fiber bundles;
- anisotropic sectors;
- local strand model.

Exit criteria:

- split/splinter patterns improve without simulating entire tree at high resolution.

---

## Phase H — Foliage physics

Add:

- foliage clusters;
- aerodynamic area;
- flutter oscillator;
- crown reconfiguration.

Exit criteria:

- close-up foliage responds coherently to gusts.

---

## Phase I — Fire

Add:

- temperature;
- moisture;
- ignition;
- radial char model;
- strength reduction;
- burning branch failure.

Exit criteria:

- wet and dry trees burn differently;
- structural failure follows burning state.

---

## Phase J — Embers and spread

Add:

- ember generation;
- wind advection;
- ignition tests;
- neighboring vegetation heat transfer.

Exit criteria:

- local fire can spread without scripted adjacency graph.

---

## Phase K — Roots

Add:

- coarse root graph;
- anchoring constraints;
- root failure.

Initially use static soil strength.

Exit criteria:

- uprooting competes with trunk fracture.

---

## Phase L — Continuum-soil coupling

Connect roots to deformable soil.

Use:

- local soil stress;
- saturation;
- plastic deformation;
- pore pressure.

Exit criteria:

- wet soil meaningfully changes uprooting behavior.

---

# 65. Production path vs research path

## Production path

```text
shader wind
+
modal trees
+
coarse rod physics near interaction
+
simple structural damage
+
cross-section cutting
+
branch-level fire
```

This can already produce highly systemic trees.

## Research path

```text
adaptive rod refinement
→ strand/fiber fracture
→ sophisticated anisotropic wood
→ local detailed combustion
→ root-soil continuum coupling
→ biological decay
```

The research path must not block shipping a useful baseline.

---

# 66. Recommended first milestone

## Milestone: "Physical Tree Prototype"

Requirements:

- Rust;
- CPU reference;
- one procedural tree;
- trunk + 5–20 branches;
- beam/Cosserat structural model;
- gravity;
- procedural wind;
- simple branch collisions;
- debug visualization.

Tests:

1. calm tree;
2. steady wind;
3. gust;
4. branch pulled by external force;
5. branch released.

No:

- fracture;
- fire;
- leaves;
- root-soil continuum;
- GPU.

Goal:

> prove that the structural skeleton behaves like a living tree rather than a chain of rigid joints.

---

# 67. Recommended second milestone

## Milestone: "Destructible Tree"

Add:

- progressive damage;
- fracture;
- local section loss;
- axe;
- saw;
- physically falling trunk;
- detached branches.

Demo:

```text
player cuts notch
→ wind changes sway
→ back cut
→ hinge bends
→ tree falls
→ branch breaks on ground
```

This is the first major gameplay showcase.

---

# 68. Recommended third milestone

## Milestone: "Forest Physics LOD"

Add:

- shader trees;
- modal trees;
- active rod trees;
- relevance-based activation;
- state transfer.

Target:

- large forest;
- only a small subset uses expensive physics.

This milestone is critical before adding more realism.

---

# 69. Recommended fourth milestone

## Milestone: "Living Fire"

Add:

- moisture;
- heating;
- ignition;
- char depth;
- structural weakening;
- embers;
- basic spread.

Demo:

```text
wet tree vs dry tree
+
wind
+
branch fracture during burning
+
embers ignite nearby dry foliage
```

---

# 70. Recommended fifth milestone

## Milestone: "Rooted Physical World"

Connect:

```text
VegetationSystem
↕
ContinuumSoil
```

Demo:

```text
dry soil:
tree resists storm

saturated soil:
root plate rotates
tree uproots
ground deforms
```

This becomes a showcase of the wider engine architecture.

---

# 71. Candidate ADRs

The agent should likely derive separate architecture records.

1. `ADR-Vegetation-Structural-Representation.md`
   - structural graph;
   - rods vs rigid chains vs FEM.

2. `ADR-Tree-Physics-LOD.md`
   - shader;
   - modal;
   - rod;
   - refined fracture.

3. `ADR-Wood-Material-Model.md`
   - anisotropy;
   - damage;
   - simplified orthotropic parameters.

4. `ADR-Tree-Fracture.md`
   - scalar section damage;
   - local fibers;
   - graph split.

5. `ADR-Tree-Cutting.md`
   - axe;
   - saw;
   - arbitrary cut geometry.

6. `ADR-Vegetation-Wind.md`
   - wind field;
   - aerodynamic loading;
   - crown reconfiguration.

7. `ADR-Foliage-Physics.md`
   - clusters;
   - flutter;
   - leaf detachment.

8. `ADR-Tree-Fire.md`
   - branch thermal model;
   - radial char approximation;
   - fire VFX separation.

9. `ADR-Root-Soil-Coupling.md`
   - anchoring;
   - uprooting;
   - continuum soil integration.

10. `ADR-Vegetation-State-Persistence.md`
    - damage;
    - burn;
    - decay;
    - sleeping trees.

---

# 72. Candidate implementation specs

- `SPEC-Cosserat-Rod-Core.md`
- `SPEC-Tree-Structural-Graph.md`
- `SPEC-Tree-Modal-LOD.md`
- `SPEC-Tree-Wind-Loads.md`
- `SPEC-Tree-Collision.md`
- `SPEC-Wood-Damage.md`
- `SPEC-Tree-Fracture.md`
- `SPEC-Axe-Cutting.md`
- `SPEC-Saw-Cutting.md`
- `SPEC-Foliage-Clusters.md`
- `SPEC-Tree-Combustion.md`
- `SPEC-Ember-Transport.md`
- `SPEC-Root-Anchoring.md`
- `SPEC-Vegetation-GPU.md`
- `SPEC-Vegetation-Validation.md`

---

# 73. Candidate research spikes

- `SPIKE-GPU-Cosserat-Rods.md`
- `SPIKE-Strand-Wood-Fracture.md`
- `SPIKE-Adaptive-Tree-Refinement.md`
- `SPIKE-Anisotropic-Wood-Damage.md`
- `SPIKE-Modal-To-Rod-State-Transfer.md`
- `SPIKE-Radial-Wood-Combustion.md`
- `SPIKE-Root-Continuum-Coupling.md`
- `SPIKE-Forest-Fire-Spread.md`
- `SPIKE-Tree-Decay.md`

---

# 74. Open research questions

## Structural

1. Which rod formulation is most suitable for GPU implementation?
2. XPBD-style rods or an implicit Cosserat solver?
3. How many segments are required for believable trunk bending?
4. How should branch junction stiffness be modeled?
5. How to transition between modal and full rod states without energy jumps?

## Fracture

6. Is local cross-section damage sufficient for gameplay-quality chopping?
7. When is strand-level refinement visually necessary?
8. How should cracks propagate along grain?
9. How to generate fractured render surfaces from structural state?
10. How to preserve stability at the instant a structural graph splits?

## Wind

11. What wind-field representation gives the best cost/quality tradeoff?
12. How should local buildings and terrain modify vegetation wind?
13. How accurate must aerodynamic crown reconfiguration be?
14. Can wind loads be sampled per branch cluster instead of per segment?

## Foliage

15. What cluster size is acceptable?
16. How should leaf flutter frequency vary by species?
17. How much foliage state must be physical vs purely visual?

## Fire

18. Is radial 1D thermal conduction enough for trunks?
19. How should moisture transport be modeled?
20. How should char depth modify cross-section mechanics?
21. What is the minimum fire-spread model that produces systemic results?
22. When is local volumetric combustion worth activating?

## Roots

23. How detailed must the root graph be?
24. Can root anchoring be approximated by distributed constraints?
25. How to couple roots to particle soil without exploding constraint count?
26. How to persist uprooted terrain state?

## Performance

27. Which subsystem belongs on GPU?
28. How many fully active trees can the target hardware support?
29. How often must structural trees update relative to rigid physics?
30. Can modal trees update at a lower frequency?
31. How should detached branch sleeping work?

---

# 75. Reference directions and projects

These are research leads discussed in the original exploration.
The implementation agent should re-check exact titles, versions, licenses, and latest publications before using them as hard dependencies.

## Cosserat / rod mechanics

- Project Chrono FEA / Cosserat-style beam and rod elements
  https://api.chrono.projectchrono.org/

- Recent GPU and parallel Cosserat-rod research, including work on massively parallel rod simulation.

## Interactive wood fracture

Research directions include:

- anisotropic wood fracture;
- position-based wood fracture;
- fiber/strand representations;
- branch fracture through Cosserat rod structures.

The agent should specifically search SIGGRAPH / SIGGRAPH Asia 2024–2026 for:
- wood fracture;
- breaking branches;
- strand-based wood;
- Cosserat tree fracture.

## Tree dynamics

Research directions:

- modal tree animation;
- spectrum/modal tree response;
- dynamic tree models;
- wind-driven vegetation.

## SpeedTree

Useful production reference for:
- foliage LOD;
- wind authoring;
- large vegetation populations;
- card/mesh leaf representation.

https://www.speedtree.com/
https://docs.speedtree.com/

## Combustion

Research leads:

- interactive wood combustion for botanical tree models;
- FlameForge-style combustion/deformation coupling;
- vegetation wildfire simulation;
- ember/firebrand transport;
- moisture-dependent ignition.

## Wildfire / vegetation fire

Research directions include:
- branch-level fuel structure;
- pyrolysis;
- char;
- rain as a thermal sink;
- ember transport;
- wind-driven spread.

## Wood decay

Recent graphics research includes interactive modeling of fungal wood decay and its volumetric effects.

The agent should verify exact 2026 publications before using specific algorithms.

---

# 76. Critical engineering principles

1. **Do not simulate the visual mesh.**
2. **Use a sparse structural skeleton.**
3. **Treat wood as anisotropic.**
4. **Damage should alter material properties.**
5. **Cutting should alter local cross sections.**
6. **Fracture should split the structural graph.**
7. **Use modal dynamics for most trees.**
8. **Activate detailed rods only near relevant interactions.**
9. **Use local fiber refinement rather than fiber simulation everywhere.**
10. **Separate structural combustion from fire VFX.**
11. **Treat leaves primarily as clustered aerodynamic/visual elements.**
12. **Use roots as a structural interface to deformable soil.**
13. **Make Physics LOD depend on relevance, not only distance.**
14. **Keep a mature production path beside research-grade features.**
15. **Validate against mechanics, not only appearance.**

---

# 77. Long-term vision

The final vision is not simply "destructible trees".

It is a **living structural layer of the physical world**.

A tree becomes a persistent physical object whose state includes:

```text
geometry
+
structural load
+
damage
+
moisture
+
temperature
+
burn state
+
decay
+
root anchoring
+
soil state
```

Its behavior then emerges from the interaction of:

```text
wind
rain
soil
fire
vehicles
characters
other trees
gravity
```

This supports gameplay that does not require bespoke scripted reactions for each combination.

---

# 78. One-sentence project thesis

> **Build a hierarchical GPU-accelerated vegetation physics system for the Rust engine in which trees are sparse anisotropic structural graphs with adaptive rod/fiber mechanics, allowing wind, arbitrary cutting, fracture, fire, moisture, decay, vehicles, and deformable soil to interact through shared physical state rather than scripted destruction states.**
