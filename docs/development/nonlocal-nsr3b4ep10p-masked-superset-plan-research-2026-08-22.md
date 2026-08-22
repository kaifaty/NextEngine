# NSR3-B4EP10P masked superset plan research -- 2026-08-22

Status: `COMPLETE / MASKED_SUPERSET_PLAN_AUDIT_SELECTED`

## Question

B4EP10R1 routes to `evaluation_plan`: its median 1.427 s is 42.33% of
evaluation. The timed interval contains a 6,000-centre canonical energy fold
and `build_joint_owner_gather_plan`. The latter repeatedly constructs an
active target transpose for every one of 226 workspaces.

The operation counts separate these candidates without another timer. Across
the nominal transaction the energy fold visits 1,356,000 centres. Plan
construction scans 150,845,996 source slots, 131,987,230 active slots for
fill and 263,974,460 target entries for validation: at least 546,807,686
record visits, over 403 times the centre count. The selected target is the
plan, not the order-sensitive scalar energy fold.

## Existing invariant

B4EP3 already proves one fixed Verlet-style superset with 225 certified reuses
and no rebuild after the first query. Every current pair list is produced by
stable filtering and compaction of that superset. Each current source row is
therefore a stable subsequence of its superset row.

The active owner plan applies a second stable filter:

```text
superset target row
  -> keep current pair
  -> keep source with positive compression
  -> remap fixed superset slot to current compacted slot
  -> current active target row
```

If that identity holds per target, the evaluation and every HVP gather see the
same source/slot sequence and therefore perform floating additions in the
same order. This permits one fixed target CSR in the topology cache while the
per-query state is only an active-slot mapping and compression mask.

## Alternatives

1. **Parallel rebuild with partition-local histograms.** Viable, but still
   allocates and writes the same transpose 226 times. A stable counting-sort
   implementation also needs a logical-partition-by-target histogram and
   several extra passes.
2. **Memoize active plans by mask hash.** Rejected before evidence: query
   positions and compression change, so hash/compare does not establish reuse
   and collision-safe admission still needs the full mask.
3. **Atomic scatter into gradients.** Rejected: it does not preserve floating
   addition order and reopens nondeterminism already closed by B4EP10D.
4. **Persistent OpenMP team or repartitioning.** Rejected by B4EP10R1's 1.11%
   orchestration and 6.55% imbalance medians.
5. **One masked superset target CSR.** Selected for structural audit. It
   removes repeated plan construction if its filtered rows are exact and the
   extra full-row scans remain bounded.

## Risks and discriminator

The fixed plan scans entries for inactive geometric pairs and non-positive
pressure centres before skipping them. That may trade plan-build time for
target-gather bandwidth. It also adds a persistent full transpose and a
per-query superset-to-current slot map. Neither cost may be guessed from the
1.068591 pair-superset ratio because the pressure-active ratio is different.

B4EP10PD must therefore report:

- exact filtered target-row equality for all 226 active plans;
- one full plan build and 225 cache reuses;
- exact source order, current-slot order and two-target coverage;
- projected full-plan scans over all 226 evaluation and 459 HVP gathers;
- scan expansion at most `1.35x` the existing 749,890,172 target entries;
- combined nominal added payload at most 64 MiB;
- deterministic rejection of a corrupt slot mapping and target entry.

The audit changes no returned solver path, adds no timing and does not alter
OpenMP. A PASS may authorize a separate B4EP10PI opt-in implementation/A-B
contract. A structural, scan or capacity failure falls back to research of
the partition-local stable counting-sort rebuild.

## Decision

Freeze B4EP10PD before implementation. No speedup, B4E2, broad-corpus,
runtime, GPU, schema or production claim follows from this design.
