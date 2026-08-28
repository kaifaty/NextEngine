# PS-2 REALIMPACT object-grouped metal batch preregistration — 2026-08-28

## Decision

`RealImpactMetalBatchRolesAndDiscoveryInputsFrozen`.

Two byte-identical preflights freeze the first object/family-grouped batch over
all eight official REALIMPACT names containing exact `Iron` or `Metal` tokens.
Four existing observations are hash-closed; four new archives have only HTTP
HEAD identity and zero member-payload access.

This authorizes one metadata-only archive discovery. It does not authorize new
waveform bytes, candidate evaluation, statistical release credit, material
identity, quality/domain/runtime admission or a production consumer.

## Frozen lineage

| Artifact | SHA-256 / value |
| --- | --- |
| Discovery runner | `ecbaa72478c6cc4475fc204d699e334059707f0474cac284cc09e02f2ccacbfc` |
| Manifest | `873d82e3ea887f89c05fbb9ada924b43f6087062c7394b1be047e2ca54e23ae4` |
| Preflight A/B | `578e143ca137ead516c43d5320df986b4926b3855662322c5325c69ae3eaec31` |
| Existing waveform bytes hashed | `590,839,740` |
| Registry/report lineage bytes hashed | `2,432,971` |
| New member-payload bytes | `0` |

External artifacts are under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-metal-batch-discovery-v1`.

## Frozen partitions

| Role | Objects |
| --- | --- |
| Development | `17_IronSkillet`, `67_IronPlate`, `90_MetalLadle` |
| Calibration | `43_IronMortar`, `86_MetalHoledSpoon` |
| Holdout | `91_MetalSpoon` |
| Shadow | `89_MetalSpatula`, `92_MetalSpatula` |

The two exact `MetalSpatula` names share one normalized family and remain
together in shadow. Objects used to construct or inspect the seed modal
registry are excluded from holdout and shadow. The batch is still too small
for release-risk claims; its purpose is deterministic representation research.

## New archive identities

| Object | Bytes | ETag | Role |
| --- | ---: | --- | --- |
| `86_MetalHoledSpoon` | 2,310,510,455 | `"6433e1b7-89b79777"` | calibration |
| `91_MetalSpoon` | 2,312,590,327 | `"6433e360-89d753f7"` | holdout |
| `89_MetalSpatula` | 2,308,676,719 | `"6433e245-899b9c6f"` | shadow |
| `92_MetalSpatula` | 2,313,811,870 | `"6433e3e8-89e9f79e"` | shadow |

The one failed concurrent HEAD attempt for each of objects 89, 91 and 92 is
recorded; sequential HEAD then succeeded. No retry is authorized for the next
range acquisition.

## Next boundary

Read exactly the final 65,536 bytes and the 30-byte observation local header
from each new archive: eight requests and zero member payload. Audit the cache
twice. Only then may a successor freeze exact metadata/audio member ranges,
decoder shapes, candidate parameters and gates before any new waveform byte.

The successor candidate set is limited to the common-pole modal baseline, a
parametric transient and a deterministic seeded sub-band residual. Per-object
human admission is forbidden; every unsupported object selects authored clips.
