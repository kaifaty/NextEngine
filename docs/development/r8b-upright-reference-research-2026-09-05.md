# V6 standing: reference counterfactuals and TGS force scheduling

## Result and decision

`SUPPORTED_BOUNDED / DIAGNOSTIC_ONLY`. The persistent backward lean is not
explained solely by body geometry, mass or ordinary loaded-PD deflection.
An unchanged V6 body/controller reports substantial nonzero angular/joint
velocity while its pose barely changes. A one-flag native TGS counterfactual
nearly removes the root's mean velocity discrepancy and changes the standing
equilibrium. It does **not** remove the remaining oscillation or establish
robust standing/walking. No production scene/reference profile was changed.

Next: introduce explicitly identified opt-in force scheduling with unchanged
legacy bytes/behavior, then test damping/balance on that corrected substrate.
Do not continue adjusting hip/ankle targets against the old biased signal.
Foot mechanics and a compatible learning generation remain part of the goal.

## Hypotheses and discriminators

| Hypothesis | Prediction | Evidence / update |
|---|---|---|
| Fixed ankle bias is sufficient to explain/repair leaning | Changing only ankle target should reach an upright equilibrium | Both +/-70 mrad variants fall. Reject constant ankle bias as a sufficient fix. |
| Hip target must compensate ordinary loaded-PD deflection | Approximately +130 mrad compensates observed -127/-129 mrad hip angles | Constant offset falls. Feedback gain 2 survives but retains tilt; gain 4 falls. Loaded deflection alone is an inadequate causal model. |
| Between-frame dynamics or native velocity semantics bias damping | Per-physics-step pose and reported velocities disagree; changing force scheduling affects this discrepancy | All 7,200 steps show persistent velocity bias plus alternating small pose changes. One TGS flag nearly removes the mean root discrepancy without changing masses, targets, gains or safety. Supported for this exact control. |

Success oracle: full 30 seconds without fall/forbidden-contact/hard-ROM
termination, small sustained pelvis/trunk tilt (not just a favorable final
frame), physically loaded feet and meaningful velocity feedback. The current
experiment does not close all of these. Non-regression: zero-offset telemetry
reproduces the old native trajectory, and reverting the sole bridge flag
reproduces the entire new baseline JSON byte-for-byte.

## Reference experiments retained, not selected

All use body V6, 60 Hz reference, 240 Hz native physics, compiler's **16 position
/ 4 velocity TGS iterations** (not the adapter default 8/2), original fixed PD,
ROM/slew/effort/rate/power/work limits, gravity and contact termination.
No residual policy, reset-after-failure or padded completion.

| Probe arguments after body revision `6` | First terminal / motor tick | Final pelvis tilt |
|---|---|---:|
| `0 0` | timeout / 1800 | 9.6163° |
| `-70000 0` | fall / 128 | 62.1357° forward |
| `70000 0` | fall / 135 | 61.1677° backward |
| `0 130000` | fall / 233 | 61.6865° forward |
| `0 0 hip-feedback` | timeout / 1800 | 5.4736° |
| `0 0 hip-feedback-4` | fall / 342 | 47.8161° total tilt; fall predicate is not only a tilt threshold |

Diagnostic hip feedback is `-k * (2*qx/2^30) - omega_x/5`, scaled to
microradians, k=2 or 4. It uses truncating integer division and unchanged
downstream safety, not a new normative standing controller. Increasing gain
is not monotonic improvement. Do not retry these variants without changed
causal inputs. The zero-offset motor samples exactly match the prior V6 probe
after removing the newly added telemetry fields.

## Adjacent-layer finding and bounded primary-source research

Native export reads `getGlobalPose`, `getAngularVelocity` and the articulation
cache's `jointVelocity` after `fetchResults(true)`. No observed stale cache,
unit conversion or array-order defect was found in that path. Every physical
step is now recorded with actual effort, joint state and raw contact impulses.

The final baseline second has mean root omega X **+0.760378 rad/s**, versus
**-0.00053455 rad/s** from world-frame quaternion increments at 240 Hz.
Hip DOF 0 similarly reports +0.722774 rad/s versus -0.000343 rad/s mean
position change. This is not merely the 60 Hz viewer missing a fast movement.
There is also a small alternating substep pose/effort response; it does not
account for the nonzero mean velocity.

Primary sources inspected on 2026-09-05:

