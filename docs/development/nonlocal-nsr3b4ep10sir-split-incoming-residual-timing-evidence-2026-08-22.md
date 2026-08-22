# NSR3-B4EP10SIR split incoming residual timing evidence -- 2026-08-22

Status: `PASS / SOURCE_LOCAL_DISCRIMINATOR_SELECTED`

## Result

All three fresh serialized processes preserve the exact B4EP10SII semantic
result `f7b1542f...25fb2`, all five frozen physics roots and correspondence
`1e4bedbb...08a35d`. Every run reports one transaction, 226 topology calls,
226 evaluations, 459 HVPs, 4,089 executor regions and 261,696 logical
partitions. The timing-only result is byte-stable at
`62a2205d...10aa3` because durations are excluded from it.

The four disjoint category shares are stable:

| Category | Run 1 | Run 2 | Run 3 | Median | Range |
|---|---:|---:|---:|---:|---:|
| topology | 0.201375 | 0.201402 | 0.203107 | 0.201402 | 0.001732 |
| source-local | 0.587730 | 0.588019 | 0.587011 | 0.587730 | 0.001008 |
| target fold | 0.140752 | 0.140160 | 0.140276 | 0.140276 | 0.000591 |
| control | 0.070144 | 0.070418 | 0.069606 | 0.070144 | 0.000812 |

Every range is far below the frozen `0.05` limit. Source-local work owns
58.77% of the transaction and leads topology, the second category, by
`2.918189x`; it therefore passes the `>=0.20` share and `>=1.20x` leader
route.

Top-level stage-share ranges are also stable: topology `0.001732`, evaluation
`0.000934`, HVP `0.000273` and residual `0.000997`. Median executor
orchestration is only `0.011919` and median imbalance `0.071954`, both below
their `0.15` architecture thresholds. Persistent OpenMP regions and partition
rebalancing are rejected as the next research targets.

## Interpretation

This measurement does not grant another speedup claim. B4EP10SII retains its
external A/B credit and B4EP10I remains the rollback. The next permitted work
is one narrower discriminator inside source-local evaluation/HVP work; no
topology, target-fold or executor rewrite is selected.

## Build and regressions

- implementation commit: `4946af8abc21320685ed0294045df6692fb92ff3`;
- executable SHA-256: `7e603325...f6625`, size 4,262,456 bytes;
- Build ID: `74361f9b5578748200907f234e405bd60c807d8d`;
- `compile_commands.json` SHA-256: `30e9925f...a0bc`;
- B4EP10SII stdout remains `31990f6f...17ef`;
- B4EP10R1 still passes with semantic result `a296ee65...560b`.

Source SHA-256 values are `2cd43bbc...3620` for `boundary_reference.cpp`,
`e095f4f2...895d` for `boundary_reference.hpp` and `9b19e65b...3857` for
`formula_reclosure_main.cpp`. The Release build passed the repository's
`-Werror` compile policy and `git diff --check`.

## Decision

Close B4EP10SIR as PASS and research one source-local discriminator before
changing code. B4E2, broad corpus, runtime, CUDA/GPU, schema and production
remain blocked.
