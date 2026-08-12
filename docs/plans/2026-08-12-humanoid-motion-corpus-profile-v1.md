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
The LF-terminated exact sentence has SHA-256
`3419b642c89ad0ab03f04bc2b12c38b2be7e12cd41492ca4e258ac190a62bf47`.
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
soft ROM. A causal cleanup projection then limits each channel to the target
joint's published maximum velocity at 60 Hz; the per-sample projection is
preserved in the artifact and summarized in validation rather than hidden.
Root translation, root orientation/yaw, velocities, CoM, effectors, contacts
and phase are derived at 60 Hz. A deterministic signed vertical
alignment places the lowest sole collider on the ground for locomotion
(absolute bound `0.125000 m`) and the lowest solid collider on the ground for
recovery (absolute bound `0.650000 m`). The larger recovery bound covers the
different pelvis origins of a standing target and a prone/supine source; every
applied value is recorded per frame and in the clip report. No manual contact
override exists in V1; adding one creates a new corpus/profile hash.

The locomotion bound covers the measured worst independent-skeleton target
alignment (`0.120878 m` for subject 91) with less than `5 mm` integer margin;
it remains a target sole placement, not a source-motion correction.

## 3. Corpus and split closure

The exact hashes, frame ranges and individual clip records live in the
canonical JSON profile. Split ownership is performer-isolated:

| Split | Subjects | Purpose |
|---|---|---|
| train | 16, 90, 104, 140 | complete mandatory class coverage; slip/fall source 90 |
| validation | 05, 91, 111 | independent locomotion, lay-down/fall, prone/supine get-up and side transition |
| held-out | 114, 139 | independent locomotion, lay-down/fall, prone/supine get-up and side transition |

`split_group_id` is the original subject/trial identity. Crops and mirrors
retain it exactly, so no original/mirrored/cleaned sibling can cross a split.
The audit requires one independent group in every split for each of ten learned
class families, giving at least three groups per class. Directional variants do
not increase this count. Subject 139 remains held out in full even though only
selected trials are admitted. CMU subjects 91 and 105 were found to publish
byte-identical selected skeleton/motion files; subject 105 is therefore
excluded rather than misrepresented as independent evidence. The same audit
found that subject 77 publishes a byte-identical skeleton and selected get-up
motions to held-out subject 139, so subject 77 is also excluded and cannot
provide a train-group shortcut around performer isolation.

The `140_06:1..240` idle crop remains inside the source interval where both
femur and shank directions are standing-like; later authored pose work is not
mislabelled as neutral idle. The `140_03:1..273` side-transition crop ends at
the observed side-to-prone/supine support state before the later kneeling/get-up
phase. Both retain the original subject/trial `split_group_id`.

Right-leading starts, left-support stops and left side transitions may produce
one deterministic sagittal mirror inside the same source group and split.
Mirror uses only
the BodySchema symmetry mapping: root/effector X is negated, left/right arrays
are swapped, semantic joint mirror signs are applied and the reflected root
quaternion is `[x,-y,-z,w]`. Applying the transform twice must restore every
canonical integer byte exactly.

Each independently sourced backward safe-floor transition produces fixed
world-Y quarter-turn variants for forward, left and right coverage. This gives
all four required directions in each split while retaining exactly three
independent source groups: `90_17` train, `111_12` validation and the lay-down
half of `114_11` held-out. Rotation derivatives never count as new groups.

## 4. Validation and claim boundary

An admitted clip must satisfy all of the following before `TRAIN-4` advances:

- all source, license-snapshot, profile and target-descriptor hashes match;
- every target channel stays inside soft and hard ROM after solve;
- signed ground alignment stays within its partition bound, and locomotion has zero
  non-foot penetration after correction;
- contact facts follow the fixed sole/recovery height and velocity thresholds;
- start/stop, walk/turn and recovery direction checks match the declared skill;
- a loop, if later introduced, passes the profile pose/velocity wrap bounds;
- mirror-twice is byte exact, and no `split_group_id` or performer crosses a
  split;
- every mandatory learned class has train/validation/held-out source groups,
  every directional variant is present and each slow/nominal gait group has at
  least four contact-derived full gait cycles;
- deterministic re-import recreates identical NPZ and manifest hashes;
- source/target overlay previews cover every base and mirrored clip and receive
  one explicit visual disposition.

The rejected `85_15:800..1070` candidate is an acrobatic twist sequence rather
than a safe brace/fall and is not admitted. Admitted safe-floor groups are the
complete `90_17` `BannanaPeelSlip`, complete `111_12` `Lay down`, and frames
`1..1161` of `114_11` `Laying down and getting up`. Each begins upright, ends
in declared floor support and lowers retargeted CoM by more than `0.25 m`.
Source labels are not admission evidence: anatomy, contact, ground-correction
and visual review remain mandatory, and no clip is repaired by relaxing
BodySchema ROM or safety.

This profile creates no `RetargetProfileV1`, reference-tracking environment or
`PhysicalActionChunk` wire format. Those remain Proposed until `TRAIN-5` has a
concrete production consumer and a separate consumer-backed architecture
decision.