- [Isaac Lab actuator documentation](https://isaac-sim.github.io/IsaacLab/develop/source/concepts/actuators.html):
  finite stiffness permits tracking error under load; damping uses velocity.
  This explains why a biased velocity can matter, not our specific cause or
  gains. Our control is explicit fixed PD, not an Isaac implicit drive.
- [MIT humanoid dynamics notes](https://underactuated.mit.edu/humanoids.html):
  whole-body COM and contact forces/CoP govern balance; an upright segment
  angle or COM inside a potential foot box alone is not a stability proof.
- [PhysX 5.7 simulation documentation](https://nvidia-omniverse.github.io/PhysX/physx/5.7.0/docs/Simulation.html#tgs-steady-state-velocity-and-position-discrepancy):
  TGS force scheduling can produce nonzero reported steady-state velocity.
  Applying external forces each internal iteration addresses the illustrated
  spring example. Split-impulse pose/velocity differences can still exist;
  its equivalence argument expressly has an articulation/Coriolis caveat.
- [PhysX scene-flag reference](https://nvidia-omniverse.github.io/PhysX/physx/5.4.0/_api_build/struct_px_scene_flag.html):
  the flag changes force integration, including freefall distances and
  iteration dependence. It must not be silently enabled under old identities.
  The pinned local **5.9.0** `include/PxSceneDesc.h:280` confirms the same flag,
  semantics and default false. Online docs do not imply a runtime SDK upgrade.

No SDK download, disabled gravity, floating-body compensation, relaxed safety
or wholesale mass redistribution was used.

## TGS counterfactual

One temporary bridge line after `description.solverType = eTGS`:
`description.flags |= PxSceneFlag::eENABLE_EXTERNAL_FORCES_EVERY_ITERATION_TGS`.
The probe output explicitly labels this an experimental scene override, not
canonical or training-admissible; the body/compiled hashes alone do not close
the overridden physics identity. Both runs use zero reference offsets.

| Measurement | Existing scene | Per-iteration external forces |
|---|---:|---:|
| First terminal | timeout at 1800 | timeout at 1800 |
| Final pelvis / torso tilt | 9.6163° / 8.0776° | 1.9018° / 0.6503° |
| Last 10 s maximum pelvis / torso tilt | 9.6711° / 8.1885° | 7.2497° / 8.5946° |
| Last 1 s mean native root omega X, rad/s | +0.760378 | -0.0522438 |
| Last 1 s mean pose-derived omega X, rad/s | -0.00053455 | -0.0521853 |

The better final frame is **not** sustained-upright acceptance. Next control
work must address the visible oscillation and examine loaded support through
all physical substeps. The diagnostic flag and its output label were removed
after this test; the bridge is byte-identical to HEAD again.

## Exact evidence and reproduction boundary

External directory:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/upright-reference-01`.
Body/descriptor identities are in the [V6 report](r8b-principal-inertia-and-standing-2026-09-05.md).

| Artifact | SHA-256 |
|---|---|
| `baseline-substeps.json`, identical `restored-baseline.json` | `1c8c9f8f8baee777761fca3bf4848aa3bfdb4b33edc3512592257ca1d8350e1d` |
| `tgs-distributed-forces.json` | `399dec0271c29d0627212b8fdbd458078e865e810266119a55d8173d0b1d4106` |
| `offset-zero.json` | `d064fc6724e93fee69e093213e94ab47db84a69d22357008f890dc082124166d` |
| `offset-minus70k.json` | `6ccd48c8beddd745aed9a4d36c1e1d21955464f02b8746db14d36266a573223b` |
| `offset-plus70k.json` | `1b0b83c8e524579ddeb3bd5096c7562881f761cef50ca895a7cf829f5f08be25` |
| `hip-plus130k.json` | `11cbbb6d8dc6df822380fa937d6139e3a995e78b509904e924a51c0cb055490a` |
| `hip-feedback.json` | `a3f2fde77e37e3878c82fbaccd113673e5ee7b6d8e230c1178789be500139af3` |
| `hip-feedback-4.json` | `6ac5d374b8e6c887a733752e7c4d61a85a9116df82286c20c7bf1a0e3efe644e` |

Bridge source SHA before/after: `63e9df1f9ed4da92d33a1631a53107ec02c048895cecd4b404ad413d157b17cf`;
temporary one-flag source: `4ffcdae8f2fe32c5631a9c85ba9719f02ae1ef5c8758da41eb34e14ef7fb0141`;
temporary labeled probe: `474653a6e8ddb17d0c1fe34ece70e4e53bf5d62c31d7de62e5d9cc35039e330f`.
`compare_standing.py` in that directory produces the full-episode comparison
and actual-collider image `standing-tgs-comparison.png`. Those generated
artifacts remain external. Comparison script SHA-256:
`5c22ec29f0df20fb0def738eb02d826db777bc9de6d6b7be1a6a581d4fdbedd4`.
Reproducing the override needs the documented
temporary patch; the checked-in example intentionally runs only existing
production scene semantics.

## Checks and non-claims

- PASS: native zero-offset 1800-tick control before/after temporary patch;
  complete JSON byte equality, not a tolerance comparison.
- PASS: seven invalid CLI cases rejected before simulation/output.
- PASS: focused example clippy with `-D warnings`, formatting and diff checks.
- PASS: previously running full `host-check` for implementation `ba84b9a0`
  finished with explicit overall PASS; no need to restart that old check.
- NOT_RUN: training, mirror admission, disturbance robustness and foot
  articulation successor. No production reference or scene profile adoption.
