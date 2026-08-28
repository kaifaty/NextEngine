# PS-2 REALIMPACT transfer calibration

Date: 2026-08-28

## Outcome

`physical-sound-registry transfer-calibration` now executes a hash-closed,
object-disjoint REALIMPACT experiment without committing recordings or reports
to the repository. The evidence has two deliberately separate revisions:

- V1 is retained byte-for-byte as `INVALID_METRIC_CONFOUND`. Its formal gate
  crossed the preregistered thresholds, but the measurement reused tail peaks
  and estimated several unresolved low-frequency peaks from the same coarse
  damping bins. It is negative lineage, not modal-transfer evidence.
- V2 changes the extractor before opening the reserved Skull Cup, uses Shell
  Plate as calibration, and passes every preregistered modal/damping gate on
  the fresh Skull Cup holdout. Its decision is
  `RelativeModalDampingSupportedSpatialUnavailable`.
- V2 establishes only that this relative modal-frequency/damping extraction
  method transfers across the five acquired REALIMPACT rows. It does not
  establish glass identity, perceptual naturalness, absolute amplitude,
  spatial participation, exact-domain admission, validator `Pass`, content
  authority or runtime readiness.

The accepted clip-based audio path is unchanged. All eight exact-domain claims
from the PS-2 matrix remain open, so PS-3, AV-P0D and production P1 remain
blocked.

## Question and frozen hypotheses

The bounded question was whether one extractor selected without holdout input
could recover persistent relative modes and damping behavior on a different
published real object.

- H1: a separated, injective modal extractor retains enough persistent modes
  on an object-disjoint holdout to cross the frozen gates.
- H2: the apparent persistence is a peak-assignment or frequency-resolution
  artifact and disappears when those confounds are removed.
- H3: the current rows can estimate listener-dependent spatial participation.

H1 survived V2 on the narrow measurement below. V1 supports H2 for its own
extractor. H3 is not testable because exactly one listener row was acquired
per object.

## Evidence and split discipline

Both revisions bind the prior source-feasibility report SHA-256
`9a591673fb9567c9a343aeca991e62f1cb299f3a1f6f31e15c38d6050d85b9b1`
and the five previously frozen typed E2 inventory/report hashes. Absolute
amplitude is not consumed. Seven claims are explicitly prohibited in both
revisions: absolute amplitude, exact material composition, exact support
fixture, matched cross-tier conditions, physical-sound `Pass`, production
corpus admission and runtime content role.

V1 froze this access order:

| Partition | Objects |
| --- | --- |
| `dev` | Glass Goblet, Green Goblet |
| `calibration` | Blue Bowl |
| `holdout` | Shell Plate |
| sealed `reserved` | Skull Cup |

V1 manifest SHA-256 is
`4c0f39021aec1be1523921f34fb172aa06128aae44b9a100222715ecdc44f9fb`.
Three independent outputs are byte-identical at report SHA-256
`ad1e75149f8feb7016c7fb2f8c652a16e40d95112b112ddb14dda1e00a27c321`.

Inspection after the V1 holdout gate exposed the following confound:

| Row | Selected | Matched | Unique tail peaks | Modes below 250 Hz | Unique damping bins |
| --- | ---: | ---: | ---: | ---: | ---: |
| Glass Goblet | 16 | 13 | 13 | 15 | 9 |
| Green Goblet | 16 | 12 | 9 | 14 | 10 |
| Blue Bowl | 16 | 12 | 9 | 14 | 11 |
| Shell Plate | 16 | 13 | 10 | 15 | 9 |

The formal V1 decision is therefore not reinterpreted as success and its
thresholds were not changed after inspection. Shell Plate was already opened,
so it became V2 calibration evidence. Skull Cup remained unopened until the
V2 candidate and gates were frozen.

V2 froze the new split:

| Partition | Objects |
| --- | --- |
| `dev` | Glass Goblet, Green Goblet, Blue Bowl |
| `calibration` | Shell Plate |
| fresh `holdout` | Skull Cup |

There is no remaining reserved row in V2.

## V2 remediation

The V2 extractor applies the same deterministic implementation to every row
and makes the confound controls explicit:

