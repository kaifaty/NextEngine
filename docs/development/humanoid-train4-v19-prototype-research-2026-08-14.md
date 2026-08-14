# TRAIN-4 bounded V19 prototype decision — 2026-08-14

| Field | Value |
|---|---|
| Status | `COMPLETE / PROTOTYPE_REJECTED / OPTIMIZER_FREE` |
| Decision | Retain ADR-070 fresh-scene authority and build one collider-aware joint/root prototype revision |
| Requirement | `REQ-HUM-DATA-005`, `REQ-HUM-DATA-007` |
| Complete PhysX report | R27, SHA-256 `fabfef54d01ac421778fac05d9aa2a1bb062803c61f8e335ed191ed81e249e14` |
| Result | `FAIL / STOP_AND_RESEARCH`; no full V19 corpus or optimizer is authorized |

This report closes the bounded contact-manifold/reset discriminator selected by
the earlier [causal research cycle](humanoid-train4-causal-research-2026-08-14.md).
It is a working engineering decision, not an Accepted ADR, corpus admission or
training result. Generated artifacts remain in the external training store.

## Frozen experiment

The offline R18 prototype selected 17 exact 11-tick windows from
`cmu05-walk-validation`, `cmu16-walk-nominal-b` and
`cmu139-walk-heldout`, including matched passing controls. Its manifest has
canonical identity
`794e479b61c5b3041b75f86095647c1663f127763a8eddd9f3292cee0fa524ae`
and file SHA-256
`ab684f60d1eca8ca636df20efbbb376618511663c0db07f4a486a336ee6ccbad`.

The prototype introduced explicit `FLIGHT`, `HEEL_STICKING`,
`FOREFOOT_STICKING` and `FLAT_STICKING` modes, point-consistent height/speed
classification, final finite-difference velocities and root-link velocity
semantics. Frozen active-point bounds were `5 mm` normal residual,
`1 mm/frame` normal displacement and `2 mm/frame` tangential displacement.
It executed no optimizer or training run.

R27 evaluated every window in its own fresh-scene process with zero
post-create root/joint state writes. Isaac's two mandatory initialization steps
ran at `1 ns` with ground collision disabled; collision was restored before
the contact view and before the first real `1/240 s` substep. One separate
17-slot worker then ran same-case warmup plus indexed partial reset. The
non-Isaac driver required all worker reports before aggregation.

## Probe hardening before the complete result

R19–R26 were incomplete diagnostics and are not acceptance evidence:

| Runs | Observation | Resolution |
|---|---|---|
| R19 | Worker exceptions and CUDA integer indexing were not propagated reliably. | Require worker JSON, propagate failure and widen unsupported CUDA integers to `int64`. |
| R20–R21 | Scene initialization advanced approximately two physics steps and changed authored state. | Suppressing render-only work was rejected; the steps are mandatory backend initialization. |
| R22–R23 | Zero-time initialization cannot create GPU tensor handles; `1 ns` preserves pose but ground contact changes velocity. | Keep the two `1 ns` steps and disable only the already-authored ground collision during them. |
| R24 | The 12-frame projection did not cover the tracker's `0/4/8/16` lookahead. | Overlay projected values only inside the bounded window and retain the hash-closed V18 source outside it. |
| R25 | Raw float32 quaternion scale was compared without normalization. | Use the same normalized absolute-dot metric as the production tracker; keep the frozen threshold. |
| R26 | `SimulationApp.close()` ended the in-process partial phase before driver aggregation. | Isolate partial reset in a required worker process; a zero exit without JSON is invalid. |

These changes corrected measurement and process isolation. They did not relax
safety, add settling/grace, zero reference velocity or write pose after scene
creation.

## Complete R27 result

R27 ran from clean commit `3e7c54a30ec876fdb84ab352018c1c5cc304f6b2`
and emitted manifest identity
`25c5c5f7a7c3672c4a533b69f250cac904c7b6ea1667dffc6c6b34f07a349f84`.

| Criterion | Fresh scene | Indexed partial reset | Required |
|---|---:|---:|---:|
| Failed cases | `3/17` | `4/17` | no passing-control regression |
| Hard impact | `3` | `3` | strict decrease from `3` |
| Hard ROM | `0` | `1` | strict decrease from `4` |
| Joint velocity | `2` | `2` | strict decrease from `2` |
| Passing-control regressions | `2` (`7`, `9`) | `3` (`7`, `9`, `16`) | `0` |

