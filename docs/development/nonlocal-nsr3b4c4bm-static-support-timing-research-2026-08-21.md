# B4C4BM static-support timing discriminator design

Status: `COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Why measurement precedes rollout

B4C4B removes `91.54%/86.96%` of P1/P2 records admitted to cell sorting, but
replaces one combined range lookup with one fluid plus one support lookup for
every visited cell. These costs depend on support-to-fluid ratio, occupied-cell
count and standard-library binary-search behavior. Structural counters cannot
determine the sign of the latency change.

B4C4BM measures only neighborhood construction. It does not time the full
solver, B4C4A1 parent, JSON serialization, correctness comparisons, corpus
capture or static-index construction.

## Representative corpus

Record the exact canonical fluid input presented to every retained B4C4A
workspace during one selected P1/P2 macro transaction. This yields `264` P1
and `9` P2 states, including frame/forecast, interval initial, current, trial
and macro-ledger queries. Boundary support remains the original immutable
fixture input.

Before timing, build every corpus state through legacy and candidate paths and
require full neighborhood equality. Bind the ordered state identities into a
corpus hash. Correctness checking and hashing stay outside timed regions.

## Timing protocol

For each fixture sequentially:

1. construct and validate one static support index outside timing;
2. execute three unreported warmup corpus passes per path;
3. execute 21 paired measured rounds;
4. alternate `legacy -> candidate` and `candidate -> legacy` by round;
5. time each complete corpus pass with `std::chrono::steady_clock` nanoseconds;
6. consume pair count, distance count, degree and endpoint identities into a
   deterministic checksum reported after timing.

Report all samples, median, minimum, maximum, median absolute deviation,
candidate wins and median legacy/candidate ratio. P1 and P2 do not overlap, so
one case cannot distort the other's cache or CPU scheduling.

## Interpretation

There is no speed threshold and timings are not repeatability roots. Three
independent benchmark reports must agree on the deterministic parent, corpus,
correspondence and checksum material; their timing samples may differ.

The result is classified only as observed `CANDIDATE_FASTER` or
`CANDIDATE_NOT_FASTER` per fixture. A later decision may freeze a performance
gate or redesign lookup, but B4C4BM itself cannot select complete-lane
application from a fitted threshold.

## Decision

Freeze the
[B4C4BM contract](../plans/nonlocal-nonlinear-solver-research/03b4c4bm-static-support-timing-contract.md).
B4C4C, complete-lane rollout, B4D, nominal corpus, CUDA, runtime and production
remain blocked until its evidence is classified.
