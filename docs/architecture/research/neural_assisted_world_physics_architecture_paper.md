# Neural-Assisted World Physics
## Architecture Paper for Integrating Neural Networks into a Deterministic Physical World

**Document type:** architecture and research paper  
**Status:** working foundation for ADRs, implementation specifications, experiments, and validation plans  
**Context date:** August 2026  
**Target:** custom Rust game engine with a unified `PhysicalWorld / WorldDynamics` layer  
**Primary thesis:** deterministic physics remains the source of truth; neural models accelerate, approximate, prioritize, or initialize physical computation under explicit validation and automatic classical fallback.

**Related documents:**

- `physical_world_layer_architecture_for_specs.md`
- `continuum_physics_water_mud_offroad_research_brief.md`
- `vegetation_physics_destructible_trees_research.md`
- `vegetation_physics_followup_production_architecture.md`
- `arcane_world_layer_magic_architecture_paper.md`

---

# Abstract

This paper proposes an architecture for integrating neural networks into a simulated physical world containing rigid bodies, articulated characters, water, mud, soil, sand, snow, vegetation, wind, fire, thermal processes, fracture, and an arcane world layer.

The central recommendation is deliberately conservative:

> **Do not replace the physical world with a neural world model. Build a deterministic physical foundation first, then introduce learned components as bounded accelerators inside that foundation.**

A classical solver is required not only to generate initial training data. It must remain permanently available as:

- the source of physical truth;
- a reference implementation;
- a production baseline;
- a validator;
- a fallback;
- a generator of hard examples;
- a regression oracle;
- a means to retrain and improve models as the game evolves.

The recommended runtime pattern is:

```text
physical state
      ↓
neural proposal
      ↓
admissibility / OOD / residual checks
      ↓
short classical correction solve
      ↓
invariant validation
      ↓
accept
   or
full classical fallback
```

Neural components should first be used where they provide high value with bounded risk:

1. warm-starting iterative pressure and constraint solvers;
2. predicting preconditioners or correction directions;
3. selecting Physics LOD and active regions;
4. approximating unresolved subgrid effects;
5. learning difficult constitutive relations while preserving hard material constraints;
6. serving reduced-order models in calm, distant, or tightly bounded regimes;
7. optimizing spell plans, control policies, and representation transitions without owning physical invariants.

End-to-end learned simulation should be treated as an experimental backend, not as the authoritative physical substrate.

The long-term target is:

```text
Deterministic laws
+
Learned approximation
+
Invariant projection
+
Uncertainty/OOD gate
+
Automatic fallback
+
Continuous hard-case collection
```

This architecture makes neural acceleration measurable, reversible, debuggable, and progressively deployable without making the entire world dependent on the generalization behavior of one model.

---

# 1. Central architectural decision

The engine SHOULD be built in this order:

```text
1. define laws and invariants
2. implement a small deterministic reference solver
3. implement an optimized classical production solver
4. instrument both solvers
5. generate versioned training trajectories
6. train a narrow learned accelerator
7. validate every neural proposal
8. correct with the classical solver
9. fall back automatically when necessary
10. collect failures and retrain
```

The classical solver is not temporary scaffolding.

It remains part of the finished engine.

---

# 2. Why a deterministic physical foundation comes first

A learned model needs a definition of correctness.

For a physical system, visual plausibility alone is not enough.

Examples of required invariants:

## Fluids

- mass conservation;
- bounded density error;
- bounded divergence;
- volume preservation;
- no penetration through boundaries;
- conservative rigid-fluid impulse exchange.

## Granular materials and soil

- mass conservation;
- admissible stress;
- yield-surface consistency;
- bounded plastic work;
- porosity and saturation bounds;
- no spontaneous energy generation.

## Rods and trees

- valid branch topology;
- bounded constraint residual;
- plausible strain energy;
- conservative fracture impulses;
- no silent branch reconnection.

## Thermal and combustion systems

- energy balance;
- non-negative fuel mass;
- non-negative moisture;
- bounded phase fractions;
- consistent heat transfer.

## Rigid and articulated bodies

- contact non-penetration;
- joint limits;
- momentum exchange;
- stable constraints;
- no invalid inertia or transform state.

Without deterministic definitions and diagnostics, it is impossible to distinguish:

```text
fast physical approximation
```

from:

```text
visually convincing numerical failure
```

---

# 3. Determinism has multiple meanings

The architecture MUST distinguish at least three concepts.

## 3.1. Law determinism

Given:

```text
state(t)
external inputs
material parameters
boundary conditions
dt
```

the solver defines an unambiguous update rule.

This is required.

---

## 3.2. Tolerance determinism

Repeated executions produce states that may differ at the bit level but remain within specified physical tolerances.

This is usually the appropriate target for optimized GPU production solvers.

---

## 3.3. Bitwise determinism

Every output bit is identical across executions, devices, drivers, and thread schedules.

This is much harder because:

- floating-point addition is not associative;
- GPU reductions may change order;
- atomics are schedule-dependent;
- inference kernels may vary by backend;
- mixed precision changes rounding.

Recommended policy:

```text
CPU reference solver
→ maximize reproducibility

GPU production solver
→ tolerance-deterministic

networked topology events
→ server-authoritative

secondary visual dynamics
→ may be non-deterministic
```

Bitwise determinism should only be required where the game genuinely needs it.

---

# 4. The required solver hierarchy

The engine should maintain at least three solver roles.

---

## 4.1. `ReferenceSolver`

Characteristics:

```text
CPU
f64 where practical
small scenes
clear implementation
stable operation order
extensive diagnostics
slow but understandable
```

Responsibilities:

- validate formulas;
- reproduce analytical benchmarks;
- generate trusted labels;
- debug GPU divergence;
- verify conservation;
- produce regression trajectories;
- compare new algorithms.

The reference solver does not need to run the open world.

It needs to be trustworthy on representative bounded problems.

---

## 4.2. `ClassicalProductionSolver`

Characteristics:

```text
GPU-oriented
f32 or mixed precision
optimized memory layout
parallel reductions
streaming
adaptive resolution
Physics LOD
production boundaries and coupling
```

Responsibilities:

- serve as the shippable non-neural baseline;
- provide fallback;
- establish honest performance comparisons;
- run every supported scene without model dependencies.

The neural system MUST be compared against this solver, not only against a slow CPU teacher.

---

## 4.3. `HybridNeuralSolver`

Characteristics:

```text
production solver
+
neural warm starts
+
learned closures
+
learned representation decisions
+
validators
+
fallback
```

The public domain API should remain the same.

Example:

```rust
pub enum ContinuumBackend {
    ReferenceCpu,
    ClassicalGpu,
    HybridNeural,
    NeuralExperimental,
}
```

Other engine systems should not need to know the internal acceleration strategy.

---

# 5. Optional fourth role: high-fidelity offline teacher

Some training data may require a solver more expensive than the production baseline.

Examples:

- higher particle resolution;
- smaller timestep;
- more nonlinear iterations;
- two-phase air-water simulation;
- high-resolution MPM;
- detailed fiber fracture;
- offline CFD.

This solver may be:

```text
reference implementation
+
strict settings
+
higher resolution
```

or a trusted external scientific solver.

It should be treated as a data-generation tool, not automatically as the production source of truth.

---

# 6. Research position as of August 2026

Current research supports several important conclusions.

## 6.1. Hybrid warm starts are a strong first target

Neural Operator Warm Starts use a learned operator to generate an initial guess, while a classical iterative method still reaches the requested convergence tolerance. This preserves the existing discretization and solver structure. Reported gains can be substantial on selected PDE benchmarks, but must be re-measured against the engine's optimized GPU baseline. [R11]

## 6.2. Low prediction error is not sufficient for solver safety

