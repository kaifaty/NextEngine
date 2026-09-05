# Gain Tuner tool check — headless discovery repaired, candidate not selected

User-authorized tool trial on 2026-09-05; repository source `205909dc`.
No production body, controller, validator, training profile, dependency or driver
changed. No optimizer started. This is not MODEL-MIRROR-P1/P2 or V8 Isaac admission.

## Current outcome (continued from `a1b0887f`)

Process-local Robot Schema compatibility now discovers 26 links and 25 actuated
joints; all 26 real PhysX mass queries finish and match the descriptor within
2 micro kg per body. The tool's unchanged inertia routine returns 25 values.
Discovery-only repair preserves every recorded control sample exactly.

The trial also found and corrected a **harness error**: source per-radian PD
gains had been written directly into per-degree USD attributes. Old traces are
valid tool smoke evidence, but their claim to use the source physical gains is
withdrawn. Corrected export and runtime gain readback now agree for all 25 DOFs.

One candidate from the actual NVIDIA UI model, requested natural frequency
2 Hz and damping ratio 1, was exercised on the same isolated ankle trajectory.
These are operator-chosen test parameters, not an optimizer's recommendation.

| Corrected fixture run | Applied K (Nm/rad) | Applied D (Nms/rad) | Angle RMSE |
| --- | ---: | ---: | ---: |
| Source-gain control | 400 | 40 | 0.574918 degrees |
| Gain Tuner 2 Hz / ratio 1 | 2.799099 | 0.445490 | 0.900160 degrees |

The candidate error is 56.57% higher; **not selected**. This rejects that one
candidate as a tracking improvement, not Gain Tuner generally. The tool is usable
as a headless response-test aid; autonomous calibration, standing and walking
remain unverified. No production model, controller or training was changed.

## Initial outcome (historical, qualifications above apply)

- Installed Isaac Sim 5.1.0 includes NVIDIA Gain Tuner 3.0.6; no installation needed.
- GUI startup fails before fixture creation in `librtx.scenedb.plugin.so`.
- The installed GainTuner waveform generator/controller/recorder works without
  rendering on an isolated 26-body / 25-DOF fixture sourced from V8.
- A 4-second left ankle-pitch sinusoid, amplitude 0.05 rad, frequency 0.5 Hz,
  yields 961 samples, RMSE 0.0100298924 rad (0.575 degrees), maximum absolute
  error 0.0146952095 rad (0.842 degrees), maximum speed 0.147239879 rad/s.
  The plotted response lags the target. No new gains were selected or optimized.
- Automatic robot-link/mass discovery did **not** work: the tuner's `_link_mass`
  is empty, not a pending query timeout. Its accumulated-inertia map is empty.
  A bounded two-second readiness check exposed the empty selection; the same
  sinusoidal result repeats numerically. This is not an operational auto-calibrator.
  The simulator's articulation API enumerates 25 DOFs; that must not be confused
  with successful Robot Schema discovery by the tuner UI.

## Deliberately different fixture

