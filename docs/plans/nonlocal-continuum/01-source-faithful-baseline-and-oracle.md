# NR1 — Source-faithful baseline and independent oracle

Status: `EXECUTED / BASELINE_MISMATCH / REPORT_ONLY`

Execution evidence: [NR1 baseline report](../../development/nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md).

## Outcome

Build the smallest standalone implementation that reproduces the published
Nonlocal/SISSM energy terms and fixed-iteration update closely enough to locate
cost and compare later optimizations. It must not inherit PeriDyno framework
state, UI/render plugins, scene loaders or runtime ownership.

NR1 produces evidence, not a reusable engine API.

## Executable boundary

The implementation lives under
`crates/continuum-water/tools/nonlocal-feasibility` and builds into an external
directory. It contains:

- a CPU `f64` oracle with direct, readable pair and per-particle loops;
- a source-shaped CUDA `f32` baseline;
- deterministic fixture generation from explicit profile constants;
- bounded self-test and JSON-report commands;
- no Rust/Cargo workspace linkage and no call into production water code.

The CPU and CUDA implementations must not call a shared pair-contribution
function. They may share serialized fixture inputs and report schemas. This
keeps the oracle independent enough to detect sign, indexing and transpose
errors.

PeriDyno may be built externally as a secondary behavioral reference if its
Linux dependencies permit, but inability to build the full framework does not
block the standalone baseline. No upstream checkout or build artifact enters
the repository.

## Mathematical scope

Implement only the equations required for:

1. predicted positions from current position/velocity and inertia;
2. summation density over the fixed neighborhood;
3. incompressibility contribution;
4. bulk and shear viscosity contributions;
5. surface-tension contribution;
6. per-particle source and local `3x3` system accumulation;
7. one SISSM position update and final velocity reconstruction.

Boundary handling is one explicitly profiled analytical box or the exact
source-example boundary representation selected in the fixture closure. NR1
does not implement general meshes, moving rigid bodies, emitters, topology,
adaptivity or inter-material interfaces.

Every equation in code receives a comment naming the paper equation/section or
the exact upstream file/function used only as an interpretation aid. A code
choice absent from both sources is marked `NEXTENGINE_RESEARCH_CHOICE` and
listed in the report.

## Fixture closure before performance

The first implementation commit freezes a machine-readable profile record in
the executable containing:

- profile ID and version;
- particle lattice/geometry and stable initialization order;
- spacing, mass, support/horizon and rest density;
- time step, gravity and boundary constants;
- `lambda`, `mu`, `kappa`, `gamma` and enabled terms;
- fixed iteration count;
- maximum samples, neighbors and directed pairs;
- CPU/GPU numeric modes and comparison tolerances;
- paper DOI, PDF hash and PeriDyno commit.

The record is printed by `--describe-profile`. A report with a different record
hash cannot be combined with another profile's timing or correctness claim.

## Oracle matrix

The self-test covers at least:

| Case | Purpose |
|---|---|
| isolated particle | zero-neighbor/zero-term behavior and finite local solve |
| symmetric pair | equal-and-opposite source/momentum behavior |
| collinear triplet | rank-deficient/local-matrix handling |
| tetrahedral neighborhood | full `3x3` off-diagonal contribution |
| uniform interior block | near-rest density and translational symmetry |
| free surface patch | missing-neighbor and surface-term sign |
| viscosity shear pair | shear versus bulk component separation |
| tiny closed box | one through four complete fixed SISSM iterations |

For each case the CPU oracle emits density, energy components, source vector,
local matrix, next position and final velocity in stable particle/component
order. CUDA output is copied only after the complete fixed iteration and
compared element-wise under the frozen absolute-plus-relative tolerance.

The report also checks total source/impulse closure and rejects nonfinite
values before aggregation. A singular or ill-conditioned local system follows
one explicit published/upstream rule; silent regularization is forbidden.

## Source-shaped CUDA baseline

The baseline intentionally preserves the observable implementation shape found
in the inspected reference:

- one fixed neighbor list reused for all SISSM iterations;
- density recomputed once per iteration;
- distinct accumulation passes for enabled energy terms;
- full per-particle `3x3` local matrix;
- fixed iteration count with no convergence exit;
- symmetric reverse contributions accumulated through atomics;
- explicit old/new position handoff between iterations.

Buffers are admitted once for the maximum profile but the baseline records any
logical reset/copy matching the source algorithm. Framework-only allocations,
GUI state and scene plumbing are excluded. This produces a fair algorithmic
baseline without benchmarking PeriDyno's unrelated framework.

## Required commands

The finished tool exposes equivalent bounded commands:

```text
nonlocal-feasibility --describe-profile <profile>
nonlocal-feasibility --self-test
nonlocal-feasibility --check <profile> --iterations <fixed-count>
nonlocal-feasibility --benchmark <profile> --warmup 5 --runs 50
```

Commands fail nonzero on invalid profile, capacity excess, oracle mismatch,
nonfinite state, CUDA error or incomplete report. Benchmark mode performs its
self-test preflight and does not continue after a failed measured run.

## NR1 evidence report

The bounded checked-in summary records:

- exact source/profile/tool hashes;
- compiler/CUDA/device identity;
- self-test status and maximum absolute/relative discrepancy by field;
- sample, pair and neighbor-degree counts;
- fixed iteration stage times and device memory;
- one profiler attribution summary and external artifact SHA-256;
- deviations from paper/upstream behavior;
- status `BASELINE_REPRODUCED`, `BASELINE_MISMATCH` or
  `BASELINE_UNAVAILABLE`.

Raw JSON, profiler captures and binaries stay outside Git.

## Exit and stop rules

NR1 exits `BASELINE_REPRODUCED` only when all oracle cases pass and repeated
benchmark controls have identical input/profile hashes. Absolute performance
does not need to pass at NR1; it establishes the denominator for NR2.

Stop as `BASELINE_MISMATCH` after two coherent formula/indexing remediation
cycles if the same oracle class still fails. Do not tune coefficients or widen
tolerances to make the implementation agree. Stop as `BASELINE_UNAVAILABLE`
if the declared CUDA host/toolchain cannot build or execute the minimal tool.

## Non-goals

Early exit, line search, pair coloring, unique-pair semantics, mixed precision,
kernel fusion, CUDA Graphs, production boundaries and comparison with visual
paper figures. Those would obscure the source-shaped baseline.
