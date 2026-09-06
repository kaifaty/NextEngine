# ADR-126: Corrected-body canonical walking environment and pipeline smoke

| Field | Value |
| --- | --- |
| ID | ADR-126 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-06 |
| Dependencies | SPEC-27/34/35, ADR-107/110/112/114/115/116/118/119/122/124/125 |
| Supersedes | ADR-122/124/125 diagnostic-only V11 consumer restriction, additively for the exact environment and bounded pipeline run below; all calibration results and old environments remain unchanged |
| Superseded by | none |

## Decision

The user prioritizes corrected-body walking integration. Admit native environment
`nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v9` with exact
BodySchema V11, CompiledV4 per-iteration forces, 26 bodies / 25 actuators,
unchanged anatomy, gains, materials and complete safety limits. Reset is canonical
neutral with the existing seed/slot derivation; episode bound and applied command
schedule remain V8's 1200 ticks, including its final 180 zero-command actions.

Actions are 25 Q1.30 residuals in descriptor actuator order, multiplier262144,
around the existing translation-invariant walking reference V1. The reference
is not diagnostic standing V6. Explicit PD and complete engine safety own effort;
the diagnostic f64 support generator is NOT promoted or silently included.
No root-force/pose override, gain adjustment, velocity substitution or imported
23-action weights. Learning balance is allowed without a perfect hand-written
standing controller; a failed calibration gate stays failed, and an unstable
zero policy is reported rather than disguised as environment correctness.

The 94 observations are quaternion4, root-local velocities6, positions25,
velocities25, previous applied targets25, next command3, anatomical active flags2,
clock2 and whole-foot minimum heights2. Native measured velocities remain intact.
Each foot groups rearfoot and MTP. Height is the minimum of both native box
minima, so heel rise alone is not swing. Contact/terminal use exact V11 anatomical
classification with unchanged 6Ns aggregate and per-pair limits. Flags are active
contact (penetration or classifier impulse threshold), not verified load support.

Reward retains V8 formulas, 13 components and coefficients, with 25-channel
effort/rate normalizations. Per actual physics substep, sum the ground-oriented
vertical impulses for both parts before absolute value, then sum over substeps.
Slip is maximum active-member link planar L1 speed per contacting anatomical foot;
periodic load credit uses maximum member-link speed for each foot. These are link
speed proxies, not exact contact-point slip. Foot count is at most2, never4.
Lift cost uses whole-foot minimum geometry; stop credit uses anatomical active
flags. No invented impulses from padding after a safety failure: terminal input
pads frozen classification frames without advancing classifier continuity. The
actual substep count remains explicit in the frame projection.

The new manifest closes full CompiledV4 hash, contact profile and these semantics
in distinct component domains. Old V1–V8 descriptors, roots and behavior remain
unchanged. No public protocol/schema version change; existing variable-length
records expose native 25/94 widths. Python rejects incompatible handshakes/actions.

## Bounded run and claim ceiling

After native checks and exact multi-process adapter control pass, permit one
fresh-weight CUDA PPO pipeline smoke using tracked
`lab/profiles/canonical-rsl-rl-walking.v5.json`: 32 environments / 4 processes,
16 steps/update,32 updates =16384 samples, seed44,600-second optimizer budget.
Use existing RSL-RL/PPO algorithm and architecture, no hyperparameter sweep.
Freeze clean commit, binary/descriptor/profile/dependency hashes and a sole new
external run path with the existing canonical generation workflow. Preserve the
closed failure prefix; never retry unchanged after failure.

Final deterministic five-seed episodes are **report-only pipeline observations**,
not V8's independently corrected walking certificate. Even if the legacy metric
thresholds happen to pass, this smoke emits `report-only`, no selection and no
walking-quality/runtime/export/mirror claim. A full walking run needs evidence-led
follow-up and compatible anatomical-foot evaluation; it is not authorized by
changing this smoke's budget after launch. No USD/reference corpus is required
for this direct native-CPU lane (ADR-107); CUDA is neural computation only.

## Verification and rollback

Native 25-channel direct-control equality, 94-channel order/geometry/reward tests,
invalid-batch atomicity, partial reset, terminal/reset and exact native repeat;
existing anatomical impact and foot-side classification tests; Python action
units, final-observation/timeout/selection checks; exact native adapter control
with resets; immutable old V8 control; format/Clippy, boundary/content/play/replay
and Linux host-check. A new environment does not close full calibration. Retire
V9/profileV5 on failure; never change old body/weights or relax safety to obtain a
successful report. Runtime and Isaac remain outside this admission.
