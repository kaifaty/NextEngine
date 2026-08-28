# Physical sound PS-2 — exact-domain E2/E3 claim matrix

| Field | Result |
| --- | --- |
| Status | `DOMAIN_CLAIM_MATRIX_EXECUTABLE / ONE_CROSS_TIER_LINK / EIGHT_REQUIRED_CLAIMS_OPEN / FALLBACK_OUT_OF_DOMAIN` |
| Scope | Current-only external evidence audit; no corpus admission, validator release, runtime/content contract or production promotion |
| Command | `physical-sound-registry domain-claims` |
| Profile | `thin-soda-lime-glass-open-vessel-impact-v1` |
| Manifest SHA-256 | `618dfb4392b816f7bad8344a3c9b4a6aa71326087f04189586c767777894aec1` |
| Repeated report SHA-256 | `e6d078bd792ab45f09045bd272df3b39fafa0cb02c3c4e0fddb064aab95f5b60` |
| External root | `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-domain-claims-v1/` |

## Question and decision

The frozen project split proves leakage-safe corpus structure, but it does not
say whether the E2 transfer responses and E3 identified recordings jointly
support the exact physical domain in the PS-2 plan. The new gate makes that
question executable and fails closed per claim.

The measured decision is:

```text
Validated / DomainEvidenceIncomplete / FallbackOutOfDomain
```

The current evidence supports one exact object-identity bridge plus useful
transfer and recording observations. It does not support the exact composition,
geometry revision, fixture, absolute excitation, planned impact/listener
conditions or matched-condition lineage. No frozen partition contains an
exact-domain-eligible object, so PS-3, `Pass` and AV-P0D remain disabled.

## Hash-closed inputs

| Input | SHA-256 |
| --- | --- |
| Frozen corpus plan report | `e082610c90dabff3c7a328df94671dca4f84f46cd629952c3e914ce600a3ea01` |
| Partitioned E3 report | `9f5b7f8a2a94d57ea874d7a58a513d8012999b8b43e94fa34f4094de8c68d4f0` |
| Verified project split report | `7cb1532cb5e94792dd27d5fd13a1c11a2c2e5fbecbc017a2c2bcbcb74dc2a9f8` |
| REALIMPACT GlassGoblet E2 | `f33d82e3b72fa2de23c365c6abc0663a58ee668451c8f0b601416608f3a6b6db` |
| REALIMPACT GreenGoblet E2 | `9cab3bcb504dfd161150e7eb868074ad9176a9210019b4ab75a0a49932aadc53` |
| REALIMPACT Blue Bowl E2 | `5132aa224fbe81740de4cec321b2e63d9ea6f8f2df8dc69a04d13f9ea107b8b0` |
| REALIMPACT Shell Plate E2 | `68a9cee2812143b42d3ac72b6497537f824c31b16a5a3eeb3eb851d287613268` |
| REALIMPACT Skull Cup E2 | `f393c8bd0ccadd54ffbf767df2c21384ca57cf5457c8b58d0076a6e1e8e6c45a` |

Every referenced report must be outside the repository, match its declared
hash and retain its narrow current-only decision. The split report must be
`ProjectDisjointSplitVerified` and bind the exact partitioned E3 report.

## Exact object link

The V1 link adapter accepts only the already reviewed Blue Bowl identity:

```text
REALIMPACT 6_Bowl / realimpact-blue-bowl-row0000
    <-> ObjectFolder-Real object 6 / Blue_Bowl
```

Its ObjectFolder E3 group is in `calibration` and has three recording
identities. Numeric equality is not a generic identity rule. In particular,
the unit failure control rejects linking REALIMPACT `94_GlassGoblet` to
ObjectFolder-Real object 94 `Salad_Bowl` merely because both use `94`.

The other four E2 entries remain explicitly unlinked in this matrix. They still
provide typed transfer observations, but cannot borrow an E3 object identity.

## Claim result

Supported claims:

| Claim | Evidence |
| --- | --- |
| Cross-tier object identity | Frozen Blue Bowl link |
| Real-recording identity | Linked E2 transfer plus E3 recordings |
| E3 repeat-recording identity | Three ObjectFolder-Real Blue Bowl recordings |
| Force-deconvolved transfer | Linked REALIMPACT Blue Bowl row |
| Geometry, impact and listener observations | Five typed REALIMPACT E2 rows; informational only until aligned to the exact domain |

Required claims still unsupported:

| Claim | Frozen expected value | Current observation / blocker |
| --- | --- | --- |
| Material composition revision | `soda-lime-glass` | Only generic `glass`; composition revision unavailable |
| Geometry revision | `measured-calipers-and-mass-v1` | Five object meshes with unrelated revisions |
| Support fixture revision | `base-on-20mm-foam-annulus-v1` | `thread-mesh-unversioned-in-archive`; fixture revision unavailable |
| Absolute excitation profile | `instrumented-impact-hammer-v1` | Force-deconvolved transfer only; raw force-profile bytes unavailable |
| Impact condition | `rim`, `wall-midpoint` | Mesh-vertex coordinates without the planned condition/speed identity |
| Listener condition | `half-metre-axis`, `half-metre-radial` | One 230 mm REALIMPACT listener profile |
| Matched cross-tier condition | Same object, condition and repeat lineage | E2 and E3 are object-linked, not condition-linked |
| Four-partition exact-domain coverage | At least one eligible object in every partition | `dev/calibration/holdout/shadow = 0/0/0/0` |

The partitioned E3 corpus itself remains useful and unchanged:

| Partition | E3 Glass objects | E3 Glass recordings | Identity-linked objects | Exact-domain eligible |
| --- | ---: | ---: | ---: | ---: |
| `dev` | 3 | 16 | 0 | 0 |
| `calibration` | 3 | 11 | 1 | 0 |
| `holdout` | 4 | 11 | 0 | 0 |
| `shadow` | 6 | 8 | 0 | 0 |

## Reproduction and controls

Run twice with different empty output directories:

```text
cargo run -p xtask -- physical-sound-registry domain-claims \
  --manifest /home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-domain-claims-v1/manifest.json \
  --output /home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-domain-claims-v1/report-e
```

`report-e/report.json` and `report-f/report.json` are byte-identical at
`e6d078bd792ab45f09045bd272df3b39fafa0cb02c3c4e0fddb064aab95f5b60`.
Five focused tests cover argument bounds, the admitted Blue Bowl identity,
rejection of the false object-94 link, exact-axis fallback and four-partition
closure.

## Consequence and next action

The matrix closes the ambiguity between “we have geometry/position data” and
“we have geometry/position data for the exact planned condition.” Current E2
observations establish the former only. More target-only recordings or formula
tuning cannot repair the eight explicit blockers.

The next bounded step is an internet-source feasibility research cycle ordered
by these claims:

1. search published sources for stable same-object geometry, declared support,
   excitation/force, listener and repeat lineage that can link to an existing
   frozen E3 object without moving its partition;
2. if such evidence exists, add one typed link/adapter and rerun this matrix;
3. if no published source supports the acquisition-shaped V1 axes, retain V1
   as fallback-only evidence and propose an internet-native plan revision whose
   exact domain is backed by published conditions rather than invented metadata;
4. do not select PS-3, expose shadow to optimization or change the clip-based
   production baseline until an exact bounded domain closes.
