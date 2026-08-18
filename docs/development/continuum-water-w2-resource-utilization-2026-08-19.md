# Continuum water W2 resource-utilization discriminator — 2026-08-19

Status: `REPORT_ONLY / CYCLE_2_SHORT_SCALING_ROOT_EXACT / NO_W2_CREDIT`

## Question

Why does the W1 corpus take about 42 minutes while leaving most of THOTH idle,
and which bounded optimization should W2 attempt first without changing the
frozen water trajectory?

## Clean serial baseline

Commit `5b3590e9757277676f21cb2ae4f4cc61abc60cea` adds a diagnostic-only timed
entry point. The ordinary W1 call passes no timer; a unit discriminator proves
that timed and untimed calls publish identical samples, summary and frame
root. Timings never enter accepted state, convergence decisions or hashes.

The exact `water-oracle` binary SHA-256 is
`8c075d34f81927b051ce5e2190be1c59d29ae84d45186d48b17836ac2a89b0f5`.
The clean report is stored outside Git at
`/tmp/nextengine-w2-resource-serial-5b3590e.json`, SHA-256
`984281e062db0d6a15df07e97f39225b89b08416d09e101052c4084e8abafb9d`.
Its `/usr/bin/time -v` record SHA-256 is
`c9ccce51f80b70764df4362305121e82f5ae125a2d71b4a8e7a89bc0704ad8fd`.

The fixed `continuum-water-50k-stage-profile.v0` discriminator uses sealed
`CW-SEALED-001`, `48,000` fluid samples, `24,704` static boundary samples, one
warm-up and three measured substeps. It reports `32` available logical CPUs,
one configured worker, `100%` process CPU and `86,060 KiB` maximum RSS. Thus
the current implementation saturates approximately one logical CPU and at
most `3.125%` of reported logical capacity.

Raw measured outer substep times are `1,231,216,057`, `1,201,199,354` and
`1,202,153,300 ns`; density iteration counts are `40`, `36`, `36`. The short
trajectory root is
`41390c922ca043cb5daff987fef0457fe8be9e0fa08a9d8c852e38ee5884329f`.

| Stage | Three-step time | Share |
| --- | ---: | ---: |
| reconstruction | `2,212,373,023 ns` | `61.1466%` |
| density | `1,262,835,690 ns` | `34.9028%` |
| divergence | `68,443,502 ns` | `1.8916%` |
| contact | `34,480,955 ns` | `0.9530%` |
| publication | `24,016,498 ns` | `0.6637%` |
| all remaining stages | `15,992,363 ns` | `0.4421%` |
| total instrumented | `3,618,142,031 ns` | `99.9998%` after integer truncation |

Linux performance counters are unavailable to the unprivileged process
because this host has `perf_event_paranoid=4`; no system setting was changed.
Stage timing plus external process time is sufficient for this first
discriminator.

## Diagnosis and cycle 1

The serial reconstruction performs canonical fluid and boundary admission
twice for every row: once to count and preflight capacities, then again to
sample kernels. Each call also creates a temporary row vector. The first W2
cycle therefore tests one falsifiable hypothesis: retaining the first
canonical neighbor-index result in bounded flattened scratch arrays will
remove redundant searches and per-row allocations while preserving every row
order and published root.

The candidate keeps the same grid and ascending-`SampleId` suffix
sort/deduplication, records row ranges, accounts worst-case scratch capacity in
the frozen decoded-heap cap, allocates final neighbor records exactly, and
drops scratch before density-factor construction. Allocation or capacity
failure still aborts the local step before publication. Full crate tests,
focused suffix/capacity tests, clippy and boundary scan pass.

Clean commit `e15c5e22bd23af7336e8c4027dba851c323520e9` produces exact-profile
binary SHA-256
`bfd9f9fc3bcc4f15eb530466bac6f14d3f914d4a95a6ed4eb398382db207c520`.
Its report `/tmp/nextengine-w2-cycle1-e15c5e2.json` has SHA-256
`403883df598d8880bfecf1c9544565a629e26eea4eaa0fbe3b8993ed3310d6c3`;
the external time record has SHA-256
`ae7c489b7232eac6905562fb3d906fe7b9e2aafecad4d5504f859e508833b320`.

