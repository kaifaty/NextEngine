# Physical sound PS-2 — independent YCB Impact E3 pilot

Date: 2026-08-27
Status: `INDEPENDENT_E3_SOURCE_ADDED / 2_PROJECT_GROUPS / GLASS_4_OF_16 / NO_CORPUS_ADMISSION_AUTHORITY`

## Question

Can a second published project contribute independently grouped, identified
real Glass recordings without local capture, a generic redirect allowance or
invented force/geometry/support metadata?

## Implemented boundary

`physical-sound-registry internet-sources` now accepts an optional artifact
redirect policy. The only implemented policy is `osf_storage_v1`; the default
remains no redirects, so all prior AV-MSF and ObjectFolder reports are
unchanged.

The OSF policy is deliberately source-specific:

1. the manifest must name a canonical
   `https://files.de-1.osf.io/v1/resources/<node>/providers/osfstorage/<file>`
   URL;
2. the origin hostname is resolved first, every address must be public and curl
   is pinned to one address without following redirects;
3. exactly one `302` target is read and validated as
   `https://storage.googleapis.com/cos-osf-prod-files-de-1/<expected-sha256>`
   with the bounded four-parameter OSF signed-query shape;
4. the target hostname is independently resolved and public-IP pinned;
5. no further redirect is followed, and the existing streaming size/SHA-256
   checks must pass before the file enters the external content-addressed cache.

This is not a general-purpose redirect feature. A changed bucket, path, object
hash, parameter set, protocol, non-public address or second redirect fails
closed.

The new `ycb-impact-identified-recording-v1` adapter freezes exactly two
objects and four recordings per object. It validates official object/material
metadata, upstream split/condition/clip/file identity and actual audio
structure. The source files use an `.ogg` suffix but contain RIFF/WAVE IEEE
float32 stereo PCM at 48 kHz, 240,000 frames (5 seconds); the adapter reports
the actual container and encoding rather than trusting the suffix.

The adapter grants only:

- `material_identity`;
- `object_identity`;
- `real_recording`;
- `repeat_identity`.

That closes `E3IdentifiedRecording` only. Force, geometry, impact/listener
position, material composition, support condition and transfer response remain
unavailable axes.

`identified-corpus` accepts the new typed evidence alongside AV-MSF. Upstream
`train` and `test` remain source metadata; both YCB objects stay in NextEngine
`dev` and do not silently open calibration, holdout or shadow.

## Frozen official source

The source is the official [YCB impact sounds project](https://osf.io/4tcp6/)
from IRI/CSIC-UPC and CTU, specifically its public
[robot-impact component](https://osf.io/bj5w8/). The OSF API reports component
title `Robot_impact_Data`, description “Raw Audio Files collected from vertical
and horizontal poking action by a robot on a subset of YCB objects”, and last
modification `2022-09-27T11:36:47.167853`. The accompanying primary paper is
[Recognizing object surface material from impact sounds for robot manipulation](https://www.iri.upc.edu/files/scidoc/2619-Recognizing-object-surface-material-from-impact-sounds-for-robot-manipulation.pdf).

The exact `ycb_audio.xlsx` workbook is OSF file
`62330db6e919450662177fa3`, 7,994 bytes, SHA-256
`27672ecfdfaf2a1ecfc8926127ab9ab962e9cb59adb593b141caadc0459ae0fd`.
The adapter freezes these rows and source directories:

| Object | Primary/secondary material | Upstream split | Conditions | Recordings |
|---|---|---|---|---:|
| YCB 23 `Wineglass` | `Glass` / none | `train` | `horizontal-0_14`, `horizontal-0_25` | 4 |
| YCB 28 `Skillet lid` | `Glass` / `Hard Plastic` | `test` | `horizontal-0_14`, `horizontal-0_25` | 4 |

The source folder labels `0_14` and `0_25` are preserved verbatim. No physical
unit is inferred for them.

No explicit dataset license was found on the reviewed OSF project/component
surface. The source therefore remains `NOASSERTION`,
`external_research_only`; source bytes, caches, manifests and reports remain
outside the repository and distribution.

## Reproducible result

External root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-independent-e3-ycb-v1/
```

Frozen identities:

| Artifact | SHA-256 |
|---|---|
| YCB provenance review | `60864a7c670dd3ecc3378b2e1e85359f5efdf6f5e75ee57f32d26c13513f9fcb` |
| Combined source manifest | `0d35c8566fe8440b1763bff69989f54ae6f9a1f1af758134d1d56ac7ed151788` |
| Combined source report | `6954710baff8af8209f011853c8443a9b8d2a9d67450e9dd44191423feba9f76` |
| Identified-corpus manifest | `0caa8cce20144fcf0bbfc8e55e3c7804c8ee1b2922eb8d28c890b6d6ab986bde` |
| Identified-corpus report | `12258a434034e878cd3d4f361ffbb08363d1ca9c4115e28854de7aa4d7b3ffc3` |

Two fresh credential-free online fetches built separate caches. Their source
reports are byte-identical to each other and to a later no-network audit. The
identified-corpus reports over both caches and the repeated offline run are
also byte-identical.

The combined result is:

- twelve evidence-ready sources, twelve object groups and twenty-eight
  recording groups;
- two publishers and two publisher/project/revision groups;
- six primary material labels;
- all entries remain in `dev`; calibration, holdout and shadow remain unopened;
- Glass contributes four object groups and twelve recordings;
- the frozen minimum is sixteen Glass groups, so twelve remain missing;
- E3 contributes zero of the required 35 reject-mutation parent groups;
- every unavailable force/geometry/support/transfer axis remains unavailable.

The refactored WAV validator and optional redirect field preserve the prior
AV-MSF reports byte-for-byte: source report
`203f8dce49c0bd5943c07d064651fb4fed39d7a73f9122d0eb0e2a31969d47b9`
and identified report
`9379faf957c7b428bb66883b694868282d0b9dbd34c35afd627f5c70d55c04f4`.

## Decision and next action

The independent-source hypothesis is supported: the pipeline can safely add a
second published publisher/project/revision and preserve exact grouping. It is
not enough to activate PS-3. Four Glass object groups are still only one
quarter of the pre-registered minimum, and none of the E3 recordings close the
matched force/geometry/support claims required for exact-domain admission.

The next PS-2 package must re-plan the remaining published-source search from
the measured `4/16` baseline, prioritize additional independent Glass object
families and complementary E2 arrays, and keep all new groups in development
until the powered partition plan can be frozen before calibration/holdout/
shadow are opened. `Pass`, PS-3, AV-P0D and production P1 remain blocked; the
clip path remains the mandatory fallback.
