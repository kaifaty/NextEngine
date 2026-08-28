# Ceramic Cup observation causal diagnostic preflight — PS-2 — 2026-08-28

## Outcome

The bounded post-rejection diagnostic is frozen before inspecting any other
Ceramic Cup listener row. External manifest
`e7b952fe6108aae9b30fe1db956baeb83ed461d8e3923a8119dd83a50e20375b`
binds runner `92d51c021eb96a9c19eb1f21df29849b562c3fcc25b3540646aa7c16f5253ecb`,
the exact rejected parent, decoded block, value-independent row set, unchanged
V2 extractor/gates and outcome classification. Two clean runs emit
byte-identical report
`6e97bc0a14323f604e46386820931d1fb53a83b7d5ff35e01fb0b0ec323a066c`
with decision `CeramicCupObservationDiagnosticFrozen`.

The preflight performs no network access, opens no additional payload, runs no
physics and gives no quality, admission or runtime credit. It authorizes only
two repeat analyses of 27 already decoded development rows.

## Frozen parents

| Parent | SHA-256 / size |
| --- | --- |
| Rejected observation report | `56591bb8…3fd9` |
| Decode report | `9f1c2311…daf0` |
| Decoded block | `3405843a…e6ca`, `501348000` bytes |
| Python extractor source | `cf93b1fc…6772` |
| Rust transfer gate / DSP | `2624656e…f80` / `131bbf42…9ca4` |
| Rust fixture report / samples | `2e3db2d3…5572` / `c8316f81…0477` |

The preflight rehashes the full decoded block and validates the rejected parent
decision, decode dimensions, source identities and Python/Rust parity.

## Frozen axes

- height: rows `0..14`, all microphones at angle/distance `0/0`;
- angle: rows `7,67,…,547`, microphone 7 at all ten angles and distance `0`;
- distance: rows `7,22,37,52`, microphone 7 at angle `0` and all distances;
- union: 27 rows, selected from metadata order without reading their values.

Every row receives the same unchanged five V2 checks. The report also groups
selected modes below versus at/above `500 Hz` and counts fitted decay, tail
decay and positive-fit/negative-tail transitions.

## Frozen classification

- shared failure: at least `80%` of union rows fail and each axis is at least
  `50%` failed;
- reference-listener-local: at least `80%` of the other 26 rows pass;
- low-frequency association: each band has at least 16 modes and the fitted
  decay fraction below `500 Hz` is at most half the fraction above it;
- otherwise publish a mixed spatial result.

These criteria are diagnostic only. A passing row cannot replace frozen row 7,
and no result may tune a threshold or authorize denoising, mechanics, another
object or Planter access.

## Next action

Commit and transfer this checkpoint, then run the exact 27-row analysis twice
from the immutable block. Publish the frozen classification and choose the
smallest follow-up that can falsify it; do not resume mechanics merely because
some individual listener passes.
