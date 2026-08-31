# Physical Sound V15-S0b — YCB capability and cost protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `FROZEN_BEFORE_BUILD / METADATA_ONLY / ZERO_SIGNAL` |
| Roadmap | [V15 S0b](../plans/physical-sound-synthesis-roadmap-v15.md) |
| Predecessor | [V15 S0a result](physical-sound-v15-s0a-revision-aware-identity-exposure-result-2026-09-01.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Product effect | None; dataset roles remain unfrozen and authored clips remain authoritative |

## Question and bounded claim

S0b asks one source-blind question:

> Which YCB-impact object groups have stable publisher metadata that binds an
> object, material label, recording parent and available geometry/acquisition
> axes, and what would it cost to open their still-opaque payloads?

The result is a capability inventory, not a dataset admission. A folder name,
material row, paper statement or downloadable mesh route cannot by itself make
an object `training_usable` or `evaluation_complete`. S0b does not assign any
V15 role and does not authorize S1.

## Frozen sources

Only publisher-controlled HTTPS metadata is in scope:

| Source | Frozen role | Required identity |
| --- | --- | --- |
| YCB-impact object/material workbook | Exact object and primary/secondary material rows | `7,994` bytes, SHA-256 `27672ecfdfaf2a1ecfc8926127ab9ab962e9cb59adb593b141caadc0459ae0fd` |
| YCB model index | Publisher mesh routes and route-quality footnotes | `105,935` bytes, SHA-256 `d90b97268e9fd4eaf501efc22d17e27d419e9d94f498c6f6b6f63defd2a49891` |
| YCB-impact paper | Dataset-wide acquisition semantics | `7,130,560` bytes, SHA-256 `13fcf1fe0adce22a9e108c95baaeaf0c0f9a06e2fbd04cb60ee810d10254554b` |
| OSF components `bj5w8` and `hjdby` | Recording/archive parentage, file IDs, hashes and declared byte counts | Deterministic normalized snapshot produced by the acquisition stage |

The paper is the open-access copy published by UPC for DOI
[`10.1109/IROS47612.2022.9981578`](https://doi.org/10.1109/IROS47612.2022.9981578).
The model routes come from the official
[YCB model index](https://ycb-benchmarks.s3.amazonaws.com/index.html), and the
recording tree comes from the public
[YCB-impact OSF project](https://osf.io/4tcp6/).

No search result, mirror, filename guess or previously downloaded audio may
substitute for one of these identities.

## Two-stage boundary

### `acquire`

`acquire` may perform bounded GET requests only to JSON metadata collections
under `https://api.osf.io/v2/`. It recursively enumerates the `osfstorage`
trees for components `bj5w8` and `hjdby` and writes one canonical snapshot.

The acquisition stage:

- follows only OSF JSON pagination and folder-child relationships;
- permits at most `1,024` requests and `64 MiB` of metadata response bodies;
- records component, entry ID, kind, normalized materialized path, parent ID,
  byte size, publisher hashes, version and modification time;
- rejects duplicate IDs, duplicate paths, path traversal, unknown hosts,
  redirects outside `api.osf.io`, non-JSON bodies and schema drift;
- never follows `download`, `render`, storage-provider or upload links;
- omits wall-clock acquisition time so an unchanged remote tree produces
  byte-identical output.

The canonical snapshot includes exact access counters. Audio, video, force,
mesh and archive body counters must all remain zero.

### `build`

`build` is offline. It verifies the three frozen files and a canonical OSF
snapshot, then writes a fresh external output directory containing:

- `capability-inventory.json` — normalized object and recording-parent facts;
- `cost-census.json` — declared opaque payload counts/bytes and unresolved
  costs by material and acquisition condition;
- `report.json` — hashes, counts, access ledger, blockers and the S0b decision.

The same inputs must produce byte-identical output directories. Repository
paths, symlinks, replacement of an existing output and input mutation during
hashing fail closed.

## Object and recording-parent binding

Workbook IDs are canonical YCB object identities. A robot recording parent may
bind to an object only when its normalized folder name starts with the exact
three-digit YCB ID and the remaining slug normalizes to the workbook object
name. Known aliases are frozen narrowly (`tuna`/publisher `tune`, plural
collection labels and punctuation); numeric equality alone is insufficient.

Robot modes remain separate acquisition conditions inside each bound parent:

- `vertical-known` and `vertical-unknown`: top poke, fixed object/microphone
  relation, object motion restricted against the poke direction;
- `horizontal-0.14` and `horizontal-0.25`: side poke at the paper-declared
  `14 mm/s` or `25 mm/s`, where the object may slide;
- a horizontal material folder without an object child is
  `material_aggregate` and cannot grant object identity.

Manual `hit`, `scratch` and `drop` data is inventoried only at the parent level
unless the OSF tree itself binds a recording archive to one exact workbook
object. An aggregate archive never receives object-level credit.

One physical group is `(publisher, project, component revision, YCB object,
recording parent)`. Different clips under that parent are repeated
observations, not new objects. Known/unknown publisher splits are descriptive;
they are not V15 holdout roles.

## Geometry binding

The model index may prove only these states:

- `route_available` for a processed, `16k`, `64k` or `512k` archive link;
- `route_distorted` where the publisher marks the generated model as largely
  distorted;
- `route_unavailable_transparency_or_size` where the publisher withholds a
  model for that reason;
- `variant_ambiguous` where one workbook object maps to multiple model variants
  without a recording-level variant identity;
- `route_absent` otherwise.

S0b does not download a model archive. Therefore it cannot grant exact mesh
revision, archive hash, member manifest, metric scale, contact-to-surface
binding or watertightness. A stable URL is a route, not geometry evidence.

## Capability axes

Every object/parent row reports each axis as `known`, `partial`, `unknown` or
`not_applicable`, plus evidence references:

| Axis | Maximum S0b credit |
| --- | --- |
| Object identity | `known` only for exact workbook-to-parent binding |
| Material identity | `known` for exact workbook row; secondary material remains explicit |
| Recorded response | `partial`: opaque publisher files exist; signal is unopened |
| Canonical excitation | `partial` for direction and horizontal speed; force/energy remains unknown |
| Contact geometry binding | `unknown`; no contact point or struck mesh region is published |
| Geometry | `partial` when a non-ambiguous official route exists |
| Geometry scale | `unknown` until a pinned archive/member proves units and scale |
| Listener condition | `partial`: microphone model/rate and fixed vertical relation are known, coordinates/transfer are not |
| Support condition | `partial`: table and motion restriction class are known, exact support material/fixture is not |

`training_usable` requires all N1a training axes to be `known` after later
payload preflight. `evaluation_complete` additionally requires support to be
`known`. Consequently S0b itself must emit both flags as `false` and list the
missing axes rather than inferring them.

## Cost census

Cost is reported without opening payloads:

- OSF files contribute publisher-declared file count and byte size;
- files without a declared size contribute an explicit unknown count;
- model routes contribute route counts, but byte cost remains unknown until a
  later metadata-only HEAD protocol is separately frozen;
- aggregate archives are never divided into estimated per-object bytes;
- metadata request and response bytes are reported separately from projected
  payload cost.

No missing number is replaced by an estimate. This makes the S0c choice about
the smallest admissible source opening reviewable rather than optimistic.

## Exposure and role boundary

S0a remains the authority for existing ObjectFolder/RealImpact exposure. S0b
reports YCB identities already named by repository adapters as
`repository_adapter_referenced`; every other YCB identity is
`not_in_adapter_freshness_unassessed`, never fresh. S0c must merge the exact YCB
parent IDs with historical experiment evidence in a new immutable role
descriptor and rerun the exposure guard before any payload is opened.

The following do not confer a role:

- a publisher train/test or known/unknown label;
- absence from an earlier hand-picked adapter roster;
- a mesh route;
- a material total large enough to fill `4/1/1/1/1`;
- a zero-signal S0b pass.

## Mutations and focused verification

Tests must reject at least:

1. changed input bytes/hash, duplicate JSON keys and non-canonical snapshots;
2. OSF host/relationship escape, duplicate file IDs/paths and traversal;
3. workbook duplicate/missing IDs and material/name drift;
4. numeric-only or ambiguous folder aliases;
5. model-index route/footnote contradictions and variant ambiguity;
6. signal/body counters above zero;
7. repository, symlink, overwrite and repeat-exact violations.

The focused test also proves positive exact binding for Steel, Wood and Glass,
preserves vertical/horizontal parent separation, and demonstrates that every
S0b row remains ineligible for training and protected evaluation.

## Exit decision

S0b passes only if real frozen metadata builds twice byte-identically, access
counters prove zero signal/body access, and the report binds object, material,
recording parent, available mesh route and acquisition axes without upgrading
unknown facts.

The only passing decision is:

```text
S0B_YCB_CAPABILITY_PASS_S0C_ROLE_FREEZE_NEXT
```

If identity or axes cannot be bound, the adapter still publishes a deterministic
blocked inventory and closes as `SOURCE_METADATA_INSUFFICIENT`; gates are not
reduced.
