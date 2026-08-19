# Nonlocal unified continuum source audit — 2026-08-19

Status: `REPORT_ONLY / PRIMARY_SOURCES_INSPECTED / IMPLEMENTATION_NOT_STARTED`

## Question

Is the 2026 Nonlocal formulation sufficiently published and implementation-
accessible to justify a bounded Next Engine performance experiment, and which
costs should that experiment test first?

## Primary sources

| Source | Inspected identity | Bounded claim |
|---|---|---|
| [A Nonlocal Unified Variational Framework for Free Surface Flows](https://peridynamics.com/publications/2026-Liu-NUV.pdf) | DOI `10.1145/3799902.3811196`; PDF SHA-256 `610047ef32e895026c3661c57999f14ae550d8e2719ba2fcd95742371ad031c1` | unified position optimization, SISSM equations, fixed experiment profiles, limitations and timings |
| [PeriDyno UnifiedFluid example](https://github.com/peridyno/peridyno/tree/1aa892bb296fe766d2f9249c881b8605af23a69b/examples/Cuda/UnifiedFluid) | commit `1aa892bb296fe766d2f9249c881b8605af23a69b` | public CUDA example and parameterization |
| [SemiImplicitUnifiedFluidSolver](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/src/Dynamics/Cuda/ParticleSystem/SIUnifiedFluid/SemiImplicitUnifiedFluidSolver.cu) | same commit | actual buffer, iteration, density, term and accumulation structure |
| [PeriDyno license](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/LICENSE) | `Apache-2.0` | permits a provenance-preserving experiment; does not make the framework an architectural dependency |
| [Research-group publications](https://peridynamics.com/publications.html) | inspected 2026-08-19 | Pairwise Descent paper/code are `to appear`, so formulas are not yet an implementation input |

The PDF and checkout were inspected outside the repository. No external source,
binary, scene, generated report or framework dependency was added to Git.

## Confirmed mathematical scope

The paper formulates inertia, incompressibility, bulk/shear viscosity and
surface tension in one nonlinear position objective. Its SISSM method avoids a
global Hessian and updates each particle through local neighborhood reductions
and a small local matrix operation. The momentum-conserving pairwise form uses
symmetric endpoint accumulation.

This is a fluid result. Viscoelasticity, fluid-solid collision, interfacial
forces and broader non-Newtonian behavior are future directions in the paper.
Plasticity, phase change, temperature and a unified solid/fluid owner are not
implemented results and must not be attributed to it.

## Published cost evidence

All reported table examples use a `0.001 s` time step on an RTX 4080 Super.
Representative results include:

| Scenario | Samples | Iterations | Reported time |
|---|---:|---:|---:|
| water | `661k` | `5` | `43.0 ms` |
| Fig. 13(d) resolution-sweep profile | `50.7k` | `20` | `69.5 ms` |
| mud | `153k` | `40` | `239 ms` |
| high-surface-tension example | `151k` | `40` | `417 ms` |
| large coupled example | `1.7M` | `60` | `2.87 s` |

These are not Next Engine predictions: neighbor degree, material parameters,
boundaries, iterations, device and measurement scope differ. They do establish
that the released research implementation is not already a world-scale
real-time solver and that iteration count is a first-order cost variable.

## Inspected implementation shape

At the pinned commit, the solver:

1. prepares/resizes source, position and per-particle matrix arrays;
2. predicts positions;
3. executes a fixed `MaxIterationNumber` loop;
4. recomputes summation density each iteration;
5. clears source and full local-matrix accumulators;
6. launches incompressibility, viscosity and surface passes according to
   enabled flags;
7. performs the local position update;
8. assigns the new position state for the next iteration;
9. reconstructs velocity after the loop.

The example uses `DataType3f`, so the implementation is already `f32`. A
generic “move it to GPU” or “switch to float” proposal would not address the
published code.

The pair kernels symmetrically add reverse-endpoint source/matrix contributions
using atomics. Viscosity accumulates a full matrix contribution, while the
other terms have narrower patterns. Density and pair passes repeat inside a
fixed iteration schedule. These observations motivate measurements; they do
not prove that atomics, copies or any one pass dominates on the RTX 3080.

The public `LineSearchDisable` variable does not make a working global line
search visible in the inspected `compute()` loop. The paper also lists lack of
a global line search/unconditional convergence as a limitation. Consequently,
adaptive early exit or line search is an algorithm change requiring its own
residual and failure contract, not a free implementation optimization.

## Candidate optimization map

| Candidate | Why it is credible | Principal correctness risk |
|---|---|---|
| persistent buffers and position pointer swap | removes visible resize/reset/assignment traffic | stale values or changed reset scope |
| active-term specialization | water need not execute viscosity/surface work | accidentally changing the local system |
| gather-directed versus atomic scatter | trades contention for repeat arithmetic | loss of pair symmetry or different rounding spread |
| unique pairs plus segmented reduction | one pair evaluation and deterministic reduction are possible | fragment memory/setup cost |
| cell-sorted CSR/SoA neighborhoods | repeated neighbor traversal benefits from locality | changed membership/order/identity |
| convergence check every 2–4 iterations | can reduce large fixed counts when residual is already small | wrong residual, extra reduction cost or different stationary point |

CUDA Graphs, launch fusion and persistent kernels are secondary until a local
profile shows launch/reset cost is material. The previous Next Engine direct-
port discriminator already showed that high device utilization does not imply
the required algorithmic speedup.

## Rejected immediate actions

- Integrate PeriDyno into the engine: its framework, plugins and build surface
  are much broader than the bounded solver question, and Linux support is not a
  production guarantee.
- Replace DFSPH now: the Nonlocal candidate has no Next Engine oracle, roots,
  product corpus, replay profile or 4 ms evidence.
- Implement Pairwise Descent from its title: the primary paper/code are not
  public at the time of this audit.
- Start with ML/adaptivity: each changes the state/work distribution before the
  base nonlinear cost and correctness are understood.
- Claim a universal material solver: the current publication validates a
  bounded fluid formulation, not the proposed thermochemical/plastic/solid
  extensions.

## Conclusion

The primary sources are sufficient for a bounded source-faithful N0/N1
experiment. They are not sufficient for production integration or a general
continuum architecture decision. The selected next action is the standalone
[Nonlocal research roadmap](../plans/nonlocal-continuum/README.md) with a CPU
oracle, fixed-iteration GPU baseline, ordered optimization ladder and explicit
48k/local-domain/stop outcomes.
