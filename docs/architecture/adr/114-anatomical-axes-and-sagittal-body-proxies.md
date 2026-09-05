# ADR-114: Anatomical axes and sagittal body proxies

| Field | Value |
|---|---|
| ID | ADR-114 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Normative dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-069](069-biomechanics-body-schema-v2-and-solver-projection.md), [ADR-102](102-biomechanics-neutral-self-clearance-successor.md), [ADR-106](106-walking-reference-and-leg-clearance-audit.md) |
| Supersedes | ADR-069 anatomical-axis signs and ADR-106 sagittal proxy locations only in the new BodySchema V5 diagnostic. Existing profiles remain frozen. |
| Superseded by | none |

## Observation

In engine coordinates (+X right, +Y up, +Z forward), positive +X rotation
of a hanging limb (-Y) moves its endpoint backwards (-Z). V4 applies this
axis to hip, shoulder and elbow flexion as well as knee flexion. Its positive
hip-adduction axes move the thighs outwards. Limits are asymmetric, so this
is a physical range-of-motion defect, not merely a display label.

Native creation and state import reproduce the signs on both sides; a
separate target-FK check agrees. This does not prove the defect caused the
known walking candidate's trunk lean.

The source lumbar anchor is posterior to the pelvis frame origin. It is not
a compiler translation error. However, the pelvis collision box is 40.7 mm
anterior to its own sagittal COM, while the torso box is 28.556 mm posterior
to the source rib-cage sagittal bounds midpoint. Comparing box centres makes
the gap appear 90.7 mm, larger than justified by those chosen proxy centres.

## Decision

Add `nextengine.body.humanoid-biomechanics-raja-1700.v5`, revision 5:

- Reverse both hip-flexion, shoulder-flexion, elbow-flexion and hip-adduction
  axes: eight joints in total. Knee flexion remains +X; shoulder abduction,
  lumbar, ankle and axial-rotation conventions remain unchanged.
- Pelvis collider local Z becomes `-70,700 um`, its source sagittal COM.
- Torso collider local Z becomes `8,556 um`, midpoint of source rib bounds
  `[-87,827, 104,939] um`. This is a coarse collision proxy, not a skin mesh
  or exact soft-tissue envelope. Keep all dimensions and vertical offsets.
- Preserve link frames, anatomical joints' anchors, all limits, actuators,
  masses, COM, tensors, solver projection, feet, materials, effectors and
  collision exclusions. Neutral collider-centre separation becomes 21.444 mm.

The source geometry is `Geometry/hat_ribs_scap.vtp` from opensim-models commit
`a5c5f6ce9b904e618eeaf203e6efa48d457f9b41`, SHA-256
`468ff55422359dbc27d8f797faf2fb1a367e97532b089d66e25719deed695fd7`.
Only derived numeric bounds are stored here; raw geometry stays external.
The source model and numeric provenance remain in the
[original profile](../../plans/2026-08-12-humanoid-biomechanics-profile-v1.md).

V5 is an opt-in native-body diagnostic, not an automatic environment upgrade.
Its schema/source hashes differ, and the compiled hash includes the changed
axes and geometry. V2/V3/V4 factories and all existing environments remain
byte-identical. Old checkpoints must not be relabeled as compatible V5
policies. Training requires a separately identified environment and compatible
controller/reference signs; no mirror or runtime promotion is implied.

## Verification and remaining work

Focused tests must check the exact eight-axis/two-proxy delta, all unchanged
fields, bilateral positive flexion/adduction through production native state
import, preserved knee direction, and native creation/substep. Descriptor
inspection must not advertise training admission.

These are geometry and kinematic checks, not standing/walking quality. Full
principal inertia, dynamic support, foot mechanics and learned torso control
remain open. Retire V5 to roll back; never rewrite old identities.
