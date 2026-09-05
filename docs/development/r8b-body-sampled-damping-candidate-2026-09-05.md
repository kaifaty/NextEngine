# R8b stiffness-proportional sampled damping — contract revision 1

Research ID `BODY-DAMPING-01`; model preflight `SUPPORTED_BOUNDED`, native
sufficiency `REFUTED / EMPIRICAL / CORRESPONDENCE` after independent review.
Consumer: one whole-body damping correction, after the independent small-signal
model and rejected [foot-only candidate](r8b-foot-gain-candidate-2026-09-05.md).
This is not another local factor sweep: retain **all K, masses, COM, inertia,
geometry, joints and safety bounds**, replace all25 D by one derived rule.

For a constant symmetric positive response operator, D=c*K gives common modal
coordinates. With h=1/240, alpha=17/32 and k=h²*lambda(BK), write d=eta*k,
eta=c/h. The2x2 modal map has trace2-d-alpha*k and determinant
1-d+(1-alpha)*k. Strict unit-disk conditions for positive k give:
1-alpha < eta < 2/k - 1/32. Thus a common eta exists if largest k<4.
This is a conditional linear derivation, not a native humanoid stability proof.

From the two frozen operators, the largest eigenvalues of
h²*sqrt(K)*sym(B)*sqrt(K) are2.32347418 /2.32338918. Round the larger upward to
2.33 (two decimal places), choose the midpoint of the admissible interval:
eta =7/32+100/233; c=eta/240 =4831/1789440 seconds.
The exact candidate is D_q16=round_ties_even(K_q16*4831/1789440) for every joint.
There is no search or post-result tuning. Symmetrization is used only to choose
this candidate constant; evaluate its actual rounded values using BOTH original,
unsymmetrized measured operators. Require both spectral radii<1 before native
evaluation; otherwise reject this candidate under this ID.

All anatomy and stiffness remain intact. A new opt-in BodySchema identity is
required before native rollout; body-only hashes must not masquerade as V8.
Explicitly extend diagnostic admission through Accepted architecture workflow;
do not change game/training defaults or reuse old weights. Existing reference
equations and contact/safety rules remain. Exact V8 outputs are rollback controls.

Native verification budget for the one candidate: original30-second standing,
six FOOT-SERVO-01 worlds, bilateral15-second loaded-transfer inputs v1/v2,
one repeat/control and focused positive/failure tests. Original safety and
loaded heel-rise/re-contact criterion stay frozen. Stop on failures, retain
every result. Successful matrix poles or nominal stance are not full promotion.
No second damping candidate under this ID. Independent review checks equation,
rounding, actual vector/model, implementation delta and native controls before
any selected-body or repaired-stability claim. At most one initial review and
one batched repair/re-review per executable lineage.

## Independent model preflight

Frozen contract SHA256
`2f7dfaf1ed7bc5c0d7ef7ac25f5a6dad7f53bb7a99dace75c6be5f0e490d5101`.
Fresh reviewer verified all ancestor hashes without rerunning native worlds,
derived the strict interval independently via a unit-disk/left-half-plane
polynomial transformation, and reconstructed both B operators from raw trials.
Independent50-basis16-kick/drift propagation gives candidate radii
0.9998054202778694 /0.9998058239065288; maximum eigenpair residuals
2.36e-15 /3.00e-15. Original damping reproduces53.85248035524876 /53.84945569717711.
Exact rounded D in DOF order:
`[79618,61925,53079,88465,70772,53079,4423,79618,61925,53079,88465,70772,53079,4423,53079,53079,53079,31847,28309,1548,24770,31847,28309,1548,24770]`.
No load-bearing mathematical finding. The small1.94e-4 radius margin is not a
robustness certificate. The symmetric-constant-B analytic statement and rounded
unsymmetric finite numerical result are separate evidence classes.

## V9 implementation and native observations

[ADR-120](../architecture/adr/120-stiffness-proportional-damping-diagnostic.md)
implements only the explicit diagnostic: full V9 compiled admission and
separate contact/standing identities reuse old algorithms. Exact body-delta
tests preserve all anatomy/mass/K/safety. No environment/default is switched.

