# ADR-125: Support-effort calibration diagnostic

| Field | Value |
| --- | --- |
| ID | ADR-125 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-06 |
| Dependencies | SPEC-27/35, ADR-069/119/122/124 |
| Supersedes | ADR-124 V11 diagnostic input restriction and SPEC-35 PD-only diagnostic effort, additively for the exact support profile below |
| Superseded by | none |

Allow one exact V11 diagnostic safety constructor `new_bandwidth_support`.
It validates complete canonical CompiledV4 identity and binds that outer hash.
Only this constructor admits `step_substep_with_support_effort`: one signed
i64 microNm value per ordered actuator, added with checked i128 arithmetic to
the separately rounded P-D request BEFORE the existing complete effort/rate/
power/positive-work intersection. Existing target slew, soft/hard ROM,
velocity, contact and terminal rules remain unchanged. No effort is added
after safety. Wrong profile/count/state rejects without partial mutation.
Old `step_substep` remains the zero-support path with identical old roots.

New checkpoint domain `nextengine.humanoid-support-safety-checkpoint.v1\0`
followed by CompiledV4 hash and the unchanged old checkpoint payload after its
domain prefix. The supplied support vector is an explicit per-substep input,
not hidden controller state. Reset preserves profile identity. No new public
contract record, checkpoint import, learned action or runtime consumer is added.

The standing example admits strict `11 0 0 support-v1 per-iteration unchanged`
and `support-zero-v1` for zero-input correspondence. Reference remains exact
standingV6; a distinct composite diagnostic ID plus support safety root marks
the different applied law. Old modes remain unchanged. The f64 support helper
is diagnostic-only: from canonical60Hz poses and exact V11 body descriptors,
compute gravity-minus-projected modeled normal support, quantize once to
microNm ties-even and hold four substeps. It never writes root force or pose.
It is not the fixed-point runtime fallback or a contact-force measurement.

The [frozen experiment](../../development/r8b-support-effort-calibration-2026-09-06.md)
defines support geometry, arithmetic, no-support handling, controls and gate.
Success does not select V11, pass full calibration or admit training. Preserve
the passive joint friction and all old native profiles.

Checks: focused native motor/example tests, all-target Clippy/format,
boundary/content/play/replay, exact old V8/V11 native controls, zero-support
trajectory correspondence, candidate repeat and independent executable review.
Only existing workspace serde is added as a motor dev-dependency for diagnostic
JSON; no workspace configuration/public-schema/native-physics change. Broad host and
performance not required for this localized diagnostic addition.
