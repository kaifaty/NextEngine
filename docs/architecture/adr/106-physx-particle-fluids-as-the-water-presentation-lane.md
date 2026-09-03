# ADR-106: PhysX particle fluids as the optional water presentation lane

| Field | Value |
|---|---|
| ID | ADR-106 |
| Status | Proposed |
| Version | 0.1 |
| Decision date | pending (accept on the plan 23 probe evidence) |
| Proposal date | 2026-09-04 |
| Last verified | 2026-09-04 |
| Normative dependencies | [SPEC-04](../04-rendering-and-platform.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [SPEC-38](../38-continuum-material-physics.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-090](090-linux-only-v1-and-indefinitely-deferred-windows.md), [ADR-100](100-authoritative-water-volume-and-presentation-only-gpu-water.md), [ADR-102](102-presentation-particle-surface-pass.md), [ADR-104](104-water-v1-authority-is-the-exact-table-and-flow-network.md) |
| Supersedes | none; narrows one clause of ADR-100 ("no CUDA, vendor or process dependency enters `game`") for an optional, runtime-loaded presentation lane |
| Superseded by | none |

## Context

The water authority is complete and exact (ADR-104: table, flow network,
buoyancy batch; SPEC-38 practices 1-4 implemented in plans 19-22). Its
presentation is a surface ring plus stateless droplets. Task-state
decision D-011 (2026-09-04) chooses NVIDIA PhysX particle fluids
(`PxPBDParticleSystem`, GPU) as the presentation lane for volumetric
water, keeping the authority where it is. The research note
`docs/development/water-physx-presentation-research-2026-09-04.md`
records what PhysX 5.9 provides and what the repository has: the SDK is
already linked in-process (CPU, static, behind a feature), its GPU
library builds from the open 5.9 sources and is loaded at runtime by
`dlopen`, and the ADR-102 particle pass already consumes positions,
velocities and kernels.

## Decision

1. **Authority unchanged.** Nothing of ADR-100/103/104/105 moves. PhysX
   particles never enter a command, query, snapshot, save, replay,
   frame-plan or gameplay root; no type of this lane enters
   `crates/contracts`; `headless` never executes it.
2. **The lane.** A PhysX PBD fluid lives in the desktop adapter's process
   as presentation dynamics: particles are emitted from the stage's edge
   records (plan 21) and the floating boxes' immersion changes (plan 17),
   absorbed at the exact cell surfaces, bounded by the cell geometry and
   the committed body poses; the ADR-102 particle surface pass renders
   them (positions, velocities, anisotropic kernels). The exact levels
   drive the particles; nothing flows back.
3. **Optional and vendor-specific, fail-closed to the fallback.** The
   game process links the CPU PhysX bridge and loads the GPU library at
   runtime only when a run option asks for the lane and a capability
   probe (NVIDIA driver, CUDA context manager, GPU scene) succeeds. Any
   failure, device loss or absence yields the ADR-100 still surface and
   the plan 09-22 presentation, reported once. Linux x86_64 with the
   NVIDIA driver is the only host of the lane (ADR-090 stays Linux-only;
   AMD and Intel hosts see the fallback).
4. **Not determinism.** The lane is evaluator-class in ADR-058's terms:
   checks around it are bounded and statistical (particle count, bounds,
   cost, frame cadence), never hash-pinned; captures and human look gates
   are its evidence.
5. **Build and provenance.** The PhysX SDK profile gains GPU projects
   built from the pinned 5.9 sources with the host `nvcc`; the manifest
   gate pins the GPU library beside the static libraries; `xtask physx
   doctor` reports the GPU state. No CI, no network at production
   startup (ADR-058 unchanged).

## Consequences

- ADR-100's clause "no CUDA, vendor or process dependency enters `game`"
  is narrowed to: no such dependency enters `headless`, the contracts or
  any root; `game` may load the vendor GPU library at runtime for
  presentation only, behind the option and the probe.
- The first increment is the probe of plan `continuum-water/23`
  (`xtask physx pbd-probe`, research report `PHYSX-WATER-PRESENT-R1`);
  acceptance of this ADR follows its evidence. The lane in the game is
  plan 24.
- Rejected alternatives (research note): a custom Vulkan compute solver
  (vendor-neutral, but the largest build); Dimforge Nexus (no fluids yet,
  experimental); Salva (CPU, too slow at game scale); Jolt/Rapier (no
  fluids). Hard-binding the authority to PhysX (rejected: no determinism,
  no replay, no levels as a concept, megabytes of particle state to save).
