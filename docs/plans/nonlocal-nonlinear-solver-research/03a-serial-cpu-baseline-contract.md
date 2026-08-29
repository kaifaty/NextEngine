# NSR3-A -- serial CPU baseline contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY`

Identity: `nuv-newton-krylov-r0`

Candidate stack: exact full HVP, canonical cell neighborhood,
unpreconditioned Steihaug--Toint and `numerical-energy-floor-stop-v1`.

## Purpose

Establish where the selected report-only CPU solver spends time before any
optimization is chosen. This stage has no speed target and cannot award GPU,
runtime or production credit.

## Corpus

Run the unchanged NSR2-C2 centered lattice generator at side lengths
`8, 10, 12, 16` (`512, 1000, 1728, 4096` particles). Fixture construction is
outside timing. Every run begins from the exact same generated input; sequential
multi-step state is not part of this baseline.

For each size execute one warmup and seven measured solves in one process. Pin
the process to logical CPU 4 with `taskset -c 4`. The target remains compiled
with `-O3 -ffp-contract=off -fno-fast-math` and warnings-as-errors.

## Timed buckets

Use `std::chrono::steady_clock` only when the benchmark flag is active:

1. `pair_and_adjacency`: reference/current/trial cell pairs plus adjacency;
2. `objective_gradient`: initial and trial objective/gradient evaluations;
3. `hvp`: every inner and model Hessian-vector product;
4. `control_and_vector`: full solve minus the three buckets.

Record raw nanoseconds for every measured solve and publish median and p95 for
total and each bucket. The p95 for seven sorted samples is the highest sample
(`ceil(0.95*N)-1`). Timings are environmental observations and do not enter a
deterministic evidence hash.

## Correctness gates

- all seven measured executions per size are finite and successful;
- every execution of a size has the exact same final position, objective,
  stop reason, operation counts, pair counts and capacity maxima;
- 512/1000 exact state fingerprints and operation counts match NSR2-C2;
- momentum residual `<=1e-12`, maximum pairs `<=80*N`, neighbors `<=160`;
- timed buckets are nonzero where work exists and their sum does not exceed
  measured total;
- all NSR0--NSR2-C2 exact report hashes remain unchanged.

The stage passes correctness even if 1728/4096 are slow. The dominant median
bucket and growth ratios select exactly one NSR3-A1 optimization contract.
Unavailable hardware counters are recorded; timing is not replaced by inferred
counter values.

