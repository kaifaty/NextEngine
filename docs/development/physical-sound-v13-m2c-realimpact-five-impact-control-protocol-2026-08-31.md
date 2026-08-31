# Physical Sound V13-M2c — RealImpact five-impact control freeze protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / METADATA_ONLY_NETWORK / AUDIO_PAYLOAD_CLOSED` |
| Parent roadmap | [V13 M2](../plans/physical-sound-synthesis-roadmap-v13.md) |
| Control object | RealImpact `93_GreenGoblet` |
| Claim | `publisher_derived_response_control`, relative only |
| Product effect | None; authored clips remain authoritative |

## Authorized question

Can the published RealImpact object geometry, five impact parents and complete
listener metadata be hash-closed as a leave-one-impact-out control before any
new `deconvolved_0db.npy` byte is fetched or decoded?

M2c is not a fresh quality or admission source. Green Goblet has substantial
prior exposure in the historical ledger and is selected precisely as a
control-only object. It cannot train or validate the independent validator,
become admission shadow, select a material-family claim or repair ObjectFolder
object 92.

## Claim boundary

RealImpact publishes `deconvolved_0db.npy`, geometry and aligned impact/
listener arrays but no raw paired force provenance for this derived response.
M2c therefore allows only:

`publisher-derived relative response + impact parent + listener identity`

It prohibits `MeasuredTransferField`, arbitrary-force convolution, absolute
amplitude, exact support, exact glass composition, independent validation,
corpus admission and runtime use. The array is a future classical/control
input only; it is not canonical truth for ObjectFolder.

## Frozen lineage

The prior exact metadata/control lineage remains external:

| Artifact | SHA-256 |
| --- | --- |
| Existing frozen profile | `c2301789813b836b84ffb9a41380af02421490073d6585c488e276f1a5677e28` |
| Existing listener-block manifest | `fcf44d41bdd54ad3bc9df27b6c8ccc4d5ccd470a1f786641e64461e794c850de` |
| Existing acquisition report | `cef5d381f5a6d9a6a1666503a8ed7e44c70758cd4e6f9e636f8001a111d57680` |
| Existing corpus-plan report | `e082610c90dabff3c7a328df94671dca4f84f46cd629952c3e914ce600a3ea01` |

M2c reads and hashes only the frozen-profile JSON. It does not read the prior
12.5-MiB listener block, its waveform metrics or any other generated signal
artifact.

Official source identity:

| Field | Value |
| --- | --- |
| Dataset object | `93_GreenGoblet` |
| Archive URL | `https://downloads.cs.stanford.edu/viscam/RealImpact/93_GreenGoblet.zip` |
| Archive bytes | `2,311,697,935` |
| ETag | `6433e478-89c9b60f` |
| Last-Modified | `Mon, 10 Apr 2023 10:27:04 GMT` |
| Central directory | offset `2,311,696,618`, bytes `1,295`, SHA-256 `083a0677…3a4` |
| Source repository revision | `fca2bd6cbb7e9f96ac61328d2a0d51594bf01987` |

## Frozen metadata ranges

Only the ZIP central directory and these seven compressed metadata/geometry
ranges may be requested. Every response must be exact `206`, identity-encoded,
same HTTPS host, ETag, Last-Modified, full length and Content-Range.

| Entry | Data offset | Compressed | Raw bytes | CRC32 | Raw SHA-256 |
| --- | ---: | ---: | ---: | --- | --- |
| `vertexXYZ.npy` | `258` | `441` | `72,128` | `695feafe` | `cbc94c54…151e` |
| `micID.npy` | `794` | `225` | `24,128` | `082ec0c6` | `d603b615…4d6b` |
| `transformed.obj` | `2,783,916` | `557,368` | `3,528,966` | `101b1df8` | `96252fe0…7192` |
| `vertexID.npy` | `3,341,382` | `164` | `24,128` | `4f58b204` | `ad1a3143…282d` |
| `listenerXYZ.npy` | `3,341,647` | `2,923` | `72,128` | `ee426e91` | `83fa3f27…4dd4` |
| `distance.npy` | `3,344,863` | `294` | `24,128` | `570b48dd` | `95dbc48e…188a` |
| `angle.npy` | `2,311,696,326` | `292` | `24,128` | `fabb9a2e` | `ed65ac28…74a3` |

