# Physical sound PS-2 — AV-MSF identified-glass E3 pilot

Date: 2026-08-27
Status: `E3_SOURCE_READY / ONE_OBJECT_TWO_RECORDINGS / NO_CORPUS_ADMISSION_AUTHORITY`

## Question

Can the internet-source registry validate one bounded official real recording
source strongly enough to grant only the acoustic claims its published page and
payload bytes establish, while rejecting forged metadata and malformed audio?

## Frozen official source

The official [AV-MSF project page](https://zisenshao.github.io/AV-MSF/),
[repository](https://github.com/ZisenShao/AV-MSF) and
[paper](https://arxiv.org/abs/2608.05145) describe real-world impact recordings
used by Objects as Audio-Visual Modal Sound Fields. The repository `page`
branch was frozen at commit
`723df64a94480fc8f8e592c66c0d916e8b0054d1`. Its exact page card identifies:

- object: `Object 95`;
- original material: `Glass`;
- demo path: `data/demo/95`;
- contact recordings: `012,036`.

Only three immutable raw Git objects are in scope:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `index.html` | 27,239 | `b7e20b4f6af55f0037379c78b23e4841869daed70d84c240187eaa50b8900b49` |
| `impact012.wav` | 529,258 | `f7bcd74454bce303be0ed7c4a215c7438c4860d9a6107ebb1eb9a1e1604c380c` |
| `impact036.wav` | 529,258 | `e187b72cbff6153489aae55b3beaf87375427069e243295547bd751b2153d57f` |

The WAVs are each three seconds: mono 44.1 kHz IEEE float32 with 132,300
frames. The publisher surface reviewed for this import did not state a
redistribution license, so the manifest uses `NOASSERTION` and
`external_research_only`; no source byte or report is committed.

## Source-specific adapter

`av-msf-identified-recording-v1` is a fail-closed adapter rather than a generic
WAV label. Before granting evidence it requires:

- the exact publisher/project/page identity and a 40-hex Git commit;
- the exact immutable raw-commit paths implied by object and recording IDs;
- exact byte counts and SHA-256 for one page and each declared recording;
- the page's real-world/impact text and exact object/material/recording card;
- RIFF length and chunk closure, mono 44.1 kHz IEEE float32 encoding, matching
  `fact`/data frame counts, a bounded duration, finite samples and non-silence.

It validates exactly `material_identity`, `object_identity`, `real_recording`
and `repeat_identity`. That capability intersection grants only
`E3IdentifiedRecording`. It cannot grant force, geometry, impact/listener
position, support, calibration or transfer-response claims.

Focused controls prove that an alternate otherwise-valid HTTPS payload URL is
rejected before fetch, and a hash-closed WAV containing `NaN` fails without
publishing a report. Existing synthetic manifests retain their prior report
shape because adapter evidence is omitted when no typed profile exists.

## Reproducible result

External pilot root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-av-msf-e3-glass-v1/`

- manifest SHA-256:
  `a1de51fdd8f3bdcca0e7a11551b45ea91a6bf43520439da2708729351f667480`;
- provenance-review SHA-256:
  `fe8a8a290b81e6405a831095a66d44b1c12ecb441b28ed734c3d206609f8886c`;
- report SHA-256:
  `d762c92e8bd3210944ee7802fa7fef71e10776a62b4da132eb92cb0d33c56231`;
- result: `SourceSetComplete`, one `EvidenceReady` source and only
  `E3IdentifiedRecording`;
- reproducibility: two independent fresh online caches and one later offline
  audit produced byte-identical reports.

`SourceSetComplete` means only that every source declared in this one-source
manifest satisfied its adapter-backed tier. The report still states
`NO_CORPUS_ADMISSION_AUTHORITY`.

## Decision and next action

The previous generic-networking/real-adapter blocker is closed for one E3 glass
source. It does not close PS-2: one object and two recordings have no useful
grouped statistical power, no exact glass-vessel domain axes and no matched E2
spatial transfer.

The follow-up
[multi-object pilot](physical-sound-av-msf-e3-multiobject-pilot-ps2-2026-08-27.md)
has now normalized the complete ten-card AV-MSF surface. It measures ten object
groups/twenty recordings but only two Glass groups and one shared
publisher/project/revision source group. The
[REALIMPACT E2 adapter](physical-sound-realimpact-e2-adapter-ps2-2026-08-27.md)
now closes one exact transfer row while preserving fallback. Next, add
independent published E3 publishers/projects before re-planning or opening
calibration, holdout or shadow. The pre-registered requirements remain at least
35 independent reject-parent groups and 16 in-domain groups; insufficient
internet coverage remains `FallbackOutOfDomain`, never a request for local
capture or human per-sound approval.
