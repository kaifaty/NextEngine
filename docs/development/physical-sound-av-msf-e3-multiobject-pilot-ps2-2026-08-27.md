# Physical sound PS-2 — AV-MSF multi-object E3 coverage pilot

Date: 2026-08-27
Status: `E3_CATALOG_MEASURED / 10_OBJECTS_20_RECORDINGS / GLASS_2_OF_16 / NO_CORPUS_ADMISSION_AUTHORITY`

## Question

Can the complete public demo-card surface of AV-MSF be normalized into a
leakage-safe identified-real corpus, and how much independent Glass coverage
does it actually contribute to the frozen PS-2 power plan?

## Implemented boundary

`physical-sound-registry identified-corpus` consumes an external current-only
manifest, an external internet-source cache and the frozen corpus-plan report:

```text
cargo run -p xtask -- physical-sound-registry identified-corpus \
  --manifest /external/identified-manifest.json \
  --cache /external/internet-source-cache \
  --output /external/identified-corpus-report
```

The command does not trust a copied source report. It reruns the exact
`internet-sources` audit offline against cached bytes, requires every source to
be adapter-backed `E3IdentifiedRecording`, and requires the four E3
capabilities to be available: material identity, object identity, real-recording
identity and repeat identity.

It then derives publisher/project/revision, object and recording groups,
enforces one partition per source/object group, and reports coverage against
the frozen requirement of 16 in-domain groups. It refuses a manifest that
places two objects from the same publisher/project/revision source group in
different partitions. Its only successful decision is
`DevelopmentCoverageMeasured`; the report claim is explicitly
`GROUP_AND_COVERAGE_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY`.

This separate E3 normalizer is intentional. The existing `corpus-inventory`
requires exact geometry, support, excitation, impact and listener coordinates
for E1/E2 transfer evidence. AV-MSF does not publish those axes on the demo
cards, so inserting `unspecified` values would fabricate evidence.

## Frozen official source

The input freezes the [official project page](https://zisenshao.github.io/AV-MSF/),
[official repository](https://github.com/ZisenShao/AV-MSF) and
[paper](https://arxiv.org/abs/2608.05145) at page-branch commit
`723df64a94480fc8f8e592c66c0d916e8b0054d1`.

The complete frozen page exposes ten object cards, two real contact-impact WAVs
per card and six publisher material labels:

| Material label | Object groups | Recordings |
|---|---:|---:|
| Ceramic | 3 | 6 |
| Glass | 2 | 4 |
| Iron | 1 | 2 |
| Plastic | 1 | 2 |
| Polycarbonate | 1 | 2 |
| Wood | 2 | 4 |

All recordings are exact 529,258-byte RIFF/WAVE float32 mono payloads at
44.1 kHz with 132,300 frames. The adapter also accepts page card recording IDs
independently of their display order, while requiring the manifest identities
to be canonical and sorted.

No redistribution license was located on the frozen reviewed surface. The
source therefore remains `NOASSERTION`, `external_research_only`; all page/audio
bytes, manifests, caches and reports remain external to the repository.

## Reproducible result

External root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-av-msf-e3-multiobject-v1/
```

Frozen identities:

| Artifact | SHA-256 |
|---|---|
| Source manifest | `91fe4dd05277fbed9592a9a614210c8d62546954ce9e62701133cbe1f5f667d5` |
| Identified-corpus manifest | `4d19050d166740a37442ee85222490acf65289bae3d602dfc4469161b16d9f41` |
| Corpus-plan report | `e082610c90dabff3c7a328df94671dca4f84f46cd629952c3e914ce600a3ea01` |
| Internet-source audit report | `203f8dce49c0bd5943c07d064651fb4fed39d7a73f9122d0eb0e2a31969d47b9` |
| Identified-corpus report | `9379faf957c7b428bb66883b694868282d0b9dbd34c35afd627f5c70d55c04f4` |

Two fresh credential-free fetches built separate approximately 11 MiB
content-addressed caches. Their internet-source reports and identified-corpus
reports are byte-identical. A third offline normalization over the first cache
also reproduces the identified report byte-for-byte.

Measured counts:

- ten validated source records, ten object groups and twenty recording groups;
- one publisher and one publisher/project/revision source group;
- six material labels;
- all entries remain in `dev`; calibration, holdout and shadow remain unopened;
- Glass contributes two object groups and four recordings;
- the frozen minimum is 16 in-domain groups, so 14 Glass groups remain missing;
- E3 contributes zero of the required 35 reject-mutation parent groups;
- force, geometry, impact/listener position, composition, support and transfer
  response remain unavailable claim axes.

## Decision and next action

The implementation package closes normalization and exact coverage measurement
for the complete public AV-MSF demo-card surface. It also falsifies the idea
that adding the remaining AV-MSF cards could by itself close PS-2: ten objects
are still one project/revision group, and only two objects carry the target
Glass label.

The next PS-2 package must discover and adapt independent published E3
publishers/projects and implement the bounded REALIMPACT E2 transfer adapter.
Only then can the corpus plan be revised with honest group counts. Until the
required claims and grouped support exist, Glass remains
`FallbackOutOfDomain`, `Pass` remains disabled and PS-3 stays blocked.