Research from 2026 shows that a neural operator with low global \(L^2\) error can still create local physical violations that make the Jacobian indefinite and break the assumptions of memory-efficient Newton/Krylov solvers. Energy-aware, label-free fine-tuning restored a safe spectrum in the demonstrated problem. [R13]

Consequently:

> A neural warm start must be validated for solver admissibility, not only compared to a target field with MSE.

## 6.3. Solver-in-the-loop training improves long rollouts

When a network provides corrections inside a differentiable numerical solver, training through multiple solver steps exposes the model to the distribution created by its own corrections. This has produced more stable long rollouts than one-step supervised correction in several PDE classes, including 3D Navier-Stokes examples. [R3]

## 6.4. Hybrid particle simulation is more stable than neural-only rollout

Neural SPH showed that adding pressure, viscosity, and external-force components from conventional SPH substantially improves learned particle rollouts and reduces clustering-related failure. [R6]

## 6.5. Reduced-order models can eliminate many active degrees of freedom

GIOROM simulates sparse graphs and reconstructs a full representation, reporting particle-system sparsification in the range of approximately 6.6–32 times in its evaluated tasks. Its own limitations include degradation in extreme out-of-distribution regimes. [R7]

## 6.6. Fallback is already an explicit research pattern

Hybrid Neural-MPM combines neural and numerical simulation with a classical safeguard for difficult states. Its reported runtime benefit is meaningful but much more modest than the extreme speedups sometimes claimed for unconstrained surrogate comparisons. [R9]

## 6.7. Learned constitutive models are promising when hard constraints remain

UniPhy learns latent constitutive behavior across elastic material, plasticine, sand, Newtonian fluids, and non-Newtonian fluids. Newer work increasingly embeds objectivity, thermodynamic consistency, and stability into learned constitutive architectures. [R12] [R15]

## 6.8. Distribution shift is a core problem, not an edge case

SIMSHIFT was created specifically to evaluate neural surrogates under changes in geometries, configurations, and industrial simulation conditions. A model that performs well on random held-out frames can still fail on a new geometry or material regime. [R14]

## 6.9. Speedup claims require strong baselines

A systematic review found that many ML-for-PDE superiority claims used weak numerical baselines. The engine must therefore count graph construction, memory movement, inference, validation, corrections, and fallback when reporting speedup. [R16]

---

# 7. Trust tiers for learned physics

Every learned component should be assigned a trust tier.

---

## Tier 0 — observation only

The model:

- estimates cost;
- classifies regimes;
- predicts risk;
- does not change physical state.

Examples:

- fracture probability;
- OOD detection;
- representation relevance;
- performance prediction.

Risk: low.

---

## Tier 1 — initial guess

The model predicts:

- pressure multipliers;
- contact impulses;
- constraint multipliers;
- Newton initial states;
- preconditioner parameters.

The classical solver still converges to its own tolerance.

Risk: low to medium.

Recommended first production use.

---

## Tier 2 — bounded residual or closure

The model predicts:

- subgrid force;
- unresolved stress;
- turbulence closure;
- viscosity correction;
- heat-transfer correction.

The output is:

- clamped;
- projected;
- energy-bounded;
- followed by a classical step.

Risk: medium.

---

## Tier 3 — constrained constitutive law

The model predicts local material response:

```text
history + deformation + state
→ stress / yield / dissipation
```

Hard constraints enforce:

- objectivity;
- material symmetry;
- positive dissipation;
- stability;
- admissible stress.

Risk: medium to high.

---

## Tier 4 — reduced-order domain surrogate

The model advances a calm or bounded physical region with fewer degrees of freedom.

A validator decides whether to continue or promote to the classical representation.

Risk: high.

---

## Tier 5 — end-to-end neural simulator

The model predicts the next complete state.

Use only:

- in experimental backends;
- in narrow supported domains;
- with continuous validation;
- with immediate fallback;
- where topology and identity are not authoritative.

Risk: very high.

---

# 8. Preferred neural roles

The engine should prioritize learned components in the following order.

```text
1. diagnostics and relevance
2. warm start
3. preconditioner
4. refinement / LOD policy
5. bounded residual closure
6. constrained constitutive law
7. reduced-order surrogate
8. full neural simulation
```

This order maximizes useful deployment before the network is trusted with fundamental state transitions.

---

# 9. Runtime architecture

```text
                         PhysicalWorld Domain
                                  │
                                  ▼
                         Classical Discretization
                                  │
                         current residual/state
                                  │
                                  ▼
                    LearnedPhysicsAssistant
                                  │
          ┌───────────────────────┼────────────────────────┐
          │                       │                        │
     Warm Start              Closure/Residual        LOD Decision
          │                       │                        │
          └───────────────────────┼────────────────────────┘
                                  │
                                  ▼
                       Neural Proposal Package
                                  │
                                  ▼
                  Admissibility and OOD Gate
                                  │
             ┌────────────────────┴────────────────────┐
             │                                         │
           reject                                     accept
             │                                         │
             ▼                                         ▼
   Full Classical Solve                    Short Classical Correction
             │                                         │
             └────────────────────┬────────────────────┘
                                  │
                                  ▼
                       Invariant Validation
                                  │
               ┌──────────────────┴──────────────────┐
               │                                     │
             valid                                 invalid
               │                                     │
               ▼                                     ▼
            Commit                         Roll back + fallback
                                                     │
                                                     ▼
                                             Record hard case
```

---

# 10. Neural proposal package

A neural component should not return only a tensor.

It should return structured metadata.

```rust
pub struct NeuralProposal<T> {
    pub value: T,

    pub confidence: f32,
    pub support_distance: f32,

    pub predicted_residual: f32,
    pub model_id: ModelId,
    pub domain_signature: DomainSignature,
}
```

The engine may derive confidence externally rather than trusting a self-reported scalar.

---

# 11. Acceptance gate

The acceptance gate should evaluate several layers.

## 11.1. Finite-state checks

Reject:

- NaN;
- infinity;
- invalid indices;
- invalid topology;
- negative quantities that must remain positive.

---

## 11.2. Domain bounds

Check:

- density range;
- particle support;
- timestep range;
- deformation range;
- temperature range;
- material-parameter support;
- geometry class.

---

## 11.3. Local physical admissibility

Examples:

### Fluid

- bounded pressure;
- bounded divergence prediction;
- valid boundary relation.

### Solid/soil

- positive volume ratio;
- valid deformation determinant;
- stress inside or projectable to yield surface;
- non-negative dissipation.

### Rod/tree

- valid segment orientation;
- bounded strain;
- no topology mutation from the learned proposal.

---

## 11.4. Residual check

Evaluate the actual classical residual after inserting the proposal.

This is more trustworthy than prediction error alone.

---

## 11.5. Solver-safety check

For nonlinear systems, check conditions needed by the correction solver.

Examples:

- positive-definite or otherwise valid linearization;
- bounded condition estimate;
- admissible energy;
- valid volume change.

The 2026 spectrally safe warm-start results make this requirement explicit. [R13]

---

## 11.6. OOD check

Use multiple signals:

- parameter-range violation;
- geometry signature distance;
- latent feature distance;
- ensemble disagreement;
- residual mismatch;
- unexpected neighbor statistics;
- unseen material combination.

Residual failure should override learned confidence.

---

# 12. Correction and fallback policy

Suggested policy:

```text
if proposal is invalid:
    full classical solve

else if residual <= excellent_threshold:
    run minimal correction

else if residual <= acceptable_threshold:
    run shortened correction

else:
    discard proposal
    full classical solve
```

After correction:

```text
if invariants pass:
    commit
else:
    restore pre-step state
    full classical solve
```

The system MUST support rollback of the current substep.

---

# 13. Invariant validator

A common validator framework should exist across domains.