The three outer step samples fall to `932,221,569`, `860,311,684` and
`850,752,784 ns`, a mean of about `881.095 ms` and a `1.375×` whole-step
speedup. Reconstruction falls from `2,212,373,023` to `1,228,475,543 ns`
over three steps (`1.801×`) and from `61.15%` to `46.73%`; density is now
`48.00%`. CPU remains `99%` of one logical CPU. Maximum RSS rises from
`86,060` to `91,432 KiB`, remaining bounded by the frozen heap admission.

All initial, measured and final frame roots match the baseline exactly, as do
density/divergence iteration counts and short trajectory root
`41390c922ca043cb5daff987fef0457fe8be9e0fa08a9d8c852e38ee5884329f`.
Cycle 1 therefore survives its hypothesis and exact-root discriminator.

## Cycle 2 — deterministic worker substrate

The cycle-1 mean is about `881.1 ms`. Even an impossible perfect division by
all `32` logical CPUs would be about `27.5 ms`, far above the standalone
`4 ms` target. This is not a percentile verdict and does not close W2, but it
means parallel scheduling alone cannot be assumed sufficient.

Commit `c2915cfe599d635ede5d31cf923c52447c92333f` implements the smallest
crate-private worker candidate. It pins Rayon `1.12.0`, creates one local
bounded pool of at most eight threads and always decomposes row work into `64`
logical partitions. Worker count schedules those partitions; it does not
change partition boundaries, admitted-neighbor order, row-local floating-point
reduction order, global convergence reductions or canonical publication.
There is no public worker contract and no shared jobs/resource framework.

Parallel work is limited to independently indexed operations:

- canonical neighbor discovery writes partition-private fallibly allocated
  index fragments, then merges them by logical ordinal;
- kernel sampling writes disjoint preallocated neighbor ranges;
- density factors, pressure acceleration and matrix action write their exact
  canonical row slots;
- convergence, KKT, curvature, impulse and publication reductions retain the
  serial order.

The original serial functions remain the oracle and fallback. Fallible vector
reservations preserve their typed allocation error, worker errors are selected
in canonical partition order, and a failed candidate never replaces the prior
frame. Under the unit-test unwind profile an injected worker panic becomes
`WATER_WORKER_FAILURE`; under the frozen `water-oracle` `panic=abort` profile a
panic is process-fatal before publication, so it is fail-stop but cannot emit
an in-process structured report. Changing that frozen panic policy would
require an explicit profile reclosure and was not smuggled into W2.

The complete crate suite passes `91/91`, strict Clippy passes, and
`boundary-scan` passes all six checks. Focused tests compare the serial step
against worker counts `1/2/4/8`, compare logical partition counts
`1/7/64/127`, and inject both returned errors and panics before publication.

## Clean short scaling matrix

The clean exact-profile binary for `c2915cf` has SHA-256
`da6be3d85aaac1acb795d28975c3035cb7b185cbc1b6ab1a53ebf9116eeacbe5`.
It was built with Rust `1.97.1` (`8bab26f4f`, LLVM `22.1.6`), the frozen
`water-oracle` flags and target `x86_64-unknown-linux-gnu`. THOTH reports an
AMD Ryzen 9 3950X with `16` cores / `32` logical CPUs and one NUMA node.

Each row below is one clean invocation of the fixed one-warm-up,
three-measured-substep sealed-48k diagnostic. Speedup is relative to the
serial invocation in this same adjacent matrix, not to a historical run.

