# PS-2 REALIMPACT object-grouped metal batch discovery result — 2026-08-28

## Decision

`RealImpactMetalBatchDiscoveryCacheVerified`.

One frozen acquisition uses exactly eight range requests to read four 65,536
byte ZIP tails and four 30-byte observation local headers. Two offline audits
reproduce byte-identical reports. New audio and metadata member payload remains
zero bytes.

This proves observation capacity and exact ZIP identities for the frozen
calibration, holdout and shadow objects. It does not authorize waveform access,
candidate evaluation or quality/domain/runtime admission.

## Frozen lineage

| Artifact | SHA-256 / value |
| --- | --- |
| Runner / manifest / preflight | `ecbaa724…acbfc` / `873d82e3…23ae4` / `578e143c…aec31` |
| Acquisition report | `d0a051ebbe39dffdf4ae8f01a4109a439c4f0a6e89b89d7933e85829a34b0468` |
| Audit A/B | `5790f8cc1db0fc5472915b89a0a8236611c274d2c5cc3a817e9ed4d0152a1219` |
| Network / cached structural bytes | `8 / 262,264` |
| New member-payload bytes | `0` |

Artifacts remain external under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-metal-batch-discovery-v1`.

## Observation entries

| Object / role | Compressed bytes | Inferred f32 shape | Data offset |
| --- | ---: | ---: | ---: |
| `86_MetalHoledSpoon` / calibration | 2,308,176,023 | `3000×208736` | 475 |
| `91_MetalSpoon` / holdout | 2,310,790,746 | `3000×208585` | 261 |
| `89_MetalSpatula` / shadow | 2,306,531,327 | `3000×208614` | 2,140,650 |
| `92_MetalSpatula` / shadow | 2,311,492,574 | `3000×208584` | 2,317,967 |

All entries are deflated `deconvolved_0db.npy` members with a 128-byte NPY
header. The smallest row has 208,584 samples, so all objects can support 15
synchronized listener rows and the frozen 32,768-sample evaluation horizon.

## Interpretation

The family split is now mechanically executable without full archive download.
Exact metadata members are also present for angle, distance, microphone and
vertex identity. They are not opened yet; their local offsets only define the
next bounded access protocol.

The batch remains a representation experiment. A candidate that improves a
relative residual metric cannot promote material identity or perceptual
quality, and the one-object holdout plus one two-object shadow family grants no
release-risk bound.

## Next action

Freeze exact metadata ranges and one 32 MiB observation prefix per new object,
with no growth/retry. The same successor must bind decoder shapes, common-pole
modal baseline, parametric transient, deterministic seeded sub-band residual,
calibration-only selection metrics and unchanged fallback semantics before the
first member-payload byte is read.