```rust
pub trait PhysicsValidator<State> {
    type Report;

    fn validate(
        &self,
        before: &State,
        after: &State,
        context: &ValidationContext,
    ) -> Self::Report;
}
```

A report may include:

```rust
pub struct PhysicsValidationReport {
    pub valid: bool,

    pub mass_error: Option<f64>,
    pub momentum_error: Option<f64>,
    pub angular_momentum_error: Option<f64>,
    pub energy_error: Option<f64>,

    pub constraint_residual: f64,
    pub boundary_residual: f64,

    pub violation_flags: ValidationFlags,
}
```

Not every domain uses every field.

---

# 14. The network must predict solver-native quantities

The safest targets are quantities already used by the classical solver.

Good targets:

- pressure multipliers;
- Lagrange multipliers;
- contact impulses;
- Newton increments;
- preconditioner coefficients;
- constitutive potential;
- residual correction;
- refinement scores.

Riskier targets:

- arbitrary next positions;
- arbitrary next velocities;
- direct transforms;
- fracture topology;
- material creation/deletion;
- identity changes.

The closer the output is to a solver-native intermediate, the easier it is to validate and correct.

---

# 15. Warm-start models

Warm starts are recommended as the first neural acceleration.

## 15.1. Fluid warm start

Input:

```text
particle positions
particle velocities
neighbor graph
boundary samples
density error
previous multipliers
dt
material parameters
```

Output:

```text
pressure/divergence multipliers
```

Then:

```text
neural initial guess
→ DFSPH / variational correction iterations
→ convergence
```

---

## 15.2. Rigid contact warm start

Input:

```text
contact graph
relative velocities
masses / inertias
friction
previous impulses
```

Output:

```text
normal and tangent impulse initial guesses
```

The sequential impulse or complementarity solver still enforces contact constraints.

---

## 15.3. Rod constraint warm start

Input:

```text
rod state
wind load
previous multipliers
branch graph
```

Output:

```text
constraint multiplier initial state
```

The rod solver performs final projection.

---

## 15.4. Nonlinear material warm start

Input:

```text
deformation
history variables
previous Newton state
material parameters
```

Output:

```text
initial local/global nonlinear state
```

This requires stricter energy and spectral checks.

---

# 16. Learned preconditioners

A network may predict:

- diagonal/block scaling;
- multigrid transfer parameters;
- coarse corrections;
- graph partitioning;
- local inverse approximations.

This can reduce solver iterations without directly predicting the final state.

The preconditioner itself must not invalidate the mathematical assumptions of the iterative solver.

---

# 17. Learned residual and closure models

A coarse solver omits unresolved scales.

The network can predict a correction:

\[
x_{high}
\approx
x_{coarse}
+
\Delta x_{\theta}
\]

or an unresolved force/stress:

\[
f =
f_{classical}
+
f_{\theta}
\]

The correction should be bounded.

Example:

```text
coarse fluid solve
→ learned subgrid stress
→ energy/momentum projection
→ corrected integration
```

---

# 18. Solver-in-the-loop training

One-step supervised training sees only teacher states.

At runtime, the model sees states created by its previous errors.

Solver-in-the-loop training addresses this:

```text
initial state
→ neural correction
→ solver step
→ neural correction
→ solver step
→ rollout loss
```

Advantages:

- exposure to self-induced state distributions;
- direct optimization of long-term behavior;
- better stability;
- physically meaningful multi-step loss.

The differentiable solver does not need to be the final Rust runtime implementation.

It may be a parallel research/training implementation, provided it matches the production equations and is continuously cross-validated.

---

# 19. Differentiable solver strategy

The project does not need to make the entire engine differentiable.

Recommended separation:

```text
Rust production solver
→ source of truth and runtime

training solver
→ differentiable mirror for selected kernels

cross-validation suite
→ verifies both implementations
```

Useful reference infrastructure includes differentiable SPH and MPM frameworks designed for hybrid learning and long gradient propagation. [R10] [R17]

Only the subsystem being learned needs a differentiable path.

---

# 20. Learned constitutive models

Constitutive laws map local history and deformation to material response.

Conceptually:

```text
deformation gradient
strain rate
pressure
temperature
saturation
history
      ↓
constitutive model
      ↓
stress
yield
dissipation
```

This is especially attractive for:

- mud;
- wet clay;
- complex snow;
- plant tissue;
- damaged wood;
- tire-soil interfaces;
- materials calibrated from experiments.

---

# 21. Hard constraints for constitutive learning

A learned constitutive model SHOULD embed or project to:

- frame indifference / objectivity;
- material symmetry;
- stress symmetry where required;
- positive dissipation;
- stable energy;
- valid yield response;
- bounded internal variables.

Do not rely only on adding a small penalty to MSE.

Prefer:

- invariant inputs;
- energy-potential parameterization;
- convexity or monotonicity by construction where applicable;
- explicit yield projection;
- thermodynamic architecture.

Research increasingly supports hard physics constraints for robust learned material response. [R12] [R15]

---

# 22. Temporal memory and state-space models

Many materials depend on history:

- plastic strain;
- fatigue;
- hysteresis;
- drying;
- combustion;
- channel damage;
- long-term soil remolding;
- magical corruption.

A temporal model can maintain a hidden state:

```text
current local state
+
hidden material memory
→ response
+
updated memory
```

Possible architectures:

- compact recurrent model;
- state-space model;
- Mamba-like temporal module;
- graph + state-space hybrid.

Spatial interactions should still be handled by graph/neighborhood operators or the classical solver.

An emerging 2026 direction combines graph interactions and state-space temporal updates in one latent simulator, but this remains research-grade. [R18]

---

# 23. Learned Physics LOD

The `RepresentationManager` can use neural models to predict where expensive physics is needed.

Input signals:

- velocity;
- stress;
- curvature;
- contact probability;
- camera/player relevance;
- vehicle trajectory;
- predicted fracture;
- thermal front;
- uncertainty;
- current residual.

Output:

```text
stay coarse
refine
promote representation
activate full solver
```

This can save more work than accelerating a single iteration.

---

# 24. Neural LOD may not be authoritative alone

Use:

```text
neural relevance
+
physical error estimator
+
hard interaction triggers
```

Policy:

```text
network says refine
→ refine

network says coarse
+
physical residual low
→ coarse

network says coarse
+
physical residual high
→ refine
```

The network may request more fidelity.

It should not be allowed to suppress mandatory refinement detected by physics.

---

# 25. Predictive activation

The network can promote a region before an event.

Examples:

## Water

Predict a likely splash zone before impact.

## Soil

Activate continuum terrain in the future wheel-contact corridor.

## Tree

Promote modal tree to rods before likely failure.

## Fire

Refine the thermal model before the combustion front reaches a boundary.

Predictive activation reduces visible representation transitions.

---

# 26. Reduced-order neural domains

A reduced-order model simulates fewer degrees of freedom and reconstructs a denser field.

Possible use:

```text
quiet water interior
distant mud region
background atmosphere
modal forest
regional heat
regional arcane ecology
```

Do not use reduced-order state where small-scale mechanics directly affect gameplay.

Example:

A visually reconstructed vortex must not be assumed to transfer physically correct force to a boat unless that interaction is included in the model and validator.

---

# 27. Promotion from surrogate to classical state

A surrogate must produce enough state to initialize the classical representation.

Required transfer may include:

- positions;
- velocities;
- pressure/multipliers;
- stress;
- material history;
- temperature;
- uncertainty;
- conservation correction.

State transfer is a first-class specification, not an implementation detail.

---

# 28. End-to-end neural simulation

A complete learned simulator can be useful when:

- the domain is narrow;
- geometries are bounded;
- materials are known;
- interactions are limited;
- topology does not change;
- fallback is cheap.

