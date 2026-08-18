# W1 — Serial CPU DFSPH oracle

Status: `LINUX_W1_PASS / CONTINUUM-WATER-REF-P1=PASS / RESEARCH_ONLY`.

The original W0B oracle remains rejected-profile evidence. W0E selected the
constraint-separated local survivor, W0F froze its solver, geometry,
capacities and successor roots, and W0G reclosed impact-energy semantics after
the first full dam-break discriminator. W0H then replaces the inherited
globally restarting active-set PCG with one fixed accelerated projected-
gradient schedule over the same pressure QP. The current safe-Rust runner
verifies that complete root chain before executing any nominal scenario.

Linux hydro and exact free-fall pass internally. The unchanged W0F dam-break
also completes `720/720` under W0G with zero penetration, bounded solver and
momentum metrics, zero positive energy excess and an explicitly published
`638,407,503 ppb` mechanical-energy deficit. See the
[successor energy discriminator](../../development/continuum-water-w1-successor-energy-discriminator-2026-08-18.md).
The W0H candidate passes the complete internal seven-scenario discriminator,
including repeated sealed-48k and storage-order roots. W0I subsequently
rejects the old geometry-violating external curves, freezes independently
generated hard-clearance hydro/dam-break/orifice file hashes and makes exact
reference attestation a production-credit preflight. The unchanged W0H
dam-break and orifice curves pass the frozen aggregate thresholds.

At clean commit `e00999e96f0f55ae02426e806457f625d0a4844f`, the complete
seven-scenario Linux corpus passed twice with all three required external
hashes attested. Both runs reproduce corpus root
`d38d6bc8a8e98e87402202a926685dbe4867e3de6a7d8679362885be46e96835`;
their normalized reports are exactly equal after removing only diagnostic
wall-clock fields. W1 therefore closes
`CONTINUUM-WATER-REF-P1=PASS / LINUX_W1_PASS / RESEARCH_ONLY`. Detailed report
and scenario roots are in the
[hard-clearance reference evidence](../../development/continuum-water-w1-hard-clearance-reference-reclosure-2026-08-18.md).

## Outcome

Implement a safe-Rust, serial, runtime-independent DFSPH oracle that executes
the hash-frozen [W0F](00f-geometry-capacity-and-root-closure.md) formulation,
[W0G](00g-impact-energy-contract-reclosure.md) metric contract,
[W0H](00h-accelerated-pressure-profile-reclosure.md) pressure algorithm,
[W0I](00i-external-reference-geometry-attestation.md) external attestation and
nominal corpus, publishes a canonical fixed-point frame after every substep
and produces bounded typed evidence. It is not a production physics backend.

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
  W0F parent roots, W0G metric roots, W0H solver/corpus/execution roots,
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
  cubic kernel, pressure operator, divergence solve and swept analytical
  contact exactly. Solve the non-negative pressure QP with W0H's fixed
  diagonal scaling, step `0.25`, momentum schedule, projection, majorization
  guard and ascending reductions; no solver choice remains in W1.
- Accept density only when mean positive compression and mean projected-KKT
  residual are both at most `100,000 ppb` within `2..=50` exact pressure-
  operator applications.
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
- full W0F/W0G/W0H corpus at nominal scale with output artifacts outside Git;
- repeat runs on the same target/toolchain with identical canonical roots;
- analytical and SPlisHSPlasH aggregate comparison under the frozen metrics;
- fail-closed exact SHA-256 attestation of all three W0I external files before
  a production-credit trajectory starts;
- preflight rejection when any W0F parent, W0G metric or W0H
  solver/corpus/execution root differs;
- exact production/independent W0H first-step equality, zero-diagonal residual
  retention and fixed-step majorization failure coverage;
- signed/absolute/excess/deficit threshold vectors and per-output stage-energy
  closure within `2,000 ppb`.

## Exit and stop rule

`CONTINUUM-WATER-REF-P1 = PASS` requires every W0F/W0G/W0H correctness
threshold, all required W0I-attested independent references and Linux same-target
repeat/insertion equality. Timing at `10k/50k/100k` is recorded but cannot
fail W1 or be called a production budget result.

That exit is satisfied by the two clean runs at commit `e00999e`. It authorizes
W2 inside the isolated research track; it does not select a production backend
or establish the W2 standalone-performance, W3 coupling, W5 persistence or W6
promotion gates.

Windows is outside the current W1 execution scope by user decision. Exact
cross-target comparison remains deferred and not waived for production
promotion.

The main roadmap remains `PLANNED / NOT_ACTIVE` until this evidence checkpoint
is explicitly merged and the track is activated. The next bounded stage is W2
deterministic parallel equality and standalone performance; PhysX and GPU
cannot bypass that stage.

## Non-goals

Parallel execution, real-time claims, PhysX, moving bodies, public contracts,
save/restart, streaming, renderer integration and GPU.
