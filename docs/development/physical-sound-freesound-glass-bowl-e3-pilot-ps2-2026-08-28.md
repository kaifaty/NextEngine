# Physical sound PS-2 — ObjectFolder discriminator and Freesound glass-bowl E3 pilot

Date: 2026-08-28
Status: `FOUR_E3_PROJECT_GROUPS / GLASS_6_OF_16 / E3_EXPANSION_REMAINS_OPEN / NO_CORPUS_ADMISSION_AUTHORITY`

## Outcome

The next bounded PS-2 package adds one independently published Glass object
and eight repeated wood-strike recordings from Freesound pack 14905. Combined
AV-MSF, YCB Impact, CMU Heller and Freesound coverage is now:

- four publisher/project/revision groups;
- fourteen object groups and forty-one recordings;
- six Glass object groups and twenty-five Glass recordings;
- ten of the required sixteen Glass groups still missing;
- zero of the required thirty-five reject-parent groups in this E3 corpus.

Every entry remains in `dev`. Calibration, holdout and shadow remain empty,
automatic `Pass` remains disabled, and this package creates no production
content, runtime contract or P1 credit.

## Source discriminator

Two candidate paths were evaluated before adding another adapter.

### ObjectFolder-Real — strong evidence, no bounded repeat path

The official [ObjectFolder-Real download page](https://objectfolder.stanford.edu/objectfolder-real-download)
describes one hundred real household objects with repeated impact recordings,
force profiles and strike coordinates. Its Glass-labelled object inventory is
scientifically attractive, but the official acoustic downloads are ten-object
gzip tar streams of roughly 34–39 GB each. The reviewed `91–100` archive is
38,866,981,268 bytes.

A bounded prefix inspection found the first microphone/force pair for object
91, followed by large camera/video/depth members. Even after 256 MiB the stream
had not reached a second repeat. Because gzip tar has no independently
addressable central directory, downloading a bounded second repeat cannot be
proved without consuming an unbounded prefix or the complete archive.

Decision: retain ObjectFolder-Real as a high-value discovery target, but do not
add E3 credit or retry archive-prefix growth. Reconsider only if the publisher
exposes per-object audio-only payloads, a seekable archive/index, or another
official bounded retrieval route.

### Freesound pack 14905 — bounded stable-object repeats

The official [Freesound pack page](https://freesound.org/people/ascap/packs/14905/)
identifies one `medium-pitched glass bowl` and explicitly groups sounds by
striker. The selected subset is the eight cards named `wood hit medium glass
bowl 1.mp3` through `8.mp3`. Each card identifies the same author, pack and
object and shows the Attribution NonCommercial license reviewed as
[CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/).

The public HQ MP3 previews total 637,930 bytes. They remain external and
`external_research_only`; compressed previews are sufficient for the scoped E3
identity/envelope evidence but do not establish geometry, wall thickness,
support, force, position, listener or transfer-response claims.

## Implemented boundary

`physical-sound-registry internet-sources` now supports one explicit
`freesound_pack_identity_v1` normalization policy. The raw page contains
dynamic CSRF and download-count fields, so raw HTML cannot be an immutable
artifact. The policy performs one bounded credential-free HTTPS fetch and
emits a canonical JSON projection containing only:

- canonical pack URL, author and author ID;
- pack ID, title and description;
- sorted sound IDs, titles, duration/sample-rate labels, public LQ preview URLs
  and license labels.

The canonical projection is 4,245 bytes with SHA-256
`0b69f458441c57f301372b05b70d03ea17f7be494e85faba4db0bbd3ec60c59b`.
Two independent online fetches produced the same projection and source report.
Normalization cannot be combined with a redirect policy, and an unsupported or
changed page identity fails before cache publication.

The new `freesound-glass-bowl-identified-recording-v1` adapter freezes the pack
identity and eight exact HQ preview URLs, byte counts and SHA-256 values. A
small dependency-free inspector validates actual MPEG-1 Layer III frame
structure, the Xing/LAME gapless frame count, 44.1 kHz stereo format and the
declared decoded sample-frame count. `bits_per_sample = 0` in the normalized
report means compressed encoding; it is not an invented PCM precision.

The adapter grants exactly:

- `material_identity`;
- `object_identity`;
- `real_recording`;
- `repeat_identity`.

It therefore contributes only `E3IdentifiedRecording`.

## Exact external evidence

All source bytes, caches and generated reports remain outside Git under:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-independent-e3-freesound-glass-bowl-v1/`

| Artifact | SHA-256 |
| --- | --- |
| Source manifest | `1f48f56b91e0de970b0ed69eb3b590eb0ea1f55175af2bc17b14276303fd9140` |
| Source report, online A/B and offline | `ef9e77bbd8e285c561509aef91e4f98572d862cfc3bd3d5e56f7b361543a1792` |
| Identified-corpus manifest | `7784a2c64d742dd3085d63917ba15431ba7c6bf1b0ff148593e693482b97efa8` |
| Identified-corpus report, cache A/B and offline | `7f2e00c104d929ecfd9093a5d34737f28c6311f8d0bc6237cc00e58bccdff913` |
| Freesound provenance review | `8679a04702285a7789c10524166feee029fab533167231671d602d4111937a3e` |
| Frozen PS-2 corpus-plan report | `e082610c90dabff3c7a328df94671dca4f84f46cd629952c3e914ce600a3ea01` |

Both online source reports are byte-identical. Both independent-cache
identified reports and the offline repeat are byte-identical. Rerunning the
previous Heller source and identified manifests also preserves their historical
report bytes and hashes
(`531faacddc2b4c45d5a64370c7a3faf1a5429a37b7ee44f968382b5fc208d179`
and `68dd8ec320fb00a56a4e52ffd90b424e4ef4e7b04355c27f8366f6e15714090c`).

## Decision and next action

This package materially improves source diversity and Glass development
coverage, but it does not make the PS-2 sample statistically sufficient. Keep
the Freesound object in `dev`, retain authored-clip fallback, and continue
searching published sources with stable object identity and repeats. The next
bounded package should add another independent E3 object or complementary
claim-scoped E2/E1 evidence; it must not reopen ObjectFolder prefix growth
without a new bounded-access discriminator.
