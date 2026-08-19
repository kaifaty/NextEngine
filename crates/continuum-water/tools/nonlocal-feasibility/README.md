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
  --repeatability nuv-surface-16k.v0 --iterations 20 --runs 10 \
  --accumulation nuv-gather-directed-r0
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --check nuv-water-48k.v0 --iterations 5
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --benchmark nuv-water-48k.v0 --warmup 5 --runs 50
```

Each command writes one JSON value to stdout. Build trees, binaries, raw JSON
and profiler captures stay outside Git.

Commands without `--accumulation` retain the frozen `source-atomic-v0`
baseline. `nuv-gather-directed-r0` selects the NR1-RC1 owner-only directed
gather counterfactual explicitly.
