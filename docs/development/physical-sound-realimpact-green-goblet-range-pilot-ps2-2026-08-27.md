# Physical sound PS-2 — REALIMPACT GreenGoblet bounded-range E2 pilot

Date: 2026-08-27
Status: `VALIDATED / SECOND_REALIMPACT_E2_OBJECT / FALLBACK_ONLY`

## Question

Can the official 2.31 GB REALIMPACT `93_GreenGoblet.zip` contribute one exact,
independently identified `E2 transfer response` without downloading the full
archive or trusting an ad-hoc extraction script?

## Implemented boundary

`physical-sound-registry realimpact-row` now has one frozen profile,
`green-goblet-row-0-v1`. It accepts an external copy of the already reviewed
REALIMPACT repository source bundle, the frozen PS-2 corpus-plan report and an
external empty output directory. It then:

1. resolves the official HTTPS origin only to public IP addresses and pins the
   selected address for every `curl` request;
2. rejects redirects, proxies, non-HTTPS protocols, non-206 responses and any
   changed ETag, Last-Modified, Content-Range or archive length;
3. verifies the 22-byte EOCD, exact 1,295-byte central directory and twelve
   frozen ZIP members;
4. retrieves and raw-deflate-decodes six small NPY arrays and the mesh;
5. retrieves a fixed 1,048,576-byte compressed prefix of the 2.31 GB transfer
   member, decodes only its NPY header and row 0, and stops;
6. checks dtype/shape, CRC32, member hashes, acquisition axes, finite samples,
   mesh bounds and the exact impact-vertex correspondence;
7. emits an external raw float32 row, normalized PCM16 audition WAV,
   acquisition metadata, provenance review, self-contained V2 inventory
   manifest and acquisition report.

The command publishes through a private staging directory only after all
network, source and decoded-content checks pass. Research recordings and
derived audio stay outside Git.

## Exact result

| Evidence | Value |
| --- | --- |
| Official archive | `https://downloads.cs.stanford.edu/viscam/RealImpact/93_GreenGoblet.zip` |
| Archive identity | `2,311,697,935` bytes; ETag `6433e478-89c9b60f`; `2023-04-10T10:27:04Z` |
| Central-directory SHA-256 | `083a0677196ee18688c42379d869137fc48e2efd756cd418bf4ef0cfa40eb3a4` |
| Total HTTP range payload | `1,612,392` bytes (`0.069749%` of the archive) |
| Mesh | `48,174` vertices; SHA-256 `96252fe0200699f06ae148d3eeeaf8eb67a1dc87dc4da94796bf375b5dd47192` |
| Transfer array | float32 little-endian, shape `3000 x 208323`, 48 kHz |
| Selected row | row `0`, mesh vertex `31676`, position `[-0.02058199, -0.0394915, 0.1566662]` m |
| Listener | angle `0`, distance offset `0`, microphone `0`, position `[0.23, -0.04345, -0.91]` m |
| Row payload | `833,292` bytes; SHA-256 `104dd97391bf6319ccbd4dfdf48569be58097bf1f90cf8cea3ae3cb2f7498ec9` |
| Duration / level | `4.3400625` s; peak `158.9156951904297`; RMS `3.5771377718919215` |
| Audition WAV SHA-256 | `4c6398adfbe248596a46bc571a9b29b7ecf5293a7543f51c852bf8adf7763581` |
| Acquisition metadata SHA-256 | `a5c129535823c1422885a4adc3f27dcab36e8f959893d13b63b2f8efbd1bb649` |
| Inventory manifest SHA-256 | `03702bc865971b727887abd7a9e7aae172f4c4f498bfbafc535f3a3b3b97ed4c` |
| Inventory report SHA-256 | `9cab3bcb504dfd161150e7eb868074ad9176a9210019b4ab75a0a49932aadc53` |

The first and repeated online acquisitions produced identical acquisition
report, metadata, manifest, raw row, audition WAV, provenance and inventory
report hashes. The impact coordinate decoded from `vertexXYZ.npy` is exactly
the coordinate of vertex `31676` in the decoded OBJ mesh.

The V2 inventory grants only these adapter-backed capabilities:

- `force_deconvolved_transfer`;
- `geometry`;
- `impact_position`;
- `listener_position`;
- `object_identity`;
- `real_recording`.

Its provisional outcome is still `FallbackOutOfDomain`. The archive does not
publish the raw force-profile bytes, a material-composition revision, repeat
recording identity or a versioned support fixture. Those four axes remain
explicitly unavailable.

## Decision

The range-import hypothesis passes. REALIMPACT now contributes two exact E2
objects, `94_GlassGoblet` and `93_GreenGoblet`, through one typed adapter. The
new command establishes a reproducible way to inspect another compatible
archive without a full multi-gigabyte download; it is not a generic ZIP URL
fetcher and cannot widen beyond its frozen profile.

E2 object count is not E3 validator coverage. GreenGoblet therefore does not
change Glass from `5/16`, open calibration/holdout/shadow or enable automatic
`Pass`. It supplies a second real transfer/geometry target for future modal and
spatial fitting while the immediate PS-2 blocker remains eleven independent E3
Glass groups plus 35 reject-parent groups.

## Next discriminator

Search published sources for stable object-level Glass identities and repeats
that can add an independent E3 project/object group. In parallel, reuse this
frozen range profile pattern only for a specifically reviewed REALIMPACT object
whose first row adds a materially distinct geometry target. Do not download
whole archives merely to enumerate them, and do not treat additional rows from
one object as independent validator groups.
