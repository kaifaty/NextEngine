# Physical sound PS-2 — complete acquisition bundle contract

Date: 2026-08-27
Status: `E1_IMPORT_CONTRACT_READY / LOCAL_CAPTURE_RETIRED / PASS_DISABLED`

## Question

Can the external inventory distinguish a complete published raw impact bundle
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
AV-P0D. It only proves that a future published complete bundle can be accepted or
rejected without modifying code or inventing missing axes.

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

The software boundary is retained as the highest `E1 synchronized response`
import contract, but the [active internet corpus policy](physical-sound-internet-corpus-policy-ps2-2026-08-27.md)
retires local physical capture. The host's microphone inputs and lack of a force
channel are no longer relevant blockers, and the user is not expected to strike
or record an object.

The next implementation must discover and fetch already published sources into
an external cache, audit their actual capabilities and adapt complete `E1`
bundles when available. `E2` transfer responses and `E3` identified recordings
remain useful for their narrower claims rather than being upgraded to this
contract. Acquisition/provenance failures create a new source revision or
claim-scoped fallback; they never weaken the contract or trigger local capture.

The pre-registered 35 reject-parent and 16 coverage-group statistical minima,
sealed shadow and fallback policy remain unchanged. A new multi-source plan must
show internet evidence coverage before PS-3 can begin.
