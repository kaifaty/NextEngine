# Physical Sound V15-S0a — revision-aware identity and exposure protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / METADATA_ONLY / ZERO_SIGNAL` |
| Roadmap | [V15 S0a](../plans/physical-sound-synthesis-roadmap-v15.md) |
| Parent evidence | [N1b result](physical-sound-v14-n1b-real-source-metadata-inventory-result-2026-09-01.md) |
| Claim ceiling | Revision-aware physical groups and historical exposure; no dataset role credit |
| Product effect | None; SPEC-45 remains `Proposed` and authored clips remain authoritative |

## Question

Can ObjectFolder current, ObjectFolder historical and RealImpact identities be
reduced to deterministic physical-object groups while conservatively carrying
every earlier numeric, path-token and exact RealImpact-name exposure across
revision aliases, without opening source audio, force, geometry or protected
signal?

S0a fixes one concrete false-fresh risk. The old historical census recognizes
numeric `object_id` values and ObjectFolder path tokens, but many RealImpact
experiments bind an object as `93_GreenGoblet`. The current ObjectFolder table
assigns numeric ID `93` to a different object than the publisher revision used
by RealImpact. Neither numeric ID alone nor exact name alone is a sufficient
cross-revision identity policy.

S0a does not freeze train/development/protected roles, inspect archive members,
authorize waveform decode or authorize S1. It opens only S0b metadata work.

## Frozen exact inputs

| Input | Bytes | SHA-256 |
| --- | ---: | --- |
| N1b `inventory.json` | `1,118,080` | `2861bb0e42e40fe78e67dab7ed5e0897b40c42caf6ab639235948adfa38fcfc0` |
| M1c `exclusions.json` | `730,462` | `87e71fb217a217044ed1c049ef90eb663818d84e5fdd6e66514be87c89498520` |
| Pinned historical ObjectFolder table | `8,668` | `2a4d4f43245dfdafd6f52a66d8c6f6e72822654b6e738f62899d34f1dda58f32` |
| RealImpact `object_names.txt` | `738` | `3ee26ac9b34130353dc93dc16dfa448b54b309d183f4e7b8c7d2464cb807ad5a` |
| RealImpact paper text snapshot | `106,070` | `3d5c5ea23b9c65c9af7fb7b1d2e082f7507233c06681d9274285e6c4f6b0ad14` |

The paper text must retain the publisher claims that the dataset contains 50
objects purchased from ObjectFolder, that each has a high-resolution scanned
mesh and that selected objects are rigid and single-material. The input hash
binds the exact snapshot; the claim checks prevent an unrelated same-shaped
fixture from satisfying the alias rule.

The external physical-sound experiment store is the historical evidence root.
It is intentionally not represented by one pre-existing root hash: S0a builds
the exact file manifest and repeats the build against the unchanged store.

## Physical-group construction

The identity map contains all 100 historical ObjectFolder rows, all 100 current
rows already bound by N1b and all 50 RealImpact names.

1. Current and historical rows join one physical group only when numeric ID,
   normalized publisher name and material all agree. The proven range is
   expected to remain `1…70`.
2. A changed current row gets a distinct current-revision group. Equal numeric
   ID across the known `71…100` drift never joins it to historical or
   RealImpact.
3. Each RealImpact object joins the pinned historical group identified by its
   unique numeric filename prefix. This is a conservative source inference
   supported jointly by the frozen paper origin claim and the exact publisher
   `object_names.txt`; it is never used to merge into a conflicting current
   row.
4. ObjectFolder 2.0 remains a separate synthetic namespace and is outside this
   real-object identity map.

Every group ID is SHA-256 over canonical identity evidence rather than a bare
number. Each member records source/revision, publisher object ID/name/material,
alias basis and its input evidence hashes. Duplicate members, missing rows,
unknown material, inconsistent alias targets or a source row not represented
exactly once fail closed.

RealImpact may inherit historical material for source inventory only. S0a does
not grant `evaluation_complete`, independent-object or protected-role credit;
S0c must still prove axes and role eligibility.

## Historical exposure evidence

S0a unions two independently bounded channels.

### Existing numeric/path-token union