Fresh and partial outcomes differ for case `16`, `cmu16@415`: fresh passes the
full horizon, while partial reset reaches right-ankle-pitch hard ROM on tick
`10`. Eight other cases have equal outcomes but first-tick contact-pair impulse
deltas above the frozen `10,000 µN·s` bound; the maximum is
`252,409 µN·s`. Therefore indexed running-scene reset is not equivalent to a
fresh scene for this consumer.

## Collision-geometry discriminator

All three fresh failures are adjacent `cmu139` starts (`626`, `627`, `630`)
and terminate on tick `1` at `ground:body.right-ankle-roll`. Impulses are
approximately `28.224`, `23.557` and `17.301 N·s`; starts `626` and `630` were
passing controls before projection.

An independent offline recomputation used the exact R18 pose arrays,
descriptor, [`target_forward_kinematics`](../../lab/next_lab/motion_math.py)
and [`collider_minimum_y`](../../lab/next_lab/motion_math.py). It found the
right swing-foot box already below the ground at reset:

| Start | Contact mode `[left, right]` | Right sole-collider minimum |
|---:|---|---:|
| `626` | `[FOREFOOT_STICKING, FLIGHT]` | `-14,905 µm` |
| `627` | `[FOREFOOT_STICKING, FLIGHT]` | `-11,639 µm` |
| `630` | `[FLAT_STICKING, FLIGHT]` | `-3,544 µm` |

The existing corpus validator requires global minimum collider height
`>= -2 µm`. The bounded projector rechecked only active heel/forefoot points,
applied root correction to the whole body, left joint correction at zero and
did not revalidate the inactive swing-foot collider. The exact violation and
the first PhysX contact identify missing collision closure, not an impact-limit
calibration problem.

## Decisions

### Retain fresh-scene reset authority

ADR-070 already requires a fresh PhysX scene and forbids post-create root
teleport. R27 provides project-specific evidence that the indexed running-scene
implementation is not equivalent. No Accepted semantic change is needed:

- fresh-scene execution remains required evidence;
- indexed partial reset remains diagnostic-only and cannot enter TRAIN-4/5
  acceptance or optimizer input;
- a future vector implementation must replace a slot with a genuinely fresh
  scene, not reinterpret buffer clearing or state writes as reset equivalence.

### Reject the R18 projector

R18 improves bounded hard ROM but fails controls, hard impact, joint velocity,
collider non-penetration and reset equivalence. It cannot be expanded to 27
clips and must not be relabelled as a full V19 corpus.

### Build one collider-aware revision

The next smallest experiment keeps the same 17 windows, controller, safety
limits and active-point tolerances, but creates a new immutable prototype
identity with these additional invariants:

1. every physical collider is at least `-2 µm` above the ground on every
   emitted frame, including a foot in `FLIGHT`;
2. active heel/forefoot constraints still satisfy the frozen position and
   velocity bounds;
3. a conflict between stance contact and swing clearance is solved through a
   temporally coupled bounded root plus leg-chain correction; root-only
   translation is not an admissible solution;
4. final root/joint velocities are recomputed after the final pose, with
   root-link semantics and the existing hard ROM/reserve limits unchanged;
5. fresh-scene PhysX is the acceptance path. Indexed partial reset may be
   repeated only as labeled report-only evidence.

The first discriminator is the three `cmu139` windows plus `cmu16@415`; only a
no-regression result permits rerunning all 17 cases. A passing 17-case result
permits a full V19 data build, not optimization. Full TRAIN-4 still requires
local/native/visual/exhaustive gates and exactly zero required-safety events.

## Rejected alternatives

- Raising the whole root until the swing foot clears: this can lift the active
  stance points outside their frozen residual and merely transfers the defect.
- Loosening the impact or reset-impulse bound: R27 exposes geometric
  penetration and real outcome divergence.
- Clearing only Isaac sensor buffers or adding a warmup/grace interval: neither
  reconstructs a fresh PhysX scene and both contradict the frozen episode.
- Expanding R18 to the whole corpus or starting PPO: controls and targeted
  categories fail before optimization.

## Remaining uncertainty

It remains unproven that a bounded temporally coupled leg/root solve can close
swing clearance without creating ROM, velocity or transition regressions.
That is the sole next implementation question; if the four-case discriminator
fails, stop before another broad corpus revision and reconsider the selected
degrees of freedom and contact-mode transition model.
