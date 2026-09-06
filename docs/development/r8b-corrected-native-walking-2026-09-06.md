# Corrected-body native walking integration

Status: `NATIVE_25_ENVIRONMENT_IMPLEMENTED / PPO_SMOKE_COMPLETED / TELEMETRY_REPAIR_REPLAY_PASS / WALKING_NOT_LEARNED / FULL_CALIBRATION_OPEN`.
Authority: [ADR-126](../architecture/adr/126-corrected-body-walking-environment.md).
This continues the [adapter foundation](r8b-corrected-body-adapter-2026-09-06.md).

## Decision and implementation

Connect exact V11 anatomy/gains and CompiledV4 per-iteration forces through a
separately identified walking V9 environment. No gain sweep, support-feedforward
promotion or reuse of old23-action weights. Existing walking referenceV1 and PD
provide the baseline; this is not the diagnostic standingV6/support controller.
Zero-action survival is a baseline measurement, not a claim of full calibration.

Native protocol now advertises25 actions/94 observations for V9. Descriptor-order
positions, velocities, targets and action quantization remain consistent. Separate
rearfoot/MTP shapes feed whole-foot minimum height and anatomical contact/impact
classification. Link-speed slip proxies use a per-foot maximum and never count
four support feet. Failure padding does not advance anatomical contact continuity
or fabricate load impulses. V8 formulas/coefficients and25-channel normalizations
are explicit in the new manifest. All old profiles retain their original code path.

## Evidence and checks

- Five added native motor tests pass: exact descriptor/body/layout closure,
  direct25-channel PD/native snapshot equality, invalid-batch atomicity/partial
  reset, heel-rise versus whole-foot swing geometry, exact native prefix repeat
  and terminal/reset behavior. All166 motor library tests pass with `physx-sdk`.
- Seven headless protocol tests pass, including V9's25/94 handshake and old V8.
- 48 focused Python tests pass, including Q30 action units, final observations,
  timeouts and a smoke-class prohibition on walking selection/certification.
- Initial V9 multi-process comparison failed immediately at tick0/slot1:
  the direct client's profile whitelist converted normalized actions to1e6 rather
  than Q30. Native/PPO-side raw actions were correct. Add V9 to the explicit client
  mapping and test it; no physics or safety change. Do not hide this mismatch by
  weakening equality or reducing actions.
- Corrected control:640 transitions/89 terminal resets, all StepResult fields
  exact against independent raw-native workers; step-root sequence SHA256
  `db3789d1767a697bc6342fa7178ddf56c69aedf5b6bee651e5b18bff2bfce144`.
  This uses seed44, report-only4 environments/2 shards. Frozen smoke uses32/4.
- Old V8 control remains exact:640 transitions/29 resets, unchanged step-root
  sequence SHA256 `558368fb38af52123877249d6f6818d2a1e3b0db739e524a781d3b2c97f68f6a`.
- V9 zero-policy baseline terminates on `terminal.joint-safety` at158 motor
  ticks (2.6333s), final root height0.77472m. This is not a learned result and
  does not reuse the diagnostic standingV6/support reference.
- Native all-target Clippy and Rust/Python formatting/lint pass. Boundary-scan,
  content-package, play, persistence-replay and Linux host-check pass. The host
  log SHA256 is `89baaba0a98fea96d543e13fe393eebc780fa406c950e0562b5b7d1cc11dc9d3`.

External evidence root:
`/home/kaifaty/NextEngine-training/corrected-walking-native-HIjbT0`.
Initial descriptor SHA548b3b30… is retained as `descriptor-v9.json`; the final
smoke-admission wording is `descriptor-v9-smoke.json`, SHA256
`87155fc21a8bc5100a706272df7cdcae54f3ae1c5aab6187e31e7b263c73d3e4`.
The metadata revision does not change native environment manifest or dynamics.
`native-control.json` SHA256
`e11428b8e9653f1197e49f9751f94434ff08befe87de7c17e11f4275cac2a9f3`
closes controls, source identities and `zero-policy.npz` (SHA256
`f45e1ccf2869995e3685c039c180f418356a9791f9337d2003c5168f17d66044`).
The native binary was copied before host-check could replace the shared debug
path; external `next_headless` SHA256
`67a0fcb21bb227b780dab7801b985348f46979eb7efb2244bbc16b94633a6889`
is the tested native executable, not the subsequent shared no-SDK build.

## Frozen run and observed result

