# PhysX PBD probe — the GPU fluid behind the existing bridge (ADR-106 step 1)

| Field | Value |
| --- | --- |
| Research ID | `PHYSX-WATER-PRESENT-R1` (research report, not a product check) |
| Status | `RUN / G1 G2 G3 G5 G6 PASS / G4 FAIL (reading recorded)` (2026-09-04) |
| Parent | ADR-106 (Proposed); task-state D-011; research note `docs/development/water-physx-presentation-research-2026-09-04.md`; the PhysX bridge (`crates/physics-physx-ffi`, `tools/xtask/src/physx.rs`) |
| Purpose | prove on the reference host that the pinned PhysX 5.9 builds its GPU library from source, that the bridge can create a CUDA context manager, a GPU scene and a PBD fluid, step it and read positions back inside a frame budget, and that the lane fails closed without the GPU library; nothing in the game changes |

## Frozen scope

- **SDK profile.** `xtask physx setup` gains the profile
  `nextengine-physx-5.9.0-static-cpu-gpu-release-v2`: the CMake preset
  sets `PX_GENERATE_GPU_PROJECTS=True` and passes the host CUDA toolkit
  (`CUDAToolkit_ROOT_DIR`, `nvcc`); the build produces
  `libPhysXGpu_64.so` beside the static libraries; the manifest
  `nextengine-physx-sdk-v1.json` records `gpu_library` (relative path and
  SHA-256) and the CUDA toolkit version; `physx doctor` prints them. The
  static CPU libraries stay the linked ones; the GPU library is never
  linked, only found by the loader at runtime (`PX_PHYSX_GPU_SHARED_LIB_NAME`).
- **Bridge probe (additive; `bridge_abi` stays `4`, see the result).** `nextengine_physx_pbd_probe(desc,
  report)`: creates the foundation and physics as today, a
  `PxCudaContextManager` (no graphics interop), a scene with
  `eENABLE_GPU_DYNAMICS` and the GPU broad phase, one
  `PxPBDParticleSystem` with a fluid material, one `PxParticleBuffer` of
  `16,384` fluid particles on a `0.05 m` grid inside a `1 x 1 x 1 m` box
  of static planes, particle spacing `0.05 m` (rest density `1000
  kg/m^3`), and steps `120` frames at `60 Hz`, copying positions and
  velocities to the host after every `fetchResults` through the context
  manager. Report: step and readback microseconds (max, mean), particles
  inside the box plus `0.1 m` margin after every step (count), a
  SHA-256 of the final positions (run-to-run comparison, recorded, not
  gated), the GPU name and driver reported by the context manager.
- **xtask command.** `xtask physx pbd-probe` runs the probe twice and
  emits the JSON report (`PhysxPbdProbeDetailsV1`); with the environment
  variable `NEXTENGINE_PHYSX_GPU_LIBRARY` pointing at a missing file the
  command reports `gpu_available: false` with the loader's reason and
  exits successfully (fail-closed path).
- **Not in scope.** The game, the renderer, the presentation stage, any
  contract; the probe is developer tooling like `physics-backend-parity`.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 build | `xtask physx setup` under the v2 profile builds the CPU static libraries and `libPhysXGpu_64.so`; the manifest pins the GPU library; `doctor` reports it; the existing `physx-sdk` runtime tests and `physics-backend-parity` PASS on the v2 SDK | pass |
| G2 probe runs | both probe runs create the context manager, the GPU scene and the fluid, step `120` frames and read back `16,384` particles each frame | pass |
| G3 cost | step plus readback `<= 4,000 us` mean and `<= 8,000 us` max per frame on the reference host (RTX 3080) | pass |
| G4 bounds | every particle inside the box plus margin on every frame of both runs | pass |
| G5 fail-closed | with the GPU library path broken the command reports `gpu_available: false` and a reason, exits `0`, and the CPU bridge still initialises | pass |
| G6 repo unchanged | `host-check` PASS; `game` still without any PhysX feature; the boundary scan unchanged; `play`, `persistence-replay` roots unchanged | pass |