| Execution | Mean outer step | Speedup | Reconstruction mean | Density mean | Whole-command CPU | Peak RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| serial oracle | `837.264 ms` | `1.000×` | `396.735 ms` | `392.097 ms` | `100%` | `92,776 KiB` |
| 1 worker / 64 partitions | `841.645 ms` | `0.995×` | `395.830 ms` | `395.606 ms` | `100%` | `106,200 KiB` |
| 2 workers / 64 partitions | `488.813 ms` | `1.713×` | `215.064 ms` | `231.187 ms` | `162%` | `104,844 KiB` |
| 4 workers / 64 partitions | `325.059 ms` | `2.576×` | `123.686 ms` | `162.078 ms` | `243%` | `105,404 KiB` |
| 8 workers / 64 partitions | `269.643 ms` | `3.105×` | `77.473 ms` | `152.119 ms` | `372%` | `104,860 KiB` |

All five invocations have one identical initial root, warm-up root, three-frame
root vector, density iteration vector `40/36/36`, divergence iteration vector
`1/1/1`, final root and short trajectory root
`41390c922ca043cb5daff987fef0457fe8be9e0fa08a9d8c852e38ee5884329f`.
The reports bind commit `c2915cf...` and `CLEAN` tree state.
Artifacts are `/tmp/nextengine-w2-c2915cf-{serial,workers1,workers2,workers4,workers8}.json`
with matching `.time.txt` files:

| Execution | Report SHA-256 | `/usr/bin/time -v` SHA-256 |
| --- | --- | --- |
| serial | `649336e68bbec13b4a9abd00f5e769eb6202b94cb4c7061dcf0e1ec741e82696` | `53490826caa55223b57dcee32ada85e02c31364222bd1bc2f0ba16c6dfdeb7a1` |
| 1 worker | `a8685e281d8de52c3f78b05878535e86578c5dbf5d66477891ea9f421b826543` | `e129b755c50a91b342d31b634a18a68aa3512312124accfe43fe5b1909baef6b` |
| 2 workers | `38485e1bbd4149714739cc2bc913745ca3e3e8f8da13b084b98e2905e60795d1` | `78ecb3098029b964daa6cd419fcd5429dc4f42b167ba13e24bdaa43cdd70b31b` |
| 4 workers | `436a56cb9b94fa856d1926fd0c20603bbb9c6d0b3386ba149d944361a9b10a20` | `55de33c48f82c36e6a11c7d9d3cdacb7bc8b74cdfb6407eef2681cf7fde34317` |
| 8 workers | `b30a7a4e87bc2bcd4ab4c2c718054ccfb75cb9e9dfafc1cea3f06253a3656f11` | `b0a4fdb1f0888592d6c9dd2db4530c1c33c8fad4ca4f8013e223dcc83a054499` |

The 8-worker reconstruction scales `5.121×`, while density scales only
`2.578×` and improves by merely `1.066×` from four to eight workers. A bounded
dirty-tree counterfactual with `16` logical partitions preserves every root
but regresses the mean to `285.404 ms`; the fixed `64`-partition design is
therefore retained. This rejects fragment granularity as the leading remaining
cause. Repeated pressure-operator memory traffic, per-iteration barriers and
the still-serial vector/global reductions are the next causal boundary.

## Decision boundary

The best short mean is still `67.41×` the `4 ms` p95 ceiling. Even the
optimistic scheduling-only projection obtained by dividing the current serial
mean by all `32` logical CPUs is `26.16 ms`, or `6.54×` the ceiling. Therefore
scheduling tuning cannot make the frozen algorithm meet its named target.

This short diagnostic is not the formal W2 percentile workload, does not prove
all-scenario/full-trajectory worker equality and grants no ProductCheck
credit. The expensive `10k/50k/100k` windows are intentionally not run merely
to obtain a more precise failure. The next action is an explicit decision:
keep this track `RESEARCH_ONLY`, or authorize a new algorithm/data-layout
profile with new roots and renewed correctness evidence. Reducing the sample
count, changing 240 Hz cadence/budgets or promoting GPU authority remains a
separate product/architecture decision, not an optimization result.
