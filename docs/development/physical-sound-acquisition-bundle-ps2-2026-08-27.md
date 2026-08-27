# Physical sound PS-2 — complete acquisition bundle contract

Date: 2026-08-27
Status: `ACQUISITION_BUNDLE_READY / PHYSICAL_CAPTURE_NEXT / PASS_DISABLED`

## Question

Can the external inventory distinguish a complete newly recorded impact bundle
from a force-deconvolved public pilot without changing historical pilot bytes or
allowing missing metrology to become implied metadata?

## Implemented contract

The current-only `physical-sound-registry corpus-inventory` format now supports
`controlled_real_raw_synchronized_impact`. Such an entry is valid only when it
has zero unavailable components and supplies all of:

- mono little-endian float32 raw microphone samples;
- mono little-endian float32 force samples in newtons;
- identical microphone/force sample rate and sample count;
- an explicit repeat ID unique within object, impact position and listener
  condition;
- material-composition evidence;
- support-fixture revision evidence;
- microphone calibration evidence;
- force-transducer calibration evidence;
- the existing exact geometry, support, excitation, impact, listener,
  acquisition-metadata and provenance identities.

Every referenced file remains external and is hash-closed. The inventory reads
both signal arrays, verifies their exact byte counts and rejects non-finite
samples. A complete force trace must contain a positive impact. The report
publishes peak force, RMS force and the positive force integral in
newton-seconds; these are diagnostic acquisition facts, not source-model tuning
parameters.

The state machine is intentionally asymmetric:

- a force-deconvolved public pilot must declare unavailable components and may
  not attach a complete-acquisition block;
- a raw synchronized entry must attach the complete block and may not declare
  unavailable components;
- only the latter receives provisional `ResearchEligible` inventory status;
- the report itself remains `DevelopmentPilotOnly` with
  `NO_CORPUS_ADMISSION_AUTHORITY`.

This does not open calibration, holdout, shadow, registry `Pass`, PS-3 or
AV-P0D. It only proves that the next physical capture can be accepted or rejected
without modifying code or inventing missing axes.

## Controls

Focused tests cover:

- exact complete synchronized microphone/force dimensions;
- deterministic peak/RMS/positive-impulse measurement;
- missing/mismatched force dimensions;
- duplicate repeat identity;
- real-entry generator/mutation identity rejection;
- object/family/source partition leakage;
- wrong audio byte count and repository-local artifact rejection;
- finite/hash-closed repeatable inventory reports.

The positive fixture uses four 48 kHz samples with a `[0, 10, 20, 0] N` force
trace and measures positive impulse `0.000625 N*s`. It is a unit control, not a
recording and receives no corpus credit.

The previously frozen REALIMPACT report is a non-regression sentinel. Re-running
its unchanged manifest through the expanded implementation reproduces exactly:

`fed6245d5d5e79833e3acbca4a61cd1658f80fdc40268e47d51bd8785b06d3fa`.

Therefore the new optional block cannot retroactively upgrade or rewrite the
incomplete public pilot.

## Decision and next action

The software acquisition boundary is ready. The remaining PS-2 blocker is now
physical evidence, not another metadata/schema iteration.

Host preflight on 2026-08-27 found two ALSA capture devices on the integrated
ALC1220 codec, with rear microphone, front microphone and line inputs. No
calibrated force-transducer or instrumented-hammer channel is identified on the
host. An ordinary microphone-only recording would violate the synchronized
bundle and receives no evidence credit. Capture therefore waits for an
instrumented hammer/force sensor connected to a synchronized input path, or an
already recorded bundle with the same evidence.

Record one glass object with the frozen PS-2 fixture and instrumented hammer:

1. retain synchronized raw microphone and calibrated-newton arrays;
2. retain the object geometry measurement, material/composition record and all
   calibration/fixture revisions;
3. capture both declared impact positions and listener conditions with at least
   three explicit repeats;
4. import every row only into development and verify one deterministic report;
5. repair acquisition/provenance failures by making a new source revision, not
   by weakening the contract.

Only after that one-object bundle passes should acquisition scale to 40 objects,
four families and four sources. The pre-registered 35 reject-parent and 16
coverage-group targets, sealed shadow and fallback policy remain unchanged.
