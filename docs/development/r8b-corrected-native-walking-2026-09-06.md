# Corrected-body native walking integration

Status: `NATIVE_25_ENVIRONMENT_IMPLEMENTED / PIPELINE_SMOKE_PREPARED / FULL_CALIBRATION_OPEN`.
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
  content-package, play and persistence-replay pass. Linux host-check is
  still running at this checkpoint; readiness requires its final outcome.

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

## Next action and limits

Complete host-check and freeze the exact clean commit/profileV5/descriptor/binary in a
new external generation. Run only the admitted16384-sample CUDA pipeline smoke.
Keep the five-seed final episodes report-only; active-contact proxies are not the
independent anatomical-load/free-foot walking certificate. No checkpoint selection,
runtime export or Isaac correspondence claim. Full walking training and the
remaining loaded calibration/velocity-readback investigation stay separate.
