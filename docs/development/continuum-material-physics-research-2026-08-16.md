# Continuum material physics — research report, 2026-08-16

## Question and method

The source brief asks for one particle-first continuum system spanning water,
sand, mud, soil, snow and off-road coupling. This report treats the brief as a
research hypothesis, not as repository instructions. It compares that
hypothesis with current Next Engine authority/persistence constraints and
primary technical sources. The resulting architecture is recorded as
[SPEC-38](../architecture/38-continuum-material-physics.md) and
[ADR-076](../architecture/adr/076-continuum-material-physics-track.md).

Input reviewed in full: `/home/kaifaty/Downloads/continuum_physics_water_mud_offroad_research_brief.md`,
SHA-256 `d2f8bb9a6a41a62a2501841b3897aa35a86f878a6b43d8a9f835b88aec9144d1`.

## Executive conclusion

Keep the brief's strongest invariant: durable material state should be carried
by Lagrangian material samples or explicit region records, while spatial
hashes, pressure systems, grids, meshes and GPU buffers are temporary. Reject
its implied solver monoculture: DFSPH is the right first water oracle, but
APIC/MLS-MPM is the stronger initial substrate for elastoplastic sand, snow and
deformable soil.

The research result originally identified this smallest safe capability
sequence:

1. fixed-resolution CPU `f64` DFSPH laboratory and golden corpus;
2. boundaries plus one two-way rigid coupling vertical;
3. read-only presentation;
4. GPU correspondence mirror, not authority;
5. APIC/MLS-MPM dry terrain with Drucker-Prager plasticity;
6. active-region persistence;
7. saturation-coupled wet soil/mud;
8. only then a production-consumer proposal and public schemas.

The subsequent product decision selected a sealed 50k basin/crate consumer and
refined execution into the independent
[water roadmap](../plans/continuum-water/README.md). That roadmap makes the GPU
mirror non-blocking, requires exact active water persistence before promotion
and activates the main R8 integration track only after the serial CPU oracle
passes. The umbrella
[material series](../plans/continuum-material-physics/README.md) now keeps dry
sand, exact terrain persistence, saturation, free-water flux, lossy sleep and
cross-region transfer as explicit separate gates. This refinement supersedes
the linear execution order above but not the research evidence or solver-family
conclusion.

## Evidence matrix

