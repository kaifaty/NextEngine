# Body proportions and foot review

Status: `VISUALIZATION_FIXED / PHYSICAL_FOOT_SUCCESSOR_NOT_IMPLEMENTED`.
This is bounded diagnostic evidence, not an anatomy or training promotion.

## Observation and discriminating check

The user observed apparently overlong legs and an absent trunk, and requested
body/foot improvements. Competing explanations were (H1) wrong physical segment
dimensions, (H2) missing geometry in the diagnostic picture, and (H3) learned
posture. The inexpensive control uses the same closed native frame twice:
old parent-origin connections versus every actual collider. A separate view
uses descriptor initial world transforms, not learned pose or inferred FK.

H2 is supported: the previous plot omitted torso/head/hand geometry. Its torso
origin is the lumbar articulation, not the chest or head centre. Connecting it
directly to shoulders draws a misleading V. H1 as the explanation for the
missing trunk is rejected; this does not certify all anthropometric details.
H3 remains relevant to leaning/arm posture in motion, not the missing geometry.

## Measurements on the unchanged BodySchema V4

| Initial physical measurement | Result |
| --- | --- |
| Ground-to-head collider stature | 1.700 m |
| Hip joint height | 0.865010 m, 50.883% of stature |
| Shoulder joint height | 1.396500 m |
| Hip-to-shoulder vertical span | 0.531490 m |
| Hip-to-knee joint distance | 0.408050 m |
| Knee-to-ankle-pitch joint distance | 0.396472 m |
| Foot collision geometry | One rigid 260 x 110 x 60 mm box per side |

Left/right lengths agree. These are geometric measurements, not clinical
population-percentile thresholds. No limbs were shortened and no trunk was
stretched. The source [Rajagopal et al. 2016](https://nmbl.stanford.edu/wp-content/uploads/07505900.pdf)
describes a 170 cm, 75 kg model; its full muscle/joint model is more detailed
than our reduced joint-actuated projection. This dimensional agreement does
not transfer its biomechanical validation to NextEngine.

## Foot evidence and remaining physical work

The existing successful candidate has real forefoot-edge roll, with heel
height relative to toe reaching 65.4/62.0 mm across moving frames. The left
maximum occurs airborne, so it must not be reported as loaded toe-standing.
The report separately counts raised-heel/front-quarter-pressure frames:
112/90 of 900 moving-command ticks. At tick 1200 both soles are nearly flat
(heel-to-toe differences 0.056/0.027 mm). Force and pressure location here are
last-substep measurements, **not** an all-four-substep support gate. The
zero-command summary combines warm-up and deceleration, not just settled stop.

The rigid foot still cannot bend at the forefoot. [D'Hondt and colleagues'
2024 dynamic-foot study](https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1012219)
compares foot segmentation and parameter effects: both contact/tendon mechanics
and additional articulation affect gait. Its muscle-driven, compliant-contact
parameters are not transferable PD or PhysX constants. A more complex foot is
therefore an experimentally testable successor, not a proven fix for this gait.

Smallest next physical experiment: a separately identified foot candidate,
keeping total length/width and source mass properties explicit; compare flat
loaded support, heel rise with forefoot contact, release/re-contact and bilateral
symmetry against V4 before learning. If adding an MTP joint, explicitly choose
passive versus driven control and bind its ROM/stiffness/damping/energy and
new observation/action/support meaning. Do not reinterpret old checkpoint
compatibility, erase the old body or relax safety. This physical successor
and retraining were **not implemented** in this visualization change.

## Implementation and exact artifacts

- [Shared physical geometry](../../lab/scripts/native_body_geometry.py) composes
  body and collider transforms, draws all declared boxes/spheres and skips
  non-colliding carriers. Unknown shapes and invalid/missing poses reject.
- [Video renderer](../../lab/scripts/render_native_walking_video.py) and
  [contact audit](../../lab/scripts/cpu_walking_contact_audit.py) now show all
  physical shapes. Foot close-ups include the complete selected swing height.
- [Proportion audit](../../lab/scripts/audit_body_proportions.py) validates the
  closed source, measures initial proportions and existing sole mechanics,
  writes tool/artifact hashes and preserves all source bytes.
- Source is [known candidate 3999](r8b-known-candidate-reuse-2026-09-05.md),
  evaluation-manifest SHA `a8678933d1b934109b5c50d991e4228910ebd4c642dbe7b4c1d193cf24e1884c`.
- External output root:
  `/home/kaifaty/NextEngine-training/r8b-body-proportions-2026-09-05/`.
  `audit-04/report.json` closes four diagnostic PNGs and source/tool identities.
  `video-01/video-manifest.json` SHA
  `20ec83f844be2cc11e9a8c15e7ea784f1d1b24e485ac5dca8e5d6edbffdd12cf`
  closes the full 1200-frame, 60 fps, 20-second video. Original media remains.
  `audit-01`/`audit-03` are superseded previews; `audit-02` failed on a local
  CLI variable shadowing defect, subsequently fixed with a regression test.

## Verification and decision

PASS: 32 focused geometry/CLI/video/contact/candidate tests, Ruff, formatting
and diff checks. Full video frame count and complete one-second
contact sheet plus corrected foot close-ups are manually inspected.
Native training, runtime ProductChecks and optimizer are
`NOT_RUN(NO_PHYSICS_OR_TRAINING_CHANGE)`; old walking evidence is preserved,
not rerun or newly promoted. No roadmap/Accepted semantics changed.

Rollback is presentation-only: restore previous drawing code, never mutate
source physical records. The broader request for improved physical foot
mechanics remains separate from this completed visualization repair.
