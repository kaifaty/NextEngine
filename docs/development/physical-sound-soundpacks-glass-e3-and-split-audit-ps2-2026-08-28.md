# PS-2 SoundPacks Glass E3 and split-feasibility audit — 2026-08-28

| Field | Result |
| --- | --- |
| Status | `GLASS_16_OF_16 / REJECT_PARENTS_38_OF_35 / SPLIT_INFEASIBLE_3_OF_4_REJECT_PROJECTS / ALL_DEV / PASS_DISABLED` |
| Scope | Two object/family-bound Glass E3 groups plus a deterministic project-disjoint split-feasibility gate; no partition freeze, admission, quality or production credit |
| Adapter | `soundpacks-glass-recordings-identified-recording-v1` |
| Source manifest/report | `353a4ef5d978f6bdfa4ecb5c32ee5d62d87f24daac5d207e55534e6370fb5bbd` / `8dac0722fe1e84d575b5c7f89916cd91ce7ababcc127f4ba870765d149903012` |
| Combined source manifest/report | `50bde57fd9892fde20879149fe9d7083280e2479683e563f0edd9846197bbcbf` / `d1b1a199d79a3d5be848c00319c704a6945e6f9261caa91b5a5fa912e799e491` |
| Combined identified manifest/report | `5add7711d44a77c484fec2ef6c1b861c04c4eacf4907373615dfe9adf195cb19` / `48b543da13f040be827c6ef20f081a91b4c98bc1a9af45271108f66100bcb6fa` |
| Split-feasibility report | `731011e178bb0822d532a9074f0581783519f8606c597d114700deb04656b0ff` |
| External roots | `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-soundpacks-glass-recordings-discriminator-v1/` and `ps2-soundpacks-glass-e3-and-split-audit-v1/` |

## Source decision

The canonical [SoundPacks Glass Recordings page](https://soundpacks.com/free-sound-packs/glass-recordings/)
identifies `kaffekrus` as the creator and describes field recordings of apartment
windows, mirrors, drinking glasses, vases and other household objects. It binds
39 WAV samples, including 28 glass hits, to MediaFire file key
`nxuj8iiakqecpnu`. The stable [MediaFire landing page](https://www.mediafire.com/file/nxuj8iiakqecpnu/Glass_Recordings_by_kaffekrus.rar/file)
names the 33.53 MB archive. Dynamic recommendations and the expiring MediaFire
download host/token are transport state, not source identity.

The new fetch path therefore has two source-specific gates:

- `soundpacks_glass_recordings_identity_v1` validates the frozen publisher,
  description, counts, format and archive key, then emits a stable 734-byte JSON
  identity independent of recommendation widgets;
- `mediafire_file_v1` accepts only the exact canonical landing page and one
  bounded dynamic `download<digits>.mediafire.com` URL carrying the same file key
  and archive name. Generic redirects remain disabled.

The exact RAR5 archive is 35,155,966 bytes with SHA-256
`ee4e64741b33f678fdd6d1537b53bcb66417393ec233bae5531da61cef5a9aec`.
A bounded pure-Rust `rars` reader rejects SFX, encryption, split members, unsafe
paths, duplicate names and declared or aggregate expansion outside fixed limits.
The 447-byte readme and every selected embedded WAV are checked by exact byte
count and SHA-256 before float32 stereo 44.1 kHz structure, finite samples,
non-silence and frame counts are accepted.

## E3 identities and claim boundary

| Object/family | Recordings | Frames | Embedded WAV SHA-256 |
| --- | --- | --- | --- |
| `drinking-glass` | `drinking glass1..4.wav` | 132,300; 114,660; 70,560; 101,430 | `f34feec9…`, `9ae504a6…`, `6787469f…`, `24cb4b07…` |
| `metallic-vase` | `metallic vase1..3.wav` | 79,380; 70,560; 119,070 | `3d1fcefa…`, `c45d456d…`, `ccade1b6…` |

The official page, archive/readme identity, stable filenames and exact WAVs
jointly grant only material identity, pack-specific object/family identity,
real-recording identity and numbered-repeat identity. `metallic vase` remains the
publisher filename for a glass-vase family; it is not relabelled as Metal. The
evidence does not establish exact geometry, wall thickness, composition,
support, force, impact/listener position, transfer response or a single retained
physical specimen across all numbered files. The readme prohibits redistributing
the pack, so source bytes remain `external_research_only` and do not enter Git or
distributed engine content.

Two fresh online caches and an offline replay produce the same source report
SHA-256 `8dac0722…`. Two independently assembled complete cache roots then
produce byte-identical 54-source and identified-corpus reports.

## Measured corpus result

The combined development corpus moves from seven to eight independent
publisher/project/revision groups, from 52 to 54 objects, and from 118 to 125
recordings. Glass moves from `14/16` objects and 39 recordings to `16/16` and 46.
Reject parents remain `38/35` objects and 79 recordings. Both aggregate count
requirements in the frozen PS-2 plan are now satisfied.

This does not authorize a split. The current corpus has this project-level role
structure:

- eight projects contain target evidence;
- three projects contain reject-parent evidence;
- those same three projects contain both roles;
- five projects are target-only.

The new `physical-sound-registry split-feasibility` command hash-closes the
identified report and frozen corpus-plan report, verifies that the complete
pre-split E3 corpus is still in `dev`, applies the declared `20/30/25/25` basis
points by deterministic largest remainder, and requires target plus reject-parent
presence in each of `dev`, `calibration`, `holdout` and `shadow`. Eight projects
allocate as `2/2/2/2`, but only three reject-bearing projects exist for four
partitions. The exact decision is `ProjectDisjointSplitInfeasible`; both runs
produce report SHA-256 `731011e1…`.

## Consequence and next action

PS-2 aggregate coverage is complete, but project-disjoint split readiness is
not. Calibration, holdout and shadow remain empty, `Pass` stays disabled and
PS-3/AV-P0D do not start. The next smallest acquisition is at least one new
independent publisher/project/revision group containing object-bound non-Glass
reject-parent recordings. More target-only Glass from the same source cannot
remove this blocker. After that source is imported, rerun the feasibility gate;
only a feasible report may be followed by a separately hash-closed partition
assignment and sealed-shadow workflow.
