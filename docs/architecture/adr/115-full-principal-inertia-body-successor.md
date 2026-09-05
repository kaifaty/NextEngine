# ADR-115: Full principal-inertia body successor

| Field | Value |
|---|---|
| ID | ADR-115 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Normative dependencies | [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-069](069-biomechanics-body-schema-v2-and-solver-projection.md), [ADR-114](114-anatomical-axes-and-sagittal-body-proxies.md) |
| Supersedes | ADR-114's retained diagonal solver approximation only for BodySchema V6; legacy exact-norm validation only for the principal mass frame of PhysicsBodyDescriptorV3. Old body/environment bytes and ordinary V1 pose validation remain unchanged. |
| Superseded by | none |

## Decision

Add opt-in `nextengine.body.humanoid-biomechanics-raja-1700.v6`, revision 6,
with solver projection `humanoid-biomechanics-raja-1700.v3`. Preserve V5's
anatomy, geometry, mass, COM, authoritative tensors, joints, actuators, feet,
material and safety. Only the six non-diagonal solver tensors change:
bilateral shank/knee, foot and forearm/hand.

Author diagonal principal moments and canonical xyzw Q1.30 eigenframes so
`R diag(J) Rᵀ` reconstructs each original tensor. Round moments to integer
micro kg m² and validate a component-wise error at most 1 unit. The frozen
table lives in `crates/motor/src/biomechanics.rs`; runtime performs no
eigendecomposition and no automatic mass/inertia approximation. Old factories
remain frozen; schema, source, solver projection and compiled hashes separate
the new dynamics from V5 and older policies.

Body V6's squared quaternion-norm tolerance is `2^31` in Q2.60, used for
encoding these six frozen frames. This is not a joint/contact tolerance.
The full BodySchema validation still checks source tensor reconstruction and
mass projection. Float32 compiler-output reconstruction must also remain
within 1 micro kg m² per component and all principal moments must be positive
and satisfy the physical triangle inequality.

## Principal-frame descriptor boundary

`PhysicsBodyDescriptorV3.solver_principal_frame` stores the same pose fields
but has dedicated validation: translation is zero, the first nonzero
quaternion component is positive, and the exact integer squared-norm error
is at most `2^31`. All-zero, off-band and noncanonical frames fail before a
compiled result is returned. Do not call legacy `PhysicsPoseV1.validate()`
on this one mass-frame field: exact norm equality prevents almost all rounded
eigenframes, contrary to the quantized mass-frame purpose in ADR-069.

Ordinary initial/shape poses keep the exact V1 rule. No backend implicitly
normalizes an invalid authored frame. Existing V2–V5 bodies use identity mass
frames and retain exact descriptor bytes; V5 inspection's hash is pinned by
a regression test. No serialized field or FFI layout changes in this repair.

PhysX's diagonal inertia is expressed in mass space. Its existing adapter
already calls `setCMassLocalPose` and `setMassSpaceInertiaTensor`; retain that
path and fix the compiler/validation mismatch instead of discarding products
of inertia. The pinned 5.9.0 `PxRigidBody.h` explicitly describes this use.

## Verification and scope

Check the exact six-body projection-only delta, integer and independent
float32 reconstruction, malformed-frame rejection, unchanged legacy pose
validation, native anatomical motion and unchanged neutral geometry. Run the
native motor suite, host-check and applicable content/play/replay checks.

The opt-in 30-second `probe_biomechanics_body_standing` uses the unchanged
procedural controller and original safety on V5/V6; it records each observed
motor sample and stops on safety, never padding a missing horizon. Its timeout
is a nominal procedural control result, not learned posture, robustness or a
training/mirror admission. No existing environment silently selects V6.

Foot mechanics, upright balance and a compatible separately identified
training environment remain required work. Roll back by retiring V6, not by
rewriting its identity or relabeling old checkpoints.
