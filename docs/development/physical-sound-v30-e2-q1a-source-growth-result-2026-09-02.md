# Physical sound V30 E2 — Q1a source-growth result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `Q1A_RAW_GROWTH_VERIFIED_BALANCED_ROLE_POWER_REQUIRED / E3_WHOLE_PROJECT_PARTITION_OOD / REPEAT_EXACT / ZERO_SIGNAL` |
| Protocol | [Q1a-M source growth](physical-sound-v29-q1a-metal-source-growth-protocol-2026-09-02.md) |
| Roadmap | [V30 E2–E4](../plans/physical-sound-synthesis-roadmap-v30.md) |
| Product effect | None; every protected payload stays sealed and authored clips remain authority. |

## Outcome

One fresh atomic metadata capture completed after provider cooldown. Two
network-disabled audits of that same immutable capture are recursively
byte-identical. Q1a adds eight independent publisher/project revisions, nine
exact-Steel groups and three non-Metal groups, taking combined raw power to ten
projects, 32 exact-Steel groups and 73 non-Metal groups.

Those aggregate floors do not close Q1. Exhaustive whole-project allocation
cannot populate two disjoint protected roles while reserving five projects for
the remaining roles. The best frontier fills role A, but role B still lacks six
exact-Steel groups and 27 non-Metal groups. The decision is therefore a
source-power OOD result, not a role freeze or permission to open audio.

## Frozen artifact identities

All capture and audit artifacts remain outside Git under the external root
`physical-sound-v30-e2.ZXWn6G`:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `capture/capture.json` | 11,358 | `46a8427350249518655f43f9bc2ed1c9645d053a231a6d6d8cd7e564154602b7` |
| `run-a/metal-source-growth-audit.json` | 31,439 | `c40716fee9c43ebd0132bc14a0d8c77acd01feca9dc7518282a7cca5336bdccd` |
| `run-a/report.json` | 3,407 | `93d95fe7ef1e4233f3421850d488a8a48d56cd9d6394236af2fa0ca3613458f4` |

`run-a` and `run-b` pass recursive comparison with no differing file. The
profile remains 5,739 bytes with SHA-256
`b3dcf8bbae6a37e24ef03e50fb033d0576a93cb168c87596264e40042456c2e9`;
Q0-M and Q1-M identities remain unchanged.

## Publisher-markup conformance repair

The first offline audits failed before publication because all 19 selected
sound pages now leave `data-username` empty in the player element. The same
pages independently bind exact `og:url`, `og:audio:artist`, numeric uploader ID,
canonical author link and page-title author.

The owner now accepts an empty player username only when all four publisher
identity anchors agree with the preregistered author and sound URL. Any
non-empty mismatch, missing anchor, wrong URL, non-numeric uploader ID or pack
mismatch still fails closed. This is an adapter conformance repair: no source,
profile, material policy, threshold, group, role or signal changed. The same
capture then produced the exact A/B result.

Focused Q1a tests pass `13/13`; combined Q0-M/Q1-M/Q1a tests pass `36/36`,
including positive current-markup and negative author-substitution fixtures.

## Power and partition result

| Gate | Result |
| --- | --- |
| Added role-capable projects `>= 7` | `PASS`, observed `8` |
| Added exact-Steel groups `>= 9` | `PASS`, observed `9` |
| Added non-Metal groups `>= 1` | `PASS`, observed `3` |
| Combined raw exact-Steel `>= 32` | `PASS`, observed `32` |
| Combined raw non-Metal `>= 70` | `PASS`, observed `73` |
| Two protected whole-project roles plus five reserved projects | `OOD` |
| Zero protected-signal access | `PASS` |

The best exhaustive frontier is:

- role A: two projects, 18 exact-Steel and 63 non-Metal groups; no deficit;
- role B: three projects, 10 exact-Steel and eight non-Metal groups; deficits
  are six exact-Steel and 27 non-Metal groups;
- five projects remain reserved and unassigned.

The capture made 27 public HTML requests and read 1,486,198 publisher-metadata
bytes. Audit made zero network requests. Audio previews, headers, waveforms,
PCM, force samples, features, meshes, payload bytes and role signals all remain
at zero.

## Decision and next action

E2 is complete and E3 closes OOD. Q1R, Validator V1 and disclosed-real training
remain blocked. E4 must search only the measured role-B frontier: independent
project revision(s) whose whole-project contribution can close six exact-Steel
and 27 non-Metal groups while preserving five reserved projects. Prefer a
multi-object source carrying both families; repeated recordings of the same
object or redundant projects that cannot improve a feasible partition do not
count.

Freeze every new metadata source under a fresh gap-directed increment, rerun
the same offline planner, and proceed to Q1R only if the unchanged whole-project
gate becomes feasible. Do not open source payloads during E4.
