# Nonlocal continuum feasibility tool

This directory contains the quarantined, report-only implementation for the
NR1 Nonlocal/SISSM research stage. It is not linked into the Rust workspace,
does not expose an engine API and does not publish authoritative simulation
state.

The initial checkpoint contains an independent CPU `f64` oracle and frozen
machine-readable workload profiles. A later NR1 checkpoint adds the separately
implemented source-shaped CUDA `f32` baseline. The two numerical paths must not
share pair-contribution code.

Build outside the repository and run the bounded CPU controls:

```sh
cmake -S crates/continuum-water/tools/nonlocal-feasibility \
  -B /tmp/nextengine-nonlocal-feasibility-build -G Ninja
cmake --build /tmp/nextengine-nonlocal-feasibility-build
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --describe-profile nuv-water-48k.v0
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility --self-test
```

Each command writes one JSON value to stdout. Build trees, binaries, raw JSON
and profiler captures stay outside Git.
