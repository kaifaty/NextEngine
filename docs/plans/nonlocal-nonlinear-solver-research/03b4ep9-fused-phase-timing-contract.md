# NSR3-B4EP9 -- fused phase-timing contract

Status: `CLOSED / PASS / CPU_PARALLEL_ARCHITECTURE_RESEARCH_SELECTED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep9-fused-phase-timing|v1|parent=d8c6584ee93f8b3232441c061bacfb6ddb8f50d5a80805a57c6b53b0968a65e1:8a8d15c316b7e03d1eccfad527543e6a428264f8c339be71efd875b0e7079c4b|candidate=441ac76483748631f94ae4f2d092b97ba4d271fb2c380d2b6e889245527821c9:e3453dc158a8eace69b468ac84641cccdcad221dcc05695d06b6f400039e775c:8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095|implementation=a3aa054217cfd9d21193f8943effe10b7ebce7bf|timers=steady-clock;opt-in;transaction-only;non-overlap;defaults=absent|phases=topology;fused-setup;fused-pair;fused-center;fused-finalize;hvp-apply;residual|calls=transaction1;topology226;fused226;hvp459|runs=3;semantic-exact;timing-json-nondeterministic;result-excludes-time|route=median-parallelizable>=0.80;each>=0.75;range<=0.05=>cpu-parallel-research;otherwise=serial-residual-research|timing=no-throughput-claim|reference=closed|credit=b4ep10-architecture-only
```

Identity SHA-256:
`a75c1db690e26805b7f5d900a053be789316688c9ac033fd7db902f26e37cad6`.

## Implementation boundary

Add an opt-in timing trace and a dedicated command:

```text
--nominal-hydro-fused-phase-timing
```

Use `std::chrono::steady_clock`. Timers are disabled by default and must not
execute for old commands. The command retains the exact B4EP7I topology cache,
coefficient tape, fused evaluation/tape path, physics, query work and evidence
policy. It times only the transaction, not the full-state parent preflight.

Instrument seven non-overlapping quantities:

1. topology/filter/flat-CSR;
2. fused validation/allocation setup;
3. fused pair pass;
4. fused centre/adjacency pass;
5. fused finalization/CSR ownership;
6. pressure-tape HVP application;
7. residual, calculated from transaction total after checking that the phase
   sum does not exceed it.

Required exact call counts are `1/226/226/226/226/459` for transaction,
topology, each fused subphase and HVP respectively. All durations must be
positive, the measured sum must not exceed transaction total and timer failure
count must be zero.

## Semantic and regression gates

`result_sha256` excludes every duration and derived fraction. It includes the
frozen identity, status/failure, B4EP7I physical/publication roots, work chain,
work receipt, exact phase call counts and correspondence booleans. Three fresh
Release processes must exit zero with empty stderr and reproduce one identical
`result_sha256`; their complete stdout is expected to differ.

One final Release build must also reproduce exact stdout for B4EP1, B4EP3,
B4EP3I, B4EP5, B4EP7D and B4EP7I. A timer/counter or semantic mismatch fails
closed; there is no fallback.

## Routing and scope guard

For each admitted run calculate:

```text
parallelizable_fraction =
    (topology + fused_pair + fused_center + hvp_apply)
    / transaction_total
```

Select `CPU_PARALLEL_ARCHITECTURE_RESEARCH` only if the median fraction is at
least 0.80, every run is at least 0.75 and the range is at most 0.05.
Otherwise select `SERIAL_RESIDUAL_RESEARCH`.

This experiment makes no throughput claim and authorizes no parallel
implementation. B4E2, CUDA/GPU, runtime/schema, PhysX coupling and production
remain blocked.

Observed PASS: three semantic results are identical; conservative fractions
are `0.9221543365971554`, `0.92187898636494814` and
`0.92138378827357814`, with range `0.0007705483235772581`. See the
[dated evidence](../../development/nonlocal-nsr3b4ep9-fused-phase-timing-evidence-2026-08-22.md).
