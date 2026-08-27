# PS-2 declarative Freesound pack adapter — 2026-08-28

| Field | Result |
| --- | --- |
| Status | `DECLARATIVE_ADAPTER_PROVEN / COVERAGE_UNCHANGED / NEW_RAW_SOURCE_BLOCKED_BY_HTTP_403` |
| Scope | External current-only `E3IdentifiedRecording` acquisition; no corpus admission, runtime or content promotion |
| Adapter | `freesound-pack-identified-recording-v1` |
| Frozen source manifest | `29ca713ce379d3f43307220dfa7b1c7a6911f27df98ff3c2688f9f843734569a` |
| Repeated source report | `2dc5dcbda2cabe274be46a8c7a3b7a50ef645e8e00144a30f0cb221b9010d92a` |
| Frozen identified manifest | `c61dc19430fe496172944c330184a92d036c052ec760ed99d7c976a2779f8ee8` |
| Repeated identified report | `63eeea4a724e4cf1ed0ae43836e4aad641b1114d97c2fefb2270669cf364c5a5` |
| External root | `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-declarative-freesound-pack-v1/` |

## Question and decision

The two admitted Freesound source families used separate Rust modules whose
logic differed mainly in frozen constants. Repeating that implementation for
every published pack would make PS-2 acquisition scale with code changes, but
moving all trust into a free-form manifest would let a manifest self-assert a
material or object identity.

The new adapter therefore makes only the source-specific values declarative.
It still derives and verifies the canonical Freesound landing and preview
URLs, publisher/pack identities, review revision, supported Creative Commons
policy and the exact artifact set. A source receives the four E3 capabilities
only after all of the following hold:

- the normalized pack identity, author ID, title, description and selected
  sound cards match hash-closed cached bytes;
- every HQ preview has an exact byte count and SHA-256, canonical CDN path,
  MPEG-1 Layer III/Xing/LAME structure and exact gapless sample-frame count;
- object and recording evidence phrases contain a supported material token
  and occur in publisher metadata, rather than only in the manifest;
- at least two sorted recording identities belong to the same declared pack
  family;
- every E3 capability binds every pack/preview artifact.

The two old pack-specific adapters remain supported. Historical manifests and
reports are not migrated, so their exact evidence stays replayable.

## Frozen compatibility experiment

The generic profile was instantiated over the already frozen medium Glass bowl
pack 14905 and wine-glass pack 41981, using two pre-existing independent cache
roots. Both audits produced the same source report hash and kept all `15/15`
sources `EvidenceReady`. The explicit-role identified-corpus audit also
repeated byte-identically:

| Measurement | Result |
| --- | --- |
| Publisher/project revisions | `5` |
| E3 objects / recordings | `15 / 44` |
| Glass target groups / recordings | `7 / 28` |
| Missing Glass target groups | `9` |
| Explicit reject parents | `8 / 35` groups, `16` recordings, all `dev` |
| Missing reject parents | `27` |
| Calibration / holdout / shadow | empty |
| Decision | `DevelopmentCoverageMeasured / NO_CORPUS_ADMISSION_AUTHORITY` |

This is a scalability and non-regression result, not new corpus coverage.

The legacy source report remains
`78a965b2d01d13a53b5c4eeace8da135b89cfbfd1e079c1af1dbb674b2665ad0`
and the explicit-role identified report remains
`2ac5c3128a1950d4dfb36324baf7e9fe0336e1f9b2884ed1255c233c8f3b9e48`
after the implementation change.

## Failure controls

| Mutation | Manifest SHA-256 | Exact result |
| --- | --- | --- |
| Change the wine-glass material to `Steel` without matching publisher phrases | `d7e6b86a7e464166cc88739ea188cebdaa3f77e10fb7a57cf16d2108ba315670` | Rejected before fetch/report; result `bc36ffc64adaf33c43c87e7fbab68df2a892e257232566d8726938eb98b3e4e3` |
| Change the derived author ID in one preview URL | `76e72c33e268df6e931ec9fa49387786b18f1f562c05d4b3afd3a7d95543113c` | Rejected before fetch/report; result `77c09a76c0d16f9fee8dfcd6be8618db642600fdee0bd78327c027034f512660` |

Unit coverage also rejects license-policy drift and incomplete/non-material
evidence phrases.

## Bounded source research

The [astriferal Glass Bottle pack](https://freesound.org/people/astriferal/packs/29040/)
is the strongest next target candidate found. Its selected sound pages identify
one cleaned IBC cream-soda bottle, a wooden guiro striker and distinct strike
conditions, including [middle strike while suspended](https://freesound.org/people/astriferal/sounds/516170/)
and [near-rim strike](https://freesound.org/people/astriferal/sounds/516164/).
The [limpinglizards Glass bottle pack](https://freesound.org/people/limpinglizards/packs/42864/)
is a second publisher candidate with three numbered impacts of a soda bottle
onto concrete.

Neither candidate is imported in this checkpoint. The current direct host path
returns HTTP 403 for canonical Freesound pages, and the in-app browser blocks
the same page before raw DOM access. Search-index text is sufficient for source
discovery but cannot supply canonical bytes, author IDs, preview hashes or a
reproducible normalized pack identity. The official API is also not a silent
fallback because [Freesound requires an API credential](https://freesound.org/docs/api/authentication.html).
No proxy exception, guessed CDN identity or fabricated hash was added.

## Consequence and next action

Adding a reviewed Freesound pack can now be a manifest/evidence operation
instead of a new Rust adapter module. The next PS-2 acquisition step remains:

1. obtain exact raw metadata and preview bytes from a changed bounded route, or
   select a different published source with a working exact-download path;
2. run the declarative adapter on at least one new Glass family and independent
   non-Glass parents;
3. keep all new groups in `dev` until the full `16` target and `35` reject-parent
   minima exist, then freeze partitions and power analysis.

`Pass`, PS-3, AV-P0D and production P1 remain disabled. Authored clip playback
remains the mandatory runtime fallback.
