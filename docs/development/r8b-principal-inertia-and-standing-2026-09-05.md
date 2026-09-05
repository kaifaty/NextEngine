# Body V6: principal inertia and nominal standing

## Result and scope

Implemented under [ADR-115](../architecture/adr/115-full-principal-inertia-body-successor.md).
V6 keeps the V5 human anatomical axes and collider placement but replaces the
six non-diagonal solver approximations with full principal-moment/frame pairs.
No mass, anatomical COM, source tensor, segment length, collider, foot, joint,
actuator or contact rule changes. This is standard diagonalization, not a new
mathematical model or proof campaign.

The production integer validator and an independent float32-payload test
reconstruct each source tensor within 1 micro kg m² per component. The second
test applies quaternion-vector cross products to basis vectors rather than
calling the production integer rotation/reconstruction helper. It checks all
24 bodies and positive, physically realizable principal moments. Negative
controls remove the mass rotation, zero the quaternion, move it outside its
norm band, and substitute it into an ordinary exact-norm initial pose.

## Boundary defect fixed

The first V6 compile failed with `Physics(InvalidRotation)` despite passing
BodySchema validation. `physics_pose()` and the V3 physics descriptor applied
V1 exact squared-norm equality to a principal mass frame. This contradicts the
quantized eigenframe support intended in ADR-069. ADR-115 isolates the fix to
that field: zero translation, canonical sign, and Q2.60 norm deviation at most
`2^31`. Ordinary initial and collider poses retain their old validator.

No backend normalization or widened ROM/contact bounds were introduced.
The existing FFI already forwards the authored COM and principal rotation to
`setCMassLocalPose`, and principal moments to `setMassSpaceInertiaTensor`.
The pinned PhysX 5.9.0 `include/PxRigidBody.h`, lines 297–318, requires this
representation for a non-diagonal actor-space tensor; lines 227–245 state that
changing the mass pose does not move the actor itself.

Offline authoring used NumPy 1.26.4 `linalg.eigh` on the source symmetric 3x3
tensor in micro kg m², columns as eigenvectors, determinant corrected to +1,
ascending rounded eigenvalues, and existing engine xyzw Q30 encoding.
The six frozen rows are retained in the V6 body factory. Their largest exact
Q2.60 norm error is `1,225,562,393`, below `2^31`. Runtime does not eigensolve.

## Native standing discriminator

The opt-in `probe_biomechanics_body_standing` creates V5 and V6 with the
production material-complete compiler and CPU PhysX. It uses the existing
procedural standing reference, zero residual and unmodified safety, contact
classification and terminal evaluation. The reference commands only knees
and ankle pitch; those axes did not change in V5. Thus this neutral procedural
control can be reused, unlike an old learned policy with nonzero hip/arm
actions. No controller or reward tuning was performed in this discriminator.

Both bodies reach the requested 1,800 motor ticks / 7,200 physics substeps,
ending only in `terminal.timeout`. Each observed motor sample is recorded;
there is no automatic reset or padded completion. Sampled orientation results:

| Body | Final pelvis tilt from vertical | Final torso tilt from vertical |
|---|---:|---:|
| V5, old diagonal approximation | 9.1433° | 7.3650° |
| V6, full principal inertia | 9.6163° | 8.0776° |

The change does **not** improve this controller's upright posture. It fixes
the intended physical inertia representation, not the leaning symptom.
Next work should address neutral reference / balance and foot mechanics;
do not keep tuning inertial mass to make this old controller look upright.
The probe is nominal procedural evidence, not learned-policy quality,
disturbance robustness, loaded-flat-foot certification or mirror admission.

## Exact artifacts and reproduction

Directory:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/body-v6-01`.

| Artifact | SHA-256 |
|---|---|
| `descriptor.json` | `f4a7e38b6a8238a7a7b920c27624676262f469768833632b5fb88aa8e8275943` |
| `standing-v5.json` | `5c3d10076cfa35c5a7f564e9c25a0763944bbd20ddfde9a00d336d65783858b0` |
| `standing-v6.json` | `a56df498497c16e94bd5d3ef253c709d874a5c960c7604dd7834d625d0ffb33f` |

Body V6 hash:
`ceff5a55c79e84563eced3917c87dfa9ef0dd1e4b9d3a647caeadb0bf924ba01`.
Use the CPU SDK directory recorded in the
[task state](task-state/r8b-body-proportions-and-foot.md), then:

```sh
cargo run -p next_motor --features physx-sdk --example export_biomechanics_body_v6
cargo run -p next_motor --features physx-sdk --example probe_biomechanics_body_standing -- 5
cargo run -p next_motor --features physx-sdk --example probe_biomechanics_body_standing -- 6
```

Raw outputs stay external. Old V5 descriptor hash is pinned by regression;
all existing training environments retain old body selection and checkpoints.
No optimizer was started. Full user goal remains open: feet, upright dynamic
balance and a new compatible standing/walking learning environment remain.

## Checks

- PASS: 132 native `next_motor` library tests, including the new tensor and
  negative-boundary tests, frozen V5 descriptor hash and existing regressions.
- PASS: both 30-second native standing probes; this is the bounded control
  result above, not upright quality.
- PASS: `play`, `persistence-replay`, `content-package`, formatting, diff
  whitespace and local ADR/report/task-state links.
- PASS: the previously running full `host-check` completed with overall PASS
  on implementation `ba84b9a0` (Linux x86_64, Rust 1.97.1).
- NOT_RUN: separate performance benchmark and training/mirror evaluation.