Examples:

- background sloshing in known containers;
- calm distant water;
- inactive thermal regions;
- cosmetic secondary motion.

It should not own:

- permanent fracture;
- authoritative contacts;
- mass creation;
- inventory/object identity;
- soul/identity state;
- network topology events.

---

# 29. Shared neural infrastructure, specialized models

Do not begin with one universal model for water, trees, soil, fire, and magic.

Recommended architecture:

```text
LearnedPhysicsPlatform
│
├── Shared inference runtime
├── Shared graph/particle encoding utilities
├── Shared model registry
├── Shared validator interface
├── Shared OOD infrastructure
├── Shared telemetry
│
├── WaterAccelerator
├── SoilAccelerator
├── RodAccelerator
├── ThermalAccelerator
├── ContactAccelerator
└── ArcanePlanner
```

A shared backbone may be researched later.

A 2026 unified particle transformer demonstrates one architecture across cloth, solids, fluids, granular materials, and molecular systems using prediction-correction, but specialized accuracy remains an important tradeoff. [R19]

---

# 30. Data-generation architecture

```text
Scenario Generator
       ↓
Reference / Teacher Solver
       ↓
Trajectory Recorder
       ↓
Validation and Filtering
       ↓
Versioned Dataset
       ↓
Training
       ↓
Offline Evaluation
       ↓
Model Registry
       ↓
Runtime Deployment
```

---

# 31. Scenario generation

Training scenes should be procedural and parameterized.

Vary:

- geometry;
- boundary motion;
- object count;
- material parameters;
- timestep;
- resolution;
- impulses;
- initial state;
- gravity;
- temperature;
- saturation;
- wind;
- damage;
- interaction sequence.

Do not train only on attractive showcase scenes.

---

# 32. Dataset split policy

Random frame splitting is unsafe because adjacent frames are nearly identical.

Use splits by:

- complete trajectory;
- geometry family;
- material family;
- parameter ranges;
- interaction type;
- resolution;
- scene generator seed.

Required evaluation groups:

```text
ID interpolation
near-OOD
geometry OOD
material OOD
resolution OOD
interaction OOD
long-rollout
adversarial/hard cases
```

---

# 33. Dataset record schema

Conceptual metadata:

```rust
struct PhysicsSampleHeader {
    solver_commit: CommitHash,
    solver_config_hash: ConfigHash,

    scenario_family: ScenarioFamilyId,
    scenario_seed: u64,

    units_schema: UnitsSchemaId,
    domain_schema: DomainSchemaId,

    dt: f64,
    substep_index: u32,

    resolution: ResolutionDescriptor,
    material_set: MaterialSetId,
}
```

Payload may contain:

- state before;
- solver initial residual;
- iteration trace;
- converged multipliers;
- final state;
- invariants;
- boundary state;
- timing;
- failure flags.

---

# 34. Record solver traces, not only final answers

Warm-start and preconditioner training benefits from:

```text
initial guess
residual at each iteration
correction directions
converged solution
iteration count
```

This allows the model to learn:

- good initial guesses;
- expected convergence;
- solver difficulty;
- fallback prediction.

---

# 35. Hard-case dataset

Maintain a separate dataset of:

- fallback events;
- validator failures;
- OOD states;
- solver divergence;
- rare topology transitions;
- extreme materials;
- unusually high iteration counts.

Do not let common calm frames dominate training.

---

# 36. Active-learning loop

```text
game / test execution
       ↓
proposal rejected or fallback used
       ↓
snapshot saved
       ↓
high-quality solver recomputes case
       ↓
case enters hard-example queue
       ↓
model retrained
       ↓
offline gate
       ↓
new deployment
```

This lets the model adapt to the actual game rather than only synthetic research scenes.

---

# 37. Teacher versioning

Every sample MUST be tied to:

- solver source revision;
- numerical configuration;
- material definitions;
- units;
- boundary implementation;
- bug-fix version.

If the teacher solver changes, old and new data must not be silently mixed.

Possible policies:

- dataset migration;
- model retraining;
- compatibility tags;
- separate teacher generations.

---

# 38. Model artifact requirements

A deployable model package should include:

```text
model weights
architecture identifier
input/output schema
normalization parameters
supported ranges
teacher solver version
dataset hashes
training code revision
validation thresholds
OOD calibration
precision requirements
runtime backend compatibility
```

A raw weight file is not a sufficient artifact.

---

# 39. Model registry

The engine needs a registry keyed by:

```text
domain
solver version
material set
resolution class
hardware profile
model role
```

Example:

```text
domain: continuum-water
role: dfsph-warm-start
solver: v0.7.3
resolution: 2cm–5cm
materials: water-v2
model: nws-water-014
```

---

# 40. Runtime telemetry

Record:

- model usage count;
- rejection rate;
- fallback rate;
- correction iterations;
- residual before/after;
- inference time;
- total step time;
- invariant violations;
- OOD score;
- scene family;
- hardware.

This is required to know whether the model is helping in production.

---

# 41. Honest performance measurement

Measure:

```text
graph / feature construction
buffer conversion
inference
validation
correction solve
fallback cost
state transfer
synchronization
memory
```

The relevant metric is not:

```text
network forward time
```

It is:

```text
end-to-end physics step time
```

---

# 42. Strong baseline requirement

Compare against:

- optimized production solver;
- same hardware;
- same scene;
- same physical tolerance;
- same output fidelity;
- same timestep;
- same boundary conditions.

Do not claim speedup relative only to:

- unoptimized Python;
- CPU teacher;
- excessive-accuracy solver;
- different resolution;
- weaker physical tolerance.

The weak-baseline problem is well documented in ML-for-PDE evaluation. [R16]

---

# 43. Latency metrics

Report:

- mean;
- median;
- p95;
- p99;
- worst bounded case;
- frame-time spikes;
- fallback spike cost.

Games are sensitive to tail latency.

A model that improves average time but causes severe fallback spikes may be unacceptable.

---

# 44. Accuracy metrics must be domain-specific

Do not rely only on RMSE.

## Water

- mass error;
- density error;
- divergence;
- free-surface displacement;
- pressure impulse;
- rigid-body trajectory;
- rollout stability.

## Soil

- force-slip curve;
- rut depth;
- sinkage;
- yield residual;
- plastic work;
- terrain history.

## Trees

- natural frequencies;
- branch-tip trajectory;
- stress;
- failure time;
- fracture location;
- topology.

## Fire

- energy balance;
- ignition time;
- burn rate;
- char depth;
- fuel consumption;
- spread front.

## Rigid contacts

- penetration;
- restitution;
- friction impulse;
- stack stability;
- momentum.

A 2026 comparison of flow surrogates emphasizes that no single architecture or generic pointwise metric reliably captures all process-relevant behavior. [R20]

---

# 45. Training-cost amortization

A neural accelerator has an upfront cost.

Total lifecycle cost:

\[
T_{\text{hybrid,total}}
=
T_{\text{data}}
+
T_{\text{training}}
+
N
\left(
T_{\text{inference}}
+
T_{\text{validation}}
+
T_{\text{correction}}
+
T_{\text{fallback}}
\right)
\]

Classical cost:

\[
T_{\text{classical,total}}
=
N T_{\text{classical}}
\]

The model only pays off after a break-even number of simulation steps.

The project should estimate this explicitly.

---

# 46. GPU integration principles

Neural inference can lose all benefit through memory movement.

Recommended:

- read solver-native GPU buffers directly;
- avoid CPU readback;
- batch many regions/trees;
- reuse neighborhood structures when valid;
- avoid rebuilding a second graph if the solver already has one;
- use compact output heads;
- keep validation on GPU;
- use asynchronous scheduling only where dependencies permit;
- measure synchronization cost.

---

# 47. Graph reuse

