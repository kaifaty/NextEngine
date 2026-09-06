# Physical sound V20 M0 — phase-consistent metric result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT_REJECT / METRIC_CONTRACT_DEFECT / I1_SEALED` |
| Protocol | [V20 M0](physical-sound-v20-m0-phase-consistent-metric-protocol-2026-09-01.md), SHA-256 `aa27ecfe9c7c91db47568cd41a9270592cc0bb796a6ae16ebc3e72b04b25e26a` |
| Implementation | commit `b8bbeb39e574ab2fcbbe91a628e36183383baceb` |
| External runs | `v20-m0-phase-consistent-run-a` / `v20-m0-phase-consistent-run-b` |
| Complete tree | both `12f3ce7298dadff53688262f9d3f34b202b9881d7bfbf4eaf2c60a753549fbfd` |
| Product effect | None; P0c remains blocked and no I1, real, protected, cooker, demo or runtime value opened |

## Decision

M0 rejects exactly and closes this metric revision. The implementation is
deterministic and the component evidence remains valid, but the blocking
metric ensemble cannot assign a threshold with the preregistered `0.90x`
separation margin:

- raw multiresolution log magnitude misses frequency separation narrowly;
- absolute band/time decay energy confounds damping with accepted modal-gain
  changes and fails decisively.

This is a measuring-instrument defect, not evidence against B0 or F0. Their
artifacts remain immutable. I1 `1701…1712` and every real/protected role remain
sealed.

## Exact execution evidence

Both independent executions emitted the same five files and bytes:

| Artifact | SHA-256 |
| --- | --- |
| `access-ledger.json` | `5c29bd2ee5d0decdad9435a823727a47281754df227ba549cb2a19d14f4a2899` |
| `corpus.json` | `2ef1fd1f69161824dc7856a27f250a29d45b4e828fe8150399db07e422cb0e0e` |
| `manifest.json` | `c52f0c8b1cd49e84657b556066339cc2c05cb8792534959ec442af5de51def51` |
| `metrics.jsonl` | `550306a20306ea251f321a802df3e3f4b3c43917be2ccf7364ccf95994be99ae` |
| `report.json` | `2bc3c3cf39195e0c9b6fef604182cb4dabb947324cba563cf716c0fe0cf6da51` |

The run evaluated 24 development views, 192 fixed queries and 58 controls,
giving 11,136 complete finite metric rows. One tree is `9.3 MiB`, below the
`100 MiB` ceiling; each run completed in under one minute on the active host.
The access ledger records zero I1/integration/successor rows, zero real or
protected samples, zero dataset/source/checkpoint bytes and zero network
requests.

## Gate result

Nine of ten single-run gates pass in both executions:

| Gate | Result | Evidence |
| --- | --- | --- |
| component hard/physical | `PASS` | Every B0, C0 and F0 gate passes; all four hard mutations reject `12/12`. |
| complete and finite | `PASS` | `24` views, `192` cases, `58` controls and `11,136` rows are complete. |
| harmful physical owners | `PASS` | Frequency, damping, sign and mode-removal controls all reject through their declared owner. |
| identity zero | `PASS` | MRSC/MRLM/DE/TE are `<=1e-12`. |
| legacy attribution | `PASS` | Maximum absolute delta from V19 is exactly `0.0`. |
| polarity invariant | `PASS` | Acoustic metrics are `<=1e-12`; raw waveform NRMSE is at least `1.9`. |
| serialization | `PASS` | Canonical JSON round-trip is exact. |
| severity monotonicity | `PASS` | Every assigned Spearman coefficient is `>=0.9999999999999999`. |
| resource/access | `PASS` | All ceilings and zero-access fields pass. |
| metric separation | `REJECT` | Four of ten family/metric cells miss the frozen margin. |

## Failed separations

The gate compares corpus p95, including identity, B0-only, F0-only and combined
composition as acceptable controls:

| Family / owner | Max acceptable | Min harmful | Ratio | Required |
| --- | ---: | ---: | ---: | ---: |
| uniform frequency / MRLM | `0.785000298` | `0.850001436` | `0.923528202x` | `<=0.90x` |
| alternating frequency / MRLM | `0.778903306` | `0.860173494` | `0.905518841x` | `<=0.90x` |
| positive damping / DE | `1.744948036` | `1.088476852` | `1.603109918x` | `<=0.90x` |
| negative damping / DE | `1.744948036` | `1.106327228` | `1.577244048x` | `<=0.90x` |

MRSC already passes uniform and alternating frequency at
`0.890030102x/0.892955907x`. Mode removal, onset delay and impulse separation
also pass. The decay failure is causal: F0-only/combined changes the relative
modal amplitudes, so absolute time-frequency cell energy can exceed the error
from the first harmful damping step even when damping itself is exact.

## Consequence

- Do not loosen `0.90x`, square a monotonic metric, remove a physical gate or
  choose a favorable subset after this result.
- Retain raw MRLM and absolute DE as diagnostics; neither can own an I1 gate.
- Research a preregistered successor that residualizes amplitude from spectral
  shape and compares normalized decay slope rather than absolute cell energy.
- Commit that protocol and implementation before another complete execution.

The bounded successor analysis is recorded in
[M0b research](physical-sound-v20-m0b-confound-resistant-metric-research-2026-09-01.md).

