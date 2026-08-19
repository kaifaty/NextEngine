# NR2/NR3 — Nonlocal GPU optimization discriminators

Status: `SPECIFIED / UNBLOCKED_BY_NR1-RC1 / O1_NEXT / REPORT_ONLY`

The source-atomic [NR1 evidence](../../development/nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md)
did not authorize this ladder. The separately specified
[NR1-RC1 owner-only gather](03-nr1-deterministic-accumulation-reclosure.md)
has now exited `NR1_RECLOSED_GATHER_DIRECTED`; its
[execution report](../../development/nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md)
unblocks O1. `nuv-gather-directed-r0` is the correctness baseline for retained
NR2 candidates. Its adjacent RC1 timings remain observations until the ordered
ladder and required profiler captures are complete.

## Outcome

Measure whether concrete implementation changes can reduce fixed-iteration
Nonlocal/SISSM cost without changing its declared mathematical workload, then
separately evaluate algorithm-changing convergence ideas. Every retained change
has an adjacent baseline, correctness comparison and rollback point.

## Measurement discipline

For every candidate:

1. run the NR1 self-test and named tiny/full control;
2. record a cold adjacent baseline at the same commit/toolchain/device state;
3. enable exactly one candidate or one inseparable candidate group;
4. rerun correctness before timing;
5. benchmark the same profile, iteration count and warm-up/run window;
6. retain only if it improves the declared stage/total without violating a
   correctness, memory or complexity bound;
7. create one coherent commit/report boundary before the next candidate.

The decision metric uses fixed iteration counts. A profiler capture is required
after O2 and after the final retained fixed-iteration implementation; it is not
required after every small buffer cleanup.

## NR2 fixed-iteration ladder

### O1 — Persistent state and pointer swap

Replace per-step/per-iteration resize, assignment and full state copies with:

- capacity-admitted persistent SoA buffers;
- two position buffers selected by index/pointer swap;
- explicit initialization of only fields required by the next pass;
- persistent CUB/temp storage sized during preflight.

No pair, arithmetic expression, term, iteration or accumulation order changes.
Reject O1 if memory grows beyond the declared capacity model or if a stale
field can survive a reset self-test.

### O2 — Term-specialized kernels

Compile or dispatch closed kernels for the actual active term sets:

```text
Water       = incompressibility + bulk viscosity
Viscous     = incompressibility + bulk/shear viscosity
Surface     = incompressibility + surface tension
Full        = incompressibility + viscosity + surface tension
```

Inactive terms allocate no per-step work and execute no branches inside the
pair loop. Specialization may reduce a water local system only if the paper
equations prove that omitted matrix entries are identically zero; otherwise the
full `3x3` representation remains.

O2 is rejected if one compiled profile produces a different CPU/GPU oracle
result from the general-kernel control beyond the frozen tolerance.

### O3 — Accumulation-layout tournament

Benchmark three explicit layouts where supported:

| ID | Pair work | Writes | Principal risk |
|---|---|---|---|
| `scatter-atomic` | directed/source-shaped reference on correctness-passing profiles only | reverse endpoint atomics | contention and nondeterministic addition order; cannot be retained for surface |
| `gather-directed` | same directed CSR; owner reconstructs outgoing and incoming endpoint terms | owner-only writes | neighbor density reads and a different but defined `f32` association |
| `unique-pair-segmented` | one undirected pair | pair fragments then stable segmented reduction | fragment memory and reduction/setup cost |

All layouts consume the same sorted neighbor/pair identity and produce the
same mathematical source/matrix within the fixed tolerance. Performance alone
does not retain a layout whose conservation residual or run-to-run numeric
spread exceeds the declared bound. `gather-directed` enters this tournament
only after RC1 correctness; its RC1 timing is an adjacent observation, not an
NR2 retained-speedup claim by itself.

The tournament report includes atomic transactions if counters are available,
bytes/pair, pair evaluations, temporary memory and stage p95. Lack of privileged
hardware counters is recorded and does not invalidate wall-time comparison.

### O4 — Cell sorting and neighbor locality

Build particles in stable cell order and use compact CSR/SoA neighbor ranges.
The profile binds:

- packed cell-key definition and overflow checks;
- stable secondary particle ID order;
- fixed 27-cell traversal order;
- bounded range offsets and maximum degree;
- explicit map between storage order and stable fixture/sample identity.

Storage reordering may improve memory locality but cannot change fixture
identity, pair membership or report order. Generic unordered-hash iteration is
forbidden.

