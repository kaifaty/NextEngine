# Humanoid motion corpus and retarget profile V1

| Поле | Значение |
|---|---|
| Статус | Frozen `TRAIN-4` input; admission remains conditional on executable and visual validation |
| Дата | 2026-08-12 |
| Profile ID | `nextengine.motion-corpus.humanoid-biomechanics-cmu.v1` |
| Target | `nextengine.body.humanoid-biomechanics-raja-1700.v2`, revision `1` |
| Canonical config | `lab/profiles/humanoid-motion-corpus-cmu.v1.json` |
| Public/runtime contract change | none; importer, source records and retargeted bytes are private lab data |
| Training authorization | none until the complete `TRAIN-4` gate advances |

## 1. Source and rights decision

V1 uses only the Carnegie Mellon University Graphics Lab Motion Capture
Database. The provider's official homepage states: “This dataset of motions is
free for all uses.” The exact downloaded homepage snapshot has SHA-256
`4ac0024566573a10c87b668c826c2c44014b032ae57cbc5fbef2794b80c21cbc`.
The official format page snapshot has SHA-256
`61a09feeb4424de2273189806f19c8fc54f22fcf8cb94071740633da42d9bada`.

For this generation, “all uses” admits commercial training and distribution of
the resulting model output. Original AMC/ASF bytes and retargeted reference
clips are nevertheless kept in the external training store and are not
redistributed with the engine or model bundle. The corpus contains no
NextEngine gameplay capture and therefore has no player-consent dependency.

LaFAN1 was considered and excluded: its official repository applies
CC BY-NC-ND 4.0, whose non-commercial and no-derivatives restrictions are
incompatible with this commercial retarget/training purpose. AMASS, Motion-X
and other candidates remain unadmitted until their exact per-file rights and
model-output terms pass the same review.

## 2. Source format and deterministic conversion

CMU ASF/AMC records are treated as 120 Hz, left-handed `X-left, Y-up,
Z-forward` data. Engine coordinates are right-handed `X-right, Y-up,
Z-forward`; the exact vector conversion is `[-source_x, source_y, source_z]`.
The official ASF length conversion is the rational `127/2250` metres per source
unit. An inclusive crop is sampled at relative source frames `1,3,5,...`, so
every reference boundary is exactly 60 Hz without interpolation.

The private importer parses each declared ASF hierarchy and AMC channel table,
applies the authored ASF axis basis, and maps explicit semantic chains to the
23 BodySchema channels. Torso combines lower/upper back and thorax; each hip
maps femur rotation plus knee and foot; each shoulder combines clavicle and
humerus plus radius flexion. No target is selected by runtime name guessing.

Every solved target is quantized to microradians and clamped to the exact V2
soft ROM. Root translation, root orientation/yaw, velocities, CoM, effectors,
contacts and phase are derived at 60 Hz. A deterministic upward-only ground
correction may remove collider penetration, bounded at `0.250000 m` and
recorded per frame and in the clip report. No manual contact override exists in
V1; adding one creates a new corpus/profile hash.

## 3. Corpus and split closure

The exact hashes, frame ranges and individual clip records live in the
canonical JSON profile. Split ownership is performer-isolated:

| Split | Subjects | Purpose |
|---|---|---|
| train | 16, 85, 104, 140 | idle, starts, slow/nominal walk, stops, gradual turns, brace/fall, prone/supine/side get-up and side transitions |
| validation | 05 | independent nominal walk |
| held-out | 139 | independent weight shift, walk, prone and supine get-up |

`split_group_id` is the original subject/trial identity. Crops and mirrors
retain it exactly, so no original/mirrored/cleaned sibling can cross a split.
Subject 139 is held out in full even though only four of its trials are admitted.

The right-leading start, right brace/fall, left side get-up and left side
transition each produce one deterministic sagittal mirror. Mirror uses only
the BodySchema symmetry mapping: root/effector X is negated, left/right arrays
are swapped, semantic joint mirror signs are applied and the reflected root
quaternion is `[x,-y,-z,w]`. Applying the transform twice must restore every
canonical integer byte exactly.

## 4. Validation and claim boundary

An admitted clip must satisfy all of the following before `TRAIN-4` advances:

- all source, license-snapshot, profile and target-descriptor hashes match;
- every target channel stays inside soft and hard ROM after solve;
- upward ground correction stays within `0.250000 m`, and locomotion has zero
  non-foot penetration after correction;
- contact facts follow the fixed sole/recovery height and velocity thresholds;
- start/stop, walk/turn and recovery direction checks match the declared skill;
- a loop, if later introduced, passes the profile pose/velocity wrap bounds;
- mirror-twice is byte exact, and no `split_group_id` or performer crosses a
  split;
- deterministic re-import recreates identical NPZ and manifest hashes;
- source/target overlay previews cover every base and mirrored clip and receive
  one explicit visual disposition.

The `85_15` input is not admitted wholesale. Only frames `800..1070` are a
candidate brace/fall segment; its acrobatic source label is not evidence of a
safe recovery reference. If the bounded segment fails contact, anatomy,
ground-correction or visual review, it is excluded and `TRAIN-4` remains
blocked until a compatible fall source is added. It is never repaired by
relaxing BodySchema ROM or safety.

This profile creates no `RetargetProfileV1`, reference-tracking environment or
`PhysicalActionChunk` wire format. Those remain Proposed until `TRAIN-5` has a
concrete production consumer and a separate consumer-backed architecture
decision.