The exact M1c `exclusions.json` supplies both `direct_exposures` and
`path_token_exposures`. Every referenced ID exposes all physical groups that
could correspond to that ID. This deliberately over-excludes current and
historical rows under ambiguity; ambiguity is never freshness.

### Exact RealImpact-name census

S0a recursively parses regular `*.json` under the external experiment store.
For every string scalar it records exact token-bounded occurrences of the 50
frozen RealImpact names, including names embedded in URLs, archive members or
role/group strings. Evidence binds relative path, file bytes/hash, RFC 6901
pointer, pointed-value hash and matched publisher name. It never records the
full pointed value.

Exclude paths containing dependency/tool components:

```text
dependencies
venv
.venv
site-packages
__pycache__
```

Also exclude these top-level source-listing or generated-evidence prefixes:

```text
physical-sound-v13-m1
physical-sound-v14-n1b
physical-sound-v14-n1c-source-research
physical-sound-v15-s0a
ps2-source-feasibility-v1
```

N1b and the source-research directories enumerate publisher objects but do not
represent generator/protected selection. Their exact identities are already
bound as explicit S0a inputs. S0a output directories are excluded by rule so
run A cannot perturb run B. Hidden dry runs, failed experiments, manifests,
reports and MLflow records remain included.

The scan allows at most `4096` JSON files and `16 MiB` per file. Duplicate JSON
keys, non-finite values, parse failure, symlink component, file change during
read or unaccounted regular JSON fail closed. No mtime/date cutoff is allowed.

## Outputs

One fresh external directory contains canonical JSON:

```text
identity-map.json
exposure-census.json
report.json
```

`identity-map.json` contains sorted physical groups and source members.
`exposure-census.json` contains the included/excluded file manifest, numeric
and exact-name evidence, per-group exposure state and per-material candidate
counts. `report.json` binds both hashes, exact inputs, access counters and the
decision.

Candidate counts deduplicate aliases by physical-group ID. A candidate group
needs at least one N1b T2/T3 route whose archive identity is available and
whose post-alias material is in Glass/Wood/Metal. `unexposed` means no evidence
from either channel. It remains a metadata candidate, not role eligibility.

## Access accounting

The report publishes:

```text
json_files_parsed
json_bytes_read
json_scalar_values_parsed
numeric_exposure_records_read
realimpact_name_evidence_records
network_requests = 0
network_archive_body_bytes = 0
source_payload_bytes_read = 0
source_payload_members_extracted = 0
wav_headers_parsed = 0
npy_headers_parsed = 0
pcm_sample_values_decoded = 0
force_sample_values_decoded = 0
protected_signal_values_decoded = 0
```

The tool writes atomically to a fresh directory outside the repository. Two
executions against the unchanged store must be byte-identical.

## Focused guards

Tests must prove:

- repeat-exact canonical output and all source/signal counters zero;
- current/historical revision drift never aliases by number;
- RealImpact maps only to its historical group under the frozen paper-origin
  and filename-prefix evidence;
- an exact `93_GreenGoblet` string exposes that group even when numeric ID 93
  is absent from M1c evidence;
- names appearing only in an excluded N1b/source-listing subtree do not expose;
- direct and path-token M1c evidence both propagate across possible aliases;
- unknown schema/key, duplicate JSON, mutated input hash/claim/name roster,
  symlink, oversized store, unsafe output or repeat replacement fails closed.

## Decision

- `S0A_IDENTITY_EXPOSURE_PASS_METAL_SCOPE_REMAINS_POTENTIAL`: all guards pass,
  at least eight unexposed Metal metadata candidate groups remain and A/B are
  byte-identical. This opens only S0b.
- `S0A_IDENTITY_EXPOSURE_PASS_METAL_SOURCE_INSUFFICIENT`: identity/exposure is
  valid but fewer than eight Metal groups remain. V15 must close or find a new
  source before role freeze; it may not weaken the role shape.
- Any incomplete identity map, scan, accounting or repeat is
  `INVALID_S0A_IDENTITY_EXPOSURE`; no downstream package opens.

Glass and Wood counts are report facts only. They cannot change their V15
pending/fallback states in S0a.
