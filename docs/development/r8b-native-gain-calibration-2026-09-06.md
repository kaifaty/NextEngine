# BODY-GAIN-NATIVE-01 — contract revision1

Frozen before native motion, starting from repository `dfbfb04a` plus ADR-121
implementation. Consumer: determine whether the fixed eight-channel implicit
drive candidate is useful with canonical explicit240Hz PD and full safety.
This experiment is one part of the active **full model calibration** goal,
not a replacement completion criterion for that goal.

## Completion requirements for the full calibration goal

1. Current body geometry, axes/ROM, mass/COM/full inertia and collider/material
   closure must be valid and reproducible; historical V5–V8 audits are evidence
   only for unchanged fields and must be checked against the selected body.
2. All25 channels must have controlled native response in walking-relevant
   poses/loads, without hidden near-Nyquist oscillation or safety relaxation.
3. Loaded flat support, bilateral controlled heel rise, release/re-contact and
   bounded balance disturbances must work under the full native safety path.
4. Preserve predecessor identities/outputs; retain a reproducible selected
   calibrated body/controller profile and explicit compatible consumer boundary.
   Training itself is subsequent work, not evidence of calibration.

Current gaps:2–4 are not closed. Pinned Isaac traces are not native evidence.

## Fixed candidate and definitions

ADR-121 V10 clones V8, changing only eight D plus body/source identity. Source
SI values come from BODY-GAIN-02 summary SHA256
`09afa5068ba1f10a8430016c1008ea9e06dcfa4bb680d77a51737d5cb21231a6`;
quantize nearest ties-even to Q16 once. No tuning, pose-dependent inertia call,
anatomy/K/limit change. All unselected D retained, including both MTP channels.

Native SDKf259d3da… from task-state, Rust1.97.1, canonical engine right/up/forward
SI, integer control/canonical observations, fixed f32 physics,240Hz, four
substeps per60Hz reference. Existing per-iteration TGS, gravity, material,
fresh authored reset, free pelvis and real ground. Full target slew, effort,
effort-rate, velocity, power/work, ROM, contact-impact and terminal rules apply.
No reset import, episode padding or timeout treated as robust balance.

C1: exact factory delta/quantization/full compiled admission and consumer
binding; V8 standing control byte-exact to preserved output. C2: V10 completes
1800 standing ticks/7200 substeps without safety/terminal failure except timeout;
last10s root tilt<=2deg, torso<=1deg, both feet loaded, no increased MTP tail
RMS speed versus V8. Compute angles from full canonical quaternions, not the
controller's small-angle proxy. Failure of any element means no selection.

If C2 passes: execute existing6 toe-servo worlds and bilateral loaded transfer
(both ankle+MTP and ankle-only, plus zero input control), original15s horizon
and original report17 loaded heel-rise/re-contact criteria. These tests do not
certify all25 responses or disturbance recovery; those remain required next.
If C2 fails, retain failure and inspect earliest causal boundary; do not run
the same doomed standing prefix for every later transfer input.

H1: empirically improved D transfers and removes the loaded actuator problem.
H2: explicit sampled PD/coupling/contact invalidates the implicit-drive gain
transfer. H3: implementation/identity/measurement mismatch invalidates the test.
C1 separates H3; earliest native safety witness or C2 result separates practical
sufficiency from failure, but cannot alone identify every mechanism of H2.
Old nominal V8 is the successful control; preserved failed V9 is a negative
reference, not a required repeated sweep. No universal stability claim.

Budget: one V10 standing plus one exact repeat, one V8 control, conditional
servo/transfer matrix above. Freeze source/diff/commands/raw hashes, then fresh
independent review of gains, reference/safety operation order, first failure
and control identity. At most one batched apparatus repair/re-review; unresolved
load-bearing discrepancy -> INCONCLUSIVE. A verified native failure rejects
this vector; new experiments require a changed mechanism/discriminator.

## Captured native result

V10 standing stops at physics substep22 (0.091667s), before completing motor
tick6: `joint-safety: MOTOR_SAFETY_VELOCITY_VIOLATION`. Left MTP DOF6 has
position-18768urad and velocity9533921urad/s, exceeding the unchanged observed
limit8001000urad/s. The raw census contains550 applied effort channels. The
repeat is byte-exact. V8 completes7200 substeps with `terminal.timeout` and
is byte-exact to the preserved BODY-DAMPING-01 control. Thus the native C2
criterion fails; conditional servo/transfer tests are NOT_RUN by the frozen
stop rule, not passed or replaced with a shorter horizon. Independent executable
review below confirms the bounded failure, not a unique causal diagnosis.

External evidence: `/home/kaifaty/NextEngine-training/body-gain-native-JLdDAO`.
`contract-r1.md` preserves the pre-result contract. `manifest.json` seals the
complete tracked/untracked candidate diff, changed source hashes, commands,
binaries, descriptor, raw candidate/repeat/control and historical control.
Manifest SHA256:
`a68934d4fbccf48e49296500e3988b81fb43636d70b1884266ff8bcc7a2a9fcd`.
Its source snapshot predates these result additions; this is intentional.

