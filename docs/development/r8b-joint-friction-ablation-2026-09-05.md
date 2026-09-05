# Joint-friction ablation — contract revision 1

Research ID `r8b-joint-friction-ablation.v1`, after the independently checked
cold-contact comparison. The ground-contact-free central map is consistent (0.019%)
but the grounded map is not (45.478%). This localizes a boundary contribution,
not its mechanism or the cause of the whole standing trajectory.

Pinned PhysX source commit `517a0073715120e114ee055b63b26c95e00d9039` initializes
joint friction coefficient to 0.05 (`DyArticulationJointCore.h:118`). Our native
v2 articulation constructor does not set it. The pinned public header describes
that deprecated friction as proportional to transmitted spatial load. The
solver chooses the new model only if its static or viscous parameter is
nonzero (`DyFeatherstoneArticulation.cpp:4306`); zeroing the new API alone does
not disable the old default. This source matches the numeric tag discussed in
[upstream issue 498](https://github.com/NVIDIA-Omniverse/PhysX/issues/498), opened
2026-08-04 and still open when inspected. Treat the issue as a report, not a
maintainer-confirmed global diagnosis. Local source inspection is authority
for this pinned implementation detail.

## Hypothesis and finite test

Hidden transmitted-load joint friction may cause the ground-dependent small
effort sensitivity. Alternative: ground/contact or joint-limit solver response
persists independently of that coefficient. Change only v2-created joints to
explicit coefficient zero using the pinned legacy setter, before insertion in
the scene. Do not change foot/ground material friction, gravity, timestep,
iterations, limits, masses, inertia, targets, PD or safety.

This is a **temporary source-patch discriminator**, not a new accepted physics
profile. Explicit diagnostic metadata and bridge/main/helper source hashes
identify the override; normal compiled/body hashes alone do not identify it
and must not admit these artifacts. No training is running or launched.

Budget: one paired cold-response run and one 30-second original-reference
standing run with the gain-/16 shoulder candidate; if the latter completes
safely, also test the existing k=2 hip-feedback reference once. Each standing
run retains the same safety and stops at its first terminal. No gain sweep.
Retain all failures. A cold grounded relative map difference <=5% supports
the local friction mechanism; failure refutes sufficiency. Standing is judged
separately by full trajectory, not a final upright frame. One independent
review, at most one batched repair/re-review.

Rollback: remove only the temporary native setter and metadata after evidence
is captured, then require the original cold artifact to repeat byte-exactly.
Do not ship a global flag or change existing profile identities. If an actual
profile is warranted, it requires explicit identity/ADR and native regression
coverage through the normal architecture workflow. The full body/balance/feet
goal remains unproven by this bounded experiment.

## Result: sufficiency refuted; temporary patch reverted

Zero legacy joint friction leaves the entire cold pair **identical** after
removing only the two explicit override metadata fields. Grounded relative
difference remains 45.4775646455%; raised remains 0.0185195011%, using the larger
norm denominator. Thus zero friction is not sufficient to repair this cold
response. Why it makes no first-step difference is unresolved; do not infer
the initialization of transmitted-force caches from this observation.

The 30-second standing reference completes 7200 substeps with timeout. Root
tilt peaks at 9.558826 degrees (substep392), finishes at 0.831144 degrees;
last-10-second torso tilt still reaches 11.245947 degrees. This is changed
motion, not stable upright balance. The permitted k=2 hip-feedback follow-up
terminates at substep236, about 0.983 seconds, on `terminal.contact-impact`.
Its final root tilt of 0.203267 degrees does not override that failure. The
original-friction hip control lasted 2232 substeps; do not select this variant
on the basis of its last frame.

Independent `friction_review` verified the patched v2 constructor is on the
selected CompiledBodySchemaV4 creation path, all supplied hashes, trial
commands and boundary controls. A focused native rerun was byte-exact. No
load-bearing defect found; conclusion is bounded empirical refutation, not
proof that joint friction never matters.

External root:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/joint-friction-ablation-01/`.

| Evidence | SHA-256 |
| --- | --- |
| Frozen contract before results | `9bc3a4b1403d4d3e988d4753c6803415beb4d81e4895804ace31d96fec290c9f` |
| Temporary bridge source | `adb56a1a0d13865290c1ef5f04fe14fc48904dc5b873d1e9a99049ee07931b57` |
| Temporary native main | `c75c5f78a2ef098cddf74ce249d2605bc59babbf26101bd114a04943e7fe9506` |
| Temporary response helper | `535ae0e2113e3db7b2d658f1d88f54b8e18af80753e56e9b2a4d95d7ffc20513` |
| `cold-zero-friction.json` | `f16a3019522bc4b3991e6294b33936850135edae1dee36dff95035cae9f95550` |
| `standing-zero-friction.json` | `0beb8fd00bbb0355a7e3185cfd9168e3250ae739c58bd783ff88d2dd7f42e01e` |
| `hip-feedback-zero-friction.json` | `db288f758cf2c13a2b75df61db50769cc46afda044e89800e987d484dd016ef7` |
| `cold-analysis.json` | `bc32398a5c9a602b18df059c32345dcfcfc60aaff1ddfb34901febcae962ef14` |
| `restored-cold-response.json` | `04d11922098b283672a024da35c06b8c2d77e7ec51b58c4c13ec5d79355b0e0f` |

The sole behavioral patch was `joint->setFrictionCoefficient(0.0F);` in
`crates/physics-physx-ffi/native/nextengine_physx_bridge.cpp`, v2 articulation
construction after angular `setMotion` and before `setLimitParams`. Added
metadata was `native_source_override: r8b-joint-friction-ablation.v1` in main
standing output and helper response output. Both source changes are reverted;
the native bridge has no remaining diff. Restored cold output matches the
original SHA exactly, as does the old warm experiment (see paired report).
Do not use these temporary outputs as ordinary hash-admissible training data.

The [official articulation stability guide](https://nvidia-omniverse.github.io/PhysX/ovphysx/latest/guides/articulation_stability.html)
(updated 2026-08-21, inspected this investigation) discusses solver iteration,
timestep and gain/inertia interactions. This is adjacent guidance, not proof
of our cause or authority to switch the pinned profile. Remaining alternatives
include contact constraint convergence and coupled explicit-control effects.
The smallest proposed next discriminator is a separately identified bounded
solver-convergence comparison, retaining the successful raised control and
original safety. No iteration/solver/gain change has been selected or run.
No training launched. Broad ProductChecks were NOT_RUN for this reverted
ablation; native byte-exact rollback is the relevant non-regression result.
