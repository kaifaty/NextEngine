# W1 — Serial CPU DFSPH oracle

## Outcome

Implement a safe-Rust, serial, runtime-independent DFSPH oracle that executes
the W0 corpus, publishes a canonical fixed-point frame after every substep and
produces bounded typed evidence. It is not a production physics backend.

## Placement and interfaces

- Add internal crate `crates/continuum-water` with package name
  `next_continuum_water`; it does not depend on `next_contracts`, runtime,
  PhysX, renderer or ECS.
- Expose execution through `xtask continuum water oracle`; do not add another
  shipping application root.
- Keep all V1 profile/scenario/frame/report records crate-private or tool-local.
  They are not public or durable engine schemas.

Private logical records:

```text
WaterLabProfileV1 {
  exact W0 constants,
  maximum_samples,
  maximum_steps,
  maximum_neighbors_per_sample,
  scenario_hash,
  reference_corpus_hash,
}

WaterCanonicalFrameV1 {
  profile_hash,
  scenario_hash,
  step,
  sorted_samples[(SampleId, position_um[3], velocity_um_s[3])],
  frame_root,
}

WaterLabReportV1 {
  tool_commit,
  toolchain_target,
  profile/scenario/corpus hashes,
  terminal result,
  per-step iteration/residual summaries,
  conservation/reference metrics,
  canonical trajectory root,
  timing marked diagnostic-only,
}
```

Every decoded count, allocation product, step count and neighbor capacity is
checked before allocation. Reports use the existing typed `xtask` JSON/report
style and write only to an explicit output path outside Git by default.

## Solver and data structures

- Material SoA: stable `SampleId`, decoded `f64` position/velocity, density,
  DFSPH factor and pressure/divergence scratch.
- Rebuild a sorted uniform grid each substep as `(cell_key, SampleId)` entries
  plus contiguous cell ranges. Do not use hash-map iteration in a reduction.
- Traverse neighbor cells in fixed lexicographic offsets and neighbors by
  `SampleId`; use the same cubic-spline kernel family for value and gradient.
- Apply gravity, divergence solve, non-pressure baseline, density solve and
  position update in one documented fixed order.
- Disable warm start and every non-pressure optional model in V1.
- Quantize each accepted sample once with checked ties-to-even; discard the
  private float frame and decode the next step from canonical integers.

The serial oracle has no worker-count equality requirement. Insertion-order
permutations are sorted into the same sample/cell order and must produce the
same canonical trajectory root.

## Boundary and failure behavior

Analytical planes/box are part of W1. Boundary evaluation and reaction
diagnostics use fixed feature keys. A centre beyond the hard analytical wall,
nonfinite intermediate/result, checked overflow, neighbor/capacity excess or
solver non-convergence terminates the run at that first substep.

The report contains the stable first cause and last accepted root. The tool
does not retry, loosen a tolerance, retain the old frame and continue, or emit
a successful corpus after any terminal failure.

## Required checks

- unit/golden tests for cubic-spline value/gradient and analytical boundaries;
- exact ties-to-even vectors, negative zero normalization, overflow and
  nonfinite rejection;
- `N-1/N/N+1` capacity tests for samples, neighbors, steps and report bytes;
- zero/one-particle, duplicate-ID and input-order permutation cases;
- every W0 scenario at a small checked-in fixture size;
- full external W0 corpus at nominal scale with output artifacts outside Git;
- repeat runs on the same target/toolchain with identical canonical roots;
- published and SPlisHSPlasH aggregate comparison under the frozen metrics.

## Exit and stop rule

`CONTINUUM-WATER-REF-P1 = PASS` requires every W0 correctness threshold and
same-target repeat/insertion equality. Timing at `10k/50k/100k` is recorded but
cannot fail W1 or be called a production budget result.

Failure keeps the main roadmap inactive. The smallest failing scenario and
first causal metric become the next discriminator; implementation cannot move
to parallelism, PhysX or GPU to hide a serial correctness failure.

## Non-goals

Parallel execution, real-time claims, PhysX, moving bodies, public contracts,
save/restart, streaming, renderer integration and GPU.
