# BODY-LOAD-AUTHORITY-01.r1 — frozen read-only contract

Base e14e4afb; Accepted SPEC-35/ADR-119/122/124. Consumer: choose between
target-only balance-law work and a separately specified effort-compensation
path. No body/controller/native change or training is authorized by this
calculation alone. Full calibration requirements remain in BODY-GAIN-NATIVE-01.

## Claim and discriminator

Test whether a necessary static-equilibrium relaxation is feasible at exactly
three captured poses: V8 standing substep7200; V11 substeps120 and200. Use the
sealed descriptors and standing traces from `body-settling-nZsPFi` and
`body-bandwidth-lDjoJM`. Set all velocities/accelerations to zero in this
counterfactual only; it is not a claim that the recorded moving pose is static.

Keep six whole-body force/moment balance equations and the two ankle-pitch
subtree moment equations. Other joint reaction torques are unrestricted and
their equations eliminated. Root has no actuator. External support is limited
to captured foot-ground points with nonzero recorded impulse. Allow arbitrary
nonnegative normal forces and the outer square Coulomb approximation
|Fx|<=mu Fy, |Fz|<=mu Fy, mu=52429/65536, at each point. This is a relaxation
of circular friction; feasibility does not establish full-body equilibrium.
Sampled-point support may differ from other possible contacts, so results
are limited to the declared contact set, not every possible stance.

Use world +X right/+Y up/+Z forward, normalized observed xyzw quaternions,
SI float64, g=9.81m/s². All26 masses and local COMs enter whole-body balance;
each ankle's complete descendant subtree enters its scalar moment equation.
Require ankle parent-frame rotation identity and strictly interior hard ROM.
For a joint axis a and point displacement r, generalized force is
a dot (r cross F). Static balance: tau + J_contact^T F = -J_COM^T m*g_world.

Compare the same pose/contact system with three ankle torque boxes: V8
target-only, V11 target-only, physical +/-220Nm. At zero velocity, target-only
box is K*(soft_limit-q) intersect physical effort limits. Ignore slew/rate,
residual, work and controller-specific target restrictions: this optimistic
box is a superset of producible steady target-only torques, never a controller.
Q16 K is exact; use the quantized observed q. Integer torque rounding is
bounded by1microNm and cannot decide a claimed gap<=0.0001Nm.

Minimize common nonnegative expansion delta of the two torque boxes, solving
the linear program in float64 (SciPy HiGHS). Preserve every matrix, bound,
primal/dual vector, status and residual. Feasible delta<=0.0001Nm supports
only this necessary relaxation. Delta>0.001Nm with primal/dual residuals and
duality gap<=1e-6 supports target-authority deficiency for this finite pose.
Other solver states or ambiguous tolerances are INCONCLUSIVE. An infeasible
physical box/support system cannot attribute the failure to target-only K.

H1: V11 torque box excludes an equilibrium allowed by V8/physical boxes;
separate effort compensation (or different posture) is warranted. H2: V11
box admits the same necessary equilibrium; feedback/balance-law dynamics
remain the next discriminator. H3: COM/contact geometry excludes equilibrium
even with physical torques; that pose cannot isolate actuator authority.
No hypothesis establishes native stability or every walking-relevant pose.

## Budget, controls and review

Nine pose/box solves, one exact output repeat. Analytic one-axis moment/sign
and zero-gravity controls; manufactured feasible wrench control; deliberately
unsupported COM negative control. Check all primal/dual residuals independently.
One fresh reviewer receives contract, code, matrices and source/raw seals with
at most one batched repair/re-review. No ancestor native reruns or portfolio
sweep. If apparatus cannot distinguish H1/H2/H3, stop INCONCLUSIVE and retain
the smallest missing boundary. Keep generated data/scripts outside Git.

