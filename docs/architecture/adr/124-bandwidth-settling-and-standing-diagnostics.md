# ADR-124: Bandwidth settling and standing diagnostics

| Field | Value |
| --- | --- |
| ID | ADR-124 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-06 |
| Dependencies | SPEC-35, ADR-119/121/122/123 |
| Supersedes | ADR-123 input set and ADR-122 V11 standing exclusion, additively through exact new diagnostic consumers only |
| Superseded by | [ADR-125](125-support-effort-calibration-diagnostic.md) for exact support-effort diagnostic input; [ADR-126](126-corrected-body-walking-environment.md) for a separate native walking consumer |

Add explicit `abduction-tail` to the native bandwidth example: case28,
BODY-COMBINED-01.r2,200m initial elevation and300 motor ticks. Original census
and case27 remain unchanged. Input amplitude/sign/timing and full safety stay
fixed; only unloaded elevation and return observation duration differ.
The [frozen experiment](../../development/r8b-symmetric-settling-and-load-2026-09-06.md)
keeps r1's one-second-return failure distinct from the new two-second result.

After response review passes, permit exact V11 nominal standing through
explicit `new_bandwidth` contact/terminal/reference constructors. Validate
complete canonical CompiledV4 equality and subject/reset binding. Reuse the
existing articulated-foot aggregate6Ns/per-pair/continuity/terminal law and
upright reference equations without changing limits, gains, geometry or resets.

V11 contact hash: SHA256 of `nextengine.articulated-foot-contact.v4\0body=v11\0`
followed by the original32-byte articulated-foot profile hash. Standing profile
`nextengine.motor.procedural-standing-reference.v6`, state domain
`nextengine.humanoid-procedural-standing.v6\0`, then profile ID, V2 inner root
bound to V11 and new contact hash, following the existing state-root layout.
Old constructors reject V11; new ones reject other/tampered bodies.

Strict standing invocation: `11 0 0 bandwidth-v6 per-iteration unchanged`.
No offsets, startup ramp, response mode or arbitrary vectors. This admits a
diagnostic, not a calibrated selection, loaded-transfer success, training,
checkpoint reuse or mirror/runtime default. A failed standing prefix stops
later loaded trials. Preserve immutable V8 as nominal standing control.

Require native motor/example tests, admission/profile-mixing/subject/reset
checks, all-target native Clippy/format, boundary/content/play/replay, exact
old case27/standing controls, candidate repeat and independent executable
reviews of response and conditional standing boundaries. Broad host/performance
not triggered by this localized diagnostic extension; no public contract schema,
workspace dependency or runtime physics change.
