# PS-2 REALIMPACT metal batch holdout result — 2026-08-28

## Decision

`RealImpactMetalBatchHoldoutRejected`.

The previously selected `modal_plus_seeded_subband_residual_v0` improves the
log-band envelope on the unopened Metal Spoon, but it misses the frozen
listener-energy threshold and makes spectral flatness worse. Two executions
emit byte-identical reports. The two-object Spatula shadow remains sealed,
quality/domain/runtime admission remains disabled and the authored clip stays
required.

This is an immutable representation-transfer rejection. It is not a claim
about audible metal quality, material identity, mechanics or the wider
REALIMPACT corpus.

## Frozen lineage

| Artifact | SHA-256 / result |
| --- | --- |
| Runner / manifest | `8484f947…13cc` / `bffaa21c…a8664` |
| Calibration selection A/B | `83f858bf…43f2`, byte-identical |
| Holdout acquisition report | `6653fea3f2dd770b04d6d621756c2702c8002c5a4bccee0e80c26119652d32c0` |
| Holdout decode report | `5fd5ec04af4bd719419671ec026b6ea13ba827de11483317872463457238a633` |
| Holdout evaluation A/B | `fef9dd344836b8ab05935d9c70892e544f306e6844ea8c09528add67ad00cb73`, byte-identical |
| Shadow payload | `0` |

Artifacts remain external under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-metal-batch-execution-v1`.

## One-shot holdout access

Exactly three HTTP range responses were consumed for `91_MetalSpoon` with no
retry or prefix growth:

| Range | Bytes | SHA-256 |
| --- | ---: | --- |
| metadata `2310791007..2310791655` | 649 | `d2a1a4bb…c770f3` |
| metadata `2312588312..2312589021` | 710 | `0da444b2…78f769` |
| audio `261..33554692` | 33,554,432 | `65989745…789aad` |

The decoder verifies a shared `0°/0 mm`, vertex `6840` condition with
microphone IDs `0..14`. The decoded block is `15×208585` f32, 12,515,100
bytes, SHA-256 `1d5309ea…6c1505`. Decode and evaluation use no network, new
member payload or physics solver.

## Frozen evaluation

| Metric | Modal baseline | Selected candidate | Ratio | Frozen limit | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| Log-band envelope MAE, dB | 20.166067 | 10.773659 | 0.534247 | ≤ 0.95 | Pass |
| Listener-window energy MAE, dB | 5.750402 | 5.515997 | 0.959237 | ≤ 0.95 | Reject |
| Spectral-flatness absolute error, dB | 6.907048 | 7.278392 | 1.053763 | ≤ 0.95 | Reject |

The maximum gating ratio is `1.053763`, also above the frozen `1.05` maximum.
Waveform NRMSE changes from `0.901297` to `1.651029`; it remains diagnostic and
does not alter the preregistered decision.

The evidence discriminates the current hypothesis: independently seeded,
band-limited, two-exponential residual energy can transfer envelope decay, but
its listener-normalized random-phase representation does not preserve all
energy and flatness statistics on a new spoon family.

## Leakage consequence

- Do not retune thresholds, gains, decay grids or seeds against Metal Spoon.
- Do not open `89_MetalSpatula` or `92_MetalSpatula` under this rejected
  lineage; the execution runner requires a successful holdout predecessor and
  will reject shadow acquisition.
- A successor must be a new hash-closed representation hypothesis with a new
  grouped split. The opened Metal Spoon may be a diagnostic counterexample,
  never that successor's holdout.
- Before another acquisition, run a zero-network diagnostic on already opened
  rows to distinguish temporal spectral covariance, listener coupling and
  marginal-envelope explanations. Diagnostics may choose a hypothesis, not a
  production candidate.

## Checks

- holdout acquisition: three exact responses, no retry or prefix growth;
- decode: exact headers, metadata, condition, shape and PCM hash;
- evaluation A/B: byte-identical report `fef9dd34…cb73`;
- shadow access: zero bytes and not attempted;
- current ProductChecks: not run because no production consumer, schema or
  runtime contract changed.

## Next action

Freeze a zero-network residual-representation diagnostic over already opened
development/calibration/Metal-Spoon rows. Compare temporal spectral covariance
and cross-listener coherence against the modal residual, choose one falsifiable
successor hypothesis, then preregister a fresh grouped protocol before opening
any new audio.
