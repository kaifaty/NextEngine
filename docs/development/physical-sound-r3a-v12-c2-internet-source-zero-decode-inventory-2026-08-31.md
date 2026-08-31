# Physical Sound R3A V12-C2 — internet-source zero-decode inventory

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `SOURCE_AVAILABLE_WITH_NARROWED_CLAIM` |
| Accepted source family | ObjectFolder-Real raw acoustic publication |
| Structural witness | Object `51 / Fruit_Bowl / Glass`, six complete raw trials |
| Decode in this inventory | `0` microphone samples, `0` force samples |
| Next authorized step | `C3_SOURCE_ROLE_FREEZE` |
| Product authority | None; external research only, authored clips remain authoritative |

## Decision

ObjectFolder-Real satisfies the V12-C2 source-availability criterion. The
official publication and two prior byte-identical zero-decode inventories prove
that at least one stable raw trial namespace contains synchronized microphone
and force channels, object/contact identity and metadata, and that the same
microphone bytes can be joined to a published contact coordinate and object
geometry revision.

The accepted claim is deliberately narrow:

```text
exact ObjectFolder object + contact
  -> synchronized microphone counts + force-transducer counts
  -> published strike coordinate + object geometry revision
  -> one canonical, non-numerically-specified recording/listener setup
```

It is not an absolute `newtons -> pascals` transfer claim. The selected source
does not publish the required channel sensitivity/calibration, numeric
microphone pose or exact per-object support fixture. Those axes remain absent
and cannot be inferred from labels or equipment models.

## Primary-source chain

