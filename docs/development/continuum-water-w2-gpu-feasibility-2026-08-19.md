# Continuum water W2 GPU feasibility discriminator — 2026-08-19

Status: `REPORT_ONLY / DIRECT_GRAPH_PORT_MISSES_4MS / GPU_ARCHITECTURE_UNDECIDED / NO_W2_CREDIT`

## Question

Can the current sealed-48k cold W0H accelerated projected-gradient method meet
the `4 ms` water budget if its dominant reconstruction and density-operator
work is moved from the CPU to the local NVIDIA GeForce RTX 3080?

This is a bounded feasibility question, not an authorization to move canonical
water authority to the GPU, change the frozen `f64` profile or select a new
solver. SPEC-38 and ADR-076 still make the CPU authoritative and the GPU an
optional correspondence mirror.

## Clean executable

Commit `01a4c1845363a024853a1acf0ab5058b16fac6fa` adds a standalone CUDA
executable under `crates/continuum-water/tools/gpu-feasibility`. It is not
linked into the Rust workspace, exposes no public API, owns no state and cannot
publish an accepted water frame or reaction batch.

The clean detached worktree was built with CMake `4.2.3`, Ninja `1.13.2` and
CUDA `13.3.73` for compute capability `8.6`. CUDA compilation uses `-O3`,
`--fmad=false`, `--prec-div=true`, `--prec-sqrt=true` and `--ftz=false`. The
binary SHA-256 is
`355bb816aaabf7c8316c2fb96cb616801c6a29491159259769fcb677884652dd`.

The executable constructs the exact initial sealed-lattice membership and edge
counts:

- `48,000` fluid samples;
- `24,704` W0F static boundary samples;
- `1,427,612` directed fluid edges;
- `73,904` directed boundary edges.

Reconstruction uses GPU radix-sort cell indexing, dense cell ranges, row
counts, exclusive scans and flattened SoA neighbor arrays. The solve proxy uses
a cold synthetic vector, a projected candidate, scale/unscale vector stages,
one pressure-acceleration traversal, one dependent matrix-action traversal and
four global `f64` reductions per APG iteration, followed by the final pressure
application.

Two profiles are measured:

- `f64`: graph, vector and reduction arithmetic use double precision;
- `mixed32`: graph and vector arithmetic use `f32`, while global reductions
  remain `f64`.

The self-test validates the declared sample and edge counts and a finite solve
checksum before measurement.

## Scope limits

The proxy deliberately omits density/diagonal and physical RHS assembly, the
data-dependent convergence/curvature decision, divergence, contact,
integration, canonical quantization/publication and the physical correctness
corpus. Its synthetic values do not prove numerical equivalence. GPU reduction
and neighbor order are not compared to W1 roots.

Therefore these timings are an optimistic dominant-shape discriminator for a
direct graph port, not a full substep measurement and not a ProductCheck. A
future optimized implementation could use another graph layout or algorithm;
this report does not reject that broader design space.

Windows is out of scope by user decision. The GPU was not an exclusive host:
unrelated idle processes retained about `6.58 GiB` of framebuffer memory, but
preflight showed no competing compute workload and the measured minima and
percentiles are stable.

## Clean 40-iteration result

The fixed invocation is three warm-ups and 50 measured runs at 40 APG
iterations. Its JSON report is outside Git at
`/tmp/nextengine-continuum-water-gpu-01a4c18-40iter-50runs.json`, SHA-256
`76da2d989289912fc58bf6c058e15b52a573bd6d9c71f7dbab0541950a198001`.

| Profile and stage | Minimum | Median | p95 | Mean |
| --- | ---: | ---: | ---: | ---: |
| `f64` reconstruction | `1.864 ms` | `1.900 ms` | `2.035 ms` | `1.917 ms` |
| `f64` cold APG40 solve | `34.612 ms` | `34.886 ms` | `35.409 ms` | `34.944 ms` |
| **`f64` proxy total** | **`36.519 ms`** | **`36.845 ms`** | **`37.398 ms`** | **`36.861 ms`** |
| `mixed32` reconstruction | `1.313 ms` | `1.346 ms` | `1.401 ms` | `1.353 ms` |
| `mixed32` cold APG40 solve | `16.017 ms` | `16.355 ms` | `16.632 ms` | `16.353 ms` |
| **`mixed32` proxy total** | **`17.363 ms`** | **`17.720 ms`** | **`18.015 ms`** | **`17.706 ms`** |

The `f64` proxy p95 is `9.35×` the complete `4 ms` budget. The mixed proxy p95
is `4.50×` the budget, and even its best observed total is `4.34×` the budget.
Since omitted mandatory stages require nonzero time, neither direct profile can
close W2 as measured.

For scale only, comparing non-equivalent short means to the prior 8-worker CPU
profile suggests about `40×/57×` faster reconstruction and `4.35×/9.30×`
faster density-shaped work for `f64/mixed32`. The combined CPU reconstruction
plus density mean was `229.592 ms`; the GPU proxy means are `36.861 ms` and
`17.706 ms`, approximately `6.23×` and `12.97×` faster. These are useful
engineering ratios, not end-to-end or correctness credit.

## Resource use and kernel diagnosis

An adjacent 50-run utilization observation of the same clean binary reports
three consecutive active samples at `100%` SM utilization, `68–93%` memory
utilization, `1,950 MHz` SM clock and up to `265 W`. The direct port is using
the device; its miss is not explained by leaving most of the RTX 3080 idle.

Nsight Systems `2026.1.3` profiled one warm-up and one measured five-iteration
run. The report is
`/tmp/nextengine-continuum-water-gpu-01a4c18.nsys-rep`, SHA-256
`4c5b54b9f9f068388a64ed13affbc12578687d6b5935fc1a5787a864977380a1`.
Its GPU kernel summary shows:

- pressure acceleration plus matrix action consume approximately `4.59 ms`
  per five-iteration `f64` solve and `2.09 ms` per mixed solve;
- four CUB reductions per iteration consume only about `0.10 ms` per solve;
- fluid and boundary neighbor filling account for about `86%` of `f64`
  reconstruction and `81%` of mixed reconstruction.

The leading boundary is therefore repeated, dependent traversal of the
1.50-million-edge graph, with a second penalty from double precision on this
consumer GPU. Kernel-launch and reduction optimization alone cannot provide
the required `4.5–9.35×` total reduction. Nsight Compute hardware counters were
not collected because the unprivileged host returns `ERR_NVGPUCTRPERM`; no
system setting was changed.

At the measured mixed per-iteration slope, fitting just this proxy into `4 ms`
would require roughly six APG iterations rather than 40, or a comparable
operator-throughput improvement, while still leaving almost no budget for the
omitted stages. That is an algorithm/data-layout change, not a backend-only
port.

## Decision boundary

This discriminator rejects one narrow hypothesis:

> A straightforward GPU implementation of the current flattened-neighbor W0H
> APG shape on the RTX 3080 is sufficient to meet the `4 ms` water target.

It does not select `mixed32`, GPU authority, CUDA graphs, a persistent kernel,
neighbor caching, a different pressure method or a different fluid solver.
`mixed32` cannot inherit the frozen `f64` W0H/W1 roots, and even `f64` GPU
execution changes reduction order and has no canonical correspondence proof.
Under current architecture a faster GPU mirror also cannot replace the CPU
authoritative latency.

The next decision remains explicit: stop the track at `RESEARCH_ONLY`, or
authorize a new rooted algorithm/data-layout/precision and authority profile.
If research continues, the highest-value discriminator is iteration/operator
work reduction with a declared correctness boundary; launch-only tuning is not
the leading cause.
