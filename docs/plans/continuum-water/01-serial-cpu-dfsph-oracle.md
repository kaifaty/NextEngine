# W1 — Serial CPU DFSPH oracle

Status: `IMPLEMENTED / BLOCKED_ON_W0C_RECALIBRATION`.

The safe-Rust serial oracle and its tool path were implemented at commit
`74730e208cfeb70b05a3ec44b2bb9c2f5002fe97`. Clean-tree evidence passes the
exact free-fall control but stops on `CW-HYDRO-001` at the first density solve:
iteration 20 ends at `74,482,699 ppb` against the inclusive `100,000 ppb`
threshold. See the bounded
[W1 evidence report](../../development/continuum-water-w1-evidence-2026-08-17.md).
`CONTINUUM-WATER-REF-P1` remains `NOT_RUN`; this document's exit criterion is
not satisfied. The clean-tree
[W1-RC1 independent audit](../../development/continuum-water-w1-rc1-audit-2026-08-17.md)
finds no mismatch in the audited production path, so
[W0C](00c-hydro-calibration-reclosure.md) must re-close the numerical profile
before W1 resumes.
The first
[W0C diagnostic cycle](../../development/continuum-water-w0c-hydro-calibration-2026-08-17.md)
rejects both a ceiling-only repair and `ghost-cell-shell-v1`; W1 remains
blocked. Its independently reproduced `zero-velocity-settle-v1`
initialization candidate also diverges and fails the first unchanged-ceiling
probe, so W0C has escalated to a distinct non-particle boundary-formulation
discriminator.

## Outcome

Implement a safe-Rust, serial, runtime-independent DFSPH oracle that executes
the hash-frozen [W0B](00b-numeric-execution-and-corpus-closure.md) formulation
and corpus, publishes a canonical fixed-point frame after every substep and
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
  W0B document, float-profile and corpus roots,
  exact W0B constants,
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
- Rebuild the exact W0B sorted integer-cell grid each substep as
  `(cell_key, SampleId)` entries plus contiguous cell ranges. Do not use
  hash-map iteration in membership or a reduction.
- Use the W0B inclusive integer neighbor test, boundary quadrature, cubic
  kernel, equations, folds, Jacobi buffers and quantized convergence branches
  exactly; no solver choice remains in W1.
- Apply divergence, gravity, density solve, position update, validation and
  publication in the frozen order.
- Disable warm start and every non-pressure optional model in V1.
- Quantize each accepted sample once with checked ties-to-even; discard the
  private float frame and decode the next step from canonical integers.

The serial oracle has no worker-count equality requirement. Insertion-order
permutations are sorted into the same sample/cell order and must produce the
same canonical trajectory root.

## Boundary and failure behavior

Analytical planes/box and their reconstructible W0B quadrature are part of W1.
Boundary evaluation and reaction diagnostics use fixed feature keys. An outer
centre escape, clearance penetration above `2.5 mm`, nonfinite
intermediate/result, checked overflow, neighbor/capacity excess or solver
non-convergence terminates the run at that first substep with the frozen
failure string.

The report contains the stable first cause and last accepted root. The tool
does not retry, loosen a tolerance, retain the old frame and continue, or emit
a successful corpus after any terminal failure.

## Required checks

- unit/golden tests for cubic-spline value/gradient and analytical boundaries;
- exact ties-to-even vectors, negative zero normalization, overflow and
  nonfinite rejection;
- `N-1/N/N+1` capacity tests for samples, boundaries, both neighbor rows,
  steps, report bytes and reference-input bytes;
- zero/one-particle, duplicate-ID and input-order permutation cases;
- one checked-in `SMOKE_ONLY` small analogue of each W0B geometry, with a
  distinct scenario root and no corpus-credit claim;
- full external W0B corpus at nominal scale with output artifacts outside Git;
- repeat runs on the same target/toolchain with identical canonical roots;
- analytical and SPlisHSPlasH aggregate comparison under the frozen metrics;
- preflight rejection when any W0B document/profile/corpus root differs.

## Exit and stop rule

`CONTINUUM-WATER-REF-P1 = PASS` requires every W0B correctness threshold and
same-target repeat/insertion equality. Timing at `10k/50k/100k` is recorded but
cannot fail W1 or be called a production budget result.

Failure keeps the main roadmap inactive. The smallest failing scenario and
first causal metric become the next discriminator; implementation cannot move
to parallelism, PhysX or GPU to hide a serial correctness failure.

## Non-goals

Parallel execution, real-time claims, PhysX, moving bodies, public contracts,
save/restart, streaming, renderer integration and GPU.
