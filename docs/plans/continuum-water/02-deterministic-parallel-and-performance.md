# W2 — Deterministic parallel CPU and performance

## Outcome

Turn the passing serial oracle into a bounded parallel CPU candidate without
changing its canonical trajectory, then determine whether the selected 50k
product fixture can fit the current THOTH budgets.

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
  roots for every W0 scenario;
- insertion order, chunk size and worker completion permutations do not alter
  roots, iterations or terminal diagnostics;
- an allocation, worker panic or fragment-capacity fault rejects the complete
  run with no partial frame.

## Named performance workload

Define `continuum-water-50k.v1` for `ref-win-thoth-v1`:

- the W0 clean-water profile at 240 Hz;
- the sealed production basin with nominal `48,000`, hard `50,000` samples;
- fixed warm-up/measured substep counts and exact scenario/root hashes;
- release build and declared worker count chosen from the exact scaling run;
- per-substep neighbor rebuild, divergence/density iterations, publication and
  canonical-root cost included;
- setup, external report I/O and future checkpoint restore excluded and
  reported separately.

Also run identical report-only `10k` and `100k` profiles. `100k` is a stress
measurement and cannot become a production claim.

## Budget gate

The 50k workload must satisfy on THOTH:

- water-inclusive physical-frame p95 `<= 4,000 µs`;
- water-inclusive physical-frame p99 `<= 6,000 µs`;
- integrated gameplay total p95 `<= 8,000 µs` and p99 `<= 12,000 µs`;
- no missed substep, non-convergence, capacity fault, unowned span or root
  difference across profiler/worker permutations.

This is a new continuum workload using the current absolute ceilings; it does
not relabel itself as a PASS for the existing 16-humanoid `PHYS-P4` workload.
Instrumentation assigns every continuum span to the future PhysicalStep owner
row before production promotion and cannot manufacture a second budget.

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
exact correctness plus the budget gate; report-only timing cannot unblock W3.

## Non-goals

PhysX coupling, renderer, public contracts, save schema, GPU, adaptivity and
changing the W0 numerical profile.
