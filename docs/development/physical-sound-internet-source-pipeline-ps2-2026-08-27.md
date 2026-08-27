# Physical sound PS-2 — internet source registry and cache pilot

Date: 2026-08-27
Status: `SOURCE_REGISTRY_IMPLEMENTED / AV_MSF_E3_CATALOG_MEASURED / E4_READY / PASS_DISABLED`

## Question

Can PS-2 discover and retrieve published internet evidence without local
capture, unbounded downloads, repository-local datasets, inferred capabilities
or a forged acoustic evidence tier?

## Implemented boundary

`physical-sound-registry internet-sources` now consumes a current-only external
manifest and emits an external report:

```text
cargo run -p xtask -- physical-sound-registry internet-sources \
  --manifest /external/internet-sources/manifest.json \
  --cache /external/internet-cache \
  --output /external/internet-source-report \
  --fetch-missing
```

Each source binds publisher/project/revision, review date, canonical landing and
terms URLs, adapter identity, license/redistribution policy, a hash-closed
provenance review, remote artifacts and artifact-backed capability claims.
Artifacts may declare exact SHA-256 and byte count or remain discovery-only.

The fetch path is deliberately narrow:

- only credential-free canonical HTTPS URLs are accepted;
- redirects, query strings, URL userinfo, proxies and non-HTTPS protocols are
  disabled;
- the hostname is resolved before transfer, every returned address must be
  public, and curl is pinned to one deterministic public address while retaining
  TLS hostname verification;
- connect/transfer timeouts are 30/120 seconds;
- the default per-run ceiling is 512 MiB and the hard explicit ceiling is 64
  GiB; every artifact has its own smaller bound;
- bytes stream through a create-new external staging file, are counted and
  SHA-256 hashed, then are hard-linked atomically into
  `objects/<hash-prefix>/<sha256>` only after exact verification;
- repository-local cache/output, links, cache corruption, size/hash mismatch and
  overlapping output/cache fail before report publication.

Missing cryptographic integrity, an intentionally disabled fetch, a declared
download above the active limit, network failure or absent curl produce a stable
incomplete source status. They do not weaken an artifact bound or publish local
paths. A corrupt existing cache entry is a typed failure, not a refetch or
warning.

Capabilities map to exact artifact IDs and separately report byte availability
and adapter validation. `hash-closed-synthetic-v1` may validate only
`synthetic_lineage`. The typed `av-msf-identified-recording-v1` profile checks
an exact immutable project page, its object/material/recording metadata and
bounded finite float32 WAV payloads before validating only material/object/real
recording/repeat identity. Arbitrary or future adapter IDs grant no acoustic
capability. Opaque bytes therefore cannot forge `E1`–`E3`; the report has
`NO_CORPUS_ADMISSION_AUTHORITY` regardless of tier.

## Official-source pilot

The pilot uses two related official Stanford sources.

