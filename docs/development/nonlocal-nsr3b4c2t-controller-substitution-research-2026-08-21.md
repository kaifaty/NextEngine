# NSR3-B4C2T full-controller substitution research -- 2026-08-21

Status: `COMPLETE / B4B2_BINARY64_REPLAY_SELECTED`

## Question

After B4C2Q proves each private query, what remains before canonical
continuation can be designed?

## Remaining ownership boundary

The B4B2 controller executes multiple complete trajectories per macro frame:

```text
frame start/forecast spectrum
  -> level 0 trajectory
  -> level 1 trajectory
  -> embedded gate
  -> optionally finer levels
  -> commit only the selected fine level
```

It also executes independent fixed `48/96/192` trajectories used as the
physical reference. Bit-exact one-substep substitution does not prove that a
discarded adaptive level cannot leak its final position, velocity, contact
cache, reaction totals or workspace state into the committed level.

## Selected test architecture

Run the unchanged all-pairs B4B2 case as an oracle and a separate candidate
case whose frame-start, feasible-forecast, current, trial and spectrum queries
all use B4C2Q with audit disabled. The candidate is not allowed any all-pairs
or audit-oracle query. Compare:

- complete adaptive controller fields and every frame;
- accepted/executed/discarded substeps and selected refinement depth;
- fixed `48/96/192` runs, convergence and aggregates;
- positions, velocities, contacts, face populations, impulses, reactions,
  ledgers, KKT work, physical comparisons and work gates;
- the exact serialization used by the B4B2 report.

The joint trace stores an incremental query-chain digest and aggregate work
counters rather than every workspace record. B4C2Q already owns per-query
identity evidence; retaining thousands of diagnostic strings would distort
the full-controller memory experiment.

## Performance-accounting boundary

The candidate disables B4C2Q audit calls. Its all-pairs and audit counters must
all be zero. The legacy oracle is executed separately and cannot feed the
candidate. This makes the structural work counters representative of the
selected CPU algorithm, but still does not establish elapsed-time speedup:
static support indexing, allocation reuse, compact construction and SIMD are
not yet packaged.

## Decision

Freeze a complete P1/P2 binary64 B4B2 replay. PASS may authorize B4C3
canonical publish/decode transaction design only. B4D nominal execution and
all runtime/production claims remain blocked.

## Post-execution finding

The selected candidate is exact, but its trace shows `14,149` P1 and `11,860`
P2 workspace builds. Post-step diagnostics and frame aggregates rebuild states
that the accepted solve already owned. Preserve this result and assign
committed-workspace retention, immutable support indexing and nested-row
elimination to B4C4 packaging before nominal execution.