Particle and contact solvers already construct graphs/neighborhoods.

The learned model SHOULD consume the same topology where possible.

Avoid:

```text
solver builds neighbor list
+
ML builds another neighbor list
```

unless the learned topology is demonstrably necessary.

---

# 48. Precision strategy

Potential deployment:

```text
network inference:
f16 / bf16 / mixed

classical correction:
f32

reference validation:
f64 where practical
```

But reduced precision is acceptable only if:

- validation remains reliable;
- residual estimates do not underflow;
- physical thresholds remain meaningful.

---

# 49. Neural acceleration for water

Recommended progression:

## Stage W1 — DFSPH warm start

Predict:

- divergence multipliers;
- density multipliers;
- pressure state.

Classical DFSPH completes convergence.

## Stage W2 — refinement predictor

Predict:

- likely splash;
- high curvature;
- collision region;
- thin sheet;
- air entrainment.

## Stage W3 — learned subgrid closure

Predict bounded:

- vorticity correction;
- unresolved stress;
- viscosity correction.

## Stage W4 — reduced-order quiet regions

Use sparse simulation for distant/calm water.

## Stage W5 — experimental surrogate

Only in restricted scene classes.

---

# 50. Water invariants

The neural component MUST NOT silently change:

- particle mass;
- total material quantity;
- phase identity;
- boundary topology.

Required checks:

```text
density
divergence
mass
volume
energy bound
boundary residual
rigid impulse balance
```

---

# 51. Neural acceleration for mud, soil, sand, and snow

Highest-value roles:

- learned constitutive response;
- warm-start of plastic/pressure solve;
- refinement around tires;
- prediction of active deformation corridors;
- history-dependent material memory;
- reduced-order sleeping regions.

---

# 52. Soil constitutive model

Input may include:

```text
deformation gradient
strain rate
pressure
porosity
saturation
pore pressure
plastic history
temperature
```

Output:

```text
stress
yield state
dissipation
updated internal state
```

The model should use invariant features and hard admissibility constraints.

---

# 53. Tire-soil learning

Possible safe use:

```text
classical terrain state
+
tire contact state
→ learned local constitutive correction
→ classical force and momentum solve
```

Do not train a black-box function:

```text
vehicle input
→ vehicle movement
```

if the purpose is a reusable physical world.

The local model preserves interaction with arbitrary vehicles and terrain history.

---

# 54. Neural acceleration for trees

Modal dynamics already make most trees cheap.

Useful neural roles:

- modal coefficient prediction;
- modal-to-rod state initialization;
- rod warm start;
- failure-risk prediction;
- representation promotion;
- local damage estimation;
- wind-load approximation.

---

# 55. Tree topology remains classical

The neural model should not authoritatively decide:

- branch graph split;
- tree identity;
- permanent missing branches;
- root detachment.

It may predict risk or candidate fracture location.

The structural solver validates and commits topology changes.

---

# 56. Neural acceleration for fire and thermal systems

Possible roles:

- subgrid convection;
- turbulent heat transport;
- ember-generation rate;
- local heat-transfer coefficient;
- ignition probability estimator;
- representation refinement.

Hard state:

- total energy;
- fuel mass;
- moisture mass;
- char/ash fractions;
- material identity.

---

# 57. Fire closures

A learned fire closure should output bounded fluxes or rates.

Example:

```text
coarse thermal field
+
wind
+
fuel geometry
→ predicted unresolved heat flux
```

Then:

```text
energy projection
+
non-negative species update
+
classical integration
```

---

# 58. Neural acceleration for atmosphere

Potential roles:

- local wind correction around obstacles;
- turbulence closure;
- downscaling;
- wake approximation;
- predictive gust field.

The global weather/atmosphere state should remain a deterministic domain or externally specified field.

---

# 59. Neural acceleration for rigid bodies

Potential roles:

- contact impulse warm start;
- broad-phase relevance;
- contact graph ordering;
- preconditioner;
- sleeping prediction;
- soft contact approximation.

The model should never write body transforms directly in authoritative physics.

---

# 60. Neural acceleration for articulated bodies

Distinguish physics from control.

Neural motor policies may produce:

- target joint positions;
- PD targets;
- torques;
- muscle activations.

But the articulated solver remains the physical authority.

Physics acceleration may separately learn:

- contact warm starts;
- joint-constraint multipliers;
- soft-tissue approximations.

---

# 61. Neural acceleration for the arcane layer

The arcane world's conservation and identity rules should remain deterministic.

Good ML roles:

- intent-to-spell planning;
- effect-graph optimization;
- field-control approximation;
- artifact routing;
- mana-network scheduling;
- ecological forecasting;
- representation relevance.

The runtime still validates:

- mana source;
- throughput;
- energy conversion;
- coherence;
- identity permissions;
- physical coupler constraints.

---

# 62. Topology and identity firewall

The following events require authoritative non-neural validation:

```text
fracture
object creation/destruction
material phase identity change
persistent cut
root detachment
inventory transfer
entity death
soul/identity change
teleport topology
world ownership
```

A neural model may propose.

It may not silently commit.

---

# 63. Networking

Recommended multiplayer model:

```text
server
→ authoritative physical and topology events

clients
→ local hybrid acceleration and secondary motion
```

Replicate:

- accepted force/impact outcomes;
- fracture events;
- representation state where required;
- persistent material changes;
- model-independent semantic events.

Do not require all clients to produce bitwise-identical neural rollouts.

---

# 64. Model mismatch in multiplayer

Clients may run:

- different GPU inference backends;
- slightly different floating-point kernels;
- different quality tiers.

Therefore server authority should be based on:

- classical/hybrid validated result;
- topology events;
- periodic state corrections.

Cosmetic field detail can differ.

---

# 65. Savegames and model upgrades

Persistent world state MUST NOT depend on opaque neural hidden state unless that state has a versioned migration strategy.

Prefer saving physical state:

- particles/compact region state;
- stress/history;
- damage;
- temperature;
- cuts;
- reservoirs;
- topology.

On load, a new model reconstructs accelerator state from physical state.

---

# 66. Failure modes

## 66.1. Rollout drift

Mitigation:

- correction solve;
- rollout training;
- periodic projection;
- fallback.

## 66.2. Distribution shift

Mitigation:

- explicit supported ranges;
- OOD gates;
- geometry/material split tests;
- active learning.

## 66.3. False confidence

Mitigation:

- classical residual as primary signal;
- ensembles or calibration as secondary;
- hard domain checks.

## 66.4. Inference slower than solver

Mitigation:

- use narrow models;
- reuse graph;
- batch;
- measure end-to-end;
- disable model when not profitable.

## 66.5. Warm start hurts convergence

Mitigation:

- admissibility gate;
- compare initial residual;
- spectral/energy safety;
- fall back to classical initial guess.

## 66.6. Learned closure injects energy

Mitigation:

- potential-based parameterization;
- energy projection;
- bounded output;
- dissipation constraints.

## 66.7. Model/solver version drift

Mitigation:

- strict model compatibility;
- registry;
- dataset hashes;
- automatic disable on mismatch.

## 66.8. Hidden topology corruption

Mitigation:

- topology firewall;
- authoritative event processor;
- graph validity checks.

---

# 67. Recommended implementation roadmap

## Phase 0 — common physics diagnostics

Before ML:

- invariant reports;
- residual APIs;
- rollback;
- deterministic scenario runner;
- benchmark harness;
- state snapshots;
- profiler.

---

## Phase 1 — reference continuum solver

Implement:

- small CPU `f64` water solver;
- deterministic test scenes;
- complete diagnostics.

This serves as the first teacher and oracle.

---

## Phase 2 — classical GPU continuum baseline

Implement:

