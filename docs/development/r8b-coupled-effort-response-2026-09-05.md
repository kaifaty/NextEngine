# Coupled standing effort response — research contract revision 1

Research ID: `r8b-coupled-effort-response.v1`. Engineering consumer: choose the
next torso/leg actuator repair without altering anatomy/mass to mask control
oscillation. Architecture baseline `e9ebdba3`; Accepted SPEC-26/35, ADR-059/069/
115/116 retain their semantics. This is an example-only empirical probe.

## Frozen claim and ceiling

For the V6 `shoulder-yaw-gain-16` diagnostic on the explicit per-iteration force
profile, after exactly 240 recorded native steps of the original standing
reference, measure the one-step joint-velocity response to a perturbation of
one commanded joint torque about the last applied effort vector. Domain:
all 23 DOFs, both signs of 0.01 and 0.02 N m, fixed 1/240 s step, existing
16/4 solver iterations, same floating-base contact scene and native f32 path.
Canonical output is quantized to microradians / microradians per second.

Question: does a locally consistent coupled response exist over those two
amplitudes, with material off-diagonal terms in torso/leg channels? Central
differences are reported in (rad/s)/(N m), not called the inverse mass matrix.
The map includes contacts, joint constraints, solver state and the timestep.
Define local consistency as relative Frobenius difference <=5% between the
two matrices (denominator max of their Frobenius norms); report absolute
difference and per-column sensitivity as well. Off-diagonal norm is reported,
not automatically interpreted as instability. The exact negation is failure
of that finite consistency criterion; it rejects this local approximation,
not controllability or physical validity of the body.

Independent alternatives: negligible coupling; materially coupled response;
contact/limit/quantization sensitivity that invalidates a single local map.
All remain live before measurement. No full closed-loop stability theorem,
global gain optimum, modified safety or learned performance follows from this
experiment. A finite map must not be silently promoted to an SPD mass matrix.

## Controls, limits and stop rule

Reconstruct each world from its compiled initial scene and the complete applied
effort prefix, never from a pose-only restore. Require exact full canonical
snapshot equality before perturbing. Two zero-perturbation fresh worlds must
give exactly equal post-step snapshots. A unit test deliberately changes one
input to show the equality guard rejects mismatch. Persist commanded vectors
and raw joint/link/contact state for each response, rather than just a matrix.
Check command effort bounds; this is a native plant-response experiment, not
a safety-controller output/admission test. No longer rollout follows a trial.

Budget: one 240-step prefix, 92 signed perturbation worlds plus two zero
controls. Stop on any reconstruction mismatch, invalid input or native error;
report nonlinearity without amplitude retries. One independent review, with
at most one batched repair/re-review if a load-bearing defect is found. Raw
artifacts stay external; this contract precedes implementation and outcomes.

Next decision depends on the result: a consistent map permits a bounded
coupled-control analysis; an inconsistent map requires contact/quantization
localization rather than gain selection from that matrix. Neither outcome
completes the full BodySchema / foot / balance goal.

## Result: finite consistency claim REFUTED

The frozen contract above had SHA-256
`1611d6b44954d1aab577650a3cd27d4a4a951ccf6724d1caa9680e75d0501aa0`
before this outcome section. Its domain and 5% criterion are unchanged.

| Metric | Measured value |
|---|---:|
| Frobenius norm, 0.01 N m map | 1.9381999593695178 |
| Frobenius norm, 0.02 N m map | 1.5062272238941907 |
| Difference norm | 1.4207389468160574 |
| Relative difference | **73.3019800123313%** |
| Off-diagonal norm, 0.01 / 0.02 | 1.3289888458899872 / 0.5744680028078152 |
| Individual column relative sensitivity range | 20.8265%–143.4157% |

All 94 full canonical pre-step reconstructions matched. Both post-step zero
controls matched. The complete 240-step prefix and 61 motor samples also equal
the prior `gain-16-baseline.json` prefix exactly. An independent reviewer
verified all frozen hashes, source/command/DOF/sign/count correspondence and
complete canonical serialization, independently computed the matrices and
reran the native probe once: the complete stdout was byte-identical. No
load-bearing defect was found; no repair/re-review cycle was required.

Final canonical velocity rounding alone permits a matrix-difference Frobenius
error bound of 0.001725: each rounded output contributes at most 0.5 microrad/s,
the central-difference entry bounds are 0.00005 and 0.000025, and a 23×23 matrix
has Frobenius bound 23 times their sum. This cannot explain 1.420739. It does
not bound internal f32 arithmetic, hidden solver branches or contacts/limits.

Every trial retains four positive-normal-load ground points on each foot;
there is no observed support-point-count change. This does **not** prove an
unchanged friction active set or eliminate contact-solver sensitivity. Both
the numerical mechanism and its contribution to the standing oscillation
remain unresolved. Do not use this map as an inverse mass matrix or select
new gains from it; a negative diagonal entry is not negative anatomical mass.

## Exact evidence and repeatable analysis

External directory:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/coupled-response-01`.

- Native `response.json` SHA-256:
  `42d877482cc88718b5e90d5bea5123513aba33127373c2e29d376513aab93804`.
- `analysis.json` SHA-256:
  `d2a88d5c832d3e7b843395ee560348499685a788502ac3f69a8a54ce93e8210d`.
- Frozen main example SHA-256:
  `f1b88fa1d743dddb8aab88b68eeb28a4277f2f5fe73b82a337e62873e76477b9`.
- [Native response helper](../../crates/motor/examples/support/effort_response.rs)
  SHA-256: `bbc986a876d220534d86b58359eab1aa5b958d5346d2baf4371673f19e5837d5`.
- [Analysis script](../../lab/scripts/audit_coupled_effort_response.py) SHA-256:
  `20ca91af2bf38a59770c5d10548c783f2a12ca343ddfb593b988b144fd3faf38`.

Native command: use the task-state SDK environment, `cargo run -p next_motor
--features physx-sdk --example probe_biomechanics_body_standing -- 6 0 0 baseline
per-iteration shoulder-yaw-gain-16 response`. Analysis takes the native JSON
path as its single argument and emits hash-bound JSON. The response is a
one-step native plant probe, not a safety-controller admission result; the
preceding stance retains production safety. No result is a learned motion.

Independent evidence classes: EMPIRICAL, NUMERICAL, CORRESPONDENCE, bounded to
this profile/state/host/command set. Reconstruction shares the same native
simulator and is a repeatability control, not independent physical truth.
The analysis unit tests use a manufactured coupled linear map, an explicit
amplitude-dependent negative case and missing/duplicate/wrong-command/control
failures. They do not act as the native physical oracle.

## Decision and next action

Reject fitting a single local gain/inertia map from these amplitudes. Do not
retry amplitudes under this contract, and do not interpret the observed
coupling as a mathematical cause of instability. Next cheapest discriminator:
compare cold-start, same joint pose/zero velocities, with and without ground
contact using an explicitly identified whole-body vertical translation. That
separates the contact boundary from joint constraints without transferring
pose-only state as if it preserved warm solver history. If both cold controls
are consistent, the standing pose/history needs its own next discriminator.
This is a new apparatus/contract, not a retry to make the current map pass.

PASS: 2 focused native example tests, 3 Python numerical/input tests, native
example clippy with warnings denied, Ruff, format/diff checks, exact old prefix
and independent full native rerun. Full ProductChecks / training / foot
successor / perturbation robustness: NOT_RUN (example-only investigation,
no default runtime or body factory changes). The overall goal remains active.