Body hash `5506c4826e2015c5ab570aa12bf9281a41949adecd35dbd33eff7331f2186772`;
compiledV4 `c144eaae20059a941ef24ae2ac74c198427cc00a879453a9dc1ee3c44eb9f84b`;
contact `4bf364643d27df842565e07b067a53e796805d9cfe1e8e751b9a3473216147db`;
subject-zero standingV5 root
`9e970f5ae25cff133fa96e3c236329c04e80204aeb145a3cb0ba841e0b343ba2`.
The JSON harness version9 is the evidence format, not BodySchema revision;
`body_revision:10` and the above body/compiled roots identify the candidate.

## Independent executable review

One fresh review, no repair/re-review: C1 `SUPPORTED_BOUNDED` (CORRESPONDENCE,
exact integer checks, EMPIRICAL); C2 `REFUTED` (EMPIRICAL, CORRESPONDENCE).
All33 sealed entries matched. Independently recomputed SI-to-Q16 values,
compiled/contact/reference envelopes, identical fresh resets,6/1800 reference
and slew vectors and550/180000 V10/V8 applied efforts match exactly. One focused
native V10 rerun is byte-exact. No earlier ROM, velocity, contact, root or fall
failure was found, including raw contacts on the final unclassified substep.
Maximum prefix anatomical impulses2.476491/2.474830 Ns stay below6 Ns.

The left MTP pre-step22 state is q=-41436urad, v=-901416urad/s, target0.
P=1035900, D=-18032, requested1053932 micro Nm; the valid effort intersection
[-383539,616461] clamps the applied effort to616461 micro Nm. Thus the witness
is after the real rate-clamped effort, not a mislabeled raw PD command. This
does not separate contact, coupling and integration causes of the velocity jump.

Control tail recomputation from complete quaternions gives root/torso maxima
1.325099/0.149940 degrees and MTP RMS0.024579/0.111206rad/s. These are V8-only
facts. V10 tail, loaded-transfer and disturbance results do not exist.

External `independent-review-r1.md` SHA256
`3e4b13ec95db2d1011b0091d568331609bca418ff2816655f6e6c4c0779d4714`;
independent standard-library checker SHA256
`1928dafec621169c3c039e3f362b9188f56051b987292f7470d66a85702d1f09`.
No independent rebuild or PhysX-dynamics proof: the reviewer checked the sealed
binary, sources and canonical snapshots. The control shares the native solver
and is a non-regression control, not an independent physical oracle.

## Prior-art update and next discriminator

Read actual primary pages on2026-09-06, not only search snippets:

- [PhysX5.6.1 Articulations — drives/stability](https://nvidia-omniverse.github.io/PhysX/physx/5.6.1/docs/Articulations.html):
  implicit drives solve against end-step position/velocity; explicit drives
  hold effort computed at the beginning. Similarity requires low gains/small
  steps. Contacts/limits can still defeat stiff implicit drives, and extra
  iterations need not resolve competing hard constraints. This explains why
  Gain Tuner output is not a native-controller certificate, not this failure's
  unique cause or a claim that our pinned5.9 solver matches5.6.1 in every detail.
- [ovphysx stability guide](https://nvidia-omniverse.github.io/PhysX/ovphysx/latest/guides/articulation_stability.html),
  updated2026-08-21: consider timestep, coupled inertia and drive bandwidth;
  high gains are not a substitute for load compensation. Its armature,
  acceleration-drive and constraint-order suggestions are different model/
  solver choices, not authorized silent repairs to our frozen native profile.
- [Isaac Sim6 joint-gain tutorial](https://docs.isaacsim.omniverse.nvidia.com/6.0.0/openusd_tuning_tutorials/tutorial_06_joint_gains_tuning.html):
  validate isolated and simultaneous step/sine responses, contact behavior and
  velocity limits. Its numeric hand gains are not humanoid presets and its
  implicit-drive tuning heuristics do not establish explicit sampled stability.

Decision: reject this literal eight-D transfer; preserve V8 as nominal control.
Do not run another D-only sweep, increase safety limits, move mass, or replace
the controller with implicit/SPD solely because an isolated example succeeds.
The next materially different route must address native closed-loop bandwidth
(K together with D), with the previously measured coupled operator as a local
preflight only. Freeze one candidate and test actual full-safety native step/
sine response in ROM-interior poses before loaded support/transfer. A soft
unloaded success that cannot support gravity also fails the calibration goal;
that would motivate a separately specified load-compensation/controller change,
not weakening the load test. Contact/nonlinear clipping versus sampled-control
contributions remain unresolved. All25 channels, loaded transfer and disturbances
are still open; no new training/default body is selected.

## Implementation checks

PASS:155 native `next_motor` library tests;9 native example tests; all-target
native Clippy with `-D warnings`; formatting; `boundary-scan` (six boundaries),
`content-package`, `play`, `persistence-replay`. Exact V10 repeat and historical
V8 control pass as above; diff whitespace and745 local documentation links pass.
Logs are external. Broad `host-check`, performance,
training and MODEL-MIRROR admission are NOT_RUN (localized diagnostic change,
no corresponding new selection). Native standing success criterion FAILS;
passing software checks does not make the body calibrated.
