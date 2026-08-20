# NP0–NP4 — Nonlocal performance reclosure contract

Status: `COMPLETE / NONLOCAL_50K_FIXED_WORK_RECLOSURE_CANDIDATE / REPORT_ONLY / NO_W2_CREDIT`

## Scope

This contract governs the performance research authorized by the closed NR4
`NONLOCAL_48K_RECLOSURE_CANDIDATE` state. It starts from the retained
directed-gather/pointer-swap/specialized/stable-sample implementation and asks
whether a fresh Nonlocal profile deserves a later Proposed reclosure.

It does not modify the closed v0 evidence, current DFSPH roots, SPEC-38,
ADR-076, ADR-081, public contracts, runtime scheduling, persistence or GPU
authority.

## Falsifiable hypotheses

| ID | Hypothesis | Falsifier | Smallest discriminator |
|---|---|---|---|
| `HP-1` | The coherent v0 lattice hides a material storage-locality difference | permuted and advected states change neither locality metrics nor retained stage time by `5%` | same geometry with one frozen stable-ID permutation |
| `HP-2` | Compatible term fusion plus compact CSR can reduce pair-stage p95 by at least `15%` and exact-50k total p95 by at least `10%` | either combined threshold fails or numeric association/capacity fails | water-50k coherent and permuted adjacent tournament |
| `HP-3` | A certified Verlet skin can reduce amortized dynamic total p95 by at least `5%` | rebuild/filter overhead or extra pairs erase the gain | 32-substep advected water trace with identical active CSR |
| `HP-4` | A derived residual and safeguarded acceleration can reduce viscous/surface end-to-end p95 by at least `25%` | stationary-result/quality fails or iteration savings do not pay for checks | 16k coupled high-iteration controls |
| `HP-5` | A separately rooted Nonlocal 50k profile can meet standalone `<= 4 ms` p95 and `<= 6 ms` p99 on the Linux RTX 3080 | either coherent or advected decision profile misses | final same-process 50k decision campaign |
| `HP-6` | If 50k misses, a 16k local high-fidelity domain remains credible | water misses, or neither coupled 16k candidate passes `4/6 ms` and quality | retained local-domain decision campaign |

HP-1 is diagnostic: its failure removes dynamic storage sorting from NP1 but
does not fail the whole roadmap. HP-5 does not claim the future integrated
ADR-081 budget.

## Candidate classes and credit separation

| Class | May change | May not claim |
|---|---|---|
| `EXACT_WORK` | storage encoding, traversal count when algebraically redundant, launch shape, certified neighbor-list reuse | fewer nonlinear iterations or a different stationary point |
| `ALGORITHM` | residual checks, iteration count, accepted iterate, guarded accelerator | fixed-work speedup or the retained v0 root |
| `REPRESENTATION` | particle count/resolution, active domain, coarse/fine coupling | exact-50k standalone credit |

Every report names one class. Results from different classes are not added as
if they were one adjacent speedup.

## NP0 profile families

NP0 commits the exact generators and hashes before an optimization is timed.
The following identities are reserved; their final dimensions/seed/capture
hashes must appear in the NP0 specification and machine-readable report:

| Profile ID | Purpose | Work |
|---|---|---|
| `nuv-water-50k-coherent.v1` | exact-count capacity/performance control | 50,000 samples, five fixed iterations, coherent stable order |
| `nuv-water-50k-permuted.v1` | isolate storage order | identical positions/physics to coherent; one fixed bijective stable-ID permutation |
| `nuv-water-50k-advected.v1` | representative multi-substep denominator | hash-bound prior states; neighbors rebuilt per substep and frozen within solve |
| `nuv-viscous-16k-advected.v1` | coupled high-iteration control | incompressibility plus bulk/shear viscosity, 20 fixed iterations |
| `nuv-surface-16k-advected.v1` | dynamic coupled control | incompressibility plus moderate surface tension, 20 iterations |
| `nuv-surface-stiff-16k-i2.v1` | sensitive numeric control | incompressibility plus `gamma=1000` surface tension, exactly two iterations |
| `nuv-water-100k-report.v1` | scaling/memory observation | report-only; never a production promise |

The v1 profiles inherit no output digest or tolerance by name. NP0 derives and
freezes their independent CPU `f64` oracle bounds before GPU timing. The
advected corpus includes at least one topology-preserving and one
topology-changing interval; a candidate cannot choose only the easier state.
Exact dimensions, permutation and trajectory semantics are frozen in the
[NP0 corpus specification](01-np0-corpus-and-baseline.md).

## Correctness and topology gates

Every retained `EXACT_WORK` candidate must satisfy:

- CPU pair and one-iteration oracle within the predeclared v1 tolerances;
- finite state and bounded density/source/matrix/energy values;
- exact sample identity, count and mass;
- symmetric active membership and bounded directed-pair capacity;
- exact repeated output and active logical CSR for one fixed input;
- exact output/CSR correspondence to its adjacent retained baseline unless a
  pre-implementation numeric specification explicitly creates an `ALGORITHM`
  identity;
- normalized momentum residual within the frozen bound;
- stiff-surface i2 before i20 or performance timing;
- identical coherent/permuted physics after remapping to stable sample IDs.

A skin list may contain extra candidates, but the filtered active CSR must
match the canonical rebuild exactly. Its rebuild predicate and maximum-
displacement reduction are report fields.

## Performance measurement

All decision measurements run on the current Linux x86-64 RTX 3080 host with
the exact compiler/CUDA/device/power state recorded. Windows is not run and no
cross-target claim is made.

