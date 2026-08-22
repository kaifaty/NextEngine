# NSR3-B4EP10PI -- masked superset plan implementation/A-B contract

Status: `CLOSED / FAIL_PERFORMANCE / ACTIVE_PLAN_RETAINED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10pi-masked-superset-plan-implementation|v1|parent=82be83e5131ae5eb3c49a687a764c851f79a81c107417922fd7285e35a986ce1:c9b664a3f57864d8916f6db40f233e2019e0c83c9ef80e089fa6212d14b078ce|implementation=a53b411b2b96f8b1f12555f022f27a8524400b6f|commands=baseline:nominal-hydro-owner-parallel-8,candidate:nominal-hydro-masked-superset-plan-8|storage=fixed-plan-cache-owned;mapping-workspace-owned;pointer-transaction-lifetime|candidate=active-plan-builds0;fixed-build1;reuse225|gather=full-target-row;skip-missing-current-slot;skip-nonpositive-source;use-current-directed-value|energy-fold=serial-canonical|work=evaluation226;hvp459;full-scans966239080;retained749890172|capacity=actual-added<=67108864|semantics=b4ep10pd-structural-oracle;physics-roots-exact;old-commands-exact|timing=external-monotonic+gnu-time;one-warmup-each;three-pairs=AB,BA,AB;serialized;affinity=0-7|gates=candidate-exact-3of3;wins3of3;median-paired-speedup>=1.05;candidate-range-ratio<=1.10;rss-delta-kib<=16384|failure=retain-b4ep10i-active-plan|reference=closed|credit=masked-residual-timing-research-only
```

Identity SHA-256:
`45dcdee2ce3b5aee7b6f324abc68d9de392c6493a33c4a9d577d9013ca5e197b`.

## Implementation boundary

Add only:

```text
--nominal-hydro-masked-superset-plan-8
```

The candidate reuses B4EP10PD's cache-owned fixed plan and workspace mapping.
It must not call the active-plan builder. Old commands must not populate or
dereference the candidate pointer/mapping and remain byte-exact. No new
OpenMP region, atomics, floating reduction, internal timer, runtime option or
public schema is allowed.

The fixed plan pointer is admitted only while a transaction-local topology
cache is bound. Missing/wrong plan identity, mapping size, mapped slot, source,
target or compression coverage rejects the query before a workspace passes.
Every workspace is released before cache destruction.

## Exactness and work

Require unchanged frame, aggregate, trajectory and both ledger roots; frozen
query/work receipts; exact energy, KKT and adaptive-level gates; 226 evaluation
and 459 HVP calls; zero candidate failure or fallback.

Require one fixed build, 225 reuses, zero active plan builds, exactly
966,239,080 fixed entries inspected and 749,890,172 entries retained. Report
evaluation and HVP scan/retained counts separately. Actual fixed plan,
mapping and owner scratch peak must not exceed 67,108,864 bytes.

The candidate's three measured stdout streams must be byte-identical. The
final binary must preserve baseline worker-8 stdout SHA-256
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10PD stdout SHA-256
`ed205b78a818fbcef6a1b2edfef644af2451f422d809f37fa25696c1eb9f316e`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

## External A/B

On the same final Release binary and otherwise idle host:

1. pin both commands to physical CPUs `0..7` with `OMP_PLACES=threads`,
   `OMP_PROC_BIND=close` and dynamic teams off;
2. run one unmeasured warmup per command;
3. run three serialized pairs in order `AB`, `BA`, `AB`;
4. record monotonic wall nanoseconds, GNU Time user/system and maximum RSS;
5. require empty program stderr and exact stdout before admitting duration.

PASS requires candidate wins `3/3`, median same-pair baseline/candidate wall
at least 1.05, candidate wall max/min at most 1.10 and candidate median RSS no
more than 16,384 KiB above baseline median.

## Exit

PASS authorizes only separately frozen masked-path residual timing research.
Any functional failure rejects the implementation. A performance-gate failure
retains B4EP10PD structural evidence but keeps B4EP10I as selected execution.
B4E2, broad corpus, runtime/GPU/schema and production remain blocked.

## Closure

B4EP10PI fails its performance gate; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10pi-masked-plan-evidence-2026-08-22.md).
The candidate is exact, wins all three pairs, is stable and lowers RSS, but
median paired speedup is only `1.030796x` versus the frozen `1.05x`. The
active-plan path remains selected; deterministic parallel active-plan
construction research is allowed next.