### O5 — Clear/pass/launch reduction

Only after profiling shows a meaningful remaining share, test:

- producer-owned overwrite instead of separate clears;
- safe fusion of reset with the first producer;
- CUDA Graph replay for the fixed iteration schedule;
- one persistent iteration kernel only if global synchronization and failure
  reporting remain explicit.

Do not fuse passes across a data dependency merely to remove a launch. O5 is
expected to be a secondary improvement and cannot by itself justify continuing
after the main pair traversal misses the stop gate.

### O6 — Precision tournament

The source-shaped baseline already uses `f32`; therefore `f32` is not an
optimization candidate. Optional modes are:

- `f32-storage/f32-compute` baseline;
- selected `f16`/packed storage with `f32` compute for demonstrably bounded
  fields;
- `f32` local work with `f64` diagnostic reductions.

No lower-precision mode is retained without the full oracle and conservation
matrix. Precision results receive distinct profile identities and make no
canonical/replay claim.

## NR2 early-stop rules

Stop fixed-iteration optimization when any condition holds:

- two consecutive retained candidates improve total p95 by less than `10%`
  each and profiling shows no untested stage owning at least `20%`;
- the accumulated retained implementation is below `2.0x` geometric-mean
  speedup on `nuv-water-48k.v0` and `nuv-viscous-16k.v0` after O1–O4;
- correctness fails twice for the same accumulation family;
- temporary memory exceeds the predeclared capacity or prevents the 48k run;
- the optimized 48k five-iteration p95 remains above `16 ms` after O1–O4,
  leaving no credible fixed-iteration route to the `8 ms` feasibility cutoff.

An early stop proceeds directly to NR4. It does not trigger ML, adaptivity or
a larger iteration/tolerance change.

## NR3 algorithm-changing probes

NR3 is optional and begins only after the fixed-iteration NR2 result is frozen.
It uses separate profile IDs, reports and decision labels.

### A1 — Residual sampling and adaptive exit

Before code, define one residual vector and normalization derived from the
published fixed-point equation. The profile fixes:

- check cadence, initially every two or four iterations;
- absolute and relative acceptance bounds;
- minimum and maximum iteration count;
- nonfinite, stagnation and divergence failure classes;
- whether energy is evaluated and what constitutes an unacceptable increase;
- the CPU `f64` reference result used to calibrate the bound.

The global reduction and synchronization cost are included in timing. Position
delta alone is not an admissible residual. A candidate that exits early but
selects a materially different stationary point fails rather than receiving a
looser tolerance.

### A2 — Warm start

Warm start is deferred. It adds future-affecting continuation state and would
make cold restore differ from uninterrupted execution. It cannot enter the
current research result unless a separate state/profile specification defines
reset, persistence and comparison semantics.

### A3 — Line search or alternative nonlinear solver

A line search, Newton/quasi-Newton method or pairwise descent changes work and
convergence semantics. It requires published equations or a complete local
derivation plus a new bounded specification. The announced Semi-Implicit
Pairwise Descent work remains ineligible until its paper or code is public and
audited.

## Performance gates

Apply gates only after correctness:

| Gate | Required result | Meaning |
|---|---|---|
| `NR2-SPEEDUP` | retained fixed-iteration geometric-mean speedup `>= 2.0x` on 48k water and 16k viscous | optimization hypothesis survives |
| `NR2-48K-CUTOFF` | `nuv-water-48k.v0`, five fixed iterations, total p95 `<= 8 ms` | may propose full 48k reclosure |
| `NR2-LOCAL-CUTOFF` | 16k water and one coupled 16k profile total p95 `<= 8 ms` | may propose local-domain product research |
| `NR3-ADAPTIVE` | independent stationary-result/residual gates plus measured end-to-end benefit after reduction overhead | algorithm candidate only |

The current production stop target remains `4/6 ms` p95/p99. Passing an `8 ms`
cutoff is not W2 or ProductCheck PASS.

## Evidence summary

The checked-in NR2/NR3 report lists every attempted candidate, before/after
commit, exact command, correctness result, stage/total percentiles, profiler
attribution, memory, conclusion and rollback status. Raw reports and captures
remain external and hash-bound.

No percentage is added across non-adjacent runs, devices or profile identities.
The final NR4 decision cites only complete applicable gates.

## Non-goals

Production CUDA abstractions, engine scheduler integration, cross-target
equality, PhysX boundaries, adaptive particles, ML predictors, multi-GPU,
thermochemistry, plasticity and visual-quality evaluation.
