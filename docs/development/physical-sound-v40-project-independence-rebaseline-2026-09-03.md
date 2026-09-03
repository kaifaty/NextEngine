# Physical sound V40 — project-independence planning rebaseline

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Status | `PLANNING_CORRECTION / ZERO_NEW_SIGNAL / PROTECTED_FRONTIER_WITHDRAWN_PENDING_AUDIT` |
| Supersedes | V39's use of the replayed `6 exact-Steel / 23 non-Metal` deficit as a current protected-admission frontier |
| Product effect | None; no payload, role, model, validator, cooker or runtime path is opened |

## Observation

[V39 F1](physical-sound-v39-f1-source-frontier-result-2026-09-03.md)
correctly and repeat-exactly replays the V30 whole-project allocation and adds
the IETeasy metadata increment. Its baseline, however, still credits these two
large projects toward future protected roles:

| Project/revision credited by F1 | Exact Steel | Non-Metal |
| --- | ---: | ---: |
| ObjectFolder Real rendered-table identity | `17` | `63` |
| YCB Impact `osf-bj5w8-2022-09-27` | `6` | `7` |

The later V39 policy makes a whole publisher/project revision the minimum
independence unit and says that a project whose signal entered development can
never return to the protected pool. Historical evidence already proves signal
access inside both project families:

- the [YCB E3 pilot](physical-sound-ycb-independent-e3-pilot-ps2-2026-08-27.md)
  decoded eight recordings from the same `bj5w8` project revision;
- the [ObjectFolder Beer Glass result](physical-sound-r3a-v10-beer-glass-real-fit-result-2026-08-31.md)
  decoded `6,336,000` fit samples, and the
  [V24 Blue Bowl result](physical-sound-v24-x0-blue-bowl-pilot-result-2026-09-01.md)
  used three more ObjectFolder recordings;
- the V39 ObjectFolder revision key binds a rendered metadata table, not an
  independently demonstrated fresh audio-payload revision. A metadata hash
  cannot reset earlier project-family signal exposure;
- V24 also disclosed four REALIMPACT `6_Bowl` transfer rows. REALIMPACT is not
  credited by F1, but it must enter the permanent disclosed roster and cannot
  be used as a future protected project without a separately proven new
  project/revision and physical-parent boundary.

No new waveform, feature, model output or protected value was opened for this
rebaseline. The correction follows only from tracked manifests/results and the
checked-in F1 profile.

## Consequence

The `6/23` result remains an exact historical replay of the V30 planner, but it
is not a valid current protected-admission frontier under V39's stricter
project rule. ObjectFolder and YCB must be quarantined from protected credit
until an exact project-family exposure owner proves otherwise.

Removing their credited counts leaves a provisional metadata-only upper bound:

```text
8 Freesound pack projects: 9 exact-Steel / 3 non-Metal
IETeasy project:           0 exact-Steel / 4 non-Metal
provisional clean maximum: 9 exact-Steel / 7 non-Metal across 9 projects
```

This is not a new frontier: the Freesound/IETeasy projects still require one
complete historical exposure and alias audit. Relative to the unchanged raw
two-role floors (`32` exact Steel and `70` non-Metal before project-balanced
allocation), the provisional pool is at least `23` Steel and `63` non-Metal
groups short even before the five reserved-project constraint is applied.

## Decision

1. Withdraw `6/23` as current planning authority; preserve it only as an exact
   F1 replay result.
2. Start V40 with a zero-signal project-family exposure ledger over every
   historical source, revision alias and opened payload.
3. Permanently assign ObjectFolder Real, the opened YCB revision and opened
   REALIMPACT revisions to the disclosed lane unless exact evidence proves a
   genuinely independent successor revision.
4. Continue useful ML development on disclosed sources immediately after that
   roster is frozen; do not wait for protected-source sufficiency.
5. Compute a new protected frontier only from projects whose metadata identity,
   payload revision, physical parents and historical access all pass the new
   owner.

## Reconsideration condition

An excluded project may regain protected eligibility only if a hash-closed
publisher record proves that the candidate is a genuinely new project/revision
with disjoint physical parents and no opened signal, derived feature, model
target or selection influence. A changed metadata page or filename alone is
insufficient.