The primary total includes prediction, required neighbor build or amortized
certified reuse, every nonlinear iteration, convergence reductions when
enabled, local update and velocity reconstruction. Process startup, report I/O
and one-time admitted capacity allocation remain separate.

Reports include:

- min, median, p95, p99 and mean from raw unrounded samples;
- per-stage and pair-stage totals;
- directed/active/candidate pair counts, average/max degree and rebuild count;
- bytes per sample/edge, peak device memory and temporary capacity;
- coherent and dynamic storage-distance distributions;
- exact candidate/baseline input, output, CSR, binary and raw-report hashes;
- profiler stage attribution; unavailable privileged counters stay explicitly
  unavailable and are not inferred.

Measurement tiers are:

| Tier | Window | Use |
|---|---|---|
| preflight | tiny plus three reused/cold controls | reject correctness/setup failures |
| adjacent | 32 warm-ups, 96 alternating measured executions in one process | retain/reject one candidate |
| decision | 64 warm-ups, at least 512 measured substeps plus the advected trace | p95/p99 and NP4 state |

Each tier follows a fixed 256-execution GPU-conditioning window which is
outside timing and does not replace its formal warmups. This was added in NP0
after unconditioned duplicate p95 values varied by about `12%`; conditioned
fresh-process decision p95 values differed by `0.41%` coherent and `0.54%`
advected. Device telemetry remains an evidence artifact rather than a solver
input.

GPU decision timing is serialized. CPU fixture/oracle work may run in parallel
outside the timing window. Thermal throttling or competing GPU load invalidates
the affected sample set.

## Ordered stage gates

### NP0 gate

NP1 began only after all v1 generators/hashes were committed, CPU/GPU retained
controls pass, dynamic topology changes as intended, the persistent runner
emits p99-capable raw totals, and the denominator report is reproducible. NP0
met this gate on 2026-08-20; see the
[dated evidence](../../development/nonlocal-continuum-np0-evidence-2026-08-20.md).

### NP1 retention gate

An adjacent exact-work candidate must:

- pass all applicable correctness gates;
- improve its targeted stage by at least `10%` or exact-50k total by at least
  `5%`;
- regress neither coherent nor advected 50k total p95 by more than `2%`;
- remain inside its frozen memory/pair capacity;
- keep the prior identity selectable as rollback.

The final NP1 stack tests HP-2 using the stronger combined thresholds in the
hypothesis table.

### NP2 gate

Before timing, NP2 freezes a normalized fixed-point/KKT residual, energy
safeguard, check cadence, min/max iterations, high-iteration CPU stationary
reference and nonfinite/stagnation/divergence semantics. Adaptive and Anderson
candidates must pass trajectory/quality bounds and beat the retained fixed-
work total including all reductions.

### NP3 gate

No scale-changing code begins until a separate specification closes stable
identity, conservation/error receipts, boundary/handoff semantics, resolution
selection, capacity and repeated transition bounds. NP3 cannot block NP4's
standalone interpretation.

## Early-stop rules

- one repeated numeric-association failure stops that exact-work family;
- two consecutive retained NP1 candidates below `5%` adjacent total gain stop
  NP1 unless a fresh profile exposes an untested `>= 15%` stage;
- any unbounded pair/temp allocation or capacity overflow rejects the
  candidate before timing;
- if NP0 shows no meaningful coherent/dynamic locality difference, do not
  implement another cell/Morton/Hilbert storage family;
- if convergence checks cost more than the saved iterations on both coupled
  controls, stop that NP2 family;
- Pairwise Descent remains a watch item until a public paper or code is audited;
- ML, multi-GPU, runtime integration and larger tolerance are not escape paths.

## NP4 decision states

NP4 selects exactly one:

### `NONLOCAL_50K_FIXED_WORK_RECLOSURE_CANDIDATE`

The final exact-work stack passes all v1 correctness/capacity gates and both
coherent and advected water-50k decision profiles meet `<= 4 ms` p95 and
`<= 6 ms` p99. This authorizes drafting a later Proposed fixed-work solver
reclosure and broader independent corpus. It is not W2 or runtime authority.

### `NONLOCAL_50K_ALGORITHM_RECLOSURE_CANDIDATE`

Fixed work misses, but one separately identified NP2 algorithm passes its
stationary/trajectory gates and the same 50k `4/6 ms` decision window. Any
follow-up must specify the new residual, iteration and state semantics; it
cannot inherit the fixed-work root.

### `NONLOCAL_LOCAL_DOMAIN_PERFORMANCE_CANDIDATE`

The 50k target misses, while water and at least one coupled 16k advected
profile pass `4/6 ms` with correctness. Follow-up is limited to a separately
specified high-fidelity local domain plus coarse far field; the 50k target is
not reduced.

### `NONLOCAL_PERFORMANCE_RESEARCH_STOP`

Representative correctness fails, no credible candidate survives the stop
rules, or both 50k and local-domain gates miss. Preserve the lab and evidence;
do not fund integration.

### `NONLOCAL_PERFORMANCE_EVIDENCE_INCOMPLETE`

The required source, compiler, hardware or reproducible denominator is
unavailable. This is a blocked result, not a performance rejection.

## Explicit deferrals

Runtime/public schemas, PhysX coupling, save/replay, Windows, cross-target
roots, ML proposals, multi-GPU, general variable horizon, material split/merge,
thermal/plastic/solid extensions and the integrated `world-dynamics-step` gate.