- GPU particle state;
- spatial hash;
- neighbor search;
- DFSPH;
- boundaries;
- rigid coupling;
- timing breakdown.

No neural dependency.

---

## Phase 3 — dataset recorder

Record:

- solver-native features;
- iteration traces;
- residuals;
- final multipliers;
- invariants;
- scenario metadata.

---

## Phase 4 — first warm-start model

Target:

```text
DFSPH pressure/divergence multipliers
```

The model should be intentionally small.

---

## Phase 5 — runtime gate and fallback

Implement:

- OOD checks;
- residual checks;
- correction iterations;
- rollback;
- full fallback;
- telemetry.

This phase is mandatory before claiming success.

---

## Phase 6 — active hard-case collection

Use automated tests and gameplay simulations to build the hard-case dataset.

---

## Phase 7 — learned representation policy

Predict:

- water refinement;
- terrain activation;
- tree promotion;
- thermal refinement.

Physics can override the policy.

---

## Phase 8 — constrained soil constitutive model

Use classical kinematics and solve with a learned local material law.

---

## Phase 9 — solver-in-the-loop closures

Begin with one bounded correction:

- water subgrid stress;
- soil shear correction;
- thermal flux.

---

## Phase 10 — reduced-order calm regions

Use a surrogate only in a tightly defined regime.

Implement bidirectional state transfer.

---

## Phase 11 — tree and thermal accelerators

Add:

- modal/rod initialization;
- fire closure;
- predictive activation.

---

## Phase 12 — experimental universal models

Research shared particle encoders or unified latent dynamics only after specialized systems are working.

---

# 68. First concrete milestone

## Milestone: `Neural-Assisted DFSPH Warm Start`

### Classical foundation

- CPU `f64` reference;
- optimized GPU DFSPH;
- fixed-resolution water;
- static and moving boundaries;
- rigid-body interaction;
- complete diagnostics.

### Dataset

- 20–50 parameterized scenario families;
- multiple resolutions;
- varied geometry;
- calm and violent states;
- rare failure cases;
- complete iteration traces.

### Model

- graph or particle-neighborhood encoder;
- per-particle multiplier prediction;
- previous timestep solution input;
- compact inference budget.

### Runtime

```text
neural prediction
→ admissibility gate
→ residual check
→ corrective DFSPH
→ invariant validation
→ commit/fallback
```

### Success metrics

- lower end-to-end p50 and p99 physics time;
- fewer correction iterations;
- no meaningful degradation of mass/density/divergence;
- stable long rollouts;
- bounded fallback rate;
- no frame-time regression from graph/inference overhead;
- reproducible performance on OOD test suites.

---

# 69. Candidate engineering targets

These are starting hypotheses, not normative guarantees.

For the first supported scene class, investigate:

- 20–50% reduction in the target solver stage;
- no p99 regression;
- identical classical convergence tolerance;
- fallback below a defined operational threshold;
- model memory small relative to domain buffers;
- inference cheaper than saved iterations.

The final targets should be set after profiling the classical GPU solver.

---

# 70. Rust architecture proposal

```text
crates/
├── learned-physics-core
│   ├── model_id
│   ├── schemas
│   ├── domain_signature
│   ├── proposal
│   ├── precision
│   └── errors
│
├── learned-physics-runtime
│   ├── inference
│   ├── batching
│   ├── gpu_interop
│   ├── scheduling
│   └── model_registry
│
├── learned-physics-validation
│   ├── residual
│   ├── invariants
│   ├── admissibility
│   ├── ood
│   ├── spectral_safety
│   └── rollback
│
├── learned-physics-data
│   ├── recorder
│   ├── scenario_metadata
│   ├── trajectory
│   ├── hard_cases
│   └── dataset_version
│
├── learned-physics-telemetry
│   ├── timing
│   ├── fallback
│   ├── residual_stats
│   └── model_health
│
├── learned-water
│   ├── dfsph_warm_start
│   ├── refinement
│   ├── closure
│   └── reduced_order
│
├── learned-soil
│   ├── constitutive
│   ├── refinement
│   └── history
│
├── learned-vegetation
│   ├── modal
│   ├── rod_warm_start
│   └── failure_risk
│
├── learned-thermal
│   ├── closure
│   └── refinement
│
├── learned-contact
│   ├── impulse_warm_start
│   └── ordering
│
└── learned-arcane
    ├── spell_planner
    ├── field_control
    └── ecology_forecast
```

Initially, keep these as modules rather than immediately creating all crates.

---

# 71. Suggested core interfaces

```rust
pub trait LearnedAccelerator<State, Context> {
    type Proposal;

    fn supports(
        &self,
        state: &State,
        context: &Context,
    ) -> SupportReport;

    fn propose(
        &mut self,
        state: &State,
        context: &Context,
    ) -> Result<Self::Proposal, InferenceError>;
}
```

```rust
pub trait ProposalGate<State, Context, Proposal> {
    type Report;

    fn evaluate(
        &self,
        state: &State,
        context: &Context,
        proposal: &Proposal,
    ) -> Self::Report;
}
```

```rust
pub trait CorrectiveSolver<State, Context, Proposal> {
    type Diagnostics;

    fn correct(
        &mut self,
        state: &mut State,
        context: &Context,
        proposal: Option<&Proposal>,
    ) -> Self::Diagnostics;
}
```

---

# 72. Hybrid step result

```rust
pub struct HybridStepDiagnostics {
    pub model_id: Option<ModelId>,

    pub proposal_accepted: bool,
    pub fallback_used: bool,

    pub residual_before: f64,
    pub residual_after_proposal: Option<f64>,
    pub residual_final: f64,

    pub correction_iterations: u32,

    pub inference_time_ns: u64,
    pub validation_time_ns: u64,
    pub correction_time_ns: u64,
    pub total_time_ns: u64,

    pub validation: PhysicsValidationReport,
}
```

---

# 73. Simulation scheduler integration

Neural tasks are nodes in the existing dependency graph.

Example:

```text
build neighbor graph
        ↓
extract features
        ↓
run warm-start inference
        ↓
gate proposal
        ↓
classical correction
        ↓
validate
```

The scheduler should be able to disable the neural node and use the same classical path.

---

# 74. Feature normalization and units

Physical input should use explicit units or dimensionless groups.

Avoid model dependence on arbitrary world-unit scale.

Possible strategies:

- SI units internally;
- normalized local frames;
- dimensionless quantities;
- material-relative scaling.

The model artifact MUST identify its unit schema.

---

# 75. Equivariance and symmetry

Where practical, model architecture should respect:

- translation;
- rotation;
- permutation;
- material symmetries.

Examples:

- particle graph is permutation-invariant;
- vector outputs rotate correctly;
- constitutive model uses invariants.

This improves data efficiency and generalization.

---

# 76. Training objectives

Possible combined loss:

\[
\mathcal{L}
=
w_t \mathcal{L}_{target}
+
w_r \mathcal{L}_{residual}
+
w_i \mathcal{L}_{invariant}
+
w_e \mathcal{L}_{energy}
+
w_s \mathcal{L}_{spectral}
+
w_o \mathcal{L}_{rollout}
+
w_u \mathcal{L}_{uncertainty}
\]

Not every model uses every term.

The runtime validator remains necessary even if the same property appears in the loss.

---

# 77. Curriculum

Suggested training progression:

```text
simple geometry
→ moving boundaries
→ varied resolution
→ high energy
→ coupling
→ topology-adjacent cases
→ OOD hard cases
```

Do not begin with the entire world distribution.

---

# 78. Model specialization

It may be better to maintain:

- water calm model;
- water impact model;
- soil dry model;
- soil saturated model;

than one large network.

A gate can select the appropriate specialist.

However, specialist boundaries must be explicit and testable.

---

# 79. Model ensembles and uncertainty

