# Research: NVIDIA PhysX as the water presentation lane (2026-09-04)

Decision context: task-state D-011 (2026-09-04) — the water authority
stays the exact CPU model (ADR-100/103/105); PhysX particle fluids become
the presentation lane, optional and vendor-specific, with the ring and
droplet presentation of plans 09-22 as the fallback. This note records
what was verified before the ADR and the first plan.

## What PhysX 5.9 offers

- `PxPBDParticleSystem`: position-based dynamics particles with fluid
  density constraints (`PxParticlePhaseFlag::eParticlePhaseFluid`), cloth,
  inflatables and rigid coupling. The SDK documentation states that
  "simulating particle systems requires a CUDA capable GPU and a CUDA
  context manager must be provided upon creation of the particle
  system" ([Particle System, 5.4.1][ps]).
- State lives in `PxParticleBuffer`s on the GPU: "all pointers returned by
  functions of `PxParticleBuffer` are device pointers" and "all of the
  responsibility of copying data to and from the GPU is left to the user";
  positions (`PxVec4` position + inverse mass) and velocities are read
  after `fetchResultsParticleSystem()` ([PxParticleBuffer][pb]).
- Runtime particle management: `setNbActiveParticles` up to
  `getMaxParticles`, dirty flags per buffer; parameters: particle
  spacing, rest and contact offsets, fluid rest offset, rest density,
  viscosity, surface tension, cohesion, at most `96` neighbours.
- GPU dynamics: the scene needs `PxSceneFlag::eENABLE_GPU_DYNAMICS` and a
  `PxCudaContextManager`; "this feature is implemented in CUDA and
  requires SM6.0 (Pascal) or later" ([GPU simulation][gpu]); the 5.9
  changelog states "display driver supporting CUDA toolkit 12.8 and Volta
  GPU or above".
- The documentation makes no determinism statement for GPU simulation.
  ADR-058 already treats GPU mirrors as evaluator correspondence, never
  replay authority.

## What the repository has

- PhysX `5.9.0` (source revision `517a0073…`) built by `xtask physx setup`
  into the host cache as static CPU libraries with
  `PX_GENERATE_GPU_PROJECTS=False`, linked in-process through the C++
  bridge of `crates/physics-physx-ffi` behind the `physx` feature of the
  runtime; `apps/game` has no PhysX feature; `physics-backend-parity`
  compares PhysX with the reference world out of process.
- The 5.9 source tree contains the GPU sources (`source/physxgpu`,
  `gpusolver`, `gpubroadphase`, …, `61` CUDA files): `libPhysXGpu_64.so`
  is built from source with `nvcc` when GPU projects are generated and
  loaded at runtime by `PxPhysXGpuModuleLoader.cpp` through
  `dlopen("libcuda.so.1")` then `dlopen("libPhysXGpu_64.so")`. On a host
  without the NVIDIA driver the load fails with a warning and the CPU
  library keeps working: the GPU lane is optional by construction.
- License: the CPU sources are BSD-3; the GPU binaries are redistributable
  under the same conditions ([PhysX license][lic]).
- The reference host: RTX 3080 (compute `8.6`), driver `610.43.02`, CUDA
  toolkit `13.3` with `nvcc`, `libcuda.so.1` present.
- The renderer already consumes what a PBD system emits: the ADR-102
  particle surface pass takes `ParticleSurfaceUpdateV1` (positions,
  velocities, optional neighbour counts, cluster sizes, anisotropic
  kernels; capacity `65,536`), fed today with the stage's droplets only.
- ADR-100 bans CUDA, vendor and process dependencies from `game`,
  `headless` and public contracts, and requires a shipped presentation
  solver to run in the Vulkan backend or across a neutral artifact
  boundary; `physx` is a forbidden token in `crates/contracts`.

## Consequences for the lane

1. The authority split of ADR-100 is untouched: PhysX particles never
   feed a command, query, save, replay or root; the exact levels and edge
   fluxes drive the particles (emission at edges, absorption at cell
   surfaces, boundaries from the cell geometry and the bodies' committed
   poses), never the reverse.
2. ADR-100's "no CUDA dependency in `game`" clause needs a narrowing: the
   game process may *optionally* load the vendor GPU library at runtime
   for presentation, behind a run option and a capability probe, with
   the still surface and droplets as the fallback; `headless` and
   `crates/contracts` stay untouched. That is ADR-106.
3. Determinism is not claimed for the lane; the checks around it are
   statistical and bounded (particle counts, bounds, cost), never
   root-based, as for the research particle lanes of ADR-104.
4. AMD, Intel and any non-NVIDIA host see the fallback; this is a
   product decision recorded with D-011.
5. Build: the SDK profile gains GPU projects (`nvcc` on the developer
   host, no CI); the manifest gate pins the GPU library's hash beside the
   static libraries; `xtask physx doctor` reports GPU availability.

## Order of work

1. Plan 23: SDK profile with GPU projects and a bridge probe (`xtask
   physx pbd-probe`) that creates a CUDA context manager, a GPU scene and
   a PBD fluid of `16,384` particles, steps and reads back, and reports
   cost and bounds; no game change.
2. Plan 24 (after ADR-106 acceptance on the probe's evidence): the
   presentation lane in the desktop adapter's process — emission from
   the stage's edge records, absorption at cell surfaces, boundaries from
   cells and bodies, the ADR-102 pass fed with positions, velocities and
   kernels, run option and fallback, captures.
3. Later: two-way *presentation* coupling (splashes from the crate),
   foam and spray from the particle statistics.

[ps]: https://nvidia-omniverse.github.io/PhysX/physx/5.4.1/docs/ParticleSystem.html
[pb]: https://nvidia-omniverse.github.io/PhysX/physx/5.4.0/_api_build/class_px_particle_buffer.html
[gpu]: https://nvidia-omniverse.github.io/PhysX/physx/5.4.1/docs/GPURigidBodies.html
[lic]: https://nvidia-omniverse.github.io/PhysX/physx/5.4.2/docs/License.html
