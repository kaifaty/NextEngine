# PS-2 REALIMPACT metal batch payload/candidate protocol — 2026-08-28

## Decision

`RealImpactMetalBatchPayloadAndCandidateProtocolFrozen`.

Two byte-identical preflights freeze exact metadata/audio access, decoder
shapes, three candidate representations, stochastic identity, metrics,
calibration selection, holdout/shadow gates and mandatory fallback before any
new member-payload byte is read.

This authorizes implementation and a repeated zero-access execution preflight.
It does not yet authorize network acquisition, candidate execution, quality or
domain admission, runtime use or statistical release claims.

## Frozen lineage

| Artifact | SHA-256 / decision |
| --- | --- |
| Protocol runner | `c3d0b406c8f96d86508e5db2ee3e5a2c2b37763d7c4190f65780fac00c77e2ea` |
| Manifest | `2423197420b3c77b67441043323f89a6dc539ff00292007f8255856a08ab724b` |
| Preflight A/B | `bd7557418546495ccae9689e00c17ad59e94348b4aa18a896ab073e1a55709e3` / `RealImpactMetalBatchPayloadAndCandidateProtocolFrozen` |
| Discovery audit parent | `5790f8cc…a1219` |
| New payload / network in preflight | `0 / 0` |

Artifacts remain external under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-metal-batch-protocol-v1`.

## Exact access envelope

| Item | Frozen value |
| --- | ---: |
| Metadata ranges / bytes | `9 / 5,462` |
| Audio prefixes | `4 × 33,554,432 bytes` |
| Maximum requests | `13` |
| Retry / prefix growth / object substitution | forbidden |
| Decoded listener rows | `0..14` |
| Analysis horizon | `60,000` samples at `48 kHz` |
| Calibration stop | `8,192` samples |
| Future windows | `8192..16384`, `16384..32768` |

Metadata must prove one shared angle/distance/vertex and ordered microphone IDs
`0..14` before analysis. Any identity, range, inflate, shape or condition
failure stops the batch without another request.

## Frozen candidates

1. `modal_only_common_pole_v2`: unchanged rank-7 all-input-region modal
   baseline and undamped diagnostic.
2. `modal_plus_parametric_transient_v0`: eight fixed frequency bands, one
   non-negative exponential per band fitted only on samples `0..4096`, zero
   after sample `8192`, deterministic SHA-256-counter excitation.
3. `modal_plus_seeded_subband_residual_v0`: the same bands, a two-exponential
   non-negative envelope selected from a fixed eight-value decay grid on
   samples `0..8192`, rendered to sample `32768` with the same deterministic
   excitation family.

Both candidates use per-listener band RMS ratios from the calibration window.
The random seed binds manifest hash, object ID and band index; no ambient RNG,
clock or platform entropy is allowed.

## Selection and evaluation

Gating metrics are log-band-envelope MAE, listener-window-energy MAE and
spectral-flatness absolute error. Waveform NRMSE remains diagnostic because
random-phase residuals should not be selected by sample alignment.

Development/calibration select at most one candidate by frozen relative gates
and a lexicographic tie-break. Only that candidate may open the one-object
holdout; shadow opens only after the holdout report is frozen. Both Spatulas
are evaluated as one family. Threshold tuning after either partition is
forbidden.

All gates are representation-transfer gates, not perceptual-quality limits.
Any failure returns `authored_clip_required`; success still leaves
quality/domain/runtime admission false.

## Next action

Implement the hash-bound execution runner, deterministic excitation and metric
fixtures. Repeat its zero-access preflight before using the 13 exact ranges.
Acquisition, decode, development/calibration selection, holdout and shadow must
publish separate immutable stage reports so later data cannot affect earlier
decisions.
