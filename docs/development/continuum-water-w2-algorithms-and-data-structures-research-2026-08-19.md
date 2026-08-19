# Continuum water W2 algorithms and data structures research — 2026-08-19

Status: `DRAFT_RESEARCH / OPTIONS_NOT_SELECTED / NO_W2_CREDIT`

## Interim research summary

The measured `269.643 ms` sealed-48k step cannot reach the `4 ms` target by
worker scheduling alone. The useful candidates split into two different
tracks which must not share an evidence claim:

1. **Exact DFSPH data-layout probe.** Keep every frozen W0F/G/H equation,
   pair order, reduction order and published root. Replace avoidable searches,
   allocations and oversized records with radix/counting construction, prefix
   sums, CSR/SoA storage and persistent workspaces. This should be performed as
   a bounded calibration experiment, not represented as a credible route to
   `4 ms`.
2. **New particle-grid profile.** Prototype deterministic FLIP pressure
   projection on a regular staggered grid with geometric multigrid. Keep
   particles for topology, advection and publication, but move the global
   incompressibility solve from the irregular particle graph to a fixed-stencil
   grid. This is the only researched CPU candidate with an architectural path
   to the required order-of-magnitude reduction. It changes the numerical
   model and therefore requires a superseding Proposed SPEC-38/ADR-076
   decision, a new profile, roots and correctness corpus; it cannot inherit W1
   credit.

A multilevel active-set preconditioner for the existing pressure QP is a
valuable intermediate research candidate. It is closer to current semantics
than FLIP, but published results from other particle/grid discretizations do
not support claiming the approximately `67×` reduction required here.

## Measured problem, not a generic SPH problem

The clean short W2 matrix attributes the best 8-worker step as follows:

| Component | Mean | Relevant property |
| --- | ---: | --- |
| neighbor reconstruction | `77.473 ms` | grid lookup, admission, row ordering and kernel records |
| density solve | `152.119 ms` | `36–40` cold APG iterations and repeated neighbor sweeps |
| complete step | `269.643 ms` | only `372%` whole-command CPU on 16C/32T THOTH |

The current reconstruction queries 27 cells independently for fluid and
boundary supports. Each cell lookup uses two binary partition searches in a
sorted entry array, so one fluid row can perform up to `108` binary searches
before sorting/deduplicating admitted sample IDs. The pressure path then walks
the neighbor graph twice per APG iteration, separated by a dependency barrier,
and materializes several full `48k` vectors. Neighbor records contain fields
which are not consumed together, including `usize` indices and vector-valued
gradients. These are concrete data-structure opportunities; none removes the
global iteration count by itself.

