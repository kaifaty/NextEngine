# Nonlocal continuum feasibility tool

This directory contains the quarantined, report-only implementation for the
NR1 Nonlocal/SISSM research stage. It is not linked into the Rust workspace,
does not expose an engine API and does not publish authoritative simulation
state.

The tool contains an independent CPU `f64` oracle, frozen machine-readable
workload profiles and a separately implemented source-shaped CUDA `f32`
baseline. The two numerical paths do not share pair-contribution code.

Build outside the repository and run the bounded CPU controls:

```sh
cmake -S crates/continuum-water/tools/nonlocal-feasibility \
  -B /tmp/nextengine-nonlocal-feasibility-build -G Ninja
cmake --build /tmp/nextengine-nonlocal-feasibility-build
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --describe-profile nuv-water-48k.v0
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility --cpu-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility --cpu-gather-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility --self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --self-test --accumulation nuv-gather-directed-r0
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --self-test --accumulation nuv-gather-directed-r0 \
  --handoff pointer-swap-o1
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --self-test --accumulation nuv-gather-directed-r0 \
  --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --repeatability nuv-surface-16k.v0 --iterations 20 --runs 10 \
  --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --check nuv-water-48k.v0 --iterations 5
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --benchmark nuv-water-48k.v0 --warmup 5 --runs 50 \
  --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1 \
  --term-kernels nuv-terms-specialized-o2
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --self-test --accumulation nuv-unique-pair-segmented-o3 \
  --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --layout-tournament nuv-water-16k.v0 --warmup 32 --runs 96
```

Each command writes one JSON value to stdout. Build trees, binaries, raw JSON
and profiler captures stay outside Git.

Commands without `--accumulation` retain the frozen `source-atomic-v0`
baseline. `nuv-gather-directed-r0` selects the NR1-RC1 owner-only directed
gather counterfactual explicitly. Commands without `--handoff` retain the
`copy-v0` state handoff. `pointer-swap-o1` is valid only with the directed
gather path and selects the report-only NR2-O1 candidate explicitly. Commands
without `--term-kernels` retain `nuv-terms-runtime-v0`.
`nuv-terms-specialized-o2` is valid only with pointer swap and either directed
gather or the O3 segmented layout. On directed gather it selects the retained
report-only NR2-O2 path.

`nuv-unique-pair-segmented-o3` selects the report-only NR2-O3 candidate and
is valid only with `pointer-swap-o1` plus `nuv-terms-specialized-o2`. It keeps
the frozen CSR, builds and validates an immutable symmetric reverse-slot map,
evaluates each non-self pair once, writes two 48-byte endpoint fragments and
reduces them in owner CSR order. Reverse-map setup time and the additional
layout allocation are reported separately.

`--layout-tournament` co-resides historical atomic, retained gather and O3
segmented instances, then rotates their order `A/G/S`, `G/S/A`, `S/A/G` for
32 warm-up and 96 measured rounds. It is an evidence command, not runtime
integration. The O3 execution record rejected the candidate at the stiff
surface correspondence gate, so retained work continues to use
`nuv-gather-directed-r0 + pointer-swap-o1 + nuv-terms-specialized-o2`.
