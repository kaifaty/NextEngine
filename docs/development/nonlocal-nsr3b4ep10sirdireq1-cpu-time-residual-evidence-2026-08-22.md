# NSR3-B4EP10SIRDIREQ1 CPU-time residual evidence -- 2026-08-22

Status: `PASS / TOPOLOGY_STRUCTURAL_AUDIT_AUTHORIZED / NO_WALL_CREDIT`

## Result

The opt-in CPU trace passes all frozen exactness, accounting, stability and
external cross-check gates in three fresh serialized processes. Every report
has identity `6518ed98...964`, result `e5ddff76...14f`, retained SIRDI result
`b4f847cb...77e9`, retained SIRDIR result `1f66ab3c...992e`, correspondence
`1e4bedbb...a35d` and the same five physics roots. Every stderr is empty.

Each run records exactly one transaction, 226 topology, 226 evaluation and
459 HVP totals, 7,275 process phase/total intervals, 4,089 process region
intervals and 32,712 worker-active intervals. Process/thread clock resolution
is 1 ns, the checked seven-category sum is exact, worker-active CPU is no
greater than region process CPU, and there are zero clock failures.

| CPU category | Run shares | Median | Range |
|---|---:|---:|---:|
| topology | `0.291847 / 0.305401 / 0.301239` | `0.301239` | `0.013554` |
| target fold | `0.179761 / 0.186552 / 0.184880` | `0.184880` | `0.006791` |
| directed | `0.147595 / 0.154893 / 0.148680` | `0.148680` | `0.007299` |
| HVP compression | `0.088909 / 0.094057 / 0.090886` | `0.090886` | `0.005148` |
| other local | `0.122460 / 0.124653 / 0.107272` | `0.122460` | `0.017381` |
| stopped evaluation setup | `0.124573 / 0.099839 / 0.124029` | `0.124029` | `0.024734` |
| control | `0.044856 / 0.034605 / 0.043014` | `0.043014` | `0.010251` |

All ranges pass the `<= 0.03` gate. Topology is the only eligible category
above the 0.20 share threshold and leads the second eligible category,
target fold, by `1.629380x`, above the required `1.20x`. The stopped setup
category remains visible but is excluded from routing as frozen.

Internal transaction CPU is
`35.510718649 / 43.632921929 / 34.936758152 s`. GNU user+system is
`35.79 / 43.90 / 35.20 s`; external/internal ratios are
`1.007865 / 1.006121 / 1.007535`, with median `1.007535` inside
`[1.00, 1.05]`. Diagnostic wall is `4.88 / 6.07 / 4.80 s` and grants no
health, throughput or speed credit.

## Reproducibility

Implementation commit is `3b2d3e18f8da92d78da7ea0674db45ccca7c9b01`.
The Release executable has SHA-256
`5b31c60cf2df7b878d830a5ec06ab63a0cc7626a764fc415fe474fc65b803639`,
size 4,403,880 bytes and ELF Build ID
`e12c01c033af655a89ef0edfc3ab839f00b95fb7`. Its
`compile_commands.json` SHA-256 is
`79e135ca1ecbb1d76a862c944b574472ecce6fe4141684590f2da33488f03594`.

A separate probe of the old SIRDI command preserves exact stdout SHA-256
`539f1ec507e439adc50b14cdf5da616024e041a3131aa56a2002c40a83e4e7e7`
and empty stderr. Raw reports and GNU-time records are under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sirdireq1.CjreA2`;
the derived `metrics.json` SHA-256 is
`e4c4dfd2bd472cec5eea51c444f594e22a57b12eb23b2b688a062239fcdeb4e0`.

## Decision

Close B4EP10SIRDIREQ1 PASS. Retain SIRDI and authorize exactly one
timing-free structural audit of topology. The audit must identify removable
work or redundant ownership without changing formulas, order, roots or the
accepted implementation. It must be researched and frozen before code.

This evidence does not authorize another buffer experiment, a topology
implementation, wall A/B, B4E2, broad corpus, CUDA/runtime/schema integration
or production use.
