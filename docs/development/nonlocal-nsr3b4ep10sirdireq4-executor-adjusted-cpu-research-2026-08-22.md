# NSR3-B4EP10SIRDIREQ4 executor-adjusted CPU research -- 2026-08-22

Status: `COMPLETE / DERIVED_METRIC_SELECTED / QUALIFICATION_NEXT`

## Question

Why did exact Q3 whole-process CPU vary too much for candidate selection, and
can the already verified Q1 process/thread intervals define a bounded metric
for future algorithmic A/B work on the shared host?

Q3 samples span 30--39 s of process CPU and fail both win and stability gates.
Process CPU excludes external descheduling, but still includes CPU consumed by
OpenMP workers/runtime while threads enter, leave or wait inside a region. It
also remains sensitive to frequency and shared-memory contention.

## Existing exact decomposition

Q1 records, over the same transaction:

- `T`: process CPU for the complete transaction;
- `R`: process CPU inside every OpenMP region;
- `A`: the sum of per-worker thread CPU between entry to static work and exit
  from that work, before the final barrier.

All 4,089 region intervals and 32,712 active worker intervals are exact, with
`A <= R <= T`. Define:

```text
outside-region CPU  O = T - R
runtime residual    X = R - A
executor-adjusted   E = O + A = T - R + A

E + X = T
```

`E` retains serial/control/allocation work outside OpenMP regions plus CPU of
workers while their assigned partitions execute. It excludes the unstable
inside-region residual. It is an algorithmic CPU surrogate, not a claim that
all retained cycles are useful or all excluded cycles are worthless.

## Retrospective discriminator

The three already accepted Q1 runs give:

| Run | T | R | A | O | X | E |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 35,510,718,649 | 25,764,368,542 | 16,122,992,092 | 9,746,350,107 | 9,641,376,450 | 25,869,342,199 |
| 2 | 43,632,921,929 | 34,120,713,562 | 16,634,786,402 | 9,512,208,367 | 17,485,927,160 | 26,146,994,769 |
| 3 | 34,936,758,152 | 25,434,349,345 | 16,689,867,606 | 9,502,408,807 | 8,744,481,739 | 26,192,276,413 |

Range ratios are `1.248912` for `T`, `1.341521` for `R`, `1.999653` for `X`,
but only `1.035159` for `A`, `1.025672` for `O` and `1.012483` for `E`.
Thus the Q3 failure is consistent with unstable executor residual dominating
whole-process CPU variation. This is an inference from exact accounting, not
a causal proof of which runtime/host mechanism consumed those cycles.

## Qualification design

Reuse the unchanged Q1 command and derive `O`, `X` and `E` outside the binary.
No new instrumented code or candidate runs are required. Across three fresh
serialized processes pinned to `0-7`, require all Q1 semantic/count/clock and
GNU cross-check gates plus:

- executor-adjusted `E` range ratio at most `1.03`;
- outside-region `O` range ratio at most `1.05`;
- active-worker `A` range ratio at most `1.05`;
- exact checked identity `E + X = T` in every run.

PASS may authorize research and freezing of one future candidate A/B contract
using paired `E`, while reporting `T`, `X` and wall separately with no speed
credit. It cannot reopen Q3. FAIL means this shared host cannot qualify even an
algorithmic CPU surrogate and a dedicated measurement host is required.

## Limits

The metric deliberately gives no credit for reducing OpenMP entry/barrier/spin
cost because that cost lies in `X` and is unstable here. Such reductions can
still matter to production wall time, but need a qualified throughput lane.
Memory/frequency effects may still perturb `A` and `O`; the fresh stability
gates decide whether they are bounded enough for research.

No result authorizes implementation, candidate selection, Q3 rerun, wall/FPS,
B4E2, broad corpus, runtime/GPU/schema or production work.