| Question | Evidence | Bounded conclusion |
|---|---|---|
| Is DFSPH a credible water baseline? | Bender and Koschier's [Divergence-Free SPH](https://animation.rwth-aachen.de/media/papers/2015-SCA-DFSPH.pdf) solves density and velocity-divergence constraints; [SPlisHSPlasH](https://splishsplash.readthedocs.io/en/latest/file_format.html) exposes reproducible solver/tolerance profiles. | Yes for a reference solver and oracle. The paper does not establish the engine's real-time budget. |
| How should rigid boundaries couple? | Akinci et al. describe [versatile rigid-fluid coupling](https://cgl.ethz.ch/publications/papers/paperSol12.php) with momentum transfer; [density maps](https://discovery.ucl.ac.uk/id/eprint/10056696) and [MLS pressure boundaries](https://cg.informatik.uni-freiburg.de/publications/2018_CAG_localPressureBoundaries.pdf) improve curved-boundary treatment. | Start with analytical boundaries, then validate one sampled/density-map path. Reduce reactions to an engine-owned impulse batch. |
| Is adaptive split/merge ready for phase one? | Adaptive SPH work reports mass-conserving refinement but notes split/merge energy stability as a material issue ([2023 survey/method](https://onlinelibrary.wiley.com/doi/full/10.1002/cav.2136)); [SPH-ASR](https://arxiv.org/abs/2008.01326) is further research. | No. Establish fixed-resolution conservation first; adaptivity is a separately gated follow-up. |
| Should soil/snow also use SPH? | Disney's [APIC](https://disneyanimation.com/publications/the-affine-particle-in-cell-method/) preserves affine motion/angular momentum through particle-grid transfers. [MLS-MPM/CPIC](https://yuanming.taichi.graphics/publication/2018-mlsmpm/) supports two-way rigid coupling and dynamic boundaries. Published MPM work models [Drucker-Prager sand](https://doi.org/10.1145/2897824.2925906) and [elastoplastic snow](https://doi.org/10.1145/2461912.2461948). | No solver monoculture. Use APIC/MLS-MPM for history-dependent solids while retaining material state on particles. |
| What is a credible wet-soil route? | Multiphase MPM research derives porous sand-water coupling from mixture theory ([UCLA thesis](https://escholarship.org/uc/item/52b8b82q)). Chrono's [CRM terrain](https://api.projectchrono.org/vehicle_terrain_crm_api_.html) couples SPH continuum soil and rigid/FEA bodies; its [terrain overview](https://api.projectchrono.org/development/vehicle_terrain.html) contrasts that with a cheaper SCM heightfield. | First prove dry soil. Start wet soil with a bounded saturation-coupled constitutive model; escalate to two-phase poromechanics only when product cases require it. |
| Can GPU results be canonical by quantizing output? | The [Vulkan specification](https://registry.khronos.org/vulkan/specs/latest/html/vkspec.html) and [SPIR-V specification](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html) permit execution/float latitude; NVIDIA documents non-associative parallel floating-point behavior in its [CUDA best-practices guide](https://docs.nvidia.com/cuda/pdf/CUDA_C_Best_Practices_Guide.pdf). | No. Quantization hides some drift but does not prove equivalent trajectories. GPU begins as a correspondence mirror with exact environment identity. |
| Are new 2026 methods production replacements? | [Dynamic divergence-free kernels](https://arxiv.org/abs/2601.17693) are grid-free but the paper's first-order solver is slower than traditional discretizations and targets a narrower incompressible-flow setting. [Neural Monte Carlo fluid simulation](https://pranav-jain.github.io/projects/nmcfs/index.html) explicitly does not claim state-of-the-art accuracy against established grid methods. A 2026 unified variational method is listed by [PeriDyno](https://peridynamics.com/publications.html). | Keep DDFK, nonlocal variational and neural operators as isolated comparators. None replaces the first free-surface/material-state baseline. |

## Corrections to the source brief

### Retained

- particles/material samples carry material identity and constitutive history;
- grids and meshes may be ephemeral;
- fixed cadence, bounded iterations and canonical ordering are mandatory;
- rigid coupling, persistence and streaming are first-class design problems;
- rendering is a projection and may not become gameplay authority;
- active regions and moving patches are necessary performance tools.

### Changed

- `ContinuumMaterialSystem` becomes an ownership umbrella, not one solver API;
- DFSPH is water-specific, not the default for all matter;
- MLS-MPM/APIC is the default research lane for dry deformable terrain;
- GPU is correspondence-first; no cross-target exactness is presumed;
- sleeping/deactivation is an explicit approximate state transition, not a
  promised lossless particle-to-field round trip;
- current solid-contact material descriptors are not extended with continuum
  constitutive fields;
- adaptivity follows fixed-resolution validation;
- ten ADRs and broad public contracts are reduced to one Proposed umbrella
  ADR/SPEC plus consumer-driven implementation specifications.

## Next Engine integration findings

The current code and architecture already have a single physical owner, exact
canonical physics snapshots, fixed `PhysicalStep`, immutable presentation and
atomic owner-segment persistence. They also have a PhysX-only current backend
decision and a consumer-driven-contract rule. Therefore:

- continuum code cannot directly mutate ECS or PhysX state;
- two-way coupling crosses one validated stable-ID impulse boundary;
- active continuum state eventually belongs to Physical Embodiment, but
  production promotion needs a later ADR that narrows ADR-058 without
  replacing PhysX as rigid/articulation owner;
- no `crates/contracts` types should be added for the lab;
- a GPU worker uses immutable revision-bound input and cannot select a tick by
  completion time;
- render surfaces/fields remain reconstructible cache;
- a dirty region cannot be evicted until its authoritative material state is
  durably represented or the transition is rejected.

## Open uncertainties and discriminating experiments

| Uncertainty | Smallest discriminator | Decision threshold |
|---|---|---|
| DFSPH cost at gameplay scale | CPU lab at 10k/50k/100k particles with fixed scenes | Choose first production region budget only from measured p50/p95 and convergence rate |
| Analytical vs sampled boundaries | Same leak/impact corpus with both paths | Keep the simpler path unless sampled geometry materially improves error within budget |
| Coupling stability | Floating body and fast impact across 1/2/4 coupling iterations | Select the smallest fixed count meeting momentum and penetration bounds |
| Drucker-Prager sufficiency | Angle-of-repose, shear-box, sinkage and tire patch corpus | Add MCC/μ(I) only on a declared failure cluster |
| Simplified wet-soil sufficiency | Infiltration/drainage and saturation-dependent shear | Escalate to two-phase MPM only if the simplified model cannot fit both families |
| Sleep representation | Repeated active→sleep→wake cycles | Admit only if conservation/history error stays bounded without drift accumulation |
| GPU promotion model | CPU/GPU corpus on pinned Windows/Linux profiles | Choose mirror-only, recorded-result, device-closed authority or deterministic reduced model explicitly |

## Non-goals of this research result

This report does not claim solver implementation, real-time performance,
cross-target GPU determinism, production persistence, learned-model quality or
completion of any roadmap blocker. All continuum ProductChecks remain
`NOT_RUN`.
