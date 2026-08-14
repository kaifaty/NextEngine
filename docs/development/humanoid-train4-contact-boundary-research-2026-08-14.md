# TRAIN-4 contact-boundary and projection-domain research

| Field | Value |
| --- | --- |
| Date | 2026-08-14 |
| Scope | Optimizer-free bounded research for `REQ-HUM-DATA-005/007` |
| Status | `DECISION_RECORDED / IMPLEMENT_BOUNDED_V5` |
| Claim ceiling | A passing V5 may authorize only a clip-global prototype; it cannot authorize a full V19 corpus or optimization |

## Frozen problem and constraints

The V18 corpus remains inadmissible because exhaustive R14 has `204/12518`
required-safety failed start states. R27 rejected the first contact-manifold
projector and proved indexed running-scene reset non-equivalent to an ADR-070
fresh scene. Those decisions remain unchanged.

This cycle keeps the following acceptance facts fixed:

- fresh-scene execution is the only PhysX acceptance authority;
- active-point bounds remain `2 mm/frame` tangential, `1 mm/frame` normal and
  `5 mm` residual;
- every collider remains at or above `-2 µm`;
- joint reserve remains at most `2500` basis points and root vertical velocity
  at most `200060 µm/s`;
- required-safety acceptance remains exact zero;
- no grace, settling, zeroed reference velocity, safety-limit change,
  optimizer step or training run is allowed.

## Exact evidence sequence

| Evidence | Result | Interpretation |
| --- | --- | --- |
| R29 V2 fresh discriminator, file SHA-256 `6c17af2c7fa930710da179527cff83d60c31d027e25bbfd40453495270b43a06` | `FAIL 3/4`; all three `cmu139` cases still hard-impact | Initial collider penetration was removed, but the projected swing reference did not remain clear under PhysX |
| R30 V2 trajectory capture, file SHA-256 `460195fc283415f4f5bc90833102a6d832177ba4fb5ab7c5b82df687c35d699b` | Right `FLIGHT` foot returned toward ground while the expected stance was not selected | Support must take precedence over a low-clearance all-flight solve |
| R32 V3 support-precedence probe, file SHA-256 `963c51d58bcf3e9faff559031a8e5e78b7d523dfe227531e8d38260cc3019db9` | Three `cmu139` cases pass; all-flight control `cmu16@415` regresses to hard ROM | High clearance is valid only when another support is active |
| R34 V4 support-conditioned probe, file SHA-256 `5a2d34a586a9619b9b69a1e590751e535717abe1bad8379c16c2417c366da626` | Fresh-scene `PASS 4/4`, required-safety `0`, controls `0` regressions | Permits the ordered 17-case offline prototype only |
| R35 V4 all-17 manifest, canonical SHA-256 `f254f7256d4e4fb67f196298bfc6cbdbda2f7796973d6d6f0aa624e41bec6836`, file SHA-256 `daf605ea811eb51f603b031b602b2043c4544982695c33f84ccfedf7df1c31c1` | `FAIL 6/17`: one `2541/2500` joint-reserve control and five contact-transition failures at `1189..1423 µm/frame` | V4 does not close the smoothing/contact boundary |

All reports are under the external
`humanoid-motor-rebuild-v1/evaluations/TRAIN-4` root. They contain zero
optimizer steps and zero training runs.

## Competing hypotheses and counterfactuals

### H1 — retune clearance and smoothing only

A fixed six-case sweep evaluated nine `(supported clearance, smoothing pass)`
combinations from `(12000 µm, 2)` through `(20000 µm, 8)`. No combination
passed all six R35 failures. Fewer passes increased joint/root velocity; eight
passes repaired `cmu05@40`, but the five contact-boundary cases still reached
up to `1276 µm/frame` analytic normal motion.

**Conclusion:** rejected as the primary fix. The boundary condition, not one
scalar parameter, is missing.

### H2 — future flight correction leaks through active support

For `cmu16@238/239/241/407/409`, the maximum occurs on the last right
`FOREFOOT_STICKING` frame immediately before `FLIGHT`. Raw right-leg
correction is zero on the active frames, but the symmetric temporal filter
imports future flight correction backward. At `cmu16@241`, for example, the
last active frame receives approximately `3.9/-13.3/4.0 mrad` across the main
right-leg correction channels after smoothing.

A final contact reprojection after joint smoothing reduced the bounded
maximum analytic normal motion from `1423` to `837 µm/frame`. Combining it
with eight smoothing passes and a deterministic upward-only root-velocity
projection produced a read-only `17/17` local counterfactual with maxima:

- analytic normal motion `762 µm/frame`;
- joint velocity reserve `2484/2500` basis points;
- root vertical velocity `200040/200060 µm/s`;
- active residual `1959 µm` and shared normal step `584 µm`;
- minimum collider height `-2 µm`.

This counterfactual is implementation-selection evidence, not a gate report.

**Conclusion:** supported for the smallest bounded V5 implementation.

### H3 — independently solved windows can define full V19

The same candidate was compared at every overlapping absolute source frame.
Window-local solves disagreed at 60 shared frames, with maxima of `61997 µm`
for root position and `60755 µrad` for joint position. A single 801-frame
`cmu16` solve then failed globally: joint reserve `2502/2500`, analytic normal
`1304 µm/frame`, analytic tangential `9695 µm/frame`. Only `2/7` selected
`cmu16` windows passed as slices of that one trajectory. Repeating
contact/floor/velocity projections ten times reached the same fixed failure
instead of converging.

**Conclusion:** rejected. A window-local `17/17` result cannot authorize a
full corpus identity. The full lineage needs one deterministic projected value
per `(clip_id, reference_frame)` and must validate the resulting complete
trajectory, not a set of incompatible overlays.

## Decision

Implement one immutable V5 bounded prototype with:

1. eight supported correction-smoothing passes;
2. final active-contact reprojection after the leg solve;
3. a second upward-only all-collider floor;
4. deterministic upward-only root-velocity closure using the unchanged
   `200060 µm/s` bound;
5. failure if final reprojection drops an active point.

Preserve V1–V4 behavior byte-for-byte through an opt-in V5 field and a new
profile/algorithm identity. First rerun the four-case fresh-scene
discriminator. If it passes, run the ordered 17-case offline and fresh-scene
matrix. Even a `17/17` result may only authorize a clip-global prototype.

The subsequent clip-global revision must:

- solve each clip once and slice immutable frame values;
- prove overlap identity exactly;
- pass the selected controls/failures as slices of that trajectory;
- pass complete-clip invariant checks before a 27-clip V19 build is allowed.

## Rejected options and reconsideration

- Do not retry parameter-only clearance/smoothing sweeps; reconsider only if a
  new contact-boundary mechanism changes the measured response.
- Do not alternate the same contact/floor/velocity projections indefinitely;
  the 801-frame counterfactual reached an unchanged failure fixed point.
- Do not normalize away window disagreement or choose one convenient overlay;
  source-frame identity must be unique by construction.
- Do not loosen contact, collider, joint or root bounds.

Reconsider V5 only if its fresh PhysX discriminator regresses a passing
control or adds a required-safety category. Reconsider clip-global admission
only after one deterministic solver passes exact overlap, selected-window and
complete-clip checks together.
