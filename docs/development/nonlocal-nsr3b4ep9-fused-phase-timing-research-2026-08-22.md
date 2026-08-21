# NSR3-B4EP9 fused phase-timing research -- 2026-08-22

Status: `COMPLETE / CONTRACT_FROZEN / IMPLEMENTATION_AUTHORIZED`

## Question

B4EP8 leaves complete fused-workspace construction and exact pressure-tape
HVP balanced at 3.61 s and 3.59 s. Its sampled profile cannot distinguish the
inlined pair and centre passes. Is enough of the exact nominal transaction
spent in independently partitionable particle/pair work to justify research
into deterministic CPU parallelism, or should the serial residual be studied
first?

## Competing hypotheses

1. **The transaction is strongly parallelizable.** Topology, fused pair,
   fused centre and HVP application together own at least 80% of measured
   transaction time and remain stable across fresh processes.
2. **Setup/ownership/control is material.** Allocation, validation, ownership
   transfer, nonlinear control or publication prevents an 80% parallelizable
   share even though gprof's two largest call-tree categories look balanced.
3. **The attribution is unstable.** Fresh-process fractions vary by more than
   five percentage points; another architecture decision would be premature.

The timer experiment distinguishes these hypotheses without changing solver
arithmetic, topology, ownership or evidence identity.

## Measurement boundary

Add opt-in `std::chrono::steady_clock` timers to the existing research trace.
They are enabled only by a dedicated command and cover one nominal Hydro
transaction, excluding its full-state parent preflight. The non-overlapping
phases are:

- topology/filter/flat-CSR construction;
- fused validation and allocation setup;
- fused canonical pair pass;
- fused canonical centre/adjacency pass;
- fused final validation and CSR ownership transfer;
- exact pressure-tape HVP application;
- residual transaction time, derived as transaction total minus all measured
  phases.

The expected call counts are one transaction, 226 topology builds, 226 calls
of each fused subphase and 459 HVP applications. All old commands leave timing
disabled and must retain exact stdout.

Nanoseconds are diagnostic and deliberately excluded from `result_sha256`.
Each of three fresh processes must reproduce the same semantic result while
its JSON timing fields may differ. This is not a throughput benchmark.

## Routing rule

Define the conservative parallelizable fraction as:

```text
(topology + fused pair + fused centre + HVP apply) / transaction total
```

Setup/allocation, finalize/ownership and residual control are excluded from
the numerator. Authorize only B4EP10 deterministic CPU-parallel architecture
research when:

- median fraction is at least 0.80;
- every run is at least 0.75;
- maximum minus minimum fraction is at most 0.05.

Otherwise select serial residual research. Neither route authorizes threads,
GPU work, B4E2 execution, runtime/schema changes or production claims.

## Decision

Freeze B4EP9 as a measurement-only implementation. No optimization may be
stacked into the timed path.
