# Gain Tuner tool check — partial success

User-authorized tool trial on 2026-09-05; repository source `205909dc`.
No production body, controller, validator, training profile, dependency or driver
changed. No optimizer started. This is not MODEL-MIRROR-P1/P2 or V8 Isaac admission.

## Outcome

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
instead of NextEngine external PD. Source stiffness/damping and effort/speed
limits are used, but engine target slew, effort rate, power/work envelopes and
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
despite the source helper adding the tag to USD metadata. Resolve schema
registration/discovery before trusting the natural-frequency auto-gain path.
Do not fill private mass/inertia maps with invented fallback values.

## Evidence and reproduction

External directory: `/home/kaifaty/NextEngine-training/gain-tuner-rYDf8d`.
`check_gain_tuner.py` is the standalone harness; `fixture.usda`,
`ankle_response.npz`, `ankle_response.png`, `result.json` are the initial
successful headless trial. `mass-query-check/` retains the readiness discriminator.
`run.log` / `run-with-libs.log` retain both GUI failures; `headless.log` and
`mass-query-check.log` retain successes and the empty discovery finding.

Final harness SHA256 `f382544aeec9e2712aa9b0a2657c072ed59cfcffdf1076a3565b9f88076213b9`;
initial response NPZ `1924cdb93d15f460334d542e4c1774167a1c034347a330b100cb17e90c68f422`.
The final harness adds an output tag and explicit mass-readiness reporting;
the original run used a fixed 30-update wait. They are not identical harness inputs.

Run the external harness with the existing Isaac Python, `--no-render` and a
fresh `--tag`, with process-local `LD_LIBRARY_PATH` set to
`/home/kaifaty/NextEngine-training/system-libs/root/usr/lib/x86_64-linux-gnu`.
Do not rerun GUI unchanged. No GUI window or simulation process remains open.

Next: a schema-only discovery probe, requiring 26 recognized links / 25 actuated
joints and completed property queries before attempting auto-gains. GUI repair
requires a separately agreed compatible graphics environment, not blind driver
downgrade. Neither issue justifies changing BodySchema anatomy or training safety.

Checks: external harness `py_compile` PASS; bounded waveform execution PASS;
source descriptor unchanged PASS; GUI FAIL; auto-gain path NOT VERIFIED.
Repository changes are documentation only: diff/link checks, no Cargo required.
