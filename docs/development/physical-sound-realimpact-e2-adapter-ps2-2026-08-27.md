# Physical sound PS-2 — typed REALIMPACT E2 adapter

Date: 2026-08-27
Status: `TYPED_E2_ADAPTER_IMPLEMENTED / ONE_TRANSFER_ROW_VALIDATED / FALLBACK_ONLY`

## Question

Can the existing REALIMPACT GlassGoblet pilot receive `E2 transfer response`
credit without trusting self-consistent coordinates in generic external JSON or
claiming unavailable raw-force, material, repeat or fixture evidence?

## Frozen upstream identity

The adapter freezes the official [REALIMPACT project](https://samuelpclarke.com/realimpact/)
and [repository](https://github.com/samuel-clarke/RealImpact) at Git commit
`fca2bd6cbb7e9f96ac61328d2a0d51594bf01987`. Five source artifacts are required
by exact SHA-256:

| Role | Bytes | SHA-256 |
| --- | ---: | --- |
| Repository `README.md` | 515 | `3dd228b826651745f0cba8c8bfc0ff142f8fb4d9574f5adda91c272ffd8a2049` |
| Repository `LICENSE` | 1,070 | `328ed037c524ac2f73c859183ca6bf07d034dc28dae918fb3897051a6fd8b937` |
| `preprocess_measurements.py` | 6,762 | `db55f2017a037fb50a7b8b59542e85e35a7c7d5581b15fb75b25dbdf67737bbc` |
| `preprocess_annotations.py` | 1,530 | `66c4ab81d39a1ef4e26a72561dd2fa5238224f31ed089b27ca9a80d33852985a` |
| `dataset/download.sh` | 168 | `4d6c2d7967d7dc2b8c56c7bfd7fe1550202099b6a7b1207aadc40e19d553a45a` |

The source code establishes 48 kHz measurement, hammer calibration and force
deconvolution, and names the exact geometry, impact, listener and annotation
arrays. The adapter does not execute that code.

## Implemented boundary

`physical-sound-registry corpus-inventory` now accepts a current-only V2
manifest with `source_adapter.schema =
realimpact_force_deconvolved_transfer_v1`. This first profile is deliberately
narrow. It supports only `94_GlassGoblet`, transfer row `0`, and freezes the
existing acquisition metadata, deconvolved float32 row and provenance review by
their exact hashes. A future object or row requires a new profile that verifies
its source arrays instead of reusing this identity.

The adapter cross-checks:

- official repository revision and all five source files;
- canonical archive URL, HTTP identity and central-directory hash;
- object/material identity, mesh hash, vertex count and bounded mesh extent;
- transfer array path, shape, dtype, CRC and bounded byte dimensions;
- row index, impact vertex/position, listener identity/position and sample
  dimensions;
- the actual external float32 payload hash, peak and RMS;
- the four unavailable claim axes.

V2 rejects a deconvolved transfer without a typed adapter. A wrong frozen
source hash or a coordinate mismatch is fail-closed and publishes no report.
V1 remains supported so historical inventories preserve their exact bytes.

## Measured result

The external pilot lives under:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-e2-adapter-v1/`

Frozen identities:

- manifest SHA-256:
  `0701c98bbcdb30404540016eb7bf5b0d209e405f58553ff1e9cf841b24f83307`;
- V2 report SHA-256:
  `f33d82e3b72fa2de23c365c6abc0663a58ee668451c8f0b601416608f3a6b6db`;
- original V1 report SHA-256:
  `fed6245d5d5e79833e3acbca4a61cd1658f80fdc40268e47d51bd8785b06d3fa`.

Three V2 reports are byte-identical. A fresh V1 regression run is byte-identical
to the original report.

The one entry receives exactly these adapter-backed capabilities:

- `force_deconvolved_transfer`;
- `geometry`;
- `impact_position`;
- `listener_position`;
- `object_identity`;
- `real_recording`.

The report labels that evidence `E2TransferResponse`, but its overall outcome
remains `DevelopmentPilotOnly / FallbackOutOfDomain`. It receives no corpus
admission, calibration, holdout, shadow or risk credit because
`force-profile-bytes`, `material-composition-revision`,
`repeat-recording-identity` and `support-fixture-revision` are still absent.

## Decision and next action

The bounded REALIMPACT E2-adapter existence gate is closed for one transfer
row. It proves that published spatial/transfer evidence can be consumed without
local recording or invented axes. It does not close PS-2, identify a general
glass domain or authorize formula fitting against holdout/shadow.

The next PS-2 discriminator is independent published E3 publisher/project
coverage and then a coverage re-plan. Additional REALIMPACT rows are useful only
through a profile that directly verifies their official arrays and preserves
object/source grouping. `Pass`, PS-3 and AV-P0D remain disabled until the frozen
35 reject-parent and 16 in-domain group requirements can be populated without
leakage.