Native standing stops at substep18 (0.075s), bilateral ankle-pitch velocities
8.101706 /8.108368rad/s, exceeding unchanged8.001rad/s observed tolerance.
All four loaded-transfer inputs and the zero transfer control stop at the
identical18-step prefix, before tick301's first transfer input. They therefore
cannot establish heel-rise/re-contact; this is an initial standing regression.
All three grounded toe-servo worlds stop at18; all three raised worlds stop
at3 on ankle-pitch velocity, without ground endpoints/nonzero canonical impulses.
The candidate is not a replacement for nominally successful V8.

The existing independent-integer audit helper reconstructs1575 effort channels
across the six servo worlds. Its shared-helper summary is a consistency check,
not a substitute for fresh executable review. Complete standing repeat is exact.
V8 standing, toe-servo and first-step outputs are byte-exact to their captured
ancestors; V8 transfer control matches all3600 baseline steps. V9 descriptors
differ from V8 only in damping and body/compiled identity fields.

External artifacts: `/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/body-damping-01`.
`SHA256SUMS` freezes source and raw native outputs for executable review;
`summary.json` additionally records helper-based transfer/prefix checks and input
hashes. Source/commands retain SDKf259d3da…, Rust1.97.1, Python3.11.15.

Reproduce: `export_biomechanics_body_v9`; standing example arguments
`9 0 0 sampled-v4 per-iteration unchanged`; toe-servo `--body-v9`; transfer
`left/right [neutral-toe] --body-v9`. All use `--features physx-sdk` and the exact
SDK from task-state. The descriptor is CompiledV3 inspection; native traces bind V4.

PASS so far:152 native motor library tests, native all-target Clippy, focused
example tests,11 Python tests, Ruff, boundary/content/play/replay. Initial build
caught a u32/u64 damping field mismatch; corrected to u128 product/u64 result
before any native candidate evidence. No safety or native criterion weakened.
Broad host-check/performance/training not run for this package-local diagnostic.

Remaining distinction: local interior-pose zero-reference linear stability does
not establish a constrained, cold, grounded reset driven immediately toward
nonzero knee/ankle targets. Do not retune this D vector or call it a fixed body.
Next cheapest discriminator is a separately frozen reference-startup comparison,
not more anatomy/mass changes or a second gain vector under BODY-DAMPING-01.

## Independent executable review and final boundary

Fresh reviewer checked all26 manifest entries before inspecting source/results,
independently reconstructed4275 recorded effort applications across standing,
six servo worlds and five transfer worlds using rational rounding and separate
constraint intersection. All references, targets, DOF order, preceding states,
rate/power/work bounds and first observed failures agree. No preceding contact,
root/fall/world-bound failure was found in the standing prefix. All25 body
coefficients and V3/V4/contact/reference identities were independently rebuilt.
One additional reviewer native standing rerun is byte-exact. No load-bearing
finding or repair; no second review pass. Cold raised trials retain self-contact
records, so no contact-free/native universal stability claim follows.

Exact V9 identities: body
`b2ee81f6e38efe67116ced2071efc92bf27754222a1821ea8e769367c47c1dbe`,
compiledV4 `fc5650bd5f8d1e5811dba9e7822bb31ba3f9d2920689a8c6a0b0070d17135115`,
contact `cf483f5fe13431347c85ef9b50442671402d9010bebf9fcf0bae8220b5112ce8`,
subject-zero standing reset `cab7db30dcee32940b2c47116bf453cd293e4c24414a905482c85d34071c5f38`.
Standing/repeat SHA256
`68e08e58d1fa3ecb222c59d6a6c55822ff2142cb0130d2d5f9908ce19ff9af95`;
descriptor `e1eaa86ae3527345bfea30bfb00e282a743726f3833217fd0cbc7ede68b163a0`;
servo `1b59be47146fa7d519bea266df4131cd1cac41f71a8dde20223920f13fcdd2ce`;
summary `6616280dea117a030e8b278b2eba168b27802c06156991fe07673148748115fb`.

Subsequent [startup-ramp diagnostic](r8b-standing-startup-ramp-research-2026-09-05.md)
adds a separately labeled optional input to the same standing harness. Thus
the first manifest's standing-source hash is the historical reviewed surface,
not the final file hash; its no-ramp output is rechecked byte-exact under the
successor source. Body/consumer implementation hashes remain unchanged.
Do not select V9, rerun the same damping rule or transfer its coefficients into
the learning environment. Native nominal V8 remains the rollback/control.
