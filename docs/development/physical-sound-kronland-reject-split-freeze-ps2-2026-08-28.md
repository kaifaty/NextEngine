# PS-2 Kronland reject expansion and project split freeze — 2026-08-28

| Field | Result |
| --- | --- |
| Status | `REJECT_PARENTS_48_OF_35 / SPLIT_FEASIBLE / PROJECT_DISJOINT_SPLIT_VERIFIED / PASS_DISABLED` |
| Scope | Ten object-bound Wood/Metal E3 recordings, stable Kronland page identity, deterministic project-level partition freeze and independent post-split verification; no admission, quality or production credit |
| Adapter | `kronland-material-identified-recording-v1` |
| Source manifest/report | `380279ef26e9752190ebc52f9a31d611a909aff832a20050b2e45ab2022698b7` / `c9db7948f7ce33a2e9007cb0c45560031d752160c1a0275a799a697154907138` |
| Combined source manifest/report | `c913ce1cbf5cd6f966850ce8ad79fd7d4c2316687c36200f91f30188e41d92e5` / `53d546a1c6d56acdf4d0558121516a60a8428df17ad9a35ac6d4d903197a9366` |
| Pre-split identified manifest/report | `8c8b56711d135bf1b9dfc85a319e3e02bdb581ed3318524558ed38d48da7f861` / `effef6558f788cdaf523242d429e2da6246df6ec759df598181fab830388cdbe` |
| Split feasibility report | `d4a0a001fa0b793cde11b6311b64bae1364a76baf7b2c1512147a6670da15313` |
| Partitioned manifest/report | `9ddec04de271c034f1389f5fcec98ce410028b307ae565b3f890eb7f7a726c33` / `9f5b7f8a2a94d57ea874d7a58a513d8012999b8b43e94fa34f4094de8c68d4f0` |
| Split verification report | `7cb1532cb5e94792dd27d5fd13a1c11a2c2e5fbecbc017a2c2bcbcb74dc2a9f8` |
| External root | `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-kronland-reject-split-v1/` |

## Source decision

The official [Kronland material-impact publication](https://kronland.fr/publications/controlling-the-perceived-material-in-an-impact-sound-synthesizer/)
publishes five numbered original recordings for each of Wood, Metal and Glass,
alongside separately labelled synthesized and tuned derivatives. The previous
E3 adapter deliberately imported only the five Glass originals. This increment
adds the ten original Wood and Metal recordings as explicit `reject_parent`
objects under the same publisher/project/revision identity. It does not credit
the synthesized or tuned derivatives.

The live WordPress page includes rotating nonces and other dynamic state, so a
raw whole-page hash is no longer a stable source identity. The new
`kronland_material_page_identity_v1` policy emits a canonical JSON projection
containing the publication identity, recorded Wood/Metal/Glass scope and all
fifteen exact original/synthesized/tuned track labels and URLs. It rejects a
missing material scope, missing original or a derivative substituted for an
original. Two fresh online acquisitions and one offline replay produce the
same 5,127-byte projection with SHA-256
`2d01dadce8fbb43b2cc1ed7cabf1f3531873afc527a991990ca1b8a81c588c34`
and the same ten-source report. The legacy five-Glass manifest still produces
its prior report SHA-256
`a92c002defacf24d2f11523d013cb6f42b2ae54b265071372a82d5517bcf0dbf`.

All ten WAVs are mono PCM16 at 44.1 kHz, non-silent and bound by exact byte,
frame and SHA-256 expectations. The publication establishes numbered object,
material, real-recording and recording identity only. It does not establish
exact geometry, wall thickness, alloy/species, support, force, impact/listener
position or transfer response. Redistribution terms remain `NOASSERTION`, so
the bytes are `external_research_only` and remain outside Git and engine
content.

## Aggregate and feasibility result

The combined E3 corpus still has eight independent
publisher/project/revision groups because the ten objects extend the already
counted Kronland project. It grows from 54 to 64 objects and from 125 to 135
recordings. Glass remains `16/16` objects and 46 recordings. Explicit non-Glass
reject parents grow from `38/35` objects and 79 recordings to `48/35` objects
and 89 recordings.

Kronland changes from target-only to dual-role, producing this project-level
structure:

- eight target-bearing projects;
- four reject-parent-bearing projects;
- four dual-role projects and four target-only projects;
- zero reject-only or role-less projects.

Two complete source audits and two pre-split identified audits are
byte-identical. Two `split-feasibility` runs then return
`ProjectDisjointSplitFeasible` with no blockers: eight projects allocate as
`2/2/2/2`, and four reject-bearing projects can cover the four planned
partitions.

## Frozen project-disjoint assignment

`physical-sound-registry split-freeze` binds the pre-split E3 report,
feasibility report and frozen corpus plan by SHA-256. It uses plan seed
`physical-sound-ps2-glass-vessel-v1` and
`seeded_sha256_first_feasible_backtracking_v1` to find the first bounded exact
assignment that satisfies project capacities and target/reject presence. The
result keeps every object and recording from one project in one partition:

| Partition | Project groups | Objects | Recordings | Reject objects/recordings |
| --- | ---: | ---: | ---: | ---: |
| `dev` | 2 | 30 | 70 | 27 / 54 |
| `calibration` | 2 | 6 | 20 | 3 / 9 |
| `holdout` | 2 | 12 | 27 | 8 / 16 |
| `shadow` | 2 | 16 | 18 | 10 / 10 |

Each partition has two target-bearing projects and exactly one dual-role
reject-bearing project. Two independently rebuilt partitioned-corpus reports
are byte-identical. A second `split-freeze` verification compares every one of
the 135 post-split entries against the pre-split identity, expected project
partition and aggregate counts; both verification reports are byte-identical
and return `ProjectDisjointSplitVerified`.

## Consequence and next action

The former `3 reject projects / 4 partitions` blocker is closed, and the
calibration/holdout/shadow identities are now hash-closed. This is still only a
corpus-structure result. It does not show that one exact physical-sound domain
has matched geometry/support/excitation/listener evidence, does not measure a
validator threshold or false-pass risk on the frozen partitions, and does not
open shadow to optimizer feedback.

PS-2 therefore remains active with `Pass`, PS-3 admission and AV-P0D disabled.
The next smallest work package is an exact-domain claim matrix over the frozen
E2/E3 evidence: enumerate which Glass-vessel geometry, support, excitation,
position, listener/radiation and real-identity claims are actually supported,
leave unavailable axes unavailable, and select a PS-3 release candidate only
if one bounded domain is complete enough for calibration, grouped holdout and
one sealed shadow evaluation.
