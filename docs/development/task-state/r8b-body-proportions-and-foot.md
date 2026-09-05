# R8b body proportions and foot — task state

| Field | Value |
| --- | --- |
| Status | `GAIN_TUNER_AUTO_REFUTED / V8_NOMINAL / NO_NEW_TRAINING` |
| Updated | 2026-09-06 |
| Scope | Improve actual human-like BodySchema, foot mechanics, mass/inertia and leaning; visualization alone is insufficient |
| Authority | Working context only; current SPEC/ADR and exact artifacts take precedence |

## Resume in 60 seconds

- Latest result: whole-body sampled damping V9 is implemented as an opt-in
  diagnostic (ADR-120), **not selected**. Independent local-model preflight
  passes but native stance fails at18/240s, both ankle-pitch velocities>8.001rad/s.
  All loaded-transfer cases fail before the input starts. V8 controls exact;
 152 native tests and relevant checks pass. See reports20–22, not old next-action text.
- Separate60-tick startup ramp on V8/V9 gives identical first-step knee ROM
  violations (-14/-13urad; allowed minimum0 with10urad observed tolerance).
  This rules out the ramp as sufficient, not all smooth reference schemes.
  User-authorized [Gain Tuner trial](../r8b-gain-tuner-tool-check-2026-09-05.md):
  isolated schema fix now discovers26 links/25DOFs; mass queries/control pass.
  [One-DOF oracle](../r8b-gain-tuner-one-dof-research-2026-09-05.md) refutes inertia/UI units; independent controls pass.
  User authorizes separate Isaac Sim6 install on RTX3080 (3090 later); no driver/training change.
  [Isaac6 installed separately](../r8b-isaac6-install-2026-09-05.md): CUDA/smoke120 steps/Gain Tuner3.5.2 load PASS; old5.1 preserved.
  [Isaac6 one-DOF test](../r8b-gain-tuner6-one-dof-research-2026-09-06.md): inertia/K pass, D still180/pi too large. Next scoped UI unit fix; no installed fix/training yet.
- Missing torso/head in the old origin-line plot caused the apparent
  leg/trunk disproportion. All 19 physical colliders are now drawn.
- Initial V4 measures 170 cm stature, 86.5 cm hip, 139.65 cm shoulder,
  40.8 cm thigh and 39.6 cm shank. Do not shorten legs to repair that picture.
- Opt-in physical BodySchema V5/V6/V7 is implemented under ADR-114/115/117. Existing
  environments/checkpoints still use V4; full goal remains active and no
  optimizer is running. Do not treat the V5 diagnostic as a learned fix.
- Independent all-1200-frame mass audit: total 75.337 kg, legs 38.923%,
  pelvis/torso/head 51.241%. Same-pose carrier relocation changes whole COM
  by at most 0.756 mm, not evidence against dynamic inertia/control effects.
- Final torso tilt 29.94 degrees versus pelvis 10.44 degrees. Walking reward
  sees pelvis orientation but not torso orientation; causal repair untested.
- V4 torso collider is 90.7 mm behind pelvis collider. V5 places pelvis
  collider Z at source COM -70.7 mm and torso collider local Z at rib-bounds
  midpoint +8.556 mm; the gap becomes 21.444 mm. Joint anchors/COM unchanged.
- V4 has reversed hip/shoulder/elbow flexion and hip-adduction directions.
  V5 reverses those eight axes only. Bilateral production native state import
  proves the corrected directions and preserved backwards knee flexion.
- The preserved successful gait still uses a rigid box. V8 now adds a separate
  forefoot and MTP hinge; native kinematics and30-second nominal standing pass.
- V6 now carries full principal moments / frames for all six non-diagonal
  tensors. Integer and compiled-float reconstruction error <= 1 micro kg m²
  per component; old masses/COM/geometry and V5 descriptor bytes unchanged.
- Native 30-second procedural standing completes on V5 and V6. Final torso
  tilt is 7.365° / 8.078° respectively: inertia repair does NOT fix upright
  posture. The unchanged reference drives only knees/ankles (unchanged axes),
  so it is usable for this neutral control; old learned weights remain
  incompatible. No new training environment is selected.
- New discriminator: existing native TGS reports mean root omega +0.7604 rad/s
  during nearly stationary stance. Per-iteration external-force scheduling
  reduces this mean discrepancy nearly to zero with unchanged body/control.
  Both runs last 30 s; experimental final torso tilt 0.6503°, but last-10-s
  maximum 8.5946° means oscillation remains. ADR-116 now implements the exact
  behavior as opt-in CompiledBodySchemaV4 with a distinct outer hash; all 7200
  steps match the experiment, and all old-scene steps / hashes remain exact.
