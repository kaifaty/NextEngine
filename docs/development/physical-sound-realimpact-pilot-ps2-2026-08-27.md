# Physical sound PS-2 — REALIMPACT GlassGoblet pilot

Date: 2026-08-27
Status: `DEVELOPMENT_PILOT_COMPLETE / TYPED_E2_ADAPTER_VALIDATED / FALLBACK_ONLY`

The [active PS-2 internet corpus policy](physical-sound-internet-corpus-policy-ps2-2026-08-27.md)
retains this pilot as claim-scoped transfer-response evidence. Its original
overall `FallbackOutOfDomain` result and frozen hashes do not change; local
recording is no longer the next action.

## Question

Can one real controlled impact response be imported with honest geometry,
impact/listener coordinates, source identity and hashes, while automatically
rejecting any attempt to treat missing acquisition axes as a matched PS-2
domain?

## Bounded source inspection

The official [REALIMPACT project](https://samuelpclarke.com/realimpact/),
[paper](https://jiajunwu.com/papers/realimpact_cvpr.pdf),
[repository](https://github.com/samuel-clarke/RealImpact) and GlassGoblet archive
were inspected without executing downloaded code.

The paper establishes the experiment boundary: synchronized 48 kHz microphone
and calibrated force-transducer acquisition, objects resting on a thread mesh,
five impact vertices, 10 azimuths, four distances and 15 microphones. The
published preprocessing code shows that the hammer trace is gain-corrected and
deconvolved from each microphone signal.

HTTP range inspection of `94_GlassGoblet.zip` found 12 members. The archive is
2,312,738,494 bytes and contains:

- a 48,036-vertex scanned mesh with extent approximately
  `0.0925 x 0.0923 x 0.1560 m`;
- five exact impact vertex IDs/positions;
- 600 unique listener coordinates formed from 10 azimuths, four distances and
  15 microphone heights;
- a `float32[3000, 208457]` force-deconvolved transfer array;
- no downloadable force trace, raw repeat identity, versioned support fixture
  or precise glass-composition revision.

The first transfer row was extracted by bounded range decompression rather than
downloading the full 2.31 GB archive member. It binds impact mesh vertex `12351`
at `[-0.00768129, -0.03310816, -0.0026062] m` to listener
`[0.23, -0.04345, -0.91] m`, has 208,457 samples at 48 kHz and duration
`4.342854 s`.

## Implemented inventory audit

`xtask physical-sound-registry corpus-inventory` validates current-only
external development inventories. It:

- requires exact domain/object/family/source, geometry, support, excitation,
  impact and listener identities;
- binds plan report, audio payload, acquisition metadata and provenance by
  SHA-256;
- validates mono little-endian float32 byte count and rejects non-finite audio;
- audits cross-partition leakage independently for object family, object,
  source, generator revision and mutation parent;
- records unavailable components and deterministically maps any incomplete
  entry to `FallbackOutOfDomain`;
- emits only `DevelopmentPilotOnly` with
  `NO_CORPUS_ADMISSION_AUTHORITY`.

The pilot is one development entry, one object, one family and one source.
Calibration, holdout and shadow remain unopened. The exact deconvolved row has:

- raw SHA-256:
  `15c87b87423e71177e9e3b2ffd3fb0b2ea8ab7c5cbff071f519b2ddda3df325b`;
- 833,828 bytes;
- peak absolute value `368.1817626953125` and RMS
  `5.910620985854733` in force-deconvolved transfer units;
- normalized audition WAV SHA-256:
  `4574ad6fe912064fc8e0b4c11ba0eae459473f52d023b600f5e5de9647c5a2b8`.

The external pilot lives under:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-glassgoblet-pilot-v1/`

Frozen identities:

- inventory manifest SHA-256:
  `964bdc20b27b5af70aa93989864c538c7d8bcab234bcc3fb745c593bfb85642b`;
- inventory report SHA-256:
  `fed6245d5d5e79833e3acbca4a61cd1658f80fdc40268e47d51bd8785b06d3fa`;
- acquisition metadata SHA-256:
  `b9fd65f455df26c66da5bcd04d6ea0bbb580835c42932bf2167cf3d5f4861280`;
- provenance review SHA-256:
  `d16716cafd8ad41d569415130dbdf82383f87d1025dac61c01ae2595f7f9cc51`.

Two reports are byte-identical.

## Decision

The pilot proves that real controlled response bytes and spatial/geometry axes
can enter the automated external inventory without a human per-sound gate. It
also falsifies REALIMPACT's current downloadable GlassGoblet archive as a
complete matched PS-2 source. Four declared components are unavailable:

1. `force-profile-bytes`;
2. `material-composition-revision`;
3. `repeat-recording-identity`;
4. `support-fixture-revision`.

The correct automatic result is therefore `FallbackOutOfDomain`. No geometry,
force or support value was invented, and the pilot contributes zero calibration,
holdout, shadow, risk or coverage credit.

## Next action

Keep REALIMPACT as development `E2` spatial/transfer evidence. The
[typed adapter checkpoint](physical-sound-realimpact-e2-adapter-ps2-2026-08-27.md)
now binds the official repository revision, frozen GlassGoblet row, exact mesh,
impact/listener identities and transfer bytes. It grants scoped E2 capabilities
but deliberately preserves this pilot's overall `FallbackOutOfDomain` result
and missing `E1` axes.

The next blocker is independent published E3 publisher/project coverage, not
this first adapter, local recording or force hardware. Re-plan the multi-source
corpus after adding those sources; do not open shadow or start PS-3/AV-P0D until
the pre-registered grouped evidence requirement can be populated.
