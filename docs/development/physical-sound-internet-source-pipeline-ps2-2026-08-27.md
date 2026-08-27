# Physical sound PS-2 — internet source registry and cache pilot

Date: 2026-08-27
Status: `SOURCE_REGISTRY_IMPLEMENTED / E4_READY / OBJECTFOLDER_REAL_DISCOVERY_ONLY / PASS_DISABLED`

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
and adapter validation. The first implemented adapter,
`hash-closed-synthetic-v1`, may validate only `synthetic_lineage`; arbitrary or
future adapter IDs grant no acoustic capability. `E1`–`E3` therefore cannot be
forged by labelling opaque bytes. The report has
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

## Decision and next action

The generic source registry, bounded fetch/cache and claim matrix are complete.
The current blocker is now a source-specific real-data adapter plus a bounded
hash-closed published real payload, not generic networking or local hardware.

Search official dataset releases for an object-level or otherwise bounded
`E2`/`E3` artifact with published SHA-256. Implement exactly one format adapter,
then import one glass/steel/wood real source through it. ObjectFolder-Real may be
reconsidered if the publisher supplies cryptographic checksums or a bounded
object-level download; do not download the 36.37 GB unhashed archive merely to
advance the roadmap.