- k=2 hip feedback on the new profile stops at tick596 on foot impact:
  summed impulses 6.602703 / 6.338131 N s at substep2382 exceed the unchanged
  6 N s limit. All-zero targets instead violate knee ROM at substep1 (-12/-11
  microradians). Neither is an accepted stance. No gain or safety change.
- New actuator discriminator: shoulder-yaw explicit PD generates ~24 Hz actual
  rotations near 10 rad/s. Nearly passive shoulders remove that mode but drift
  to ROM; gains /16 keep both shoulders within 0.008 rad for 30 s. Trunk/leg
  oscillation remains. With the same k=2 hip feedback, left-foot impulse is
  6.952268 N s at substep2229; termination at2232. Shoulder repair alone is not
  the balance fix. These are newly hashed diagnostic schemas, not selected V7.
- Coupled response experiment v1 is complete and independently reviewed:
  94 reconstructions and zero controls are exact, but 0.01/0.02 N m central
  maps differ by 73.302%, failing the frozen 5% criterion. Final velocity
  rounding cannot explain this (0.001725 bound vs1.420739 difference norm).
  All trials retain four loaded points per foot; friction/constraint/solver
  sensitivity is unresolved. Do not fit gains/inverse mass from this matrix.
- Cold comparison completed: grounded central maps differ 45.477565%, raised
  ground-contact-free maps 0.0185195% (max-norm denominator). Independent review
  and exact native rerun pass. This supports a cold contact-boundary contribution,
  not the mechanism or warm standing cause; limit kinks remain possible.
- Joint-friction-zero ablation is REFUTED as a sufficient fix: complete cold
  outputs unchanged except metadata. Standing lasts 30 s but torso last-10-s
  peak is 11.246 degrees; k=2 hip feedback fails on impact at substep236.
  Temporary native setter and metadata are reverted. Restored cold and warm
  outputs are byte-exact. Do not select zero friction from the upright final frame.
- 16->64 position iterations is also rejected as sufficient: cold discrepancy
  rises to87.716%, hip feedback still fails impact at6784. Patch reverted and
  original cold hash exact. Stop using contact-map smoothness as a prerequisite
  for standing/training or sweeping iterations to repair actual motion.
- Quiet upright candidate now passes the frozen nominal30-second test:
  shoulder-yaw gains/16 plus damping/4 on eight hip/knee/torso channels, then
  k=2 hip position feedback WITHOUT its old omega/5 term. Last10-second root
  max1.303656 degrees, torso0.181539; selected-eight tail RMS0.00391637 rad/s. Both feet
  loaded and essentially flat. Whole-run torso transient still8.192405 degrees.
  Independent native rerun byte-exact. This is not robust balance or learning.
- Damping-only suppresses near-Nyquist RMS98.59% but leans3.44 degrees. Keeping
  the hip rate term restores upright appearance but tail joint RMS0.897787;
  do not select it by its final frame. No mass/inertia/geometry changed.
- ADR-117 now implements the exact quiet candidate as V7 body and standing V2
  in the motor library, with full compiled-input validation and subject/reset
  state binding. All7200 native steps/1801 samples equal the candidate. Old
  modes repeat byte-exactly. No training environment or game route is switched.
- Next: disturbance/foot mechanics and a separately identified learning environment.
  Do not repeat the finished mass audit,
  axis-sign investigation or diagonalization to tune the leaning symptom.
  All runtime changes need new identities and native checks before training.
- Foot input audit now independently closed: upstream2016/2023 toe inertia
  becomes planar [100,1100,1000] micro kg m², not a finite-thickness repair.
  Source toe COM is3.4 mm past the current box front; visual toes reach45.718 mm
  past it. Do not simply partition the old box. The finite-volume articulated
  proxy is implemented as V8 (report15). ADR-119 adds raw anatomical-foot6 Ns
  aggregation with preserved pair limits and standing V3 (report16). V8 lasts30s;
  torso tail max0.14994°, each foot active7200 substeps; toe mean loads12.47/14.83N.
  MTP velocity RMS0.0247/0.1112rad/s still has near-Nyquist power: do not infer
  quiet joints or retune from a picture. Next loaded heel-rise/re-contact, then
  disturbances. Old V7 output remains byte-exact; no training selection.
- Loaded transfer fails (report17); FOOT-SERVO-01 is INCONCLUSIVE under its
  empty-contact firewall (report18). First-step inputs (report19) independently
  resolve zero->zero, knees->MTP-1.095rad/s, ankles->nearly-8, small ankles->-0.153.
  All125 effort channels and native controls exact. Sampled-control work is
  now completed in reports20–22; do not restart proximal input isolation.

## Required context

1. [Routing](../../architecture/agent-routing.md), current body/motor rows if
   changing physics; read their full SPEC/ADR set before that next change.
