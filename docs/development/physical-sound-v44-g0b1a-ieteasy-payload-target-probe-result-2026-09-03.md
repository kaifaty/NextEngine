# Physical sound V44 G0B1a — IETeasy payload/target probe result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT / 15_OF_15_PAYLOADS_HASH_VERIFIED / MONO_48_KHZ_MEDIA_PASS / 15_OF_15_ONSET_AND_TARGET_PROBES_COMPLETE / TARGET_EXTRACTOR_AUDIT_REQUIRED / CORPUS_CREDIT_ZERO / PSEL_BLOCKED`

Planning authority: [Roadmap V44](../plans/physical-sound-synthesis-roadmap-v44.md)

## Outcome

G0B1a converts the metadata-only IETeasy lead into exact payload evidence
without selecting recordings by sound quality. Before any audio access, the
tracked selection profile froze one publisher record per physical specimen:
repetition `1` for all `15` catalogue-eligible parents, ordered by physical
parent ID. The resulting `1,657,896` bytes were fetched from the official
Mendeley routes into the external research store and every file matched the
publisher byte count and SHA-256 frozen by G0B0.

The terminal decision is
`G0B1A_IETEASY_PAYLOAD_REPEATABLE_TARGET_EXTRACTOR_AUDIT_REQUIRED`. All `15`
files are valid mono 48 kHz MP3, decode deterministically, contain a usable
impact onset and cover the complete three-second C0 canonical window. The C0
target extractor also completes on every record. It is not promoted directly
into a corpus successor because all `15/15` probes saturate its frozen
`maximum_modes=12` cap and all decoded segments peak above unity before C0
normalization. Those observations require a bounded extractor/normalization
audit; they are not grounds for choosing a favorable mode count after seeing
the data.

No current supported-parent, role, PSEL, validator, model, admission, cooker,
demo or runtime credit is granted. The IETeasy project remains one disclosed
project and the current planning deficit remains `49` parents. Authored clips
remain the only production path.

## Signal-blind selection and access boundary

The selection was fixed by four data-independent rules:

1. include every G0B0 `prospective_catalogue_eligible` IETeasy parent;
2. select exactly `repetition_index=1` for each parent;
3. order parents by `lineage.physical_parent_id` and break waveform ties by
   `record_id`;
4. assign the whole project the future `generator_train` role intent, without
   role credit, so one project can never leak across train/development or
   validator roles.

The profile caps each source payload at `262,144` bytes, the whole selection at
`2,097,152` bytes and decoded duration at `1–30` seconds. The owner performs no
network request. It accepts only regular non-symlink external payload files
named by their expected SHA-256, verifies bytes/hash first, then runs the
hash-bound `/usr/bin/ffprobe` and `/usr/bin/ffmpeg` binaries. Candidate, model,
validator and protected values remain unread.

## Exact media and target observations

| Observation | Result |
| --- | ---: |
| Selected/verified payloads | `15 / 15` |
| Selected payload bytes | `1,657,896` |
| Physical parents / project revisions | `15 / 1` |
| Steel parents | `5` |
| Codec / channels / rate | `MP3 / mono / 48,000 Hz` for `15 / 15` |
| Publisher-reported duration range | `7,300–11,975 ms` |
| Decoded sample-count range | `350,402–574,802` |
| Detected onset range | samples `113,649–282,052` (`2,367.688–5,876.083 ms`) |
| Minimum audio remaining after onset | `4,561.167 ms` |
| Complete unpadded C0 source windows | `15 / 15` |
| C0 targets extracted | `15 / 15` |
| Target probes at the 12-mode cap | `15 / 15` |
| Pre-normalization raw-peak range | `1.196791981–1.590879894` |
| Candidate/model/validator/protected reads | `0 / 0 / 0 / 0` |

The target output stores only hashes and bounded structural diagnostics; it
does not publish audio or feature values into Git. C0's existing three-second
PCM normalization makes every probe finite and reproducible, but G0B1a does
not infer absolute force or comparable cross-source loudness from normalized
recordings.

