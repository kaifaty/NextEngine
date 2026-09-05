# ADR-117: Quiet upright body and standing reference

| Field | Value |
| --- | --- |
| ID | ADR-117 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-115](115-full-principal-inertia-body-successor.md), [ADR-116](116-explicit-per-iteration-force-scheduling.md), [ADR-059](059-event-sourced-physx-continuation-reconstruction.md) |
| Supersedes | ADR-115's retained V6 actuator gains only for explicitly selected BodySchema V7; V1 procedural standing remains unchanged and available. No existing environment, training or runtime route is switched. |
| Superseded by | none |

## Decision and evidence

The [coupled damping](../../development/r8b-coupled-damping-discriminator-2026-09-05.md)
and [hip rate-term ablation](../../development/r8b-hip-rate-feedback-discriminator-2026-09-05.md)
independently reproduce quiet nominal upright standing without mass changes.
Retain the demonstrated settings as opt-in reusable motor implementation,
not private example-only edits or new meanings under old hashes.

`nextengine.body.humanoid-biomechanics-raja-1700.v7`, revision7, derives from
V6. Bilateral shoulder-yaw stiffness and damping are divided by16. Damping
alone is divided by4 on bilateral hip-pitch, hip-yaw and knee, torso-pitch and
torso-yaw (eight channels). All values use exact integer Q16 division. Keep
all other fields except schema ID/revision/source provenance unchanged:
geometry, joint axes/anchors/limits, source and principal inertia, mass/COM,
collision exclusions, material, effort/power/rate limits and safety.
Source domain is `nextengine.source.raja-1700.sampled-standing-actuators.v7`.
The new schema and compiled descriptor hashes identify the changed dynamics.

Add `BiomechanicsProceduralStandingControllerV2` with profile
`nextengine.motor.procedural-standing-reference.v2`. It accepts only a complete
canonical V7 `CompiledBodySchemaV4` (ADR-116 force schedule). Validate the full
compilation against the canonical factory at construction, not only an editable
hash label. This reset-time check is outside the substep loop. Keep V1's exact
knee/ankle/neutral law and root-Z reset anchoring. Add to both hip-pitch targets
`-2 * trunc(2_000_000 * root.qx / 2^30)` microradians at the motor tick. This is
the frozen small-angle quaternion proxy, not Euler pitch. Do not add the old
direct hip `omega_x / 5` term. V1 ankle rate feedback and ordinary joint PD
remain. Preserve V1's ties-to-even arithmetic; hip division truncates towards
zero exactly as in the native discriminator.

V2 state identity binds its domain/profile, complete compiled outer hash,
subject PersistentId and V1 reset state root. Compiled body hashes describe
the reusable physical profile and are subject-independent, so the subject
must be bound separately. Targets still enter production safety before every
240-Hz effort application; contact and fall criteria are not relaxed. The
reference is stateless between ticks except for immutable reset anchoring.
No FFI, native scene, gameplay command, persistence format or ABI changes.

## Verification and boundaries

Require exact V7-versus-reviewed-candidate fields apart from explicit source
identity, new body/compiled hashes, signed integer reference vectors and no
hip rate dependence, reset/subject state identity, rejection of wrong body
and tampered compilation, preserved old factories/modes, full7200-step native
target/effort/pose/contact equality to the candidate and normal safety timeout.
Run native motor and example tests, format/Clippy, host-check and applicable
play/replay/content checks; export the V7 inspection descriptor.

This accepts reusable body/control code only. It does not select a training
environment, admit old weights or an Isaac mirror, or promote a gameplay
runtime policy. The nominal test does not establish disturbance recovery,
walking, forefoot articulation or global stability. Those require distinct
bounded evidence and compatible consumers. Rollback means selecting the
preserved V6/V1 profile, never changing V7/V2 semantics in place.