Possible uncertainty tools:

- small ensemble;
- multiple output heads;
- latent-distance model;
- conformal calibration;
- temporal inconsistency;
- disagreement with coarse physics.

Uncertainty is a routing aid.

It is not a substitute for residual validation.

---

# 80. Online learning policy

Do not update production model weights silently during gameplay.

Recommended:

```text
runtime collects hard cases
→ offline retraining
→ reproducible validation
→ signed/versioned deployment
```

Test-time adaptation may be researched later, but uncontrolled online updates make debugging and multiplayer consistency difficult.

---

# 81. Security and robustness

Treat model files and schemas as executable engine assets.

Validate:

- dimensions;
- hashes;
- supported solver version;
- numeric ranges;
- memory requirements;
- output bounds.

A malformed or incompatible model should disable acceleration, not crash physics.

---

# 82. Debug visualization

Visualize:

- neural-supported regions;
- accepted/rejected proposals;
- OOD score;
- residual heatmap;
- correction magnitude;
- fallback regions;
- saved iterations;
- model version;
- physical violations.

For particles:

```text
color by predicted pressure error
color by uncertainty
color by fallback
```

For trees:

```text
color by promotion probability
color by failure risk
```

---

# 83. Regression suite

Every model release should pass:

- deterministic unit tests;
- classical solver regressions;
- ID benchmark;
- OOD benchmark;
- long rollout;
- stress scenes;
- performance suite;
- model-disabled equivalence path;
- save/load;
- multiplayer event consistency.

---

# 84. Acceptance criteria for a model release

A model can ship only if:

- supported domain is documented;
- all hard physical checks pass;
- fallback works;
- no regression when model is disabled;
- p99 latency is acceptable;
- memory budget is acceptable;
- performance is measured end-to-end;
- OOD suite is reported;
- model/solver compatibility is pinned.

---

# 85. Rollback and canary deployment

The engine should support:

- instant model disable;
- previous model version;
- per-domain rollout;
- per-hardware rollout;
- telemetry comparison;
- automatic rollback on violation/fallback spikes.

This is ordinary production engineering, not optional ML tooling.

---

# 86. What not to do

Do not:

1. train a model before defining invariants;
2. compare only against a slow CPU solver;
3. predict the entire world state as the first experiment;
4. trust low visual/MSE error as proof of solver safety;
5. allow the network to commit topology changes directly;
6. save opaque neural hidden state as the only world state;
7. rebuild expensive ML graphs unnecessarily;
8. ignore inference and validation overhead;
9. train and test on adjacent frames from the same trajectory;
10. let fallback paths rot from lack of use;
11. assume one universal model will outperform specialized solvers;
12. remove the classical solver after deployment.

---

# 87. Production and research branches

## Production branch

```text
classical solver
+
warm starts
+
bounded closures
+
learned LOD
+
fallback
```

## Research branch

```text
reduced-order neural domains
+
shared particle foundation models
+
graph-state-space simulators
+
end-to-end surrogates
+
test-time adaptation
```

Research models should not block production physics.

---

# 88. Candidate ADRs

1. `ADR-Deterministic-Physics-Foundation.md`
2. `ADR-Physics-Solver-Hierarchy.md`
3. `ADR-Learned-Physics-Trust-Tiers.md`
4. `ADR-Neural-Proposal-Gate.md`
5. `ADR-Classical-Fallback.md`
6. `ADR-Physics-Invariant-Validation.md`
7. `ADR-Physics-Dataset-Versioning.md`
8. `ADR-Teacher-Solver-Versioning.md`
9. `ADR-Model-Registry.md`
10. `ADR-OOD-Detection.md`
11. `ADR-Solver-In-The-Loop-Training.md`
12. `ADR-Learned-Constitutive-Models.md`
13. `ADR-Neural-Representation-Policy.md`
14. `ADR-Topology-Identity-Firewall.md`
15. `ADR-Hybrid-Physics-Networking.md`
16. `ADR-Hybrid-Physics-Persistence.md`
17. `ADR-End-To-End-Performance-Evaluation.md`
18. `ADR-Neural-Physics-Online-Learning-Policy.md`

---

# 89. Candidate specifications

- `SPEC-Reference-Physics-Solver.md`
- `SPEC-Classical-GPU-Baseline.md`
- `SPEC-Physics-Diagnostics.md`
- `SPEC-Physics-State-Snapshot.md`
- `SPEC-Physics-Rollback.md`
- `SPEC-Physics-Dataset-Format.md`
- `SPEC-Scenario-Generator.md`
- `SPEC-Hard-Case-Recorder.md`
- `SPEC-Model-Artifact-Format.md`
- `SPEC-Learned-Physics-Runtime.md`
- `SPEC-Neural-Proposal.md`
- `SPEC-Proposal-Admissibility-Gate.md`
- `SPEC-Physical-Residual-API.md`
- `SPEC-OOD-Gate.md`
- `SPEC-Invariant-Validator.md`
- `SPEC-Classical-Fallback.md`
- `SPEC-Hybrid-Step-Telemetry.md`
- `SPEC-Model-Solver-Compatibility.md`
- `SPEC-DFSPH-Warm-Start-Model.md`
- `SPEC-DFSPH-Hybrid-Runtime.md`
- `SPEC-Learned-Refinement-Policy.md`
- `SPEC-Learned-Soil-Constitutive-Law.md`
- `SPEC-Learned-Thermal-Closure.md`
- `SPEC-Modal-Rod-Neural-Transition.md`
- `SPEC-Contact-Impulse-Warm-Start.md`
- `SPEC-Reduced-Order-Domain-Promotion.md`
- `SPEC-Hybrid-Physics-Benchmarking.md`
- `SPEC-Hybrid-Physics-Multiplayer.md`

---

# 90. Candidate research spikes

- `SPIKE-DFSPH-Neural-Warm-Start.md`
- `SPIKE-Neural-Preconditioner.md`
- `SPIKE-Spectrally-Safe-Warm-Start.md`
- `SPIKE-Solver-In-The-Loop-SPH.md`
- `SPIKE-Particle-OOD-Detection.md`
- `SPIKE-Learned-Refinement.md`
- `SPIKE-Constrained-Soil-Constitutive-Model.md`
- `SPIKE-Graph-Mamba-Material-History.md`
- `SPIKE-Water-Reduced-Order-Model.md`
- `SPIKE-Contact-Impulse-Prediction.md`
- `SPIKE-Tree-Failure-Risk.md`
- `SPIKE-Thermal-Subgrid-Closure.md`
- `SPIKE-Unified-Particle-Backbone.md`
- `SPIKE-Hybrid-GPU-Buffer-Interop.md`
- `SPIKE-Physics-Model-Quantization.md`

---

# 91. Normative requirements

## MUST

- Classical physics MUST remain available as a complete supported backend.
- A trusted reference or teacher solver MUST exist for each learned physical subsystem.
- Neural output MUST pass finite-state and domain-admissibility checks.
- Solver-native residuals MUST be measured before accepting a warm start.
- Hard physical invariants MUST be checked after correction.
- Failed validation MUST trigger rollback and classical fallback.
- Topology and identity changes MUST require authoritative non-neural validation.
- Models MUST be versioned against solver, schema, units, materials, and datasets.
- Performance MUST be measured end-to-end against an optimized classical baseline.
- OOD and hard-case evaluation MUST be separate from random held-out validation.
- Runtime MUST support disabling every learned component.
- Persistent world state MUST remain interpretable without a specific neural model.

## SHOULD