The particle count, box, spacing, frame count and budgets are frozen; a
change after the run is a new revision with its own evidence. The
run-to-run hash equality is recorded, not gated.

## Result (2026-09-04)

Implementation: `tools/xtask/src/physx.rs` (profile v2, `PhysxGpuManifestV1`,
`cuda_toolkit()`, `patch_physx_for_cuda_13`, `physx pbd-probe`),
`crates/physics-physx-ffi` (`ne_physx_pbd_probe`, `PbdProbeDesc` /
`PbdProbeReport`, mock and stub, bridge ABI `5`).

Apparatus corrections, recorded: the pinned PhysX 5.9 targets CUDA
toolkit 12.8 and the host has 13.3, so `physx setup` applies three
idempotent source patches to the cached sources when a toolkit is
present: (1) the GPU architecture list follows
`CMAKE_CUDA_ARCHITECTURES` (`86`) instead of the fixed list starting at
Volta, which nvcc 13 rejects; (2) `cuCtxCreate` takes the 13.x
signature under a `CUDA_VERSION` guard; (3) `CudaKernelWrangler.cpp`
gains no-op stubs for `__cudaGetKernel` and `__cudaLaunchKernel`, the
runtime entry points nvcc 13 emits in its host stubs (PhysX resolves the
older ones the same way and launches through the driver API; a first
attempt to link `cudart_static` instead collided with those stubs and
was dropped). The build uses the public CMake tree (`compiler/public`),
not the root FetchContent tree. Without a CUDA toolkit the profile
builds the CPU SDK and records `gpu: unavailable`.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 build | v2 profile builds the static CPU libraries and `libPhysXGpu_64.so` (`41.8 MB`, sm_86, CUDA `13.3`, SHA-256 in the manifest); `physx doctor` reports it; `physics-backend-parity` PASS on the v2 SDK (`100,000` substeps, `10,000` permutations); the `physx-sdk` crate tests PASS (`7` + `2`) | pass |
| G2 probe runs | both runs: `NVIDIA GeForce RTX 3080`, CUDA context, GPU scene and fluid created, `120` frames, `16,384` particles read back every frame | pass |
| G3 cost | step plus readback: run 1 mean `1,756 + 80 us`, max `4,443 + 163 us`; run 2 mean `1,533 + 68 us`, max `3,930 + 115 us` (limits `4,000` mean, `8,000` max) | pass |
| G4 bounds | at the worst frame `3` of `16,384` particles lie outside the box plus `0.1 m` margin in both runs (the seeded column is `2.05 m` tall in a `1 m` footprint; the wall planes meet at the corners where PBD particles can slip) | FAIL against the frozen "every particle"; reading recorded |
| G5 fail-closed | with `NEXTENGINE_PHYSX_GPU_LIBRARY=/nonexistent/...` the command reports `gpu_available: false`, reason "gpu library or cuda device unavailable", the loader's warning, exit `0`; the CPU bridge initialised for the run | pass |
| G6 repo unchanged | `host-check` PASS (workspace fmt, clippy, tests, boundary scan), `play` root `5f0c8bcd…` and `persistence-replay` root `03901d76…` unchanged, `next_motor` closure tests PASS with the ABI kept at `4`; `game` has no PhysX feature; `crates/contracts` untouched | pass |

Apparatus correction (recorded): the frozen text bumped the bridge ABI
to `5`; the ABI number is part of the biomechanics closure identity of
`next_motor` (ADR-058's training substrate), so a bump moves those
roots. The probe is an additive entry point with every existing call
unchanged, so the ABI stays `4` and only the build profile (`v2`) and
the manifest's `gpu` record move.

Recorded, not gated: the two runs' final positions differ
(`df1c3ba9…` against `bdff37ce…`) — GPU PBD is not run-to-run
deterministic, as ADR-106 assumes.
