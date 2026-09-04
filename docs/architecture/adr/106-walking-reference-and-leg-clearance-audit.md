# ADR-106: Walking reference and leg-clearance audit

| Field | Value |
|---|---|
| ID | ADR-106 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-04 |
| Last verified | 2026-09-04 |
| Normative dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-102](102-biomechanics-neutral-self-clearance-successor.md), [ADR-105](105-r8b-dense-tracking-walking-counterfactual.md) |
| Supersedes | ADR-102's body selection only for the walking V5 diagnostic; ADR-105's action/reference selection only for the V4/V5 diagnostics. Historical profiles, standing and promotion gates are unchanged. |
| Superseded by | none |

## Context

The failed walking V3 policy never leaves double support. Its standing
reference also adds absolute forward displacement to ankle pitch, progressively
consuming the residual range over the commanded path. Removing this position
anchor is necessary for a translation-invariant walking action.

Increasing residual range on the old body exposes self-contacts between the
leg collision proxies. The neutral thigh gap is only `4,520 um`; the shank gap
is `39,020 um`. Both are below the pinned pair contact distance of `40,000 um`.
The [bounded investigation](../../development/r8b-walking-action-basis-2026-09-04.md)
records positive single-support controls after a separately identified proxy
revision. This is evidence about collision geometry, not a claim that the
anatomical joints were spherical or incapable of stepping.

## Decision

Add two diagnostic environment/descriptor successors:

- Walking V4 retains BodySchema V3 and the original residual scales. Its
  procedural reference removes only absolute forward-position feedback;
  pitch, pitch-rate, forward-velocity feedback and all safety remain active.
- Walking V5 uses BodySchema
  `nextengine.body.humanoid-biomechanics-raja-1700.v4`, revision 4, the same
  translation-invariant reference and a `262,144` Q16 residual multiplier
  (`4x`). The multiplier applies after the original rounded residual and
  before the unchanged soft-ROM, skill and slew intersection. Its accepted
  range is strictly positive and at most `4x`; invalid inputs fail atomically.

BodySchema V4 changes only six collision proxies: both thigh boxes have
`55,000 um` lateral half-width, both shank boxes have `45,000 um` transverse
half-extents, and both knee spheres have `50,000 um` radius. Neutral lateral
gaps become `44,520`, `59,020` and `49,020 um`, respectively. Anatomical joint
axes/limits, link poses, mass, CoM, inertia, materials, collision exclusions,
actuators and effectors remain V3-exact. V3 standing and older profiles retain
their original bytes. The new body and action meanings receive distinct
body, descriptor, environment, action, reference and correspondence hashes.

The V5 training recipe is Proposed and has no inherited standing checkpoint:
V3-body standing weights do not have the same action/body meaning. Neither
recipe is activated by this ADR and neither grants an optimizer budget.
Commands and the eleven V3 reward components remain unchanged, so any later
gait-credit change must be isolated in another environment identity.

## Evidence boundary

`WALKING-ACTION-REACHABILITY-P0` is a small diagnostic: zero, left-swing and
right-swing episodes each have 105 motor ticks. It requires no terminal,
continuous single support for at least eight ticks on each side and positive
forward displacement of at least `0.01 m`. The exact Q1.30 tape is reversible
through the float32 Isaac action input. It does not prove a full alternating
gait, improved commanded tracking, standing on the revised body, or
MODEL-MIRROR-P1. Forward displacement alone is weak evidence because the zero
control also drifts forward.

Stop recording at the first terminal commit; preserve that commit and the
observed horizon. Do not record an automatic reset as the terminal trajectory,
pad missing samples into success, or label an incomplete horizon passed.

Canonical CPU passes the bilateral reachability diagnostic. Isaac GPU fails
on joint safety at tick 96, and Isaac CPU fails at tick 105. A report-only
canonical-damping counterfactual does not close the discrepancy. These results
keep the original paired alternation/correspondence and optimizer gates open.
No safety tolerance, contact distance or canonical solver setting is relaxed.

## Verification and rollback

Verify V3 body preservation, V4 proxy-only differences, walking translation
invariance, multiplier rejection/safety, descriptor selection and CPU/Torch
reward goldens. Run the native motor tests and relevant host/product checks.
Retain external diagnostic inputs and failed outputs. Revert by retiring the
V4/V5 recipes as diagnostics; never relabel their artifacts as standing V3 or
claim that they repair the existing MODEL-MIRROR-P1 failure.