2. [SPEC-35](../../architecture/35-deterministic-humanoid-training-substrate.md),
   [ADR-069](../../architecture/adr/069-biomechanics-body-schema-v2-and-solver-projection.md),
   [ADR-102](../../architecture/adr/102-biomechanics-neutral-self-clearance-successor.md),
   [ADR-106](../../architecture/adr/106-walking-reference-and-leg-clearance-audit.md).
3. [Current review/evidence](../r8b-body-proportions-and-foot-review-2026-09-05.md)
   and [preserved walking result](../r8b-known-candidate-reuse-2026-09-05.md).
4. [Reviewed mass/posture evidence and implemented V5 discriminator](../r8b-mass-and-posture-research-2026-09-05.md).
5. [ADR-114](../../architecture/adr/114-anatomical-axes-and-sagittal-body-proxies.md).
6. [V6 implementation and standing discriminator](../r8b-principal-inertia-and-standing-2026-09-05.md)
   and [ADR-115](../../architecture/adr/115-full-principal-inertia-body-successor.md).
7. [Reference failures and isolated TGS mechanism](../r8b-upright-reference-research-2026-09-05.md).
8. [Implemented force-schedule profile and rejected reference follow-up](../r8b-force-schedule-profile-2026-09-05.md)
   and [ADR-116](../../architecture/adr/116-explicit-per-iteration-force-scheduling.md).
9. [Shoulder actuator discriminator and remaining coupled response](../r8b-actuator-oscillation-discriminator-2026-09-05.md).
10. [Rejected local coupled map and independent correspondence](../r8b-coupled-effort-response-2026-09-05.md).
11. [Cold contact-boundary result](../r8b-cold-contact-response-2026-09-05.md)
    and [rejected joint-friction ablation](../r8b-joint-friction-ablation-2026-09-05.md).
12. [Rejected iteration intervention](../r8b-contact-iteration-discriminator-2026-09-05.md),
    [effective coupled damping discriminator](../r8b-coupled-damping-discriminator-2026-09-05.md)
    and [quiet upright hip reference](../r8b-hip-rate-feedback-discriminator-2026-09-05.md).
13. [Implemented V7/standing V2](../r8b-body-v7-upright-profile-2026-09-05.md)
    and [ADR-117](../../architecture/adr/117-quiet-upright-body-and-standing-reference.md).
14. [Reviewed foot source/geometry inputs and successor constraints](../r8b-foot-successor-inputs-research-2026-09-05.md).
15. [Implemented articulated foot and checks](../r8b-articulated-foot-v8-2026-09-05.md)
    and [ADR-118](../../architecture/adr/118-articulated-volumetric-foot-body.md).
16. [Articulated contact/standing result](../r8b-articulated-standing-2026-09-05.md)
    and [ADR-119](../../architecture/adr/119-articulated-foot-standing-diagnostics.md).
17. [Failed loaded-transfer discriminators and next research boundary](../r8b-loaded-foot-transfer-2026-09-05.md).
18. [Unloaded coupled-foot response and toe-servo research](../r8b-toe-servo-research-2026-09-05.md).
19. [Resolved first-step proximal input isolation](../r8b-foot-first-step-research-2026-09-05.md).
20. [Sampled-control local model](../r8b-sampled-foot-control-research-2026-09-05.md)
    and [rejected foot-only preflight](../r8b-foot-gain-candidate-2026-09-05.md).
21. [V9 damping-only implementation and native rejection](../r8b-body-sampled-damping-candidate-2026-09-05.md)
    and [ADR-120](../../architecture/adr/120-stiffness-proportional-damping-diagnostic.md).
22. [Cold reference startup comparison](../r8b-standing-startup-ramp-research-2026-09-05.md).

## Decision and remaining uncertainty

- Observation: same native frame with actual colliders has a complete trunk
  and head; the parent-origin diagram does not represent physical dimensions.
- Decision: correct shared diagnostic geometry and opt-in physical V5 axes /
  sagittal proxies; preserve V4 and the successful checkpoint as control.
  No new anatomical segment lengths or arbitrary mass redistribution.
- Rejected: shortening legs from the stick picture; compulsory flat feet in
  every gait phase; changing body bytes under the old identity.
- Remaining: benefit of forefoot articulation versus contact-shape/parameter
  improvements. The current report cannot distinguish these physical options.
- Reconsider proportions only on landmark/collider/source measurements, not
  a learned pose or a skeleton missing geometry.
- Historical V4/V5 solver tensor loss is large in some rotation directions
  (foot ~24%, forearm-hand ~42% relative quadratic-form error). It is not a
  measured gait error. Full projection is implemented in V6, preserving
  source masses/COM. Its first compile exposed V3 physics descriptor's
  erroneous V1 exact-norm check for principal frames. ADR-115 isolates a
  bounded mass-frame validator; initial/shape poses keep their old rules.
  Native creation/motion and compiled payload reconstruction pass. No direct
  SDK mass-property getter was added; the existing setCMassLocalPose /
  setMassSpaceInertiaTensor path was inspected in pinned 5.9.0 source.
