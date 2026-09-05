# ADR-120: Stiffness-proportional damping diagnostic

| Field | Value |
| --- | --- |
| ID | ADR-120 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), ADR-069/115/116/117/118/119 |
| Supersedes | ADR-118 exact-V8-only four-sole admission and ADR-119 exact-V8-only consumer admission through explicit V9 constructors; ADR-119 harness mode restriction for a separately labeled diagnostic startup input; all predecessor profiles/no-ramp outputs remain frozen |
| Superseded by | none |

## Decision and scope

Implement opt-in BodySchema V9, `nextengine.body.humanoid-biomechanics-raja-1700.v9`,
revision9. Clone V8 and replace every actuator damping with
`round_ties_even(stiffness_q16 * 4831 / 1789440)`. The ratio has units seconds.
Compute the product in u128 and checked-convert the quotient to u64. Preserve
all stiffness, masses, COM, inertia, anatomy, limits, contact materials and
exclusions. Source identity domain:
`nextengine.source.raja-1700.stiffness-proportional-damping.v9`.
The [bounded research contract](../../development/r8b-body-sampled-damping-candidate-2026-09-05.md)
defines its finite test budget; this is not a selected robust balance solution.

CompiledV3 additionally admits four soles only for the complete canonical V9
factory, not arbitrary renamed or modified articulated bodies. CompiledV4's
force scheduling is unchanged and its complete payload identifies the candidate.

Keep old contact/standing/terminal constructors restricted to V8. Add explicit
`new_sampled_damping` constructors to the same implementation types, validating
the entire canonical V9 compilation, never only its editable hash. Reuse the
unchanged algorithms, with no public arbitrary-body bypass or gain setter.

V9 contact profile is SHA256 of
`nextengine.articulated-foot-contact.v2\0body=v9\0` followed by the32 raw bytes
of the existing articulated-foot profile hash. All anatomical6 Ns and per-pair
limits, order/aggregation, failure atomicity and continuity rules remain.
Contact/terminal roots retain full compiled-body and subject binding.

Standing profile `nextengine.motor.procedural-standing-reference.v4` retains
all V3 equations and neutral MTP targets. State root uses domain
`nextengine.humanoid-procedural-standing.v4\0`, profile ID, existing V2 inner
state root instantiated against V9, and V9 contact-profile bytes, in that order.
No change to existing V3 state roots. The descriptor is native diagnostic-only;
its inspection compiled hash remains V3, while native traces identify V4.

The standing harness may additionally accept a final `startup-ramp` argument
only for exact V8/articulated-v3 or V9/sampled-v4 with unchanged actuators,
per-iteration forces and zero offsets. Multiply the complete base reference
by min(tick,60)/60 using signed ties-to-even before production safety. Emit
`diagnostic_reference_input` with `STANDING-STARTUP-01.v1`, duration/rule and
diagnostic-only admission. The ordinary reference identity still describes
the underlying controller, not this additional input. No new selected runtime
reference is implied. Preserve all no-ramp outputs and both failed outcomes.

## Verification and rollback

Require exact damping/identity-only body delta, rounded-vector checks, unchanged
geometry and predecessor hashes; reject tampered, renamed and wrong-version
inputs in compiler/consumers. Check identical reference equations, distinct
subject/reset/profile roots, contact boundaries and wrong-profile rejection.
Run native motor/example tests and Clippy/format, boundary/content/play/replay,
nominal30-second standing, six toe-servo worlds and both bilateral loaded-transfer
inputs with original criteria. Preserve failures and byte-exact V8 controls.

One candidate only. Local matrix poles inside the unit circle do not establish
native contact/clipping/reference-feedback stability. No optimizer, old-weight
reuse, training generation, mirror or game default changes. Rollback selects
unchanged V8; a failed V9 remains a diagnostic, not a weakened safety profile.