All entries use raw ZIP deflate. The runner must verify compressed byte count,
decompressed byte count, CRC32 and full raw SHA-256 before interpreting
metadata.

The forbidden audio entry is
`93_GreenGoblet/preprocessed/deconvolved_0db.npy`, data offset `3,345,262`,
compressed bytes `2,308,350,969`, raw bytes `2,499,876,128`, CRC32
`d41ca14a`, frozen logical shape `(3000, 208323)` and dtype `<f4`. No HTTP range
may overlap its compressed interval.

## Metadata invariants

The six NPY arrays must be canonical little-endian current NumPy containers:

- `vertexXYZ` and `listenerXYZ`: finite float64 shape `(3000,3)`;
- `vertexID`, `micID`, `distance` and `angle`: int64 shape `(3000,)`;
- rows form exactly five contiguous impact blocks of `600`;
- within each block, `vertexID` and `vertexXYZ` are constant;
- the five impact-parent `(vertexID, XYZ)` pairs are distinct;
- listener `(angle,distance,micID,XYZ)` tuples are identical across all five
  blocks and unique inside one block;
- each block contains exactly one canonical listener with
  `angle=0`, `distance=0`, `micID=7`;
- the canonical rows must be global indexes `7,607,1207,1807,2407`;
- the OBJ is nonempty, finite text, has at least one vertex and face, and its
  raw hash is the geometry revision `mesh-96252fe02006-v1`.

No impact or listener value may choose a model capacity or threshold. The
runner records the five discovered impact IDs/coordinates only after the above
structure passes.

## Frozen control roles

Impact parents are indexed by source block `0…4`. M2c defines five symmetric
leave-one-impact-out folds:

| Fold | Fit impact parents | Held control parent |
| --- | --- | --- |
| `loio-0` | `1,2,3,4` | `0` |
| `loio-1` | `0,2,3,4` | `1` |
| `loio-2` | `0,1,3,4` | `2` |
| `loio-3` | `0,1,2,4` | `3` |
| `loio-4` | `0,1,2,3` | `4` |

Only the canonical listener row belongs to the M3–M5 contact-control lane.
The other `2,995` listener rows are radiation metadata and remain signal-
closed until a separately preregistered listener-field revision. Because each
parent is held exactly once, fold aggregation cannot select a favorable impact.

## Read budgets

Per independent M2c run:

| Counter | Maximum |
| --- | ---: |
| Network range responses | `8` |
| Compressed source bytes | `563,002` |
| Raw metadata/geometry bytes decoded | `3,769,734` |
| Metadata scalar values decoded | `30,000` |
| Mesh vertices/faces parsed | structure only |
| Audio compressed bytes requested/read | `0` |
| Audio sample values decoded | `0` |
| Prior listener-block payload bytes read | `0` |
| Source payloads emitted | `0` |

The `30,000` scalars are `9,000 + 9,000` float coordinates and four `3,000`
int arrays. Output contains only bounded descriptors, hashes, five impact
parents, the canonical listener descriptor, folds and counters. It stays in a
fresh external directory.

## Decision

Two independent executions must produce byte-identical manifest/report JSON.
Success is `READY_FOR_M2D_CANONICAL_SOURCE_RECOVERY`; any source, range,
container, geometry, partition, canonical-row or zero-audio mismatch fails
closed.

A pass proves only that one already-exposed derived-response control is exact
and leakage-bounded. It gives no real formula, glass quality, validator, atlas,
runtime or ProductCheck credit and does not unblock M3 without a viable fresh
canonical source decision.