[ObjectFolder 2.0](https://github.com/rhgao/ObjectFolder) was frozen at commit
`3c6cd8930b2dcbadb6d94dadf2745c956bdcd236`. Three small official files were
downloaded twice through fresh caches and verified:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `objects.csv` | 67,378 | `5565b7e8f739194616d5f276e49b6e3bbf34f447f2a07b79ad02b1c23d8b1557` |
| `README.md` | 9,608 | `bf37922f51280a981a6bd2a27cba3e8a9698f05d5d52be8d09c8a925eed8b1c0` |
| `LICENSE` | 18,104 | `fd457083300523c6d8a8910c7a1e07eb86424e5d614a21950bd0d4941a0336c1` |

Only `E4SyntheticGenerated` is adapter-validated. The available object/material
catalog bytes are reported, but do not become real identity evidence.

The official [ObjectFolder-Real download page](https://objectfolder.stanford.edu/objectfolder-real-download)
describes 100 real household objects, 30–50 six-second impact recordings per
object, strike coordinates on the mesh and ground-truth contact-force profiles.
Its table includes glass, steel and wood objects. The first acoustic archive's
official HTTPS response reports 36,367,088,523 bytes and byte-range support, but
the page publishes no SHA-256. The pipeline therefore assigns
`MissingIntegrityMetadata`, performs no 36.37 GB download and grants no
`E1`–`E3` capability. This is the intended fail-closed result, not a dataset
quality rejection.

The second official-source pilot freezes the
[AV-MSF](https://zisenshao.github.io/AV-MSF/) page branch at Git commit
`723df64a94480fc8f8e592c66c0d916e8b0054d1`. Object 95 is explicitly labelled
`Glass` with contact recordings `012` and `036`. The adapter validates the
27,239-byte page and two 529,258-byte, mono 44.1 kHz float32 WAVs by exact
SHA-256, page card and audio structure. It grants exactly
`E3IdentifiedRecording`; missing force, geometry, positions, support and
calibration receive no credit. See the
[AV-MSF pilot](physical-sound-av-msf-e3-pilot-ps2-2026-08-27.md).

The follow-up
[multi-object pilot](physical-sound-av-msf-e3-multiobject-pilot-ps2-2026-08-27.md)
freezes the complete ten-card public demo surface at the same revision. Its
source audit validates ten objects and twenty recordings. The separate
`identified-corpus` normalizer derives leakage-safe source/object/recording
groups and measures Glass coverage without inventing E1/E2 axes.

## Frozen result

External evidence lives under:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-internet-sources-v1/`

- manifest SHA-256:
  `88d54a0311c34b002a2402f1d54da7fd359126f371e236961e552fe89c831655`;
- report SHA-256:
  `7e7cc242ab4970d0ed89add667a530c01d0772568d3e4e15018adbfb4075be82`;
- decision: `SourceSetIncomplete`;
- sources: two total, one evidence-ready;
- supported tiers: ObjectFolder implicit `E4SyntheticGenerated` only;
- ObjectFolder-Real: `DiscoveryOnly`.

Online reports from two fresh content-addressed caches and a later offline report
are byte-identical. External source bytes, cache objects and reports remain out
of the repository.

The separate AV-MSF E3 pilot lives under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-av-msf-e3-glass-v1/`.
Its manifest/report hashes are
`a1de51fdd8f3bdcca0e7a11551b45ea91a6bf43520439da2708729351f667480` and
`d762c92e8bd3210944ee7802fa7fef71e10776a62b4da132eb92cb0d33c56231`.
Two fresh online caches and one offline audit again produced byte-identical
reports.

The complete-card pilot lives under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-av-msf-e3-multiobject-v1/`.
Its source-manifest, source-report and identified-report SHA-256 values are
`91fe4dd05277fbed9592a9a614210c8d62546954ce9e62701133cbe1f5f667d5`,
`203f8dce49c0bd5943c07d064651fb4fed39d7a73f9122d0eb0e2a31969d47b9`
and `9379faf957c7b428bb66883b694868282d0b9dbd34c35afd627f5c70d55c04f4`.
Two fresh caches and an offline normalization are byte-identical.

## Decision and next action

The generic source registry, bounded real-data adapter and multi-object E3
normalizer are complete. The full AV-MSF public card surface contributes ten
objects and twenty recordings but only one publisher/project/revision group;
Glass contributes two object groups against the frozen minimum of sixteen.
The blocker is now independent-publisher grouped coverage plus complementary
E2 spatial/transfer evidence, not networking, adapter existence, AV-MSF card
enumeration or local hardware.

Discover and adapt independent published E3 publishers/projects, implement the
bounded REALIMPACT E2 adapter, then re-plan available groups before opening
calibration, holdout or shadow. ObjectFolder-Real may be reconsidered if the
publisher supplies cryptographic checksums or a bounded object-level download;
do not download the 36.37 GB unhashed archive merely to advance the roadmap.
