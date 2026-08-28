# PS-2 REALIMPACT metal batch calibration selection — 2026-08-28

## Decision

`RealImpactMetalBatchCandidateSelected` with
`modal_plus_seeded_subband_residual_v0`.

The fit-role acquisition/decode succeeds and two byte-identical
development/calibration executions select one representation before any
holdout or shadow access. This is relative representation evidence only, not
perceptual quality, material identity, exact-domain admission or runtime
credit.

## Frozen lineage

| Artifact | SHA-256 / result |
| --- | --- |
| Runner / manifest | `8484f947…13cc` / `bffaa21c…a8664` |
| Repeated execution preflight | `1af90ed4…e4f68` |
| Fit acquisition report | `454933ae9745f2d9d0e3871a37aa8b7794daa1de28a2d68b529d3d1e4c5163d8` |
| Fit decode report | `af7af9fb84824e3046e90c201f464f904cd05b58b513ec9cc6cfb567d94e068a` |
| Selection A/B | `83f858bf8822f12649677b21845d3755ffcc12580417e9779aab9974d1ce43f2` |
| Selection repeat | byte-identical |
| Holdout / shadow payload | `0 / 0` |

Artifacts remain external under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-metal-batch-execution-v1`.

## Fit-role access and decode

Exactly two HTTP range responses were consumed for `86_MetalHoledSpoon`:

| Range | Bytes | SHA-256 |
| --- | ---: | --- |
| metadata `2310507171..2310508545` | 1,375 | `37f4d8cd…5f93b` |
| audio `475..33554906` | 33,554,432 | `b9801c85…e72ed` |

The decoder verifies four typed metadata arrays and a shared `0°/0 mm`, vertex
`38440` condition with microphone IDs `0..14`. The decoded block is
`15×208736` f32, 12,524,160 bytes, SHA-256 `d1f0f8e7…e8c48`. No retry,
prefix growth or extra request occurred.

## Selection result

The modal baseline stays inside its frozen work envelope on all five fit
objects: retained mode counts are `30, 26, 12, 13, 11` for Iron Skillet, Iron
Plate, Metal Ladle, Iron Mortar and Metal Holed Spoon respectively.

Calibration uses only Iron Mortar and Metal Holed Spoon:

| Candidate | Envelope ratio | Listener-energy ratio | Flatness ratio | Max object/metric | Improved fraction | Result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| truncated parametric transient | 0.493789 | 1.000000 | 1.000000 | 1.000000 | 0.0 | Reject |
| seeded two-exponential residual | 0.387055 | 0.709616 | 0.552735 | 0.943533 | 1.0 | Select |

The selected residual improves every frozen aggregate gate. On Holed Spoon its
envelope/energy/flatness ratios are `0.366567/0.943533/0.542231`; on Iron
Mortar they are `0.407542/0.475699/0.563239`.

Waveform NRMSE remains diagnostic and is not used for selection. Random-phase
residual synthesis is not expected to preserve sample alignment; the selected
candidate therefore does not claim transparent resynthesis or audible metal
quality.

## Leakage consequence

This immutable report is the prerequisite for opening only the exact
`91_MetalSpoon` holdout ranges. Both `89_MetalSpatula` and
`92_MetalSpatula` remain sealed. Thresholds, candidate ID and all DSP rules are
now immutable for holdout/shadow; failure returns `authored_clip_required`.

## Checks

- acquisition: two exact responses, no retry, no failure;
- decode: exact header/condition/shape/hashes;
- selection A/B: byte-identical report;
- Ruff format/check, Python byte compilation and `git diff --check`: pass;
- current ProductChecks: not run because no production consumer or runtime
  contract changed.

## Next action

Acquire and decode only `91_MetalSpoon`, evaluate only the selected seeded
residual, then commit its immutable holdout report before any Spatula request.
