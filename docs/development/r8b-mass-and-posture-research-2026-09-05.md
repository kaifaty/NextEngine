# R8b mass and posture investigation — contract revision 1

Status: `MASS_AUDIT_SUPPORTED_BOUNDED / PHYSICAL_SUCCESSOR_OPEN`; current full goal is a more human-like physical body
with mass-placement and leaning problems corrected. Diagnostic/renderer
completion is not completion of that goal.

## Bounded question and definitions

Engineering consumer: decide whether the next physical/control successor
needs altered source masses, a different serial-carrier projection, explicit
trunk/COM control, or more than one of these. Body V4 and known candidate 3999
remain unchanged controls under SPEC-35/ADR-069/102/106/113.

The first numerical experiment measures all 1200 native frames of the closed
seed-1001 episode and the descriptor initial pose. It computes whole-body
COM = sum(m_i * (p_i + R_i*c_i)) / sum(m_i), in world X-right/Y-up/Z-forward
metres and kilograms. IEEE f64 diagnostic rotations normalize recorded Q30;
this is not a new canonical fixed-point physics observable.

It separately measures pelvis and torso world-up tilt (yaw invariant), and
the instantaneous change in whole-body COM if serial carrier mass is placed
at its group's physical yaw-link COM in the **same recorded pose**. This
counterfactual is not a simulated mass change and cannot assess its inertia,
control response or stability. Foot-midpoint offsets include the swing foot
and are not a support polygon or a balance-success criterion.

## Competing hypotheses and decision boundary

| Hypothesis | Discriminator | Limit |
| --- | --- | --- |
| Gross mass/group/frame error | Exact source rows, compiler/FFI mapping, weighted initial and native COM | Plausible COM does not certify anthropometry |
| Serial carriers displace COM enough to explain lean | Neutral preservation and whole-episode same-pose relocation delta | Small delta cannot rule out dynamic inertia effects |
| Controller objective permits trunk lean | Compare physical torso tilt to pelvis tilt and the exact reward inputs | Does not prove one new reward fixes the policy |

Budget: one source episode plus manufactured translation, rotation, weighted
mass and carrier-placement controls; no optimizer or simulator restart.
Finite results apply only to this identity/episode. Stop this discriminator
after measuring the whole trace; never interpret a static geometric COM test
as a theorem of dynamic stability. Independent review follows the frozen
tool/output before using the result to select a physical correction.

## Initial read-only triage

- Native compiler sends `solver_center_of_mass_micrometres` to FFI; the bridge
  applies `setCMassLocalPose`, `setMass` and `setMassSpaceInertiaTensor`.
  This is source inspection, not a new loaded-scene round-trip.
- Direct preliminary computation gives total mass 75.337 kg, neutral COM
  approximately (0, 0.93584, -0.08659) m. No whole-mass-in-legs explanation is
  evident. Exact group fractions and full-trace results remain to be audited.
- At tick 1200 pelvis tilt is 10.44 degrees while torso tilt is 29.94 degrees.
  The current walking reward facts include root rotation but not torso
  rotation. A pelvis-only upright cost therefore cannot distinguish all
  possible trunk orientations at fixed root; this is a control blind spot,
  not proof that the source torso mass is wrong.

## Reviewed mass/pose result

The frozen [audit](../../lab/scripts/audit_body_mass_balance.py) and
[five manufactured tests](../../lab/tests/test_body_mass_balance.py) were
independently reviewed and recomputed over all 1200 frames. Status:
`SUPPORTED_BOUNDED`, not a dynamic-stability proof.

| Observable | Result |
| --- | --- |
| Total mass | 75.337 kg |
| Pelvis | 11.777 kg, 15.6324% |
| Torso/head including its carriers | 26.8266 kg, 35.6088% |
| Both arms including carriers | 7.410 kg, 9.8358% |
| Both legs including carriers | 29.3234 kg, 38.9230% |
| Initial COM, world XYZ metres | (0, 0.9358392784, -0.0865885007) |
| Maximum same-pose carrier relocation COM change | 0.755550 mm, tick 1023 |
| Maximum pelvis / torso tilt | 17.623914 degrees at tick 244 / 39.400410 degrees at tick 280 |
| Final pelvis / torso tilt | 10.440285 / 29.942325 degrees |

