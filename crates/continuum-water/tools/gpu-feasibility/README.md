# Continuum-water GPU feasibility proxy

This directory contains a standalone, report-only CUDA discriminator for the
sealed 48k water workload. It is not linked into the Rust workspace, exposes no
public API, owns no state and cannot publish a water frame or reaction batch.

The proxy reconstructs the exact initial sealed-lattice neighbor membership and
edge counts for `48,000` fluid samples and `24,704` W0F static boundary
samples. It then times the dominant cold W0H density-solve shape: 40
iterations, one pressure-acceleration graph pass, one dependent matrix-action
graph pass and four global reductions per iteration. The `f64` mode uses double
precision throughout. The `mixed32` mode uses `f32` graph/vector arithmetic
and `f64` global reductions.

This is deliberately not `CONTINUUM-MIRROR-P1` and grants no W2 or ProductCheck
credit. It uses synthetic solve vectors and does not execute density/diagonal
assembly, the data-dependent convergence branch, divergence, contact,
integration, canonical publication or the physical correctness corpus. Its
only valid use is to reject or retain a direct-port GPU performance hypothesis
before a production architecture decision.

Build and run on the declared Linux CUDA host:

```sh
cmake -S crates/continuum-water/tools/gpu-feasibility \
  -B /tmp/nextengine-continuum-water-gpu-feasibility-build -G Ninja
cmake --build /tmp/nextengine-continuum-water-gpu-feasibility-build
/tmp/nextengine-continuum-water-gpu-feasibility-build/continuum-water-gpu-feasibility \
  --self-test
/tmp/nextengine-continuum-water-gpu-feasibility-build/continuum-water-gpu-feasibility \
  --warmup 5 --runs 20 --iterations 40
```

The measured command writes one bounded JSON report to stdout. Redirect it to
an untracked path such as `/tmp`; generated reports and binaries do not belong
in Git.
