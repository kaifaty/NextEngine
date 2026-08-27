# Physical sound PS-2 — Freesound wine-glass cached E3 increment

Date: 2026-08-28

Status: `FIVE_E3_PROJECT_GROUPS / GLASS_7_OF_16 / CACHED_E3_VALIDATED / DIRECT_FETCH_RECHECK_OPEN / NO_CORPUS_ADMISSION_AUTHORITY`

## Outcome

Freesound pack 41981 adds one independently published wine-glass family and
three numbered real recordings to the development corpus. Combined AV-MSF,
YCB Impact, CMU Heller and both Freesound projects now measure:

- 5 publishers and 5 publisher/project/revision groups;
- 15 object/family groups and 44 recordings;
- 7 Glass groups and 28 Glass recordings against the preregistered `16`-group
  minimum;
- 9 missing Glass groups and 35 missing reject-parent groups;
- every entry remains in `dev`; calibration, holdout and shadow remain empty;
- automatic `Pass`, PS-3 and AV-P0D remain disabled.

The increment is cache-validated rather than an online-repeat claim. This is an
explicit acquisition limitation, not a quality or corpus-admission failure.

## Source and bounded claim

The official [Freesound pack page](https://freesound.org/people/wasserbjorn/packs/41981/)
identifies author `wasserbjorn`, pack `41981` (`Metal Clank`) and twelve sound
cards. The selected family is:

| Recording | Sound id | Publisher title | Frames at 44.1 kHz stereo |
| --- | --- | --- | ---: |
| `001` | `761160` | `Knife hits wine glass 1` | 631433 |
| `002` | `761161` | `Knife hits wine glass 2` | 962183 |
| `003` | `761162` | `Knife hits wine glass 3` | 992251 |

Every selected card carries the same recording note (`Recorded with WA jr 47
and LA120`) and `Creative Commons 0`. The exact expression is `CC0-1.0`; the
audio still remains outside Git.

The evidence grants only `material_identity`, pack-specific wine-glass family
`object_identity`, `real_recording` and three numbered `repeat_identity` values.
It does not prove that the same physical glass was retained between files and
does not grant geometry, composition, force, support, impact/listener position,
calibration, radiation or transfer-response credit.

## Executable increment

The registry now contains:

- `freesound-wine-glass-identified-recording-v1`, which freezes publisher,
  project, review, license, pack, family and recording identities;
- strict MPEG-1 Layer III/Xing/LAME audits for the three public HQ previews;
- a Freesound pack normalizer that preserves the previous described-pack bytes
  and produces an empty canonical description when a pack has none;
- typed identified-corpus normalization for the new adapter;
- positive declaration/normalizer coverage plus the existing corruption and
  partition-leakage failures.

No public schema, runtime role, acoustic material, contact projection or
production ProductCheck is changed.

## Exact integrity

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| HQ preview `761160` | 260587 | `f477d0e2de75bdf81e3e2ec6b2e220b7faa0466412073d547237b13da7711085` |
| HQ preview `761161` | 396582 | `4d371e9d9583796b524e664093aeedb295006da8a05f143cb7409ce8e7ee5991` |
| HQ preview `761162` | 409997 | `7be43b4a78f3a9775e62225cd3c96d1eeab1c3f975c8e4a6650d0e1c2a029065` |
| Canonical pack identity | 3673 | `e1eceb0d378fa1f243d2f869f32903b4dd9ac89831110dbdcde797118d14c559` |
| Provenance review | 2611 | `d79b33e14b70cef92fdf366164923cf249ab5b4485a62862cafb447cd132a41e` |

The pack identity binds canonical URL, author/user id, pack id/title, the empty
authored description, and all twelve sorted card identities, durations, sample
rates, preview URLs and license labels. Volatile ratings, counters and session
state are excluded.

## Acquisition discriminator

Ordinary HTTPS initially retrieved the exact MP3 bytes above. The registry's
bounded route deliberately disables environment proxies and pins a validated
public endpoint. On this host, direct requests to the current Freesound address
returned HTTP 403, and a probe marked all four new artifacts `FetchFailed`.

The security boundary was not weakened. Exact artifacts were imported into two
external cache roots and independently verified; two source reports and three
identified-corpus reports are byte-identical. They are not described as two
independent online fetches. Direct-fetch readiness may be restored only when the
unchanged bounded route returns 2xx and reproduces every hash.

External evidence root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-freesound-wine-glass-v1/`

| Evidence | SHA-256 |
| --- | --- |
| Direct-fetch discriminator | `4b1472a7c7c0441565f3ea455de8dbe920dcf78710f4ce4650a890edb7c248a7` |
| Incomplete direct-fetch probe report | `7a6ccbc1714da7416d0ff4ff4295dcb0eb1e268a11fbf24f9b6fa606ae41d83f` |
| Source manifest | `a621541565bd32dfa396e151f86f7c4405dcb435a7c253849f56d2fd4b45862b` |
| Source reports A/B/offline | `78a965b2d01d13a53b5c4eeace8da135b89cfbfd1e079c1af1dbb674b2665ad0` |
| Identified manifest | `23c28cfdc970065f23a998f65844b949211de6a8fbef29dbafc610157424b067` |
| Identified reports A/B/offline | `fca10741118661e8661eba5e0233824b68d2095fcf1ddb57a3e7942d49561cf2` |
| Frozen corpus-plan report | `e082610c90dabff3c7a328df94671dca4f84f46cd629952c3e914ce600a3ea01` |

## Decision and next action

Keep the wine-glass family in `dev`, preserve clip fallback and continue with a
different stable-object Glass E3 project. Do not spend the next package adding a
proxy exception or repeatedly probing Freesound. In parallel, design the first
explicit reject-parent import so the existing non-Glass E3 objects can be used
without silently treating material labels as preregistered mutation parents.