Clean implementation commit `83ef7eeb3bbb1fa2f7712b918bac292140b1f5a1` froze
`generation/generation-manifest.json`, SHA256
`8a01ae96c9dfaa506a55f04dcc8d15f5a49cfaf5258001adda01e356216babef`.
Both freeze and launch controls match: 2560 transitions / 501 resets, native
step-root sequence `1960e74c57939e1bf0e4eb89f6a7537ae67393f2b67b6754eff3fed53ddab5c1`.
The sole `generation/runs/TRAIN-1` completed all 32 PPO updates / 16384 samples
on CUDA with fresh weights, no resume or checkpoint selection. Measured collection
time was 32.758s, learning 2.467s; this is report-only debug-build throughput.
All losses, model and optimizer tensors are finite. Final checkpoint is
`model_31.pt`, SHA256 `4178ad80f1a5c59a48267fa473ae1f0ada53d8473450cdb11e5f01659bcb8f12`.
Run manifest SHA256:
`6290180b62d52fa958aaec5520921d1b3313e838fe2b0ae84efea4025485c026`;
metrics SHA256: `fc2c6d2162c09517d81eb3098529fe18662704907493e1837dee0f9236b0aee7`.

Training had **zero moving-command samples**. Terminal counts: self-collision304,
joint-safety261, impact56, fall2. The mean of rolling episode-length metrics rose
from12.787 ticks (first8 updates) to34.466 (last8); this is not a held-out quality
improvement. Action noise stayed near0.25. This smoke does not record KL/gradient
telemetry, so finite losses alone do not establish optimizer health.

All five declared final seeds terminate at90 ticks (1.5s), joint-safety,
forward displacement **−0.423749m**, zero single support. These are identical
trajectories during the initial zero-command phase, not five independent diverse
walking trials. The final deterministic policy is worse than the zero-policy
158-tick baseline. Walking is **not learned**; do not extend this run in place.

## Post-run telemetry defect and bounded repair

Read-only artifact audit found that the native encoder's duplicated joint
telemetry still sliced observations at10..33/33..56, and the Python parser
bounded these arrays to23. PPO observations/actions were already94/25, but exported
q omitted two joints and exported v mixed two q channels with21 v channels.
Initial all-field adapter equality could not detect a shared encoder defect.
Do not use old V9 `joint_position_*` / `joint_velocity_*` tails for complete-body
analysis, including `zero-policy.npz`; its full `observation_raw` remains usable.
Original files/manifests/checkpoint are preserved unchanged.

Commit `ebb648440160358b95b80d73494ccd8df053b03e` makes these slices and parsing
descriptor-sized, rejects short joint tails, and adds native V8/V9 wire-to-
observation equality plus Python 23/25-channel/short-tail regression tests.
Eight native protocol tests and33 focused Python tests, formatting and strict
headless Clippy pass. Full host-check is not repeated for this localized encoder/
parser-only repair; the earlier full check covers the integration, not this commit.

No-optimizer replay under the repaired binary reproduces all five evaluation
outcomes, every non-joint NPZ field and the original first23 q channels exactly,
while exporting all25 q channels. New adapter control preserves the exact2560-
transition step roots; old V8 control preserves640 transitions /29 resets exactly.
Repaired binary SHA256:
`f4b57e12cebd6f7f0c522ff3838b85d9b386c9492e127f6ead584151522b99a8`.
`telemetry-replay/replay-manifest.json` SHA256:
`3cc9aeb70521828c04b804b96fcad428d988f8a65ad6b2a027bab5f566b2c728`.

The diagnostics skill's helper supports reference-overfit manifests, not this
canonical schema. The external `audit_smoke.py` therefore verifies generation,
all8 declared artifacts, finite state, sample counters and five NPZs directly.
`smoke-audit.json` SHA256
`0b2f9257048ca1c90e5d3eff187cacde215675cd4931430a4cd9ffdb6c37de54`
reports integrity PASS and original joint telemetry FAIL; replay is a separate
repair proof, not a rewrite of that failed check.

## Decision and smallest next action

The native25 pipeline can execute PPO. Original trajectory export was defective
and is now separately repaired/verified; neither fact establishes walking quality
or full calibration. The first learning barrier observed is safety termination
before the movement lesson, not demonstrated numerical divergence. Exact failing
joint/channel and the contribution of initial mean action versus exploration
remain unlocalized. Next: a bounded unchanged-body prefix audit of zero, seeded
initial mean/noisy and final mean actions, with complete q/v and post-safety
targets, before choosing a balance curriculum or changing an exploration variable.
Do not infer that another gain sweep, longer unchanged PPO, imported23 weights,
weaker safety, velocity substitution or support-feedforward promotion is justified.
No optimizer remains active. Active-contact proxies are not independent anatomical
load/free-foot certification; runtime export and Isaac correspondence remain open.