- candidates contain 8, 12 or 16 modes and all use a 65,536-sample FFT;
- peaks are limited to 250–12,000 Hz, at least 12 Hz and 45 cents apart, and at
  least -45 dB relative to the strongest peak;
- fit-to-tail assignment is greedy minimum-error and one-to-one within 40
  cents, so one tail peak cannot satisfy several fitted modes;
- damping uses a 16,384-sample FFT and a 2,048-sample hop over the frozen
  windows, preventing selected modes from sharing a coarse damping bin;
- candidate selection uses the Shell Plate calibration loss only after all
  candidates pass the dev sanity check. Skull Cup is evaluated exactly once
  after selection.

The calibration losses were:

| Candidate | Shell Plate calibration loss |
| --- | ---: |
| `injective-modal-8-fft65536-v2` | 4.3307117541934055 |
| `injective-modal-12-fft65536-v2` | 4.551517847634973 |
| `injective-modal-16-fft65536-v2` | 3.84184123971641 |

The last candidate was selected. V2 manifest SHA-256 is
`52dbdc59235dfe88f1533dab5c1b11e1226318e1e50727300b84bdb94441e383`;
its selected calibration profile SHA-256 is
`b364dc9af2a172f8f08dd76c1eb5484438afc0c318eda96f5d14660b18c7a6a1`.
Two complete reports are byte-identical at SHA-256
`01346767b596630061fe437e98d5213bf426e49e5b96c22565a77acfea50d444`.

## Fresh holdout result

| Skull Cup gate | Observed | Threshold | Result |
| --- | ---: | ---: | --- |
| Selected modes | 16 | at least 6 | pass |
| Persistent modes | 9 | — | observation |
| Persistent-mode recall | 0.5625 | at least 0.5 | pass |
| Median frequency error | 28.00989504479041 cents | at most 40 cents | pass |
| Decaying-mode fraction | 0.5625 | at least 0.5 | pass |
| Median tail-prediction RMSE | 6.013301162052201 dB | at most 24 dB | pass |

Each V2 row has 16 distinct damping bins. Its matched and unique tail-peak
counts agree: Glass Goblet 12, Green Goblet 7, Blue Bowl 9, Shell Plate 9 and
Skull Cup 9. The V1 confounds therefore do not explain the V2 threshold result.

This remains a small transfer-structure experiment. The holdout modes are
mostly between 250 and 557 Hz with two higher peaks near 1.94 and 4.44 kHz; the
result must not be generalized into a material identity or a complete audible
object model.

## Spatial result

The spatial decision is `NotEvaluableSingleListenerRowPerObject`: one observed
row per object versus a minimum of two. Published listener-grid identity says
where a row belongs; it is not a substitute for multiple measured responses.
No spatial-participation score or credit is produced.

## Reproduction

The manifests and results remain external under:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-transfer-calibration-v1/`
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-transfer-calibration-v2/`

Each run used:

```text
cargo run -p xtask -- physical-sound-registry transfer-calibration \
  --manifest <external-manifest.json> \
  --output <new-empty-external-directory>
```

Repository tests cover exact V1/V2 splits and prohibitions, calibration-only
candidate selection, the conjunctive holdout gate, fresh-boundary access,
deterministic synthetic DSP behavior, non-finite input rejection and
one-to-one matching. Re-running V1 with the V2-capable implementation preserves
the original V1 report hash exactly.

## Decision and smallest next action

Keep V1 as immutable negative evidence and V2 as modal/damping-only positive
evidence. Neither revision changes the PS-2 exact-domain matrix or authorizes a
validator release.

The next bounded package is a multi-listener REALIMPACT acquisition pilot:

1. extend the typed row adapter to hash-close at least two distinct published
   listener rows for one already-development object;
2. prove their exact impact, listener and row identities from the source
   metadata before interpreting array order;
3. preregister an injective cross-listener participation discriminator before
   opening its calibration/holdout rows;
4. retain `NotEvaluable` unless object-disjoint multi-listener calibration and
   holdout evidence exists; one-object success is only an acquisition/control
   result.

If the published row identities cannot be proved, preserve the current
modal/damping evidence and leave spatial participation unavailable rather than
inventing metadata or requesting local capture.