The next extractor audit must be frozen before examining any alternative
target values. It should compare the unchanged `12`-mode representation to a
single declared higher-cap diagnostic, verify that target distances and
descriptor-control conclusions are not rank artifacts, and test normalized
PCM/target identity under a deterministic overshoot policy. If the cap or
normalization materially changes the representation, a versioned C0 target
successor is required; otherwise the existing C0 policy may be reused with an
explicit saturation flag.

## Frozen implementation

- signal-blind selection profile:
  `lab/profiles/physical-sound-v44-g0b1a-iet-payload-selection.v1.json`,
  `2,908 bytes`,
  `sha256=225011894ef31ec959f399ba0137cc407dd000ad20ca10c13f5060630b1df242`;
- execution profile:
  `lab/profiles/physical-sound-v44-g0b1a-iet-payload-probe.v1.json`,
  `4,144 bytes`,
  `sha256=02b5ce99409c6bf15acef7bd1d6a540bf5291e606aee967afc30274218a473f4`;
- owner:
  `lab/scripts/physical_sound_v44_g0b1a_iet_payload_probe_v1.py`,
  `29,413 bytes`,
  `sha256=7538ecdd874ceda518678934ff1829bf2039ab023df2f0d83b5709b9a07e84af`;
- focused tests:
  `lab/tests/test_physical_sound_v44_g0b1a_iet_payload_probe_v1.py`,
  `11,489 bytes`,
  `sha256=43abd8d09c3eb80e89d5925e2df47f2547e66cebcb006a2504623edd1596a2ed`.

The execution profile binds SPEC-45, the G0B0 result and prospective contract,
the immutable selection, the C0 owner/policies, exact NumPy `2.5.2` and SciPy
`1.18.0`, and the exact ffmpeg/ffprobe binaries. The live roadmap is not a hash
dependency.

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v44-g0b1a-iet-payload-2026-09-03`

`payload/` contains the `15` publisher payloads under their SHA-256 names and
is not tracked. `run-a` and `run-b` consume the same immutable profiles,
prospective manifest and payload root and are byte-identical. Each official
output has six files and `63,894` bytes. SHA-256 of the canonical sorted array
of `{bytes,path,sha256}` rows is
`a692679055584dbe648f12232a0db94a3088b22bc477109cec1813c6587b8bb6`.

| Artifact | SHA-256 |
| --- | --- |
| `access-ledger.json` | `3e2842f30a1cfe671d8afaa74a3eae66e34ddff85624bf1d27e098bd7c44dab2` |
| `media-probe.json` | `0cbfab2ac2638fae3f9242ba061caffdbbac69d7f070630b1746d3b4e9fc7dda` |
| `profile.json` | `02b5ce99409c6bf15acef7bd1d6a540bf5291e606aee967afc30274218a473f4` |
| `report.json` | `7b8ab02c50e495202717151873ed590ad1e21a021e0877c3758d829e87a0ccd1` |
| `selection-profile.json` | `225011894ef31ec959f399ba0137cc407dd000ad20ca10c13f5060630b1df242` |
| `selection.json` | `635c7fb68d8e54c6074fe29e82449eef67f1e12b62b0fcec5c36e994965da484` |

## Verification and next action

- focused suite: `PASS`, `6/6` tests;
- official external A/B and recursive byte comparison: `PASS`;
- publisher byte/SHA-256, MP3 structure, mono/48 kHz, duration, decode, onset,
  full source-window and target extraction checks: `PASS`, `15/15`;
- hash mutation, wrong sample rate, duplicate repetition, repository payload,
  repository output and forbidden network imports reject before publication;
- Ruff `0.14.1` format/check: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL` only on the pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
  G0B1a adds no source-layout escape hatch and receives no boundary-scan credit;
- runtime ProductChecks: `NOT_RUN`; this is external research tooling under
  SPEC-45 `Proposed` and changes no production consumer.

G0B1b is next: preregister and run the bounded 12-mode/overshoot target audit,
then freeze the `15` selected records into a disclosed corpus successor only
if that audit admits reuse. In parallel, metadata-first source search must
find at least a second independent Steel descriptor-to-signal project. Until
both materialization and source power close, PSEL, B1, validator/model
training, protected access, cooking and demo integration remain blocked.
