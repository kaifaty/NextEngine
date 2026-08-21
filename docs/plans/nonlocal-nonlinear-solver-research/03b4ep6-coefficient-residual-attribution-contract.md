# NSR3-B4EP6 -- coefficient-candidate residual-attribution contract

Status: `CLOSED / PASS / B4EP7_EVALUATION_BASE_DESIGN_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep6-coefficient-residual-attribution|v1|parent=46224e0e70c3fa1a21b3a1fa3b8e5aec81f10fcc6312d99314caec6c91e45e40:9f47bda20166eb985b62b418f53acad2348311b309de66d01c53490b73abd8e9:5cd61e3eb82f6a3cedfe4d7d8e39cbb5aca65c6e00c5ef9cd9e7555a71b9bf23:dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b|implementation=7263d8929491b66ea94eb74713fab4c136efe5be|sources=6fe62f0d4a095062059a5387d9c85e8d20ce5b96cb951575597a840c98304d63:f7f6ea10b2a378ea7a291ae9924278b8c64c9b59e2958727f9b8052ea696511d:cdeaf706f179eb716257d55b5f2dbf0371add899978bd7c86d8ea8309b2288fa|toolchain=gcc15.2;release=-O3,-DNDEBUG,-g,-pg,-ffp-contract=off,-fno-fast-math;gprof=2.46|run=one-b4ep5;watchdog=900s|gate=stdout-byte-exact;gmon-nonempty;flat-self-samples>0|categories=hvp;workspace;workspace-filter-csr;workspace-eval-tape;workspace-coeff;control|selection=top-leader>=1.20x-runner-up;workspace-subleader>=1.20x;otherwise=phase-timing|timing=external|decision=one-next-design|credit=b4ep7-design-only
```

Identity SHA-256:
`75b8a8b7ba137674091413e8f0eb7ace46f32a2c09059de260c06c36955db24d`.

## Build and run

Create a clean external CMake Release build with:

```text
CMAKE_CXX_FLAGS_RELEASE=-O3 -DNDEBUG -g -pg -ffp-contract=off -fno-fast-math
CMAKE_EXE_LINKER_FLAGS=-pg
CMAKE_EXPORT_COMPILE_COMMANDS=ON
```

Use GCC 15.2.0 and GNU gprof 2.46. Build only
`nonlocal-formula-reclosure`. From an empty external run directory execute
once under the 900-second watchdog and external `/usr/bin/time -v`:

```text
nonlocal-formula-reclosure --nominal-hydro-hvp-coefficient-ablation
```

Do not change host performance-event policy.

## Admission

Require exact identity/source hashes, exit zero, empty stderr and stdout
exactly 6,729 bytes with SHA-256
`dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b`.
Require semantic result
`5cd61e3eb82f6a3cedfe4d7d8e39cbb5aca65c6e00c5ef9cd9e7555a71b9bf23`,
nonempty `gmon.out`, nonzero flat samples, flat/call-graph text and hashes,
compile commands, instrumented binary hash and Build ID.

Preserve exact profile call counts and resolve optimized/folded symbols by
their callers and children. Instrumented wall/RSS are not Release evidence.

## Attribution and exit

Report inclusive HVP, complete workspace and residual control. Split workspace
into topology/filter/CSR, evaluation/base tape and coefficient population.
Select a leader only at `>=1.20x` runner-up; otherwise select
`SCOPED_INTERNAL_PHASE_TIMING` and no optimization. A stdout mismatch is
`PROFILER_CORRESPONDENCE_FAIL`; empty samples are `PROFILER_NO_SAMPLES`.

B4E2, references, runtime/CUDA, solver-policy changes and production remain
blocked.

Observed PASS: workspace 4.66 s versus HVP 3.45 s (`1.3507x`); within
workspace, evaluation/base tape 2.73 s versus topology/CSR 1.09 s
(`2.5046x`). See the
[dated evidence](../../development/nonlocal-nsr3b4ep6-coefficient-residual-attribution-evidence-2026-08-22.md).
