# Physical sound PS-2 — CMU Heller independent E3 pilot

Date: 2026-08-27
Status: `THIRD_PROJECT_GROUP_ADDED / GLASS_5_OF_16 / DEVELOPMENT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY`

## Question

Can a third published project add another explicitly identified Glass object
without local capture, unsafe archive extraction or counting different
impactors on one object as independent target objects?

## Source review and bounded claim

The official [CMU AuditoryLab Sound Events Database](https://www.auditorylab.org/sound-events-database)
states that each sound-event type has five exemplars recorded under similar
controlled conditions. Its versioned [Impact Events item](https://kilthub.cmu.edu/articles/media/Impact_Events/20205035)
is DOI `10.1184/R1/20205035.v1`, posted 2022-07-07, and publishes two audio
archives plus recording notes.

The adapter credits only the archive directory explicitly named `Marbles
Dropped in Glass Vase` and its five numbered WAV files. This establishes one
stable Glass object/event family with five repeats for E3 material/object,
real-recording and repeat identity. The WAVs are multi-impact event recordings,
not isolated force-calibrated strikes. They grant no force, geometry, impact or
listener position, support, composition or transfer-response evidence.

The reviewed `Mirror` directories are excluded because the source does not
explicitly label their material as Glass. The `Red Vase` recordings are also
excluded because its material is unspecified. Different balls and hammers on
the same mirror are not counted as new target objects.

The item is `In Copyright` with an explicit research-use grant that excludes
commercial-product incorporation or compensated transfer and requires the
stated acknowledgements. It is therefore frozen as
`LicenseRef-Heller-Sound-Events-Research-Only` / `external_research_only`.
Source archives, extracted WAVs, caches and reports remain outside the
repository and distribution.

## Implemented boundary

`physical-sound-registry internet-sources` adds the typed
`heller-impact-identified-recording-v1` adapter and one source-specific
`figshare_kilt_hub_v1` redirect policy:

1. only `https://ndownloader.figshare.com/files/<decimal-id>` is accepted;
2. the origin is public-IP checked and pinned without following redirects;
3. exactly one `302` target must use the approved CMU KiltHub S3 bucket, the
   same file ID, a bounded canonical filename and the six-field AWS V4
   signed-query shape;
4. the target is independently public-IP checked and pinned, then streamed
   through the existing exact byte/SHA-256 cache boundary with no second
   redirect;
5. generic redirects remain disabled.

The new bounded ZIP reader accepts only safe unique paths, non-encrypted
stored/deflate entries, at most 1,024 entries, at most 512 MiB total
uncompressed bytes and at most 64 MiB per entry. It reads only the five frozen
paths into memory and validates each entry's exact byte count/SHA-256 before a
strict RIFF/WAVE PCM16 stereo 44.1 kHz check. Nothing is extracted to the
filesystem by the adapter.

The source files are:

| Artifact | Figshare file | Bytes | SHA-256 |
|---|---:|---:|---|
| `Impacts_audio1.zip` | `36113411` | 38,059,995 | `1d57964b4f48d277b6cf567a0bce938b3b8901f6b43ca6787db1036e8d737783` |
| `impact_recordingNotes.rtf` | `36113402` | 42,010 | `c38fa7cc1d49534e6979609d6019354efe040b52dcadf667e8f619c9f2dcf8a1` |

The five WAVs contain 489,472–719,124 sample frames. Their individual exact
hashes and archive paths are frozen by the adapter and emitted in its typed
evidence report.

## Reproducible result

External root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-independent-e3-heller-v1/
```

| Artifact | SHA-256 |
|---|---|
| Heller provenance review | `cfe53522901cd8317bc7fb379c7a0f7399a653c7d5e5ffe2d6744951ba4d3bc1` |
| Combined source manifest | `6377ea4019ca37ada7a9ca1c1a6e8bff71647a83d2b8d9ea39ee074caf31567e` |
| Combined source report | `531faacddc2b4c45d5a64370c7a3faf1a5429a37b7ee44f968382b5fc208d179` |
| Identified-corpus manifest | `0ac9b2785d35ef52e2ebada3fab848434a06e8458f6149d7934fd5c9f01d335b` |
| Identified-corpus report | `68dd8ec320fb00a56a4e52ffd90b424e4ef4e7b04355c27f8366f6e15714090c` |

Two fresh credential-free online caches and one offline source audit emit
byte-identical source reports. Identified-corpus reports over both caches and a
second offline repeat are also byte-identical.

Combined AV-MSF + YCB Impact + Heller result:

- 13 evidence-ready objects and 33 E3 recordings;
- three publishers and three publisher/project/revision groups;
- Glass contributes five object groups and 17 recordings;
- eleven of the required sixteen Glass groups remain missing;
- zero of the required 35 reject-mutation parent groups are supplied by E3;
- every entry stays in `dev`; calibration, holdout and shadow remain unopened.

The previous combined YCB source and identified reports remain byte-identical
at `6954710b…feba9f76` and `12258a43…7b3ffc3`, respectively.

## Re-plan and next action

The source is admissible for one bounded E3 object, but it falsifies the idea
that the remaining CMU impact archive can cheaply close Glass coverage: mirror
and red-vase rows lack explicit Glass identity and multiple impactors do not
create independent object groups.

The next bounded source candidate is Greatest Hits because its official page
publishes material/action labels under CC BY 4.0 and substantially more real
interactions. Its smallest official labeled video package is still 20 GB, so
the next package must first obtain a hash-closed material/object inventory and
prove bounded object-level retrieval or reject it before downloading the full
archive. In parallel, REALIMPACT remains the preferred E2 expansion only when
additional official preprocessed/raw rows become actually downloadable. Until
then Glass remains fallback-only, `Pass` and PS-3 remain blocked, and the clip
path remains the production baseline.
