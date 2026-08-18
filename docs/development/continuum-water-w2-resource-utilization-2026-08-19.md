# Continuum water W2 resource-utilization discriminator — 2026-08-19

Status: `REPORT_ONLY / CYCLE_1_COMPLETE_ROOT_EXACT / NO_W2_CREDIT`

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

## Risk and next discriminator

The cycle-1 mean is about `881.1 ms`. Even an impossible perfect division by
all `32` logical CPUs would be about `27.5 ms`, far above the standalone
`4 ms` target. This is not a percentile verdict and does not close W2, but it
means parallel scheduling alone cannot be assumed sufficient. Cycle 2 must
now implement stable logical partitions with canonical merge, compare serial
and worker `1/2/4/8` roots, and apply the two-cycle stop rule without changing
sample count, cadence, thresholds or CPU authority.
