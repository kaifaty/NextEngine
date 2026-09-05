# ADR-121: Screened damping native diagnostic

| Field | Value |
| --- | --- |
| ID | ADR-121 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-06 |
| Dependencies | SPEC-35, ADR-069/115/116/117/118/119/120 |
| Supersedes | ADR-118/120 four-sole and ADR-119/120 consumer admission only through explicit exact-V10 constructors; all previous profiles remain unchanged |
| Superseded by | none |

## Decision

Add opt-in `nextengine.body.humanoid-biomechanics-raja-1700.v10`, revision10,
as a native test of the eight fixed D values from
[BODY-GAIN-02](../../development/r8b-whole-body-gain-calibration-2026-09-06.md).
Clone V8, preserving all anatomy, mass/inertia, stiffness, materials, constraints
and safety. Change only identity/provenance and the following damping Q16:

| Joint | D Q16 |
| --- | ---: |
| bilateral ankle pitch | 319360 |
| bilateral ankle roll | 118376 |
| left shoulder yaw | 26503 |
| right shoulder yaw | 26504 |
| bilateral elbow | 420501 |

These are nearest-ties-even quantizations of the fixed measured SI vector,
not runtime Gain Tuner calculations. The one-LSB shoulder difference is retained
as part of the exact candidate, not silently symmetrized. Mirrored action-value
rules remain unchanged. Provenance domain:
`nextengine.source.raja-1700.screened-eight-damping.v10`.

CompiledV3 four-sole admission requires equality to the complete V10 factory.
CompiledV4 retains its force schedule. Add `new_screened_damping` constructors
to existing articulated contact/terminal/standing implementations. Each validates
the entire canonical compilation and subject/reset binding. No generic gain
setter, arbitrary-body bypass, new safety tolerance or native ABI change.

V10 contact identity hashes `nextengine.articulated-foot-contact.v3\0body=v10\0`
followed by32 raw bytes of the original articulated-foot profile hash. Retain
every aggregate6 Ns/per-pair limit, continuity and failure rule. Standing profile
`nextengine.motor.procedural-standing-reference.v5` retains all V3/V4 reference
equations. State root uses `nextengine.humanoid-procedural-standing.v5\0`, new
profile ID, V2 inner state root bound to V10, and new contact-profile bytes.
An internal closed profile enum may replace the old boolean without changing
V8/V9 state-root bytes or equations.

## Verification and scope

Strict standing mode: `10 0 0 screened-v5 per-iteration unchanged`; no offsets,
startup ramp or response mode. Existing toe-servo and transfer examples may
select `--body-v10`, keeping their original inputs, limits and raw evidence.
Descriptor export remains native diagnostic-only, never training/mirror input.

Require exact eight-D/identity-only delta, independent Q16 rounding, full-input
rejection, old/new profile rejection, subject/reset roots and unchanged contact
boundary tests. Run native motor/example tests, format/Clippy, boundary/content/
play/replay and exact V8 standing control. Frozen native test budget and success
criteria are in [BODY-GAIN-NATIVE-01](../../development/r8b-native-gain-calibration-2026-09-06.md).
Nominal standing failure stops promotion; do not tune another vector under
this identity. Successful standing alone does not close loaded transfer,
perturbation recovery or the full calibration goal.

This accepts a reproducible diagnostic, not a calibrated body selection.
No optimizer, checkpoint reuse, environment or runtime default changes. Rollback
selects immutable V8; implicit-drive Isaac success is not explicit-PD evidence.