Independent computation used scalar quaternion cross products, `math.fsum`,
`acos` tilt and a complete relocated weighted sum rather than the candidate
delta shortcut. Maximum disagreement was 2.67e-15 m / 1.96e-12 degrees.
Compiler/mirror/FFI correspondence was inspected: local COM is transformed
once by the recorded actor pose. This is not a loaded-scene mass readback.
Descriptor values precede native f32 conversion; summed f32 mass differs by
approximately -1.52e-7 kg. Carrier inertia/control effects remain untested.

Exact external evidence:

- Root: `/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05`.
- `audit-01/report.json` SHA-256
  `ca96636a33750c1f4ca49cdf573b2e90e1925e881a0baac6e34a0c1725dbebf3`.
- Audit source SHA-256
  `18ba426c117f12c867c4900cada993d751999ee618b6e2a423ed590234be0f82`;
  tests `38e12ab8a27bf437f4012680cfe54f5cf48c2cc0b2295f1cad818ac2eb98b51c`.
- Input descriptor SHA-256
  `d4e43b3e4ad0d08fc69a0327cd086d133375926de1eb4d9c296ef7987e6c560e`;
  known-candidate evaluation manifest
  `a8678933d1b934109b5c50d991e4228910ebd4c642dbe7b4c1d193cf24e1884c`.
- Entry-tool hash does not seal imported helpers. Independent recomputation
  closes this result; future reuse must retain dependency/environment identity.

Reproduction uses the pinned external Python with `PYTHONPATH=lab`:

```sh
python -m lab.scripts.audit_body_mass_balance \
  --evaluation /home/kaifaty/NextEngine-training/r8b-known-walking-candidate-v1/evaluation-01 \
  --output /absolute/fresh/external/audit-directory
python -m unittest lab.tests.test_body_mass_balance
```

## New discriminator: neutral sagittal geometry and inertia

The user's subsequent neutral side view identifies a separate real issue:
torso/pelvis offset exists before any learned action. Exact V4 authored Z
coordinates (positive forward), relative to the pelvis origin:

| Landmark | Z, mm |
| --- | ---: |
| Pelvis collider centre | -30.0 |
| Hip joint | -56.276 |
| Pelvis COM | -70.7 |
| Lumbar joint / torso body origin | -100.7 |
| Torso collider centre | -120.7 |
| Torso/head group COM | -130.7 |
| Shoulder joint | -97.545 |

The torso collider centre is therefore 90.7 mm behind the pelvis collider
centre; group COM is 60.0 mm behind pelvis COM. Neither quantity is a joint
angle or proof of instability. The previous visualization correction solved
missing geometry, **not** sagittal anthropometric validation.

The exact original source was recovered externally as
`source-01/Rajagopal2015.osim`, SHA-256
`b8a31616557f73f798898c03a9beee723ba2987e646a688375d60c4327d90bff`,
matching [the already licensed source](../plans/2026-08-12-humanoid-biomechanics-profile-v1.md).
Its `back` joint has parent location `(-0.1007, 0.0815, 0)` in source
X-forward/Y-up/Z-right, zero parent/child orientation and zero default lumbar
angles. Source torso COM is `(-0.03, 0.32, 0)`. Thus this offset is not an
accidental sign inversion in the current compiler. Source agreement alone
does not justify our proxy geometry or all-zero standing posture. The source
pelvis frame is not the centre of its collision box.