- Source toes have a non-realizable diagonal inertia; do not copy it into a
  new dynamic MTP link. Current merged foot tensors pass that elementary
  check. Source2023 audit and V8 finite-volume proxy are complete (reports14/15).
- Success oracle for physical successor: loaded flat support, controlled
  heel rise, release/re-contact and bilateral symmetry, original safety;
  later learned quality needs a new compatible generation.
- Constant ankle +/-70 mrad and hip +130 mrad all fall. Hip feedback k=2
  survives at 5.47° pelvis tilt, k=4 falls. Do not tune against the old biased
  velocity or reuse these as selected profiles. Ordinary gravity deflection
  alone is no longer an adequate explanation. The one-flag TGS experiment
  changes force scheduling and is now implemented under a new identity with
  preserved old descriptor/native traces. Balance validation remains open.
  No global bridge flag or fake measured velocity fix. Do not replace the
  explicit PD with SPD based solely on that paper: clipping/contact and its
  anchored-root demos do not establish our free-standing result.

## Handoff

- Rust V6 body and mass-frame repair plus opt-in compiled V4 force schedule
  implemented. No environment/default/runtime authority changes.
- Historical V5/V6/force-schedule native and host checks passed; exact records
  remain in reports6–8. Current body/contact checks are in reports15–17.
- External V5 descriptor/image: `/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/body-v5-01`.
- V6 descriptor and both original standing outputs are in `body-v6-01`.
  New `upright-reference-01` includes every-step joints, effort, raw contacts
  and root pose/velocities, failed references, one-flag TGS experiment and
  `standing-tgs-comparison.png`. Full zero-offset JSON repeats byte-exactly
  after the temporary flag is reverted. Exact hashes / reproducible temporary
  patch are in that report. `profiled-*.json` now records the real opt-in
  implementation and rejected reference cases; hashes are in the newer report.
- New outer compiled V6 hash:
  `66d6a5b01ea1a26294b92e050bbdc79556cffd63cabdd7382d15ea0543d6c7ed`.
  Use explicit Cargo SDK feature invocation, not the shared unhashed debug
  binary during host-check: its no-SDK configuration can replace that path.
- Build with `NEXTENGINE_PHYSX_SDK_DIR=/home/kaifaty/.cache/nextengine/physx/sdk/f259d3da157cc6120b378b53ee14c10805be89698242b03d7417f699ca711c3b`.
  Global active SDK has a different build profile and is correctly rejected;
  do not change global active locator or weaken manifest validation.
- Initial previews and failed CLI artifact retained externally; use audit-04.
- Initial V8 foot implementation was kinematic-only. Its144-test
  suite failed rear mass-volume containment (143 passed); collider top +5 mm
  repairs it without changing sole/mass/COM. Final144 tests, native Clippy,
  boundary/content pass. Exact hashes and remaining consumers are in report15.
- Historical actuator/cold-map/friction/iteration and V6-reference artifacts
  remain external with exact source/hash/check records in reports9–12. Negative
  paths above remain binding; do not repeat their amplitude/gain/iteration sweeps.
- `body-damping-01` holds V9 native results, review SHA256SUMS, four exact V8
  controls and independent4275-effort-channel correspondence. Standing SHA
  `68e08e58d1fa3ecb222c59d6a6c55822ff2142cb0130d2d5f9908ce19ff9af95`.
  The later harness-only startup option preserves this no-ramp output exactly.
- Current implemented command: `cargo run -p next_motor --features physx-sdk
  --example probe_biomechanics_body_standing -- 7 0 0 upright-v2 per-iteration unchanged`.
  V7 body hash43d9f3e1…, compiled hash5bc1bd85…, subject-zero reference root
  cec3d35b…. External `body-v7-01/standing-subject-bound.json` SHA6eef44ec…;
  descriptor SHA18de43f4…. Full exact hashes and checks in report13.
  First subject-state collision test failed and was repaired by explicit
  PersistentId binding, not a weaker test; physical trajectory unchanged.
  Final138 native tests, five example tests, native all-target Clippy,
  format/diff/links and play/replay/content pass. Broad host-check passed
  workspace Clippy/tests, failed final source-layout scan on old diagnostic
  `#[path]`. Replaced with conventional nested modules; all six boundary
  checks and native example/Clippy revalidation pass. Full wrapper not rerun;
  exact failed/repaired status is preserved in report13, session32268 closed.
- Mass audit is independently `SUPPORTED_BOUNDED`, with no load-bearing
  arithmetic defect. Its entry-script hash omits helper hashes; independent
  recomputation closes this result only. Do not repeatedly rerun/re-review it.
- Routed SPEC/ADR/root/R8 and Rust reads are complete; do not restart on each continuation.
