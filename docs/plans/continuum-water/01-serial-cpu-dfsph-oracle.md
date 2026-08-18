# W1 — Serial CPU DFSPH oracle

Status: `IMPLEMENTED / LINUX_CORPUS_IN_PROGRESS / EXTERNAL_REFERENCES_PENDING`.

The original W0B oracle remains rejected-profile evidence. W0E selected the
constraint-separated local survivor, W0F froze its solver, geometry,
capacities and successor roots, and W0G reclosed impact-energy semantics after
the first full dam-break discriminator. The current safe-Rust runner verifies
that complete root chain before executing any nominal scenario.

Linux hydro and exact free-fall pass internally. The unchanged W0F dam-break
also completes `720/720` under W0G with zero penetration, bounded solver and
momentum metrics, zero positive energy excess and an explicitly published
`638,407,503 ppb` mechanical-energy deficit. It remains
`REFERENCE_PENDING`; this is not corpus or ProductCheck credit. See the
[successor energy discriminator](../../development/continuum-water-w1-successor-energy-discriminator-2026-08-18.md).
`CONTINUUM-WATER-REF-P1` remains `NOT_RUN`.

## Outcome

Implement a safe-Rust, serial, runtime-independent DFSPH oracle that executes
the hash-frozen [W0F](00f-geometry-capacity-and-root-closure.md) formulation,
[W0G](00g-impact-energy-contract-reclosure.md) metric contract and nominal
corpus, publishes a canonical fixed-point frame after every substep and
produces bounded typed evidence. It is not a production physics backend.

## Placement and interfaces

- Add internal crate `crates/continuum-water` with package name
  `next_continuum_water`; it does not depend on `next_contracts`, runtime,
  PhysX, renderer or ECS.
- Expose successor execution through `xtask continuum water run-w1-linux`;
  retain `continuum water oracle` only for immutable W0B evidence. Do not add
  another shipping application root.
- Keep all V1 profile/scenario/frame/report records crate-private or tool-local.
  They are not public or durable engine schemas.

Private logical records:

```text
WaterLabProfileV1 {
  W0F parent roots, W0G metric/corpus/execution roots,
  exact successor constants,
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
- Rebuild the inherited W0F sorted integer-cell grid each substep as
  `(cell_key, SampleId)` entries plus contiguous cell ranges. Do not use
  hash-map iteration in membership or a reduction.
- Use the W0F inclusive integer neighbor test, support-complete boundary,
  cubic kernel, projected active-set PCG density solve, divergence solve,
  swept analytical contact and quantized convergence branches exactly; no
  solver choice remains in W1.
- Apply divergence, gravity, density solve, position update, validation and
  publication in the frozen order.
- Disable warm start and every non-pressure optional model in V1.
- Quantize each accepted sample once with checked ties-to-even; discard the
  private float frame and decode the next step from canonical integers.

The serial oracle has no worker-count equality requirement. Insertion-order
permutations are sorted into the same sample/cell order and must produce the
same canonical trajectory root.

## Boundary and failure behavior

The W0F axis-aligned outer/internal manifest and reconstructible density
support are part of W1.
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
- one checked-in `SMOKE_ONLY` small analogue of each historical geometry, with a
  distinct scenario root and no corpus-credit claim;
- full W0F/W0G corpus at nominal scale with output artifacts outside Git;
- repeat runs on the same target/toolchain with identical canonical roots;
- analytical and SPlisHSPlasH aggregate comparison under the frozen metrics;
- preflight rejection when any W0F parent or W0G metric/corpus root differs;
- signed/absolute/excess/deficit threshold vectors and per-output stage-energy
  closure within `2,000 ppb`.

## Exit and stop rule

`CONTINUUM-WATER-REF-P1 = PASS` requires every W0F/W0G correctness threshold,
all required independent references and Linux same-target repeat/insertion
equality. Timing at `10k/50k/100k` is recorded but cannot fail W1 or be called
a production budget result.

Windows is outside the current W1 execution scope by user decision. Exact
cross-target comparison remains deferred and not waived for production
promotion.

Failure keeps the main roadmap inactive. The smallest failing scenario and
first causal metric become the next discriminator; implementation cannot move
to parallelism, PhysX or GPU to hide a serial correctness failure.

## Non-goals

Parallel execution, real-time claims, PhysX, moving bodies, public contracts,
save/restart, streaming, renderer integration and GPU.