1. The pinned official [ObjectFolder-Real download source](https://github.com/objectfolder/objectfolder.github.io/blob/d058ba09e7e7a1a8358d64d7b48e5f588b377eb8/source/_pages/objectfolder-real-download.md)
   states that each impact sound has a mesh strike coordinate and ground-truth
   contact-force profile. Exact commit:
   `d058ba09e7e7a1a8358d64d7b48e5f588b377eb8`; raw Markdown is `8,668` bytes,
   SHA-256 `2a4d4f43245dfdafd6f52a66d8c6f6e72822654b6e738f62899d34f1dda58f32`.
2. The official [CVPR 2023 paper](https://ai.stanford.edu/~rhgao/publications/ObjectFolder_CVPR2023.pdf)
   specifies an impact hammer with force transducer and a free-field microphone,
   synchronized acquisition, normal-direction strikes and the support families
   used for light, ordinary and heavy objects.
3. The pinned official [contact-localization benchmark](https://github.com/objectfolder/contact-localization/tree/4bb002f519cab9d250bbbe045a6df0248bf1639f)
   binds microphone records and mesh-coordinate labels under the same
   object/contact key. Its current HEAD remains
   `4bb002f519cab9d250bbbe045a6df0248bf1639f`.
4. The prior [A1R force-source freeze](physical-sound-r3a-v10-a1r-object51-force-source-freeze-2026-08-31.md)
   verifies six complete raw object-51 contacts, each with `mic.wav`,
   `Force.wav`, scalar force metadata and recording metadata. All six raw
   microphone files are byte-identical to the coordinate-bearing compact
   publication.

RealImpact remains a secondary transfer/radiation control. Its official
[repository](https://github.com/samuel-clarke/RealImpact) currently says that
only preprocessed data are downloadable and the raw dataset is still being
packaged. It therefore does not satisfy paired-raw C2 admission today.

## Current endpoint identity

Read-only checks on `2026-08-31` returned:

| Endpoint | HTTP identity |
| --- | --- |
| `audio_data_51_60.tar.gz` | `200`, `34,360,300,077` bytes, ETag `"63e36c0f-80008922d"`, Last-Modified `2023-02-08 09:31:59 GMT`, byte ranges accepted |
| `mesh_16k.tar.gz` | `200`, `417,661,010` bytes, ETag `"64042f2a-18e50052"`, Last-Modified `2023-03-05 05:56:58 GMT`, byte ranges accepted |

The exact raw archive URL is
`https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/audio_data_51_60.tar.gz`.
The inventory does not download the full archive. It reuses the previously
hash-closed `512 MiB` prefix solely as structural evidence.

## Exact zero-decode witness

The existing independent runs remain byte-identical:

- manifest SHA-256:
  `3041c19d07f78fc9bdfd092ced72e07b5e795b235a286a23ede68108556b6ed5`;
- report SHA-256:
  `b421743e147132305aa0826f3dc2ded0c2a476151e9c2cad11f035995e917cb9`;
- prefix SHA-256:
  `f58e242af7b1b9f500b86e438f2c90ad6cb3f18d29914e1431c69c6a20c78584`;
- selected raw members: `24`, selected bytes hashed: `6,912,924`;
- raw/compact microphone byte identities: `6/6`;
- WAV headers validated: `18`;
- decoded microphone/force values: `0 / 0`;
- selected payloads emitted: `0`.

The structural witness contacts are `27,15,4,3,9,18` in raw archive order.
They prove source shape only. They are already opened by V10 work and are not
eligible as the fresh C3/C4 target or as independent validator evidence.

## Axis matrix

| Required fact | Result | Bounded consequence |
| --- | --- | --- |
| Paired raw force and microphone | `PASS` | Same raw object/contact directory contains both channels. |
| Common timebase | `PASS_SYNC_ONLY` | Paper states synchronization; paired headers are mono `48 kHz`, `288,000` frames. Absolute channel delay remains uncalibrated. |
| Stable object/trial identity | `PASS` | Raw and compact microphone byte identity joins the namespaces. |
| Impact coordinate | `PASS_LABEL` | Published XYZ exists; it is not proven to be an exact full-mesh barycentric vertex. |
| Geometry and scale | `PASS_REVISION` | Official point cloud/scale and stable mesh endpoint exist; C3 must freeze one exact geometry artifact. |
| Listener/microphone pose | `ABSENT_NUMERIC` | Only one canonical recorded setup may be claimed; no radiation field. |
| Support/fixture | `ABSENT_PER_OBJECT` | Paper gives support families, not the exact object/trial assignment. |
| Force units/sensitivity | `ABSENT` | Model may use normalized force counts, not calibrated newtons. |
| Microphone units/sensitivity | `ABSENT` | Model may use normalized microphone counts, not pascals/SPL. |
| Saturation/headroom metadata | `ABSENT` | C3 must freeze a waveform-independent header/integer-range check and C4 must fail closed after authorized decode. |
| Parent grouping | `PASS_PROJECT_OBJECT_CONTACT` | Object/contact groups are stable; one project cannot prove cross-project generalization. |
| Stable download/provenance | `PASS` | Official HTTPS endpoint, pinned site source and pinned benchmark repository. |
| Redistribution | `RESEARCH_ONLY` | Raw payload stays external and is not redistributed or packaged by NextEngine. |

The official page says recordings are six seconds while the paper says five;
the observed raw and compact headers are six seconds. C3 must freeze the bytes
and headers, not silently resolve this publication discrepancy by prose.

## Provenance inconsistency guard

The current rendered download page and the pinned website-source commit disagree
on some object identities in the `81…100` range. For example, the pinned source
maps object `91` to `Solid_Spoon / Steel` and object `93` to
`Glass_Green / Glass`, while the rendered table observed during this inventory
shows a shifted mapping.

Object `51` is unaffected. C3 must nevertheless bind any fresh target to one
exact pinned object table plus archive paths and compact byte identities. An
unpinned material name from the live HTML cannot select a target or validate a
material-family claim.

## C3 entry conditions

C2 is complete, but PCM remains closed. C3 must now:

1. choose a fresh exact object/contact set not opened by V8–V10 evidence;
2. freeze the full source/geometry identities and missing-axis claim;
3. assign parent-disjoint estimator fit, generator development,
   representation holdout, validator calibration, validator method holdout and
   admission shadow roles without signal-derived selection;
4. set exact per-role member/sample read budgets and zero-read preflight;
5. define normalized-count semantics and fail closed on header, clipping,
   pairing, coordinate or lineage mismatch.

Only a committed C3 manifest may authorize the first bounded raw-channel read.