The original DFSPH work computes neighborhoods, density and the diagonal
factor once per step, uses parallel Jacobi iterations and recommends warm
starting. Its reported `4 ms`-step examples average about `4.5` density and
`2.8` divergence iterations, and warm starting yielded about a threefold
speedup in its tests. Those numbers explain why a cold `36–40` iteration
profile is structurally expensive, but they are not performance predictions
for this implementation. Source: [Bender and Koschier, Divergence-Free SPH
(2017)](https://dankoschier.github.io/resources/papers/BK17.pdf).

## Track A — exact algorithms and layouts for the frozen DFSPH profile

These candidates can in principle preserve the present roots. Every candidate
must still prove serial equality and worker `1/2/4/8` equality; “same neighbor
set” is insufficient if it changes floating-point pair order.

| Priority | Candidate | Replaces | Root constraint | Expected role |
| --- | --- | --- | --- | --- |
| A1 | persistent ping-pong workspace | per-iteration `Vec` allocation/zeroing | index-identical writes; same scalar operation order | remove allocator traffic and full-buffer churn |
| A2 | packed cell key + stable radix/counting sort | comparison sort of grid entries | secondary key remains ascending `SampleId` | linear-time grid construction with regular memory access |
| A3 | sparse cell directory in CSR form | up to 108 row-local binary searches | exact 27-cell visit order retained | O(1) range lookup after prefix construction |
| A4 | SoA pressure adjacency with `u32` indices | wide AoS neighbor records | same row and neighbor ordinal | lower bandwidth; expose unit-stride scalar streams |
| A5 | k-way merge of sorted cell runs | per-row general sort/dedup | output must be ascending `SampleId` | exploit at most 27 already-sorted runs |
| A6 | deterministic Verlet/skin cache | full grid rebuild every substep | exact-distance refilter and canonical order every step | amortize reconstruction only when motion is small |
| A7 | delta-compressed neighbor IDs | raw adjacency indices | lossless decode to canonical ordinal | trade decode ALU for memory bandwidth; benchmark first |
| A8 | prefix-weighted logical partitions | equal row-count partitions | fixed boundaries derived from canonical row costs | improve load balance without dynamic output order |

### A1 — persistent solver workspace

Allocate all APG vectors once after capacity admission and reuse them as
ping-pong buffers. Express pressure acceleration and matrix action over
preallocated output slices. Do not fuse the two neighbor passes: the second
pass consumes the completed first-pass vector, so a fusion would change the
operator. This candidate is low risk and should be implemented before exotic
compression because it also makes later bandwidth measurements interpretable.

### A2/A3 — radix-built sparse cell CSR

The competitive-programming construction is:

1. bias and pack bounded integer cell coordinates into an integer key;
2. stable radix-sort `(cell_key, SampleId)` or counting-sort bounded key
   digits;
3. mark every key transition;
4. exclusive-prefix-sum the marks into a compact cell directory;
5. query the 27 prescribed keys through a deterministic fixed-capacity lookup.

A dense array for the entire global coordinate allowance would waste memory.
Use either a basin-bounded dense directory after explicit preflight, or a
sparse sorted directory plus deterministic open addressing. The hash table is
only a key-to-range index: iteration never follows hash-table order. This
retains canonical cell and `SampleId` ordering while avoiding two
`partition_point` calls per visited cell.

The construction follows the fixed-radius-neighbor pattern of sorting by cell
and building range offsets with scans. Hoetzlein presents a counting/radix
variant and SoA layout, although its measured implementation is GPU-oriented
and cannot establish our CPU gain: [Fast Fixed-Radius Nearest Neighbors
(2014)](https://ramakarl.com/pdfs/2014_Hoetzlein_Fast_Neighbors.pdf).

### A4 — SoA/CSR adjacency

Use `u32` for sample/boundary ordinals under the frozen `50k`/`32,768`
capacities, and store pressure-consumed fields separately:

```text
row_offsets: [u32; particle_count + 1]
other:       [u32; directed_edge_count]
grad_x/y/z:  [f64; directed_edge_count]
kernel:      [f64; directed_edge_count]  // only for stages that need it
```

This is the graph representation known in competitive programming as CSR or a
packed forward-star. It removes repeated per-edge `usize` and lets pressure
sweeps read only their required columns. A CPU SPH study found that changing
AoS to SoA reduced cache misses and improved its scalar kernels, but the
reported gains vary strongly by architecture and its full `2.6×/4.8×`
results combine more than layout alone. Treat it as directional evidence:
[Efficient and Scalable Implementation of Truly Incompressible SPH on CPU
(2016)](https://arxiv.org/abs/1612.06090).

Compressed neighbor lists can save substantial memory by Morton ordering and
delta coding, but decoding may lose on a graph revisited dozens of times. The
published method reports up to `87%` neighbor-list memory savings; it does not
prove lower latency for our APG loop: [Compressed Neighbour Lists for SPH
(2019)](https://diglib.eg.org/items/2bbc0b3f-e865-4792-81fc-65edd8b2f2cc).

### A5 — bounded k-way merge

If entries inside each cell are already ordered by `SampleId`, merge the 27
runs with a small fixed min-heap or loser tree and discard equal IDs at the
output boundary. This changes `O(k log k)` general sorting of at most 128
neighbors to `O(k log 27)` with predictable storage. A fixed-size radix sort
is an alternative. Both need a microbenchmark: for such small rows, Rust's
standard sort may still win.

### A6/A7 — conditional candidates

A Verlet cache may keep a larger neighbor set for several steps, but every
use must refilter the exact support radius and emit the same ascending IDs.
The rebuild decision must derive only from canonical positions and fixed
profile constants; a save/restart cold rebuild must reproduce the same frame
root. It can remove at most the reconstruction share and therefore cannot
close the total target.

Delta IDs, varints and Morton/Z ordering improve locality only if decode cost
is lower than the avoided memory traffic. Morton order may schedule storage or
partitions; it must never reorder a floating-point row reduction. An
undirected half-edge representation is deferred: deterministic parallel
scatter would require edge coloring or contribution buffers followed by
stable segmented reduction, likely replacing saved memory with synchronization.

### A8 — resource utilization and SIMD boundary

The current 64 logical partitions contain nearly equal row counts, not equal
neighbor work. Build a canonical prefix sum of `fluid_degree + solid_degree`
and place fixed partition cuts with lower-bound searches near equal cumulative
weights. This is the classic linear-partition/prefix-sum technique. Completion
may occur in any order because each partition still writes disjoint canonical
row ranges; error selection and final reduction retain ordinal order. Compare
per-partition edge counts and elapsed tails before keeping it.

Work stealing cannot remove the dependency barrier between pressure
acceleration and matrix action. Parallelizing convergence sums with an
ordinary reduction tree would also change floating-point order. A reproducible
superaccumulator or rooted fixed reduction tree is possible only as a new
numeric profile; for the exact probe, keep global scalar reductions canonical.

The exact build disables SSE3/SSSE3/SSE4/AVX/AVX2/FMA, leaving only the
baseline x86-64 feature set. SoA may still let LLVM process independent rows
with baseline SIMD, but `checked_scalar` after each term and indirect neighbor
loads are likely to inhibit it. Inspect generated assembly and compare roots;
do not infer vectorization from source layout. Enabling AVX2 or FMA is a
separate cross-target numeric/codegen decision, not an invisible optimization.

## Competitive-programming toolbox: applicability

| Technique | Use here | Verdict |
| --- | --- | --- |
| radix/counting sort | grid keys and fixed-width sample IDs | **use** |
| prefix sums / scan | cell offsets, row offsets, active-row compaction | **use** |
| CSR / forward-star | immutable directed neighbor graph | **use** |
| coordinate compression | sparse occupied cells within bounded world coordinates | **use** |
| deterministic open addressing | cell-key to CSR-range lookup only | **use with fixed probing/load factor** |
| k-way merge / loser tree | merge 27 sorted cell runs | **microbenchmark** |
| bitset + rank/select prefix | compact active pressure rows while retaining row order | **measure active fraction first** |
| offline sort-unique | deterministic construction and diagnostics | **use outside hot iteration** |
| double buffering | APG vectors and partition fragments | **use** |
| DSU | connectivity components | **not on the hot path**; pressure still couples each component globally |
| Fenwick/segment tree | dynamic range aggregation | **reject**; no matching range query |
| kd-tree / sweep line | general spatial search | **reject** for fixed-radius, nearly uniform 3D samples |
| generic unordered hash iteration | neighbor emission | **reject**; unstable order and poor locality |
| dynamic graph coloring | conflict-free edge updates | **defer**; construction/barrier cost and exact-order burden |

Bitset active compaction is safe only if inactive rows demonstrably dominate.
The pressure operator also depends on the other endpoint, so simply skipping
zero-pressure particles can omit required terms. The discriminator must first
record active-row and active-edge fractions over the corpus.

## Track B — change the pressure solver, retain particle discretization

### B1 — warm-started DFSPH

Warm starting has the strongest DFSPH-specific published evidence for reducing
iteration count. It is nevertheless incompatible with the current contract:
W0H explicitly freezes cold APG, and V1 authority does not persist pressure.
A hidden cache would make cold restore and warm continuation diverge. An
admissible version must publish/version the continuation state or prove a
canonical reconstruction; either choice is a new profile and new roots.

### B2 — multilevel active-set preconditioning

The present non-negative pressure problem is a box-constrained convex QP. A
multilevel active-set preconditioner combined with MPRGP has been demonstrated
specifically for pressure Poisson problems with non-negative constraints:
[Takahashi and Batty, A Multilevel Active-Set Preconditioner for
Box-Constrained Pressure Poisson Solvers
(2022)](https://diglib.eg.org/items/1db25776-26f1-4397-a27b-277cc49863bf).
It is a materially better match than unconstrained CG or generic AMG.

For a particle PPE, plain-aggregation AMG used as a BiCGSTAB preconditioner
also outperformed unpreconditioned BiCGSTAB especially at larger problem
sizes, but that evidence is for MPS rather than our DFSPH QP:
[Matsunaga et al., Solution of pressure Poisson equation in particle method
using algebraic multigrid (2016)](https://www.jstage.jst.go.jp/article/jsces/2016/0/2016_20160012/_article/-char/en).

The research prototype would coarsen the canonical particle graph, root every
aggregation/tie rule, retain projected/KKT termination, and rebuild the
hierarchy from canonical state. It should be rejected early unless it reduces
operator applications by at least an order of magnitude without increasing
setup enough to erase the gain. No cited result justifies promising that.

### B3 — solver swaps not recommended

PCISPH, IISPH and PBF still perform iterative neighborhood work and change
error/energy behavior. The DFSPH paper's own comparisons do not identify them
as a faster replacement at comparable incompressibility goals. PBF is useful
for visually plausible real-time liquid but changes constraint semantics:
[Position Based Fluids (2013)](https://matthias-research.github.io/pages/publications/pbf_sig_preprint.pdf).
They are not the next candidate for this product contract.

## Track C — particle-grid representation

### C1 — deterministic FLIP + geometric multigrid

FLIP keeps particles for material topology and advection, transfers velocity
to a regular staggered grid, projects grid velocity to incompressibility, and
transfers the velocity change back to particles. The pressure matrix then has
a compact fixed stencil over roughly the active grid cells, rather than two
indirect irregular-neighbor sweeps for each of `36–40` iterations. A geometric
V-cycle has naturally nested levels, bounded work per level and regular SoA
memory access.

This direction is established by the particle-grid free-surface method of
[Zhu and Bridson (2005)](https://doi.org/10.1145/1073204.1073298). APIC adds an
affine velocity representation to reduce PIC dissipation/noise and conserve
angular momentum: [Jiang et al., The Affine Particle-In-Cell Method
(2015)](https://doi.org/10.1145/2766996). APIC's affine matrix per particle
would expand canonical state unless deterministically reconstructed, so the
first discriminator should use a frozen FLIP/PIC transfer rule and add APIC
only after state semantics are closed.

An IISPH-FLIP hybrid reports a performance factor of seven over standard SPH
at equal particle count while reducing memory and maintaining small density
deviation: [Cornelis et al., IISPH-FLIP for Incompressible Fluids
(2014)](https://diglib.eg.org/items/856a8b1a-b8e0-468f-901c-1c19ed6bc8ce).
Even applying that factor naively to `269.643 ms` leaves about `38.5 ms`, so it
supports the direction but not the target claim. Regular-grid multigrid, not
hybridization alone, is the performance hypothesis to test.

The new profile must close free-surface cell classification, solid cut-cell or
face weights, pressure null-space handling, particle-to-grid/grid-to-particle
transfer, particle reseeding policy, crate reaction extraction and exact
publication. Hydrostatic, dam-break, orifice, hard-clearance and energy gates
must be rerun under new roots.

### C2 — shallow-water/height-field alternative

A 2D finite-volume shallow-water solver on a regular grid is much more likely
to fit `4 ms`, using tiled arrays, fixed stencils and multigrid or tridiagonal
subsolves. It cannot represent the currently required 3D dam break, submerged
crate, aperture jet, overturning splash and disconnected water volumes. This
is a valid product simplification only if the consumer contract changes from
volumetric water to a basin-surface model; it is not an implementation
optimization.

## Ranked experiments and stop gates

### E1 — exact-layout calibration probe

Implement in order: A1 persistent workspace, A2/A3 radix-built cell CSR, then
A4 `u32` SoA adjacency. Keep each change separately measurable and root exact.
Run only the short serial/`1/2/4/8` profile. Stop this route as a performance
solution if the combined 8-worker mean remains above `120 ms`; that threshold
still misses production by `30×` but prevents more low-leverage tuning. Retain
useful cleanups only when they reduce time without making proofs or memory
caps worse.

### E2 — particle-grid feasibility kernel

First write a short Proposed decision that explicitly supersedes SPEC-38 and
ADR-076's “CPU DFSPH is the sole canonical candidate” clause, and update the
routing traceability in the same change. Then create a separate profile and
prototype, not a conditional branch in the DFSPH oracle. The minimal kernel
includes particle/grid transfer, fixed solid basin faces, pressure projection
with deterministic geometric V-cycles, back-transfer and canonical particle
publication. Exclude renderer and PhysX.

Predeclare these discriminator gates:

- identical roots for worker counts `1/2/4/8` and repeated cold runs;
- mass, hydrostatic column, dam-break front and aperture-flow aggregate gates;
- hard basin clearance with no forbidden state;
- a sealed-48k one-step mean `<= 8 ms` before funding full corpus reclosure;
- bounded memory and no wall-clock-dependent iteration or convergence rule.

The `8 ms` gate is not production PASS. It is a feasibility cutoff: a minimal
kernel slower than twice the final standalone budget is unlikely to absorb
cut cells, reaction extraction, diagnostics and publication later.

### E3 — optional multilevel-QP spike

Run B2 only if preserving particle-pressure semantics has higher product value
than reaching the schedule quickly. Compare setup plus solve against cold APG,
record operator counts and exact KKT result, and stop unless one step is below
`40 ms` with a plausible path to single-digit milliseconds. Do not mutate W0H
or reuse W1 roots during the spike.

## Preliminary recommendation — not selected

The current research leans toward E1 as one bounded engineering probe and E2
as the primary redesign experiment. This is not an accepted choice and does
not authorize implementation. E1 would tell us how much of the current cost
is representation overhead and leave a faster exact oracle; E2 would test the
only researched CPU architecture with regular enough work to plausibly
approach the target. E3 remains an explicitly time-boxed
semantic-preservation fallback. If E2 were selected and missed its `8 ms`
discriminator, the next discussion would return to the product decision:
lower spatial/cadence requirements, accept a height-field model, authorize GPU
authority, or keep volumetric water `RESEARCH_ONLY`.

No performance factor in this report is additive, and none of the external
benchmarks is directly comparable to the frozen THOTH fixture. Only local
clean measurements can promote a candidate.