A separate exploratory tensor calculation found all 24 current authoritative
link tensors positive with positive principal-moment triangle margins.
However, current solver projection discards products of inertia. For the
foot and forearm-hand rows, preliminary maximum relative quadratic-form
errors are approximately 24.34% and 41.55%. These are NOT measured trajectory
errors or an independently reviewed new experiment. They justify checking a
source-preserving principal-axis projection before changing source masses.
[PhysX's documented mass-frame interface](https://nvidia-omniverse.github.io/PhysX/physx/5.1.3/_build/physx/latest/class_px_rigid_body.html)
requires diagonalization plus a mass-frame orientation for a non-diagonal
actor-space tensor; native 5.9 behavior still needs its own focused check.

Do not directly split the original toes into a new dynamic link using its
old diagonal inertia: source `(0.0001, 0.0002, 0.001)` kg m^2 violates a
principal-moment triangle inequality. The current merged foot tensor does
not have that violation. The 2023 source correction is a cross-check, not
permission to silently change the frozen source lineage.

## Decision and validation boundary

Do not move all mass into the torso, shorten legs from the stick view, or
claim inertia alone explains learned lean. Keep the successful V4 policy as
a control. Next physical work must address neutral sagittal geometry,
source-consistent inertia, foot mechanics and torso-aware control with
separately identified profiles and native standing/walking checks.

PASS: five focused tests, Ruff format/lint and independent bounded numerical
review. Native simulation, training and runtime ProductChecks:
`NotRun(DiagnosticOnlyNoPhysicalChange)`. The active goal remains open until
physical changes and meaningful standing/walking non-regression establish
the requested end state.

## Physical V5 implementation — axis and sagittal proxy discriminator

The diagnostic-only boundary above describes the original mass-audit commit,
not the subsequent implementation. [ADR-114](../architecture/adr/114-anatomical-axes-and-sagittal-body-proxies.md)
now records the opt-in physical successor.

Competing explanations for the neutral picture were wrong lumbar translation,
misplaced collision proxies, or learned posture alone. Exact original OSIM
lumbar translation rejects the compiler-translation explanation; neutral V4
rules out learning as the sole explanation of the box-centre gap. The original
rib geometry has source forward bounds `[-0.087827, 0.104939] m`, midpoint
`0.008556 m`. Source axes (forward, up, right) map to engine (right, up,
forward). V5 uses that midpoint for torso collider Z, and source sagittal COM
`-0.0707 m` for pelvis collider Z. It preserves the broad proxy dimensions;
bone bounds are not claimed to define the soft-tissue envelope.

The downloaded `r_pelvis.vtp` global geometry is not used as pelvis authority:
[upstream issue 171](https://github.com/opensim-org/opensim-models/issues/171)
reports differences from Rajagopal-specific pelvis geometry. Original model
mass/COM and joint coordinates remain the numeric authority. Raw geometry
stays external in `source-geometry-01`, not in the repository/distribution.

A separate kinematic check exposed the reversed limb-flexion and hip-adduction
axes. At +30 degrees with identity **xyzw** root, V4 target FK gives right
knee forward displacement `-0.20289615 m` from hip flexion, elbow
`-0.14489746 m` from shoulder flexion, palm `-0.15049038 m` from elbow flexion.
Knee flexion correctly moves the ankle backwards. An initial exploratory call
used wxyz identity and was discarded before conclusions; the corrected call
and the production native test both use engine conventions.

V5 reverses those eight bilateral anatomical axes while keeping their numeric
ROM, effort limits and all other joints unchanged. The native test creates
both V4 and V5 through `CompiledBodySchemaV3`, imports a +30-degree pose through
the production restore API and reads physical endpoints / forearm orientation.
It verifies V4's backwards flexion as a negative control, V5's forward flexion,
inward adduction and unchanged backwards knee bending on both sides. This is
kinematic correctness, not a causal claim about the learned trunk lean.

Exact delta tests permit only eight axes, two collider local Z coordinates,
and new schema/source identities. Mass, COM, full and projected inertia, link
binds, effectors, feet, dimensions, materials and exclusions are unchanged.
The neutral gap is `21.444 mm` versus V4's `90.7 mm`; whole-body neutral COM is
unchanged. All V2–V5 neutral stature/sole/non-excluded-AABB checks pass.

External output directory:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/body-v5-01`.
`descriptor.json` SHA-256:
`1e3c0cefaa92cda91e1c1ae893126df0da4beee852d69a509efae597bbe86a54`.
Body schema hash:
`5a485b67d250ac40c1232176ab9de982954541a6a2080078494aea1b5054288b`.
`sagittal-comparison.png` was visually inspected; it compares actual compiled
colliders and unchanged centres of mass, not learned trajectories.

Validation: 130 native `next_motor` library tests passed; focused clippy with
all targets and warnings denied passed after replacing a redundant test Vec
with an array. Initial default-SDK build failed closed on a different global
SDK build profile. Re-running with the existing exact CPU SDK `f259d3da…`
passed; the global locator and manifest checks were not changed.

`content-package` (123 records / 64 chunks) and all six `boundary-scan`
checks also passed. Formatting, diff
whitespace and local report/ADR/task-state links passed. Broad `host-check`,
gameplay `play`, full application `persistence-replay` and performance were
not run for this opt-in body-data change; the package's native replay and
old-environment regression tests were included in the 130 passing tests.

Remaining: source-preserving full principal inertia, foot mechanics,
V5-compatible controller/reference directions, native dynamic support and a
separately identified training environment. No optimizer was started. The old
V4 checkpoint remains a control and is not called V5-compatible. A native
neutral substep is not a standing/walking quality result.
