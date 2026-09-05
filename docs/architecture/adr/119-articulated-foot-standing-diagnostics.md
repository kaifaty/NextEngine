# ADR-119: Articulated-foot standing diagnostics

| Field | Value |
| --- | --- |
| ID | ADR-119 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-069](069-biomechanics-body-schema-v2-and-solver-projection.md), [ADR-117](117-quiet-upright-body-and-standing-reference.md), [ADR-118](118-articulated-volumetric-foot-body.md) |
| Supersedes | ADR-118's exclusion of contact/standing consumers only for the exact new diagnostics below. No training environment, learned weights, mirror or runtime admission. |
| Superseded by | none |

## Decision

Permit bounded native standing diagnostics on the exact canonical V8 body /
CompiledBodySchemaV4. Three consumers validate the complete supplied compilation
against the canonical factory, not just its hash label. Old constructors and
profiles retain their old behavior and roots.

### Anatomical-foot contacts

`BiomechanicsContactClassifierV2` retains V1 per-actor-pair classification,
limits and raw endpoint validation. The exact left and right ankle-roll/MTP
actors form two anatomical feet. For each substep sum all their ground-contact
impulses in the common canonical ground-first orientation, **before** active
contact filtering. Use checked i128 component sums and checked u128 squared
norms. A norm strictly above 6,000,000 micro N s marks the active contacts of
that foot as hard-impact violations. Equality is allowed. Preserve individual
segment limits even if vectors of different segments partially cancel.
Left/right feet are never combined. Self-contact and non-foot limits retain V1.

Track anatomical support continuity separately from V1 pair counters: a foot
is active if either segment meets V1's penetration/impulse activity condition.
Rear-to-toe transfer retains that counter; one empty substep clears it. Retain
pair continuity and per-pair class/material decisions. `SoleSupport` remains
a foot contact-role label, not a proof of flat sole orientation; geometry and
loaded contact locations must be checked separately in the native trial.

The profile hash is SHA-256 of the explicit implementation profile string
`nextengine.articulated-foot-contact.v1` and its frozen rules. New frame and
continuity roots bind that profile, full compiled hash and subject ID; frame
roots include unfiltered anatomical impulse sums, and continuity roots include
both foot counters. Reset clears both legacy continuity and new foot state.
Any invalid endpoint, checked overflow or classification failure leaves state
unchanged. These hashes are diagnostic identities, not training manifests.

### Terminal and reference consumers

`BiomechanicsTerminalEvaluator::new_articulated` accepts only the new contact
profile. It preserves the four-substep rule, impact/fall/joint-safety priorities,
limits and timeout; legacy/new profile mixing fails without latching. Its
state root additionally binds the compiled body and subject. Legacy constructor
state/decision bytes remain unchanged.

`BiomechanicsProceduralStandingControllerV3`, profile
`nextengine.motor.procedural-standing-reference.v3`, applies the existing V2
upright law to V8's 25 ordered channels. Both new MTP targets are neutral zero;
all existing knee/ankle/hip equations and target-slew/effort safety remain.
Its state root binds the new reference ID, full compiled body, subject, reset
anchor and new contact profile. It does not accept V7 or tampered inputs.
This transfers a reference law for measurement, not a claim of V8 stability.

Extend the existing native standing example only with a strict V8 mode:
`8 0 0 articulated-v3 per-iteration unchanged`, no offsets or response mode.
Keep old mode outputs exact. Record raw contacts/joints and anatomical-foot
impulses/counters. Stop on existing safety/terminal failure or 1,800 motor ticks.
Do not start an optimizer or silently relax safety after a failed diagnostic.

## Checks and limits

Test the inclusive aggregate ceiling, separate sides, inactive-pair contribution,
retained segment limits under cancellation, callback reversal/order/manifold
partition, transfer/gap/reset continuity, malformed/overflow atomicity, terminal
profile rejection and aggregate impact disposition, exact body/subject/reset
binding and neutral toe targets. Run native motor and example tests, Clippy,
format, boundary/content and unchanged V7 diagnostic correspondence. Record
the bounded V8 standing result, including failures, before posture claims.

Loaded heel rise, re-contact, disturbance recovery and compatible learning
remain open regardless of a nominal timeout. Neither contact labels nor
successful compilation establish these. Rollback selects the unchanged V7/V2
control and retains the V8 diagnostic as evidence; no profile is rewritten.
