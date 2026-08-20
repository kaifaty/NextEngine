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
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --self-test --accumulation nuv-gather-directed-r0 \
  --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2 \
  --storage cell-sorted-o4
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --locality-tournament nuv-water-48k.v0 --warmup 32 --runs 96
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --retained-tournament nuv-water-48k.v0 --warmup 32 --runs 96
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --np0-baseline nuv-water-50k-coherent.v1 --warmup 64 --runs 512
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --np0-baseline nuv-water-50k-advected.v1 --warmup 64 --runs 512
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p1-check nuv-surface-stiff-16k-i2.v1 --iterations 2
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p1-tournament nuv-water-50k-advected.v1 --warmup 32 --runs 96
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p2-check nuv-water-100k-report.v1 --iterations 5
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p2-tournament nuv-water-50k-advected.v1 --warmup 32 --runs 96
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

Commands without `--storage` retain stable fixture/sample order.
`cell-sorted-o4` is valid only with retained gather, pointer swap and O2 term
specialization. It stores solver fields in stable packed-cell order while
remapping captures and logical CSR back to sample-ID order. Map setup, exact
map/CSR digests, memory and neighbor-storage distances are reported
separately.

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

`--locality-tournament` co-resides stable-sample and cell-sorted instances and
alternates their order for 32 warm-up and 96 measured rounds. Exact remapped
output/CSR, storage capacity and before/after state are required before its
same-process timings are admissible.

`--retained-tournament` is the final NR2 aggregate comparator on the two
correctness-valid HN-3 workloads. It co-resides and alternates the historical
source-atomic/copy/runtime denominator with the final retained
gather/pointer-swap/specialized/stable-sample stack. It is not valid for the
known-failed atomic stiff-surface profile.

`--np0-baseline` accepts only the separately rooted v1 profiles. It preserves
the retained NR4 solver identity, validates a bounded v1 CPU subset and stiff
surface i2 control, and runs one preallocated same-process benchmark. Advected
profiles feed each accepted output position/velocity into the next substep,
rebuild neighbors from that reference and reset only at a complete 32-step
epoch boundary. The command records every trace-state/output/CSR hash, raw
stage timing, degree/locality distribution and capacity result. A fixed 256-run
GPU-conditioning window precedes (and is separate from) the requested 32/64
formal warmups so P8-to-boost transitions do not contaminate percentiles.

The dynamic surface performance profile uses `gamma=100`; the independent
`nuv-surface-stiff-16k-i2.v1` correctness gate retains `gamma=1000`. The
rejected combination `gamma=1000`, 20 iterations and 32 sequential substeps
exceeded its bounded CSR at step one and is not a supported performance
profile.

`--p1-check` and `--p1-tournament` select the exact-work
`fused-owner-terms-p1` traversal on top of the retained NP0 identity. Density
remains a separate global barrier. Enabled post-density terms share one owner
CSR visit but retain independent f32 accumulators and the original
incompressibility/viscosity/surface commit order. The check requires exact
tiny/stiff/target output and CSR. The tournament co-resides retained and P1
instances, conditions both, alternates their order and advances dynamic traces
in lockstep. P1 adds no device storage and remains selectable as a rollback
layer for P2.

`--p2-check` and `--p2-tournament` compare retained P1's 32-bit neighbor IDs
with `compact-csr-u16-p2`. Eligible fixtures build directly into a 16-bit
neighbor array; row offsets remain checked 32-bit values. Capture widens IDs
outside timing for representation-neutral logical CSR hashing. Profiles above
65,535 samples explicitly fall back to the retained 32-bit representation.
