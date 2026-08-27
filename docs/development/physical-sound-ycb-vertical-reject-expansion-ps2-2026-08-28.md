# PS-2 YCB vertical reject-parent expansion — 2026-08-28

| Field | Result |
| --- | --- |
| Status | `27_E3_OBJECTS_READY / REJECT_PARENTS_38_OF_35 / GLASS_9_OF_16 / SPLITS_UNOPENED / PASS_DISABLED` |
| Scope | Published current-only `E3IdentifiedRecording`; no per-impact segmentation, corpus admission, quality acceptance, runtime or content promotion |
| Adapter | Additive `ycb-impact-identified-recording-v1` vertical profile plus bounded Ogg/Vorbis validator |
| Source manifest/report | `ce1fa11f3bc007e13c0299cec2f7f4866158cf87862e45cb0dfc064f1b74c1b9` / `2dd991ee5dbc8ee823d2e03cf2df30ef7cb8fb53ada2099029f53ab9ff8e3619` |
| Combined source manifest/report | `b77cdba8e1596827827010c97e4367b49d3f5d52f0602b6632b36020a6d88d4b` / `53fb33747685576ec261cd3011e0acc8fd2a75b1aa731fa9d32497820510f2e3` |
| Combined identified manifest/report | `1b64719bccdeb78ff4b0f5e458fd97289b5aa96d00aa8f4dc3487c7dba4081b7` / `1e4b5e790b230ae5c9ab5204cdbda2e5d4b427982f2cc87b3c8cfa1bf316f107` |
| External root | `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-ycb-vertical-reject-e3-v1/` |

## Question and source discriminator

Can the official YCB Impact robot component close the remaining non-Glass
reject-parent count without local capture or inferred object labels?

The horizontal-poke tree cannot do this safely: outside Glass it aggregates
multiple objects under material/split/condition folders and names only
`Clip_N.ogg`. The workbook proves object materials but does not bind those
horizontal clips to individual objects. No non-Glass horizontal object credit
is claimed.

The official [robot-impact component](https://osf.io/bj5w8/) exposes a distinct
vertical tree. Its current OSF structure has 30 `Known_Objects` and 15
`Unknown_Objects` child folders, each named with a YCB ID and object label. The
official `ycb_audio.xlsx` workbook then binds that ID to the object name and
primary/secondary material. This closes recording-to-object-to-material
identity without inventing a mapping. The component and its collection method
are described by the official [project](https://osf.io/4tcp6/) and
[publication](https://www.iri.upc.edu/publications/show/2619).

## Frozen selection

The package intentionally selects the smallest balanced increment that exceeds
the missing count: three objects from each of nine non-Glass primary material
classes, with two distinct publisher files per object.

| Primary material | Selected objects |
| --- | --- |
| Aluminium | Master chef can, Tomato soup can, Tune fish can |
| Paper | Cracker box, Pudding box, Sugar box |
| Ceramic | Bowl, Mug, Plate |
| Foam | Foam brick, Sponge, Washer sponge |
| Steel | Hammer, Padlock, Skillet |
| Hard Plastic | Clear box, Pitcher base, Pitcher lid |
| Soft Plastic | Apple, Banana, Mustard bottle |
| Other Plastic | Cups, Large clamp, Small marker |
| Wood | Colored wood block, Nine-hole peg test, Wood block |

All 54 selected audio SHA-256 identities are unique. Known/unknown labels are
preserved as source metadata but do not open a NextEngine partition. Every
object remains in `dev`, and every object shares the already existing
IRI/CSIC-UPC/CTU YCB project/revision group.

The stored `.ogx` files are Ogg/Vorbis stereo at 44.1 kHz rather than the RIFF
payload used by the earlier horizontal subset. The new in-process validator
checks one logical stream, Ogg capture/version/flags, serial and page sequence,
page CRCs, packet continuation, Vorbis identification/comment/setup headers,
channel/rate identity, terminal EOS and a 60-second decoded-frame ceiling. The
selected files range from 1,055,754 to 2,370,816 sample frames (23.94–53.76
seconds). Each file contains an unsegmented sequence of robot pokes; the
pipeline counts publisher clip files as repeats and makes no single-impact
count or isolated-transient claim.

The adapter grants only material identity, object identity, real-recording
identity and repeat-file identity. It still grants no absolute force, tip or
object geometry, impact/listener position, support condition, material
composition or transfer response.

## Reproducibility and measured coverage

Two independent credential-free fetches produced separate 55-object caches
(54 audio payloads plus one deduplicated workbook) and byte-identical reports.
Two complete offline cache roots then reproduced the combined source and
identified-corpus reports byte-for-byte.

| Measurement | Before | After |
| --- | ---: | ---: |
| E3 objects | 20 | 47 |
| E3 recordings | 59 | 113 |
| Publisher/project/revision groups | 6 | 6 |
| Material labels | 8 | 13 |
| Glass target groups/recordings | 9 / 34 | 9 / 34 |
| Reject-parent groups/recordings | 11 / 25 | 38 / 79 |
| Missing reject-parent groups | 24 | 0 |

The report now marks the development-only reject-parent count sufficient
against the frozen minimum of 35. This is only a coverage measurement. It does
not create generated negative controls, a partitioned false-pass estimate or
admission authority. Glass remains seven object groups short, and all
calibration/holdout/shadow partitions remain empty.

The external provenance review is SHA-256
`a58adc05ffee03ab39f81129faaa14b85c9b1b900dc39f4c2e81da140129ccc2`.
No explicit redistribution grant was found on the reviewed official surfaces,
so the files remain `NOASSERTION`, `external_research_only` and outside the
repository/distribution.

## Failure controls and consequence

The first two online audits both rejected the same Plate clip when a provisional
50-second ceiling was used; its exact terminal granule is 2,370,816 frames
(53.76 seconds). The final frozen ceiling is 60 seconds, and both complete
online and offline audits pass. Unit controls reject mutated Ogg checksums and
Vorbis channel metadata. Adapter declaration controls reject a vertical object
whose official material is changed to Glass.

PS-2 remains open. The next smallest action is no longer generic non-Glass
count expansion: acquire seven independently identified Glass object groups
from stable exact-download sources while expanding complementary E2/E1
geometry, excitation, support and transfer evidence. Only after complete
target evidence exists may the project freeze project-disjoint calibration,
holdout and shadow splits and attempt PS-3. Authored clips remain the mandatory
production fallback.