Source inspection descriptor:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/body-v8-01/descriptor-r2.json`
SHA256 `0fe0252fe8e1c63caaf323bc4c555b4efacf81765db0a542ac851e116e07cf7f`.
It explicitly says `native-body-diagnostic-only`. The production mirror validator
still rejects its 26/25 topology; it was neither changed nor monkey-patched.

The separate external tool script authors a labeled **asset-tool fixture**, not
an admitted translated training bundle. It copies masses, principal inertias,
joint frames/ROM, shape geometry and exclusions from the hash-checked descriptor.
Source counts and total mass are asserted; source hash is rechecked after the run.
No trajectory correspondence or complete independent exporter audit is claimed.

Differences are intentional and disqualify transfer of the result to standing:
pelvis fixed to world, gravity zero, no ground; built-in implicit force drives
instead of NextEngine external PD. Source effort/speed limits are used; source
stiffness/damping require the corrected export described above. Target slew,
effort rate, power/work envelopes and
terminal classifier are absent. Isaac World uses CPU PhysX dynamics at 240 Hz,
TGS 16 position / 4 velocity iterations. This is not a GPU learning benchmark.
Only left ankle pitch is commanded; all other drive targets remain zero.

No publisher metadata declares engine admission. The saved scene labels its
scope `PINNED_ZERO_GRAVITY_IMPLICIT_DRIVE_TOOL_SMOKE_NOT_MIRROR`.

## GUI and discovery findings

First GUI launch also lacked loader paths for already-installed libxml2/libGLU.
Using the existing external compatibility-library directory removed that issue,
but the same RTX startup segfault remained. No body was loaded in either launch.
Host: RTX 3080, driver 610.43.02, Ubuntu 26.04. NVIDIA reports the same driver / 
Isaac Sim 5.1 crash combination and says this driver branch is not validated:
[NVIDIA support thread](https://forums.developer.nvidia.com/t/isaacsim-crash-when-update-gpu-driver/371975).
Driver incompatibility is a supported explanation, not a locally proven cause
because no alternate-driver control was performed. No driver change is authorized.

Isaac Lab 2.3.2's headless experience avoids the crashing renderer and completes
the waveform test. The standard extension's `GainTuner.setup` selects robot links
using `HasAPI("IsaacLinkAPI")`; that selection is empty for this authored fixture,
despite the source helper adding the tag to USD metadata. This discovery defect
is repaired in the isolated follow-up below, not in the installed extension.
Do not fill private mass/inertia maps with invented fallback values.

## Evidence and reproduction

External directory: `/home/kaifaty/NextEngine-training/gain-tuner-rYDf8d`.
`check_gain_tuner.py` is the standalone harness; `fixture.usda`,
`ankle_response.npz`, `ankle_response.png`, `result.json` are the initial
successful headless trial. `mass-query-check/` retains the readiness discriminator.
`run.log` / `run-with-libs.log` retain both GUI failures; `headless.log` and
`mass-query-check.log` retain successes and the empty discovery finding.

Initial-turn final harness SHA256 `f382544aeec9e2712aa9b0a2657c072ed59cfcffdf1076a3565b9f88076213b9`;
initial response NPZ `1924cdb93d15f460334d542e4c1774167a1c034347a330b100cb17e90c68f422`.
The final harness adds an output tag and explicit mass-readiness reporting;
the original run used a fixed 30-update wait. They are not identical harness inputs.

Run the external harness with the existing Isaac Python, `--no-render` and a
fresh `--tag`, with process-local `LD_LIBRARY_PATH` set to
`/home/kaifaty/NextEngine-training/system-libs/root/usr/lib/x86_64-linux-gnu`.
Do not rerun GUI unchanged. No GUI window or simulation process remains open.

The original next action (schema-only probe) is now complete. GUI repair still
requires a separately agreed compatible graphics environment, not blind driver
downgrade. Neither issue justifies changing BodySchema anatomy or training safety.

Initial checks: external harness `py_compile` PASS; bounded waveform execution PASS;
source descriptor unchanged PASS; GUI FAIL; auto-gain path NOT VERIFIED.
Repository changes are documentation only: diff/link checks, no Cargo required.

## Follow-up: discriminator and minimal compatibility repair

Competing hypotheses were missing authored tags, unregistered informal schemas,
and asynchronous mass-query readiness. `schema_probe.py` opens the unchanged
fixture without physics stepping: all 53 tagged prims retain their authored
robot/link/joint metadata, but `HasAPI` is false and all three schema registry
types are `Tf.Type.Unknown`. Registered `PhysicsRigidBodyAPI` succeeds on all
26 bodies. Both official recursive discovery functions return zero. Together
with the earlier empty query dictionary, this supports a schema-recognition
defect, not absent mass data or a longer required wait.

OpenUSD documents that `AddAppliedSchema` accepts names without requiring a
valid registered API type; `HasAPI` resolves a schema type. See the actual
[UsdPrim API](https://openusd.org/dev/api/class_usd_prim.html). Our local probe
also shows that `GetAppliedSchemas()` filters out these unregistered tags, so
using it alone would not repair this installation.

`robot_schema_compat.py` loads hash-checked, process-local copies of NVIDIA's
robot-schema utility module and Gain Tuner backend. It changes exactly eight
and one informal-tag checks respectively to composed `apiSchemas` metadata
membership. No mass queries, inertia formulas, controllers, registered PhysX
checks or installed files are changed. Original modules remain unmodified.
The original licensed modules are read locally; no vendor code is redistributed.

Controls pass: absent prim / wrong tag remain false; a stronger session-layer
deletion removes a link from discovery, re-adding restores it; the real rigid
body API remains recognized. Original discovery still returns zero. Real
property queries supply every mass; no private map is filled with substitute
values. `discovery-fixed/` reproduces all original NPZ arrays exactly.

## Gain units and the bounded candidate

NVIDIA's [legged-robot rigging tutorial](https://docs.isaacsim.omniverse.nvidia.com/5.0.0/robot_setup_tutorials/tutorial_rig_legged_robot.html)
requires conversion between USD degrees and controller radians. The installed
experimental articulation's `set_dof_gains`/`get_dof_gains` implementations
confirm the conversion. The harness now authors `K_usd=K_si*pi/180` and likewise
for D. The historical direct write inflated both physical gains by `180/pi`.
This correction concerns only this external fixture, not an established defect
in the production mirror exporter.

The corrected control checks all 25 live stiffness/damping pairs against the
source (relative tolerance 2e-6), before and after testing. The candidate uses
the installed `JointItem`, `NATURAL_FREQUENCY` mode and its normal callbacks.
Its USD K/D are 0.0488534890 / 0.00777527426; live articulation readback confirms
the SI values in the table. All other live gains remain exactly unchanged.
Both corrected runs finish 961 samples over 4 seconds with finite outputs.
This is not proof of safety under engine constraints or loaded motion.

The inertia calculation itself is **not certified by successful discovery**.
Inspection shows the installed backend forms `norm(I.T*I)` and combines forward
and backward branches; its mapping to physical joint-axis inertia and the UI's
frequency units still needs an independent one-DOF oracle before trusting its
frequency labels across morphologies. Do not silently replace that arithmetic
or transfer the calculated numbers into native PD. This trial changed discovery
only and tested the tool's unmodified candidate; it did not prove its formulas.

Next, if continuing auto-calibration validation: one known-inertia revolute-joint
control to check frequency/units, not another full-body gain sweep. Reconsider
automatic gain transfer only with that oracle and a controller-matched test.
GUI remains the separate known renderer failure; do not repeat it unchanged.

## Follow-up evidence and checks

Same external directory; new files: `schema_probe.{py,json,log}`,
`robot_schema_compat.py`, `validate_compat.py`, `compat_controls.{json,log}`,
`discovery-fixed/`, `auto-2hz/` (historical-unit exploratory candidate),
`si-control/`, `si-auto-2hz/`, `compare_results.py`, `comparison.{json,png}`.
Only the two `si-*` directories support the current gain comparison.

Final harness SHA256 `97be1dd6d4046bd70d34ac39965d1e1f57e733a7408ff0af1cb45272864825f5`;
compatibility helper `d1838b051a985518e4abd9893cbfb24bb020d08c690438ce34966b66a21837a5`;
comparison script `9d302ab68388b5951b7fb4ffad9f7246dbc4719ab0b3ce674d4fd58040dbfaca`;
comparison JSON `5e087f4688db06d39229e82845ea6d6747bead1abc64abc968e025a4e9a66ac5`.
The comparison JSON retains exact result/trace/probe/control hashes.

Reproduce corrected control with `--no-render --schema-compat --tag <fresh>`;
add `--auto-frequency 2` for the sole candidate. Existing process-local loader
path and headless experience above remain required. `--legacy-gain-units` exists
only to reproduce the old defect, never as a calibration default.

PASS: five external scripts compile; metadata positive/negative/composition
controls; 26 real completed mass queries; 25 inertia entries; exact discovery-only
trace control; source/live SI gain checks; bounded candidate execution and live
gain readback; unchanged source SHA. Candidate improvement FAIL. Inertia/frequency
oracle NOT RUN; GUI previous FAIL, not rerun. No process remains running.
Repository diff and changed local-link checks PASS; Cargo/host-check NOT RUN
(documentation-only repository diff, external experiment does not use Rust).
