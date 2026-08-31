# Physical Sound V13-M1c — historical census and fresh shortlist protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / ZERO_SOURCE_SIGNAL` |
| Roadmap | [V13 M1](../plans/physical-sound-synthesis-roadmap-v13.md) |
| Parent | [M1b result](physical-sound-v13-m1b-exposure-ledger-v0-result-2026-08-31.md) |
| Candidate lane | `ObjectFolder Real / canonical_impact_field` |
| Product effect | None; M2 remains blocked until this protocol passes |

## Question

Can a byte-closed census of all prior first-party physical-sound JSON prove
that at least one official ObjectFolder Real glass object has never appeared
in any prior object identity or object-path token, without opening waveform,
force, geometry or model payloads?

M1c supplies the first real `historical_union` input to the M1b builder. It
does not reconstruct unavailable exact sample counts from every retired schema.
Instead it conservatively treats every prior exact object reference as exposed
and admits a fresh candidate only by exhaustive absence.

## Frozen census boundary

The source root is the external physical-sound experiment store. Include every
regular `*.json` file recursively except paths with any component:

```text
dependencies
venv
.venv
site-packages
__pycache__
```

Also exclude top-level generated evidence directories whose names start with:

```text
physical-sound-v13-m1
```

Those directories contain only synthetic M1 records/ledgers and would make an
A/B census self-referential. Hidden historical dry runs, MLflow artifacts,
reports, manifests, models, checkpoints and failed experiments remain in the
census. No date/mtime filter is allowed.

For every included JSON, record sorted relative path, bytes, SHA-256, parsed
schema identity or explicit `NO_SCHEMA/NON_OBJECT`, and whether it contributed
an exclusion token. Duplicate JSON keys, parse failure, symlink, path escape,
file change during the scan, more than `4096` files or an artifact over the
M1b limit fail closed.

The manifest freezes included files and stable excluded tool files, including
counts/bytes and hashes of their sorted relative-path lists. Generated M1
evidence directories are excluded as whole subtrees and are intentionally not
enumerated: the frozen prefix rule is recorded, so A output cannot perturb B
diagnostics merely by existing. No other excluded file is silently omitted.

## Official candidate universe

Use the already cached official ObjectFolder Real download HTML:

```text
ps2-source-feasibility-v1/sources/objectfolder-real-download.html
bytes  = 34749
sha256 = 0111f57a336fdeb805eb03543fb10b2cb174f8d0bcb7ef5f0e30c17324e06db0
```

It must parse into exactly IDs `1..100`, each with one non-empty name and
material. This agrees with the [official current table](https://objectfolder.stanford.edu/objectfolder-real-download)
and the already frozen object `91 / Glass_Green / Glass` source manifest.

Do not use the stale cached Markdown file with hash
`2a4d4f43…58f32`: its third table column is shifted and labels object `93`,
not `91`, as `Glass_Green`. That conflict makes it inadmissible as identity
authority even though the file remains historical census evidence.

The M1c candidate universe is the official HTML rows whose material is exactly
`Glass`. M1c does not rank by sound, listen to samples or choose a winner.

## Conservative object-exposure extraction

Walk every included parsed JSON with deterministic RFC 6901 pointers. An exact
integer/string ID in `1..100` is a direct object exposure when found at:

- any object field named `object_id` or `dataset_object_id`;
- any scalar member of a field named `object_ids`;
- `id` directly below an object field named `object`.

Deduplicate to the first lexicographic pointer per `(artifact, object ID)`.
Every direct exposure is bound into the M1b catalog twice through the exact
pointed-value hash: as `project_id` and `object_id`. The normalized entry is:

```text
source_namespace = conservative-objectfolder-real-id
project_id       = <object ID>
object_id        = <object ID>
payload_kind     = metadata
role             = historical_unknown
access_kind      = derived_signal_decoded
values_decoded   = 0
protected        = true
```

`values_decoded=0` means the old exact count is unavailable, not that the old
experiment read no signal. `derived_signal_decoded + historical_unknown`
deliberately disqualifies the object. The M1c global ledger summary must never
be attached to a new `SourceQualified` record.

In parallel, scan every JSON string for exact ObjectFolder path/group tokens:

```text
audio/<ID>/
contacts/<ID>/
force/<ID>/
<ID>/audio/
<ID>/contacts/
<ID>/force/
objectfolder-real-object-<ID>_
```

Path-token hits are exclusion evidence even when no legacy schema exposes a
direct object field. They are listed with artifact and JSON Pointer. They do
not enter the M1b catalog because the pointed string is not itself an object
ID; the shortlist uses the union of direct and path-token IDs.

Any ambiguity is exclusion, never freshness. Cross-source false positives are
acceptable; false negatives are not.

## M1b catalog and outputs

All included JSON files appear in one canonical M1b catalog. Files with direct
object evidence use `parse_json=true`; the rest are hash-only. The partition
policy is `historical_union`. M1b rechecks every file from disk before writing
the ledger.

One external atomic M1c result directory contains:

```text
census.json
catalog.json
universe.json
exclusions.json
shortlist.json
ledger/catalog.json
ledger/ledger.json
ledger/summary.json
ledger/report.json
report.json
```

The shortlist contains each official glass row with:

```text
object_id
name
material
direct_exposure_count
path_token_exposure_count
decision: FreshMetadataOnly | ExcludedPriorExposure
```

Only `FreshMetadataOnly` rows appear in `fresh_candidates`. This decision
authorizes at most M2 zero-signal source/role freeze. It does not authorize
archive download beyond metadata/range preflight, waveform decode, formula
fit, validator use or runtime promotion.

## Accounting and repeat gate

The M1c report separates:

- census JSON bytes/files/scalars parsed;
- official HTML bytes parsed;
- M1b artifact JSON bytes hashed and metadata files parsed;
- build-time network, source bytes, waveform/force/signal values and protected
  signal values, all exactly zero.

Two runs against the unchanged store must make `census`, `catalog`,
`universe`, `exclusions`, `shortlist`, M1b ledger/summary and both reports
byte-identical. M1c output directories are excluded by rule, not timing.

Negative tests must reject:

- official HTML byte/hash/table drift or non-`1..100` IDs;
- stale Markdown used as universe input;
- changed JSON hash/size between census and ledger build;
- duplicate JSON key, malformed JSON, symlink or path escape;
- missing direct exposure evidence;
- one hidden exact object ID or path token incorrectly classified fresh;
- an intentionally referenced candidate remaining in the shortlist;
- nonzero network/source/signal build counter;
- output inside the repository.

## Decision and stop rule

- `M1C_HISTORICAL_CENSUS_FRESH_SHORTLIST_PASS`: A/B are byte-identical, every
  included JSON is hash-accounted, M1b accepts the historical union, at least
  one official glass row has zero direct/path-token hits and all access
  counters are zero. This opens only M2 zero-signal role freeze.
- `NO_FRESH_GLASS_CANDIDATE`: census is valid but every glass row is exposed;
  research must add a different internet source family, not relax extraction.
- Any incomplete census, unstable file, parser, lineage, accounting or repeat
  failure is `INVALID_M1C_CENSUS`; M2 remains blocked.
