# Nonlocal accumulation reclosure research — 2026-08-19

Status: `RESEARCH_COMPLETE / GATHER_DIRECTED_SELECTED / IMPLEMENTED / NR1_RECLOSED`

Execution result:
[NR1-RC1 reclosure evidence](nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md)
records every ordered gate PASS and unblocks NR2 at O1.

## Question

What is the smallest change that can test whether NR1's stiff-surface mismatch
comes from run-dependent GPU accumulation order rather than the published
Nonlocal/SISSM equations, without weakening the oracle, changing the physical
workload or prematurely starting performance optimization?

## Evidence already established

The [NR1 evidence](nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md)
separates two facts:

- the independent CPU `f64` and CUDA `f32` tiny oracle passes all eleven named
  fixtures, including the surface pair function and one through four complete
  iterations;
- `source-atomic-v0` first exceeds repeated state tolerances on the 16k stiff
  surface control at iteration two while profile/input hashes and both CSR
  neighbor arrays remain byte-identical.

The current CUDA kernels assign one thread to every source particle. Each
thread reduces its local endpoint in a stable loop but scatters the reverse
endpoint with `atomicAdd`. Different blocks can therefore add to one target in
a hardware-scheduled order. This is a meaningful distinction for `f32`:
[NVIDIA's floating-point guide](https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/mathematical-functions.html)
states that floating-point addition is not associative and that changing the
operation order changes rounding.

The distinction is also explicit in NVIDIA's reduction library. The installed
CUDA `13.3` / CCCL `3.3.4` headers and the current
[CUB `DeviceReduce` documentation](https://nvidia.github.io/cccl/unstable/cub/api/structcub_1_1DeviceReduce.html)
separate a fixed-order run-to-run path from a faster `not_guaranteed` atomic
path whose order may differ between runs. CUB's
[determinism contract](https://nvidia.github.io/cccl/unstable/cccl/determinism.html)
limits `run_to_run` to the same input, build and GPU; tuning and architecture
may change the reduction tree. This supports the order hypothesis but does not
prove that every atomic surface workload must diverge.

The host-side documentation identity used for that check is:

| Item | Identity |
|---|---|
| CUDA compiler/toolkit | `13.3.73` / `13.3` |
| CCCL/CUB headers | `3.3.4` |
| `cub/device/device_reduce.cuh` | SHA-256 `d9747d8ce73d19ca8eba020651d4f7f58c9cc23bbeff8b5261852f525d0c9b99` |
| `cub/device/device_segmented_reduce.cuh` | SHA-256 `451bcda6f44eba60833fc7b1c357892b38d9c4a1cc4fdc705fb0068655a874c9` |

The installed segmented header defaults to same-GPU run-to-run determinism
and explicitly rejects `gpu_to_gpu` determinism. RC1 therefore does not depend
on a stronger CUB guarantee than the active toolchain provides.

The research-group publication page was checked again on 2026-08-19.
[Semi-Implicit Pairwise Descent for Nonlocal Continuum Mechanics](https://peridynamics.com/publications.html)
still lists both paper and code as `to appear`. Its title is not an admissible
source for a replacement nonlinear method.

## Exact accumulation identity

Let `E` be the frozen symmetric directed neighbor relation, excluding the
self entry skipped by the radius guard. For directed edge `i -> j`, let
`L_ij` be the contribution written to source particle `i`, `R_ij` the reverse
contribution written to `j`, and `K^L_ij`, `K^R_ij` their local-matrix parts.
The final source-shaped scatter value at owner `i` is, in real arithmetic,

```text
A_i = sum_(i,j in E) L_ij + sum_(j,i in E) R_ji.
```

Because the measured CSR is symmetric, one owner thread can instead traverse
`N(i)` in its existing stable order and compute

```text
G_i = sum_(j in N(i)) (L_ij + R_ji),
```

and the equivalent matrix expression. Thus `A_i = G_i` in real arithmetic.
The change selects a defined `f32` association rather than an arrival-defined
one; it is not expected to be bit-identical to `source-atomic-v0`.

The term-specific incoming contribution is closed as follows:

| Term | Owner `i` must gather from neighbor `j` | Important constraint |
|---|---|---|
| incompressibility | outgoing `L_ij(c_i)` plus incoming `R_ji(c_j)` | the incoming term uses neighbor density ratio `c_j`; reusing `c_i` changes the equation |
| bulk/shear viscosity | outgoing `L_ij` plus incoming `R_ji`; the pair matrix is invariant under reference-direction sign | the first implementation evaluates both named endpoint expressions rather than assuming a factor of two |
| surface tension | outgoing `L_ij` plus incoming `R_ji`; radial coefficients are symmetric | retain the default bidirectional pair function and both source-shaped directed contributions |

Using current/reference positions `y`/`X`, density ratio `c`, symmetric radial
coefficient `a`, viscosity pair matrix `P`, and surface coefficients `d`/`b`,
the owner expressions are:

```text
incompressibility:
  L_ij = -a*y_j + c_i*a*(y_j-y_i)
  R_ji = -a*y_j + c_j*a*(y_j-y_i)
  K_i  += (-a)I + (-a)I

viscosity, with D_ij = X_j-X_i and P_ij = P_ji:
  L_ij = P*y_j - P*D_ij
  R_ji = P*y_j - P*D_ij
  K_i  += P + P

surface tension:
  L_ij = d*y_j + b*(y_j-y_i)
  R_ji = d*y_j + b*(y_j-y_i)
  K_i  += dI + dI
```

The first implementation retains the two additions shown above instead of
rewriting them as one doubled coefficient.

For viscosity and surface tension the two owner contributions simplify to the
same real-valued expression. That algebraic simplification is deliberately not
part of the first reclosure: using `2*x` would combine the topology repair with
a second floating-expression change. Incompressibility cannot use that
simplification because its density ratios can differ.

## Candidate comparison

| Candidate | Repeatability mechanism | Additional storage | Research conclusion |
|---|---|---:|---|
| `gather-directed-r0` | one owner thread, fixed CSR order, owner-only output | no pair fragments; existing CSR and output buffers | **selected first** |
| `unique-pair-segmented-r0` | stable owner-key order plus segmented reduction | `O(P)` endpoint fragments, keys and reduction/sort scratch | retain only as fallback after a gather formula or cost failure |
| `fixed-point-atomic-r0` | exact integer addition after per-field quantization | scale/range metadata; possibly wider accumulators | defer: changes the numerical representation and needs overflow/quantization proofs |
| `f64-atomic` | smaller rounding error only | wider source/matrix buffers | reject as remedy: atomic arrival order remains undefined and RTX 3080 `f64` throughput is costly |
| warp-aggregated atomics | fewer atomics | warp partials | reject as remedy: global atomic arrival order remains run-dependent |
| graph coloring | conflict-free colored passes | coloring and color ranges | reject first: many passes/setup for a problem owner gather solves directly |
| persistent/cooperative scatter | attempts to constrain scheduling | scheduler/global-sync machinery | reject: block scheduling still does not define one global floating add order |

`gather-directed-r0` is not intrinsically double the current pair work. The
source-shaped baseline already visits both directions of every symmetric pair
and computes local plus reverse endpoint expressions on each visit. Gather
keeps one directed CSR visit per owner, reconstructs the incoming expression
locally and removes reverse global atomics. It does add a neighbor-density read
for incompressibility and changes accumulation association, so performance is
an empirical question rather than an assumed win.

## Segmented-reduction resource bound

The installed CUB
[`DeviceSegmentedReduce`](https://nvidia.github.io/cccl/unstable/cub/api/structcub_1_1DeviceSegmentedReduce.html)
does promise run-to-run determinism for pseudo-associative reductions on the
same GPU, but not across different compute capabilities. It is therefore a
valid fallback mechanism, not a cross-target authority contract.

NR1 measured `5,190,588` directed entries for water-48k. Removing one self
entry per sample leaves `2,571,294` undirected non-self pairs and `5,142,588`
endpoint fragments. Before sort/reduction scratch or a second ping-pong copy:

| Fragment layout | 48k values | Owner keys | 16k values | Owner keys |
|---|---:|---:|---:|---:|
| source `vec3` + diagonal (`16 B`) | `78.470 MiB` | `19.617 MiB` | `25.691 MiB` | `6.423 MiB` |
| source `vec3` + full matrix (`48 B`) | `235.409 MiB` | `19.617 MiB` | `77.073 MiB` | `6.423 MiB` |

The full NR1 water-48k allocation is only `32,725,766 B` (`31.209 MiB`). A
viscosity-capable fragment pass therefore multiplies traffic and live memory
before CUB scratch is counted. Symmetric-matrix packing can reduce the factor,
but would be another expression/layout optimization. This makes segmented
reduction a poor first discriminator even though its determinism guarantee is
usable.

## Selected smallest experiment

The selected remediation identity is `nuv-gather-directed-r0`. It changes only
the three pair-accumulation kernels. Density, frozen CSR, coefficients, active
terms, `f32` mode, local solve, time step, fixed iteration counts, compiler
flags and all NR0 tolerances remain unchanged. Existing clears remain in the
first experiment even if owner overwrite makes them redundant, so clear/fusion
credit stays in NR2 O5.

The discriminator order is:

1. independently implement a CPU `f64` directed-gather path and compare it
   with the existing CPU scatter oracle on all eleven tiny fixtures;
2. implement the CUDA owner-only gather and pass the existing CPU/CUDA tiny
   matrix, including source, full matrix, position, velocity and momentum;
3. run a ten-repeat, two-iteration `nuv-surface-16k.v0` control and require
   byte-identical ordered output-state digests for the same binary/toolkit/GPU;
4. only after step 3, run ten repeats at the frozen 20 iterations and require
   the existing finite, field-tolerance and momentum gates;
5. only after surface correctness, execute the water/viscous correctness
   controls and bounded adjacent timings.

The 16k two-iteration control is intentionally retained instead of inventing a
new tiny fixture. NR1's largest tiny fixture can fit in one 256-thread CUDA
block and therefore did not exercise the inter-block atomic ordering implicated
by the failure. The known 16k control costs milliseconds and already has a
successful negative observation, so it is the smallest evidence-backed
cross-block discriminator.

The exact repeatability claim is intentionally narrow: same input, executable,
CUDA/CCCL version, driver, RTX 3080 device and launch profile. Different GPUs,
toolkits, compiler flags or future tuning receive correspondence tolerances,
not an inherited byte-equality claim. This research candidate remains a GPU
mirror and cannot satisfy ADR-081's production `CanonicalFloatExecutionProfile`
or become canonical authority.

## Resume and stop decisions

- If every gate passes, record `NR1_RECLOSED_GATHER_DIRECTED` and allow NR2 to
  begin. Preserve `source-atomic-v0` as the immutable source-shaped record and
  use it as the HN-3 performance denominator only for the water-48k and
  viscous-16k profiles on which its correctness passed.
- If CPU scatter/gather disagree beyond the frozen oracle bounds, stop as
  `GATHER_FORMULA_MISMATCH`; do not reach CUDA timing.
- If owner-only CUDA output still varies with byte-identical CSR, stop as
  `ORDER_HYPOTHESIS_FALSIFIED` and inspect the earliest differing producer
  before trying segmented reduction.
- If gather is stable but violates the CPU/tolerance or momentum gates, stop as
  `GATHER_NUMERIC_MISMATCH`; do not widen tolerances or simplify by a factor of
  two.
- If correctness passes but cost is materially worse, preserve the result and
  evaluate `unique-pair-segmented-r0` only under a separately frozen memory
  capacity. Correctness reclosure itself receives no performance speedup
  credit until the adjacent NR2 report is complete.

## Conclusion

The architectural response was validated: preserve the failed source-shaped
baseline and reclose a separately named deterministic accumulation candidate.
The selected **owner-only directed gather** passes the stiff-surface control,
requires no pair-fragment capacity and supports the leading shared-write-order
hypothesis on the frozen environment. NR2 is therefore unblocked; the original
comparison remains research rationale rather than retained optimization credit.
