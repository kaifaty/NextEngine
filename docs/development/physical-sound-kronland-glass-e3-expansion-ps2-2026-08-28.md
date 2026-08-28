# PS-2 Kronland Glass E3 expansion — 2026-08-28

| Field | Result |
| --- | --- |
| Status | `SEVEN_E3_PROJECT_GROUPS / GLASS_14_OF_16 / REJECT_PARENTS_38_OF_35 / ALL_DEV / PASS_DISABLED` |
| Scope | Five published original Glass recordings as five numbered E3 objects; no geometry, force, split, admission, quality or production credit |
| Adapter | `kronland-material-identified-recording-v1` |
| Source manifest/report | `97e017fc50fda5191246ce8733e7fea10072b06fdb8f96bc1b78aa35d5781ef2` / `a92c002defacf24d2f11523d013cb6f42b2ae54b265071372a82d5517bcf0dbf` |
| Combined source manifest/report | `50d11e367619a85f61d42e8f4828583d6aaf5d9cf7969c95b139b9f0a6425017` / `2a8792a27e7df42bf5fcdf9a1e8832056aeca5c9d5cd1314b7ecfc6797023ee9` |
| Combined identified manifest/report | `d2683a9610c51400cd8d1005f9d10e57969d78f3bd5ced2d0fc3649527819479` / `e2534028e7077d64d76e79705e5386b26bc63688765d52571a1975ffbe88dcb6` |
| External root | `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-kronland-glass-e3-v1/` |

## Bounded research decision

The missing Glass target count survived multiple acquisition cycles, so the
next attempt compared competing exact-download routes rather than tuning an
existing source. [FSDnoisy18k](https://zenodo.org/records/2529934) supplies a
strong clean Glass material envelope but no stable physical-object identity in
its published metadata. The [REALIMPACT project](https://samuelpclarke.com/realimpact/)
publishes five small demo WAVs, but the demo page does not bind the displayed
object to an exact dataset object/material row. Neither receives new E3 credit.

The official [Kronland material-impact publication](https://kronland.fr/publications/controlling-the-perceived-material-in-an-impact-sound-synthesizer/)
does close the narrower identity chain. It states that everyday-life Wood,
Metal and Glass objects were recorded, analyzed, synthesized and tuned, then
publishes five numbered Glass tracks with separate `original`, `synthesized`
and `tuned` links. Only the five `_expe.wav` original recordings are imported.
The adapter does not infer whether a numbered object is a vessel, plate or
another shape.

## Exact source identity

The official project page is `235,670` bytes with SHA-256
`d7171401972ff80416bfe8be4d50aaeaf3b5f3164dae9aa790796b8bbaf24beb`.
Its exact link labels bind the following files:

| Object | Official track | Bytes | Frames at 44.1 kHz | SHA-256 |
| --- | --- | ---: | ---: | --- |
| `glass-v1` | `Glass 1 original` / `AST_v1_expe.wav` | 119,618 | 57,761 | `2e950d3a2257a4af2e99407c644fd61280f9f651a200e22d3ca0d8ca7207b400` |
| `glass-v2` | `Glass 2 original` / `AST_v2_expe.wav` | 93,768 | 44,836 | `87a649a0be249600bf417c51a6fb410a0201a1fc27a667202e19c2a442adab5d` |
| `glass-v4` | `Glass 3 original` / `AST_v4_expe.wav` | 53,506 | 24,705 | `185cb111f532c9760d8c2e956171824a1cb04d48c2ab940fd9e8f5a3bb8031ac` |
| `glass-v5` | `Glass 4 original` / `AST_v5_expe.wav` | 60,832 | 28,368 | `869c13ce80568cf2c1b5b6cdb62d5aac58b03470d21f2a146491546e2202b982` |
| `glass-v6` | `Glass 5 original` / `AST_v6_expe.wav` | 96,680 | 46,292 | `ec7adb4e2fe6a3b68e13501d993be66addcda285b60eb7d974bbe3fac069ab0a` |

Two independent HTTPS acquisitions are byte-identical. Every WAV also matches
the earlier hash-closed AV-P0B benchmark bytes, providing a separate historical
identity check. The in-process validator requires mono PCM16, 44.1 kHz,
non-silence and the exact per-object frame count.

The first registry fetch exposed a transport defect: the safe resolver checked
all DNS answers but selected the lexicographically first public address, which
was IPv6 on a host without an IPv6 route. The fetcher now deterministically
prefers a public IPv4 address when both families are published, while retaining
all-address public-IP validation and pinned `curl --resolve`. The initial
fail-closed report remains external diagnostic evidence; both final caches and
reports reproduce independently.

## Claim boundary and measured consequence

For each numbered object the typed adapter binds the exact page and original
WAV to material identity, object identity, real-recording identity and recording
identity. Unit controls reject a changed material declaration or a page that
relabels the exact track as synthesized. Unknown redistribution terms remain
`NOASSERTION / external_research_only`; no recording enters Git or distributed
content.

The combined report moves from 47 objects/113 recordings in six project groups
to 52 objects/118 recordings in seven. Glass moves from `9/16` objects and 34
recordings to `14/16` and 39; reject parents remain `38/35` and 79 recordings.
All entries remain in `dev`, so calibration/holdout/shadow counts are still
zero and PS-2 remains open. No page metadata provides geometry, composition,
support, excitation, impact/listener position or transfer response.

The next smallest PS-2 action is to acquire two more exact Glass E3 object
groups, preferably from at least one additional project, while continuing
complementary E2/E1 axis coverage. Only after the full target minimum exists
may the project freeze project-disjoint calibration/holdout/shadow splits.
