# Explicit force-schedule profile: implementation and standing follow-up

Implemented under [ADR-116](../architecture/adr/116-explicit-per-iteration-force-scheduling.md).
`CompiledBodySchemaV4` selects
`nextengine.physics.humanoid-per-iteration-external-forces.v1`; the V6-body
outer compiled hash is
`66d6a5b01ea1a26294b92e050bbdc79556cffd63cabdd7382d15ea0543d6c7ed`.
Body V6 and the inner V3 compilation are unchanged. No default environment,
learned checkpoint, reward, PD gains, mass or safety threshold changes.

## Production boundary and verification

The versioned additive C entry accepts the existing scene record plus a u32
schedule. ABI 4 layouts and old entry points remain unchanged; old scene
construction delegates to schedule 0. The new compiler selects schedule 1
through its `create_world` method. Missing material, unknown tag and repeated
configuration fail without consuming a valid scene configuration opportunity.
Rust never converts arbitrary native integers into enum values. No new native
allocation owner, retained pointer or thread-sharing permission was introduced.

Tests cover the new frozen hash, complete equality of the inner V3 compilation,
body sensitivity, native dynamics distinction, and 64-step exact same-effort
fresh-world reconstruction. This is not a full new-environment replay gate.
The new profiled baseline matches **all 1801 motor samples and all 7200 physics
samples** of the earlier one-line TGS experiment, including joint state,
contacts and applied efforts. The preserved legacy scene likewise matches
the old complete trace and compiled hash exactly. Probe metadata has a new
schema/profile label, so whole JSON equality across probe schema revisions
is not claimed.

## Reference follow-up: do not select the failed candidates

Hypothesis: the old feedback law may behave better on a corrected velocity
signal; alternatively, its reference / discrete PD / contact dynamics still
require repair. Test the same k=2 hip feedback, not another gain sweep, on the
new profile. It lowers tilt but stops at motor tick **596**, physics substep
**2384**, after a `terminal.contact-impact` detected within that motor tick.
At substep **2382**, summed ground impulses are **6.602703 N s** on the left
foot and **6.338131 N s** on the right, both exceeding the unchanged **6 N s**
foot impact limit. Individual contact points alone hide the aggregate spike.
The nearly upright final torso (0.200°) is not success.

The last second's mean total normal force is 737.861 N (body weight ~739.056 N),
but the single-step spike still fails safety. There are no zero-total-load
substeps in that final second: do not explain the spike as several wholly
unloaded steps accumulating their entire normal impulse. Exact cause of the
spike remains unresolved; the reference / actuator / contact response needs
a discriminating experiment before selecting another controller.

A separate reference-only neutral-target counterfactual tests whether the
legacy feedback itself is necessary. It stops on the **first** physical step:
knees reach -12 / -11 microradians outside the exact zero lower hard limit.
This does not prove the body cannot stand; a reset/reference on a hard-limit
boundary cannot provide the desired interior stance margin. Do not relax the
hard-ROM validator or repeat all-zero targets as an adequate stance.

The original reference with the new force schedule remains the 30-second
successful safety control, with residual oscillation. Its maximum individual
foot impulses are 3.271516 / 3.354904 N s; mean total normal force in the final
second is 739.053 N and both feet together have positive load on every one of
those 240 steps. This is bounded load evidence, not certification of flat
support through all motion or a completed human-foot model.

## Adjacent-layer research and next discriminator

- Our biomechanics path uses explicit PD in `safety_control.rs`, not native
  implicit joint drives. Ankle pitch gains are 400/40, ankle roll 300/30
  (stiffness/damping in their corresponding SI units); effort/rate/power/work
  limiting remains downstream. High-frequency effort/contact response merits
  checking against local effective inertia before more root-feedback tuning.
- [Tan, Liu and Turk, Stable PD](https://www.jie-tan.net/project/spd.pdf),
  sections 3–4, distinguishes numerical control instability from the physics
  integrator and uses next-step state in the torque calculation. Its articulated
  formulation needs coupled mass/bias/external-force information. Some demos
  anchor the root: they do not establish free-standing balance, and its
  stability claims cannot simply be transferred to our clipped explicit PD.
  The author-hosted paper was opened; the Georgia Tech mirror timed out.
- [OpenSim model issue 185](https://github.com/opensim-org/opensim-models/issues/185)
  reports the factor-ten toe Izz difference between gait2392 and Rajagopal.
  It is open, not a maintainer-confirmed correction. This supports inspecting
  the alternate original numeric model before foot articulation, not copying
  an unverified replacement into V6. The source toe triangle violation remains
  a reason not to split it unchanged into a dynamic link.

Next: distinguish the actuator's discrete high-frequency response from global
balance/reference error, with the current profiled baseline as control and
unchanged safety. Any accepted actuator/body change gets a separate identity.
Then improve foot mechanics and select a compatible standing/walking generation.

## Exact artifacts

Directory:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/upright-reference-01`.

| Artifact | SHA-256 |
|---|---|
| `profiled-tgs.json` | `43d48fef706cfc87714cf75f0fa701a874654d4f792f8ff0ff58a5119ebd72ec` |
| `profiled-hip-feedback.json` | `c1a11f3a5d4c960bc6bfb57c8ddd2c71c3eaafeb5129e83012737bcd09629351` |
| `profiled-neutral-targets.json` | `c2addf3621f9eb2bf0dbb21be688a1b1f990245c97248e3ee038cc46fafd18cd` |
| `profiled-legacy-control-02.json` | `c81c1062816e85d75d63f8637799d30948555d74ff8e79d1477ee495a4836dd6` |

Use `cargo run -p next_motor --features physx-sdk --example
probe_biomechanics_body_standing -- 6 0 0 baseline per-iteration` with the SDK
directory in the task state. Replace `baseline` with `hip-feedback` or
`neutral-targets` only to reproduce those rejected cases. Omit all arguments
after `6` for the legacy control. A direct call of the shared debug binary
during host-check hit the no-SDK build and produced an empty
`profiled-legacy-control.json`; it is not a physics result. Prefer the explicit
Cargo feature invocation because other build configurations can replace that
unhashed example path.

## Checks

- PASS: 134 native motor tests; 3 native FFI tests; 6 mock lifecycle tests.
- PASS: both complete exact native trace comparisons described above.
- PASS: play, persistence-replay and content-package.
- PASS: platform portable contract; SDL/ash candidate NOT_RUN_ADAPTER_DISABLED.
- PASS: full host-check (session 10364 completed), including workspace clippy.
- PASS: native-feature all-target clippy for motor / PhysX adapter / FFI with
  warnings denied; formatting and diff whitespace checks.
- NOT_RUN: new-environment replay/admission, training, perturbation robustness,
  Miri/native sanitizer and a separate performance benchmark. No new pointer
  ownership or default runtime hot-path choice is introduced by this change.
