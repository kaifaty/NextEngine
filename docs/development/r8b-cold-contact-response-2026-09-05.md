# Cold contact response — contract revision 1

Research ID `r8b-cold-contact-response.v1`, baseline `5a07513a`. Consumer:
decide whether to investigate contact-specific solver response or internal
joint/constraint response before changing body control. Accepted SPEC-26/35
and ADR-059/069/115/116 remain unchanged. Example-only empirical experiment.

## Domain and discriminator

Use the V6 shoulder-yaw-gain-/16 diagnostic, all authored joint positions and
all velocities zero. Compare the original initial placement with a second
schema differing only by a +500000 micrometre vertical root-bind translation
and its explicit schema/source identity. Every physical body must shift by
500000 +/-1 micrometres with identical orientations, masses, inertias, shapes,
joint frames and joint states. No pose-only continuation or old warm solver
state is imported. The initial scene is rebuilt for every trial.

For each of the two cases, apply zero baseline joint torque, plus signed
0.01/0.02 N m perturbations on each of 23 DOFs for exactly one 1/240 s step.
Two zero controls and 92 signed trials per case: total 188 fresh trial worlds.
Gravity, floor, f32 PhysX, 16/4 iterations and explicit per-iteration forces
stay fixed. This is not a PD rollout or a standing safety admission test.

Retain the prior central-difference units (rad/s)/(N m), relative Frobenius
consistency criterion <=5%, raw commands and complete canonical states.
No new amplitude tuning is allowed. The hypothesis is not that either map
must pass: compare the two outcomes before selecting a mechanism.

- Raised consistent, grounded inconsistent: supports a contact-boundary
  contribution at this cold pose; does not prove the later warm-state cause.
- Both consistent: cold joint constraints/ground contact alone do not reproduce
  the prior 240-step failure; history or later pose remains necessary to test.
- Raised inconsistent: ground contact is not necessary for this cold-state
  failure; inspect joints, solver arithmetic and conditioning.
- Boundary-control failure: INCONCLUSIVE, not a physical outcome.

Require exact pre-state and zero-post-state reconstruction per case, exact
schema delta and translation control, zero recorded ground impulse/contact in
the raised case and positive ground load in the grounded zero control. Report
joint-limit proximity explicitly. A cold unilateral-limit kink is not
negative mass, and an initial no-load case is not a valid grounded control.
Integer velocity rounding is bounded as in the prior contract; internal
float arithmetic/constraints are not covered by that bound.

Stop on first native/identity/reconstruction failure, without changed inputs
or retries. One independent source/raw-data review plus at most one batched
repair/re-review. No global stability, inverse mass, new body-default or
training claim follows. Preserve the old warm response as exact non-regression.

## Outcome and evidence

`SUPPORTED_BOUNDED`: the cold ground boundary contributes to the measured
amplitude sensitivity. The two central maps differ by **45.4775646455%** on
the floor and **0.0185195011%** when raised, using the larger matrix norm as
denominator. Only the raised case passes the frozen 5% test. This does not
identify which contact-solver mechanism is responsible or explain the entire
warm standing trajectory.

All 188 reconstructions and repeated controls are exact. The 24 links shift
exactly 500000 micrometres with unchanged orientations and zero joint states
and velocities. Grounded zero control has 10 ground records, 8 loaded, with
2.696130 N s summed absolute vertical impulse. All raised trial posts have
zero ground records; zero-impulse self-contact records remain. Therefore call
this **ground-contact-free**, not contact-free.

Knees and elbows start at their zero lower limits. Grounded knees reach
-12 microradians (left) and -11 to -10 (right); raised knees reach 0 to 2,
elbows 0 to 11. This is not a safety-controlled stance. Passing a central
consistency test at a unilateral limit does not prove differentiability or
identify inverse mass. The final canonical velocity-rounding Frobenius bound
is 0.001725: conservative ratios remain >45.3255% grounded and <0.07737%
raised. Internal float/solver errors are not bounded by that calculation.

External root:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/cold-contact-response-01/`.

| Evidence | SHA-256 |
| --- | --- |
| `cold-response.json` | `04d11922098b283672a024da35c06b8c2d77e7ec51b58c4c13ec5d79355b0e0f` |
| `analysis.json` | `dd914815139ed7fa2f630ed18d077632e660d48b54ae5c60344674e4821128ed` |
| Frozen contract before this results section | `8ab752eca031f6a5eed2edcf49d91682c57758684fdc08c846b1ebb12a4b2b5f` |
| Native example main at measurement | `b48e138bc6eeb974b4f26d48cbe67c34f363aef2b83981d17e7e92a5413db605` |
| Native response helper at measurement | `3ab1b65066b9e288777ec392240390268736254d865dc34ac78104e15b1a55e4` |
| `restored-warm-response.json` | `42d877482cc88718b5e90d5bea5123513aba33127373c2e29d376513aab93804` |

Reproduce using the explicit pinned SDK directory recorded in
[task-state](task-state/r8b-body-proportions-and-foot.md), then
`cargo run -p next_motor --features physx-sdk --example probe_biomechanics_body_standing -- 6 0 0 baseline per-iteration shoulder-yaw-gain-16 cold-response`.
Analyze with `lab/scripts/audit_coupled_effort_response.py` in the existing
training Python environment. Analysis output binds both raw and script hashes.

Independent `cold_response_review` found no load-bearing apparatus defect,
recomputed the result and reproduced native output byte-exactly. The refactored
warm mode also reproduces the prior SHA `42d87748…` exactly. Focused validation:
3 native example tests and 5 Python tests passed; cold boundary tests reject
ground records in the raised case, absent ground load, override mismatch,
wrong translation, warm prefix and nonzero baseline. Broader ProductChecks
are not rerun for this diagnostic-only change; no runtime profile is selected.

Next discriminator was the
[joint-friction ablation](r8b-joint-friction-ablation-2026-09-05.md). Do not
fit a controller from the inconsistent grounded matrix or retune amplitudes
under this contract. Reconsider only with a distinct, declared causal change.