- The first learned accelerator SHOULD be a warm start or preconditioner.
- Models SHOULD predict solver-native intermediate quantities.
- The engine SHOULD reuse existing solver graphs and GPU buffers.
- Learned closures SHOULD be bounded and projected.
- Learned constitutive models SHOULD enforce objectivity and thermodynamic admissibility by construction.
- The engine SHOULD collect fallback cases for offline active learning.
- Representation decisions SHOULD combine neural predictions with physical error estimators.
- Networked games SHOULD replicate authoritative physical events rather than neural hidden state.
- Model deployment SHOULD support rollback and canary evaluation.
- Tail latency SHOULD be treated as a first-class metric.

## MAY

- Reduced-order neural domains MAY serve calm or distant regions.
- State-space/Mamba-like models MAY represent long material history.
- Shared particle encoders MAY be researched after specialized models.
- Test-time adaptation MAY be explored behind an experimental flag.
- End-to-end neural simulation MAY exist as a non-authoritative experimental backend.

---

# 92. Updated PhysicalWorld architecture

```text
                         WorldDynamics
                               │
                    PhysicalWorld Scheduler
                               │
       ┌───────────────────────┼────────────────────────┐
       │                       │                        │
 Specialized Solvers    RepresentationManager   CouplingGraph
       │                       │                        │
       └───────────────────────┼────────────────────────┘
                               │
                               ▼
                    LearnedPhysicsPlatform
                               │
      ┌────────────────────────┼──────────────────────────┐
      │                        │                          │
 Warm Starts             Closures/Materials        Relevance/LOD
      │                        │                          │
      └────────────────────────┼──────────────────────────┘
                               │
                               ▼
                    Validation + OOD + Fallback
                               │
                               ▼
                       Authoritative State
```

The neural layer assists the world.

It does not replace ownership of the world.

---

# 93. Final implementation order

```text
1. laws and invariants
2. CPU reference solver
3. optimized classical GPU solver
4. profiler and diagnostics
5. state snapshots and rollback
6. dataset recorder
7. narrow warm-start model
8. admissibility and OOD gate
9. classical correction and fallback
10. active hard-case loop
11. learned Physics LOD
12. bounded closure
13. constrained constitutive models
14. reduced-order domains
15. experimental shared/universal models
```

---

# 94. Project thesis

> **Build the physical world as a deterministic, validated system of specialized solvers, then add neural networks as narrow, measurable accelerators that propose initial guesses, closures, constitutive responses, and representation decisions. Every proposal is checked against solver residuals and physical invariants, corrected by classical numerics, and rejected in favor of a full fallback whenever it leaves the supported physical domain.**

---

# 95. Final recommendation

The first practical research project should be:

```text
Neural-Assisted DFSPH Warm Start
```

It is the correct initial target because:

- the iterative bottleneck is isolated;
- the target is solver-native;
- validation is clear;
- the classical solver remains authoritative;
- failure is recoverable;
- the same infrastructure generalizes to mud and other continuum materials;
- speedup can be measured honestly.

After this succeeds, the same platform can support:

```text
learned soil response
learned representation promotion
learned thermal closure
rod/contact warm starts
reduced-order quiet regions
arcane planning and field control
```

The intended end state is not a neural physics engine replacing equations.

It is a physical engine whose expensive computations are increasingly anticipated, initialized, compressed, and focused by learned models while deterministic laws remain the final arbiter.

---

# References

## [R1] Graph Network-based Simulators

A. Sanchez-Gonzalez, J. Godwin, T. Pfaff, R. Ying, J. Leskovec, P. W. Battaglia.  
**Learning to Simulate Complex Physics with Graph Networks.** 2020.  
https://arxiv.org/abs/2002.09405

## [R2] MeshGraphNets

T. Pfaff, M. Fortunato, A. Sanchez-Gonzalez, P. W. Battaglia.  
**Learning Mesh-Based Simulation with Graph Networks.** 2020.  
https://arxiv.org/abs/2010.03409

## [R3] Solver-in-the-Loop

K. Um, R. Brand, Y. Fei, P. Holl, N. Thuerey.  
**Solver-in-the-Loop: Learning from Differentiable Physics to Interact with Iterative PDE-Solvers.** 2020.  
https://arxiv.org/abs/2007.00016

## [R4] NeuralMPM

**A Neural Material Point Method for Particle-based Emulation.** 2024.  
https://arxiv.org/abs/2408.15753

## [R5] JAX-SPH

**JAX-SPH: A Differentiable Smoothed Particle Hydrodynamics Framework.** 2024.  
https://arxiv.org/abs/2403.04750

## [R6] Neural SPH

A. P. Toshev, J. A. Erbesdobler, N. A. Adams, J. Brandstetter.  
**Neural SPH: Improved Neural Modeling of Lagrangian Fluid Dynamics.** 2024.  
https://arxiv.org/abs/2402.06275

## [R7] GIOROM

H. Viswanath, Y. Chang, J. Berner, P. Y. Chen, A. Bera.  
**Reduced-Order Neural Operators: Learning Lagrangian Dynamics on Highly Sparse Graphs.** 2024–2026 revisions.  
https://arxiv.org/abs/2407.03925

## [R8] Weak Baselines in ML-for-PDE

C. McGreivy, A. Hakim.  
**Weak baselines and reporting biases lead to overoptimism in machine learning for fluid-related partial differential equations.** 2024.  
https://arxiv.org/abs/2407.07218

## [R9] Hybrid Neural-MPM

J. Xu, H. Huang, C. Zou, M. Savva, Y. Wei, W. Chen.  
**Hybrid Neural-MPM for Interactive Fluid Simulations in Real-Time.** 2025.  
https://arxiv.org/abs/2505.18926

## [R10] diffSPH

R. Winchenbach, N. Thuerey.  
**diffSPH: Differentiable Smoothed Particle Hydrodynamics for Adjoint Optimization and Machine Learning.** 2025.  
https://arxiv.org/abs/2507.21684

## [R11] NOWS

M. S. Eshaghi et al.  
**Neural Operator Warm Starts for Accelerating Iterative Solvers.** 2025.  
https://arxiv.org/abs/2511.02481

## [R12] UniPhy

H. Mittal et al.  
**UniPhy: Learning a Unified Constitutive Model for Inverse Physics Simulation.** 2025.  
https://arxiv.org/abs/2505.16971

## [R13] Spectrally Safe Warm Starts

J. Oh, Y. Lee, J. Darbon, G. E. Karniadakis.  
**Spectrally Safe Neural Operator Warm-Starts for Large-Scale Newton Solvers.** 2026.  
https://arxiv.org/abs/2606.21828

## [R14] SIMSHIFT

P. Setinek et al.  
**SIMSHIFT: A Benchmark for Adapting Neural Surrogates to Distribution Shifts.** 2025.  
https://arxiv.org/abs/2506.12007

## [R15] Thermodynamics-Constrained Constitutive Learning

**Learning inelastic constitutive models from stress-strain data using thermodynamics-constrained neural networks.** 2026.  
https://arxiv.org/abs/2605.16837

## [R16] Review of ML-for-PDE Evaluation

See [R8], and the 2026 review:  
**Partial Differential Equations in the Age of Machine Learning.**  
https://arxiv.org/abs/2603.07655

## [R17] JAX-MPM

**JAX-MPM: A Learning-Augmented Differentiable Meshfree Simulator.** 2025.  
https://arxiv.org/abs/2507.04192

## [R18] Graph Mamba Operator

**Graph Mamba Operator: A Latent Simulator for Interacting Dynamical Systems.** 2026.  
https://arxiv.org/abs/2606.09432

## [R19] Unified Particle Transformer

**Unified Simulation of Lagrangian Particle Dynamics via a Single Transformer Architecture.** 2026.  
https://arxiv.org/abs/2605.15305

## [R20] Flow Surrogate Evaluation

**No Free Lunch in Flow Surrogates under Time-Varying Operating Conditions.** 2026.  
https://arxiv.org/abs/2607.23667