Primary sources read2026-09-06:
[MIT humanoid notes](https://underactuated.mit.edu/humanoids.html) derive
whole-body COM/wrench balance and unilateral contact restrictions;
[PhysX5.6.1 articulation documentation](https://nvidia-omniverse.github.io/PhysX/physx/5.6.1/docs/Articulations.html)
distinguishes root/joint gravity forces and contact-free gravity compensation.
Neither makes a contact-free G(q) call a floating-base balance controller.
Pinned native5.9 behavior is not inferred from this static f64 calculation.

## Captured results and newly identified claim ceiling

External directory: `/home/kaifaty/NextEngine-training/body-load-authority-IscN5S`.
Manifest SHA256
`8e7f0e6b69ca9a27d18b414ac8d0f07029466f1993476e9628317ab3811f1d4b`.
Python3.12.13, NumPy2.5.2, SciPy1.18.0; source, matrices, primal/dual outputs,
raw inputs and pre-result contract are sealed. Repeat output is byte-exact.
Before any solve the initial script rejected incorrect expected foot names;
the assertion was corrected to actual ankle-roll/MTP body IDs. No result was
selected or discarded by that setup correction.

| Pose | V8 target-box expansion | V11 target-box expansion | Physical-box expansion |
| --- | ---: | ---: | ---: |
| V8 substep7200 | 0Nm | 18.5771521114Nm | 0Nm |
| V11 substep120 | 0Nm | 3.3954213378Nm | 0Nm |
| V11 substep200 | INCONCLUSIVE | INCONCLUSIVE | INCONCLUSIVE |

The last row returns SciPy status4 (HiGHS Unknown / primal Infeasible), not a
valid infeasibility certificate. Preserve it without a solver-option sweep.
Zero-gravity and manufactured wrench controls give zero expansion; the
deliberately unsupported wrench returns status2. Optimal solve residuals are
below1.3e-13 and duality gaps below8.6e-14 in the declared SI formulation.
At V8's pose the V11 ankle boxes end at9.475834/9.457381Nm; the minimum-expanded
solution uses28.052986/28.034533Nm. Expansion is a common box radius, not a
unique required torque split. At V11 step120 both deficits are on negative
torque limits; do not treat one fixed positive bias as the answer.

**Important model-validity restriction:** the bridge preserves legacy native
joint friction. The [earlier friction ablation](r8b-joint-friction-ablation-2026-09-05.md)
and reread pinned source show default0.05 transmitted-spatial-load friction,
not represented in the descriptor-only static program. Consequently r1's
phrase "necessary relaxation" applies to its frictionless-joint force-balance
model, not automatically to the full native plant. Native passive ankle torque
could supply part of the computed deficit. Neither H1 as native impossibility
nor unique causality of the observed fall follows from this calculation.
This limitation was found during current adjacent-layer inspection and sent
to the independent reviewer before its verdict; original contract/results
remain unchanged.

Pinned source commit517a0073715120e114ee055b63b26c95e00d9039:
`DyFeatherstoneArticulation.cpp` setup computes transmitted spatial magnitude
times coefficient times stepDt; the internal constraint loop clamps a
velocity-dependent friction impulse. Our bridge has no friction setter.
No new native ablation is needed to rediscover the already rejected zero-
friction fix. The SDK `computeGravityCompensation` documentation also explicitly
omits contacts/drives; floating-base output includes six root forces/torques.
Applying just that function's joint entries cannot be called support-aware
gravity compensation. The existing lab inverse-dynamics kernel is pinned to
24bodies/23DOFs; it was inspected but not reused for26body/25DOF V11.

Next engineering action: formulate a contact-aware support-effort experiment
on unchanged V11, with no fictitious root actuator, then test it natively
through the existing effort/rate/power/work intersection. Adding compensation
requires an explicit successor input/profile; adding effort after safety,
exceeding target ROM to synthesize torque, or silently reusing the23DOF kernel
is rejected. This is a mechanism experiment justified by the loaded failure,
not a claim that the static program proves it will succeed. Retain zero-
compensation byte regression and all loaded calibration criteria.

## Independent review and verification

Fresh report:
`/home/kaifaty/NextEngine-training/body-load-authority-review-RrbDxh/review.md`,
SHA256 `034ec5161849ea6a8bb0953fb61417163e1278e93f2329fbfcd254af2ac86455`.
Independent checker SHA256
`c2752e869739ed816584dfe24a4ee0ef115079ff1e0360d6d833cbcbf93ed5c8`;
result SHA256
`1a777a857b895aabdd290286462e5d5e1f9a065920fd1f8c0ccda809143cf21f`.
The reviewer independently reconstructs all nine systems using per-body
world-origin wrenches and translated subtree cuts, Hamilton quaternion
rotation and rational target-box calculations. All matrices/bounds match.
Exact rational arithmetic over stored binary64 data checks eight positive
primal/dual witnesses: largest residual1.32e-13, stationarity1.12e-15,
duality gap8.46e-14; dual sign violation2.45e-10 is within declared tolerance,
so these remain numerical certificates, not exact dual-feasibility proofs.

Verdict: frictionless two-pose deficits SUPPORTED_BOUNDED; native necessary-
relaxation correspondence INCONCLUSIVE because passive ankle friction is
omitted. This is a material finding, not a closed objection. Third-pose
status4 stays INCONCLUSIVE. No model repair, solver retry or native rerun was
performed; no re-review is needed for the explicitly restricted report.

PASS: external script syntax,10 sealed file checks, byte-exact repeat,
independent numerical/matrix review, git diff whitespace and139 local document
links. Native/Cargo/ProductChecks: NOT_RUN(NoExecutableRepositoryChange).
Existing native failure remains valid; no new controller has yet been tested.
