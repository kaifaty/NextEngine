# W2 — Deterministic parallel CPU and performance

## Outcome

Turn the passing serial oracle into a bounded parallel CPU candidate without
changing its canonical trajectory, then determine whether the selected 50k
product fixture can fit its standalone THOTH stop target.

## Current implementation checkpoint

W2 starts with a bounded Linux-only stage discriminator rather than another
full serial corpus repetition. `continuum water profile-w2-linux` fixes the
sealed `48,000`-sample scenario, one warm-up substep and three measured
substeps. It records raw wall-clock costs for decode, reconstruction, initial
diagnostics, divergence, gravity, density, contact, integration and canonical
publication, plus the short trajectory root and available logical
parallelism. Timers exist only in the diagnostic call and never enter accepted
state, convergence decisions or hashes.

This `continuum-water-50k-stage-profile.v0` projection is a bottleneck
discriminator only: setup/report I/O are separate, process CPU utilization is
captured externally, and it grants no W2 percentile, correctness or ProductCheck
credit. The formal `continuum-water-50k.v1` warm-up/measured windows remain to
be frozen after the worker design and root-equality harness exist.

The clean serial baseline at commit `5b3590e` uses `100%` of one CPU on a host
with `32` available logical CPUs. Three measured sealed-48k substeps average
about `1.212 s`: reconstruction accounts for `61.15%` and density for
`34.90%`. Cycle 1 therefore retains each row's first canonical admitted-index
result in bounded flattened scratch instead of repeating grid admission and
temporary row allocation. At clean commit `e15c5e2`, all short-run roots and
iterations remain exact, reconstruction is `1.801×` faster and the whole step
is `1.375×` faster. Density/reconstruction now jointly own `94.73%`; cycle 2
partitions both without changing row or reduction order. See the
[resource-utilization discriminator](../../development/continuum-water-w2-resource-utilization-2026-08-19.md).

Cycle 2 at clean commit `c2915cf` uses a crate-private local pool, `64` stable
logical partitions and canonical indexed placement. The serial oracle and all
row-local/global reduction orders remain intact. The short sealed-48k matrix
has exact initial, per-step, final and trajectory roots for serial and workers
`1/2/4/8`; an 8-worker step falls to `269.643 ms`, a `3.105×` adjacent-run
speedup, while reconstruction scales `5.121×` and density only `2.578×`.
Whole-command utilization reaches `372%` CPU and peak RSS is `104,860 KiB`.

This remains `REPORT_ONLY / NO_W2_CREDIT`: full W0B trajectory equality and
the formal percentile windows were not run. The short mean still misses the
`4 ms` target by `67.41×`, and even perfect division of the current serial
mean across all `32` logical CPUs would leave `26.16 ms`. Long gate repetitions
are deferred pending an explicit choice between `STOP_RESEARCH_ONLY` and a new
algorithm/data-layout profile; another scheduler-only tuning pass is not an
evidence-backed next step.

## Correctness-preserving parallel plan

- Keep the serial implementation as the oracle and fallback test path.
- Preserve the sorted uniform-grid representation and stable logical sample
  partitions. Worker count schedules partitions only; it cannot change cell,
  pair or reduction order.
- Each worker writes private bounded fragments. Merge fragments by the same
  canonical `(cell key, SampleId, field/reduction key)` order used by serial.
- Use `f64` only within the step and the same single fixed-point publication
  boundary. Do not replace reductions with atomics, unordered floats or
  approximate neighbor sets.
- Add no generic jobs/resource framework. Use the smallest crate-private
  parallel mechanism supported by the measured workload.

Exact requirements:

- serial and worker counts `1/2/4/8` produce identical frame and trajectory
  roots for every W0B scenario;
- insertion order, chunk size and worker completion permutations do not alter
  roots, iterations or terminal diagnostics;
- an allocation, worker panic or fragment-capacity fault rejects the complete
  run with no partial frame.

The frozen exact profile uses `panic=abort`: an unexpected panic therefore
terminates the isolated run before publication. The unwind test profile also
proves the structured `WATER_WORKER_FAILURE` path. Requiring a structured
in-process report after an exact-profile panic would change the frozen profile
and needs a separate reclosure.

## Named performance workload

Define `continuum-water-50k.v1` for `ref-win-thoth-v1`:

- the W0B clean-water profile at 240 Hz;
- the sealed production basin with nominal `48,000`, hard `50,000` samples;
- fixed warm-up/measured substep counts and exact scenario/root hashes;
- release build and declared worker count chosen from the exact scaling run;
- per-substep neighbor rebuild, divergence/density iterations, publication and
  canonical-root cost included;
- setup, external report I/O and future checkpoint restore excluded and
  reported separately.

Also run identical report-only `10k` and `100k` profiles. `100k` is a stress
measurement and cannot become a production claim.

## Standalone stop gate

The 50k workload must satisfy on THOTH:

- water-inclusive physical-frame p95 `<= 4,000 µs`;
- water-inclusive physical-frame p99 `<= 6,000 µs`;
- no missed substep, non-convergence, capacity fault, unowned span or root
  difference across profiler/worker permutations.

These values are an isolated solver stop target, not `PHYS-P4`, an integrated
gameplay PASS or permission to sum subsystem budgets. W6 must introduce and
pass the successor mutually exclusive `world-dynamics-step` row whose one-tick
window includes every physical substep, merge, validation and publication.

## Two-cycle stop policy

An optimization cycle consists of one falsifiable bottleneck hypothesis, one
bounded implementation change, serial/root non-regression, and a complete
10k/50k/100k measurement. The two initial admissible cycles are:

1. sorted-grid construction and memory-layout/locality improvement without
   changing pair order;
2. deterministic parallel partition/merge and, only if profiling identifies
   grid sorting as the limiter, a stable radix-sort comparison.

After two coherent cycles, a 50k budget miss changes W2 to
`STOP_RESEARCH_ONLY`. Reducing sample count, increasing budget, changing
cadence/thresholds or promoting GPU authority is not an optimization and
requires a new explicit architecture/product decision.

## Evidence and exit

The W2 report binds tool commit, Rust/toolchain, target, THOTH fingerprint,
scenario/profile hashes, canonical roots, run boundaries, raw percentile
samples, iteration tails, memory peaks and per-stage costs. A PASS requires
exact correctness plus this standalone stop gate; it cannot satisfy the W6
combined budget.

## Non-goals

PhysX coupling, renderer, public contracts, save schema, GPU, adaptivity and
changing the W0B numerical profile.
