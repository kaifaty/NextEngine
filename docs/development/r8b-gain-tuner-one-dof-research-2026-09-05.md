# Gain Tuner one-DOF oracle — GAIN-ORACLE-01 r1

Repository snapshot `d94206386fdf550da8af8888356a5cd129b10937`.
External tool research only; no native model/controller, production mirror,
training, installed package or driver change. Prior scope is recorded in the
[tool trial](r8b-gain-tuner-tool-check-2026-09-05.md). SPEC-35 and Accepted
ADR-118/119/120 boundaries remain unchanged; no ProductCheck is promoted.

## Result

**C1/C2 REFUTED; C3 SUPPORTED_BOUNDED**, independently reviewed. Evidence
classes: analytic derivation, numerical, empirical and correspondence.
The evidence distinguishes two defects in the
installed Gain Tuner 3.0.6 / Isaac Sim 5.1.0 force-revolute path: scalar inertia
aggregation and the UI gain unit boundary. The analytic SI control succeeds.
The result is not a diagnosis of NextEngine's body anatomy or native controller.

| Principal moments (kg m²) | Correct X-axis inertia | Tuner scalar |
| --- | ---: | ---: |
| 0.1, 0.1, 0.1 | 0.1 | 0.0173205093 |
| 0.2, 0.2, 0.2 | 0.2 | 0.0692820372 |
| 0.1, 0.15, 0.15 | 0.1 | 0.0333541616 |

Doubling all moments quadruples the reported scalar. Changing only transverse
moments changes it despite the same X-axis inertia. Both contradict the frozen
physical-inertia oracle for this fixed-base, centered single revolute joint.

With **known correct inertia supplied to the UI model alone**, live physical K
and D are both 57.29578 times the requested SI values. This is a separate test
condition, not an invented mass-query result or a repair of the backend map.
Explicit per-degree conversion after that UI calculation restores the desired
gains and the analytic response in these fixtures.

## Frozen contract and controls

Before execution, external `contract.md` revision1 froze three positive,
physically realizable inertia tensors, a world-fixed base and centered rotor,
one revolute X joint, identity principal/joint frames, mass1kg, zero gravity,
no colliders/contacts, zero friction/armature/body angular damping. No morphology
exporter is involved. The physical plant has precisely one moving DOF.

C1 requires reported inertia equal Ix within1e-5 relative error. C2 requires
live SI K/D equal `Ix*(4*pi)^2` / `2*Ix*4*pi` for requested2Hz/ratio1 within1e-5.
C3 is the independently derived 0->0.05rad critical-damping response over1s:
normalized max position error <=3% at240Hz, <=1% at960Hz and strictly reduced.
These thresholds were frozen before the run, not selected from its outputs.

Each resolution runs all three fixtures under four conditions: analytic SI,
unmodified tool inertia+UI, known-Ix UI, known-Ix UI plus explicit degree
conversion. Every condition resets position/velocity/targets; real PhysX mass
queries verify the authored rotor moments/mass/COM; live K/D is read before
and after each response. Force cap1e6Nm, ROM+/-90deg and speed100rad/s are
non-binding for these small motions, not NextEngine safety envelopes.

Arithmetic: USD/PhysXf32, NumPyf64; CPU implicit PhysX, TGS16/4, no rendering.
Isaac Python3.11.15, in-simulator NumPy1.26.4. Offline SciPy reports1.15.3;
the in-simulator SciPy version was not separately captured. No RNG or learned policy.
Only the previously hash-checked process-local informal-schema discovery fix
is loaded. Vendor mass queries, inertia math and JointItem remain unchanged.

## Independent reference and observations

For rotation restricted to unit axis a through COM, kinetic energy gives
`J=aᵀ I a=Ix`; the scalar dynamics are `J*qdd+D*qd+K*(q-q_target)=0`.
For requested `w=4*pi` and ratio1, `K=J*w²`, `D=2*J*w`, and with zero initial
state `q(t)=.05*(1-(1+w*t)*exp(-w*t))`. These formulas do not call Gain Tuner.
A separate SciPy matrix exponential checks responses against their actual live
gains, including overdamped cases; it does not certify solver exactness.

The installed backend instead reduces its accumulated tensor using the
Frobenius norm of `I.T*I`. On these centered diagonal single-link fixtures,
this is `sqrt(Ix^4+Iy^4+Iz^4)`, with dimensions (kg m²)², not axial inertia.
The source routine and real queried inputs reproduce the table. This is not
a proof that simply substituting axis projection repairs a general branched,
floating or contact-constrained robot.

The installed JointItem computes gains from frequency and writes raw USD
attributes without converting its rotational gain results. NVIDIA's
[rigging documentation](https://docs.isaacsim.omniverse.nvidia.com/5.0.0/robot_setup_tutorials/tutorial_rig_legged_robot.html)
specifies `K_deg=K_rad*pi/180` and the same conversion for damping. The actual
articulation readback independently exposes the missing factor. Requested2Hz
with correct J passed to that UI yields live-gain natural frequency15.1388Hz
and damping ratio7.5694, not2Hz/1; these are **computed from measured gains**,
not frequencies estimated from oscillation peaks.

Across the three analytic controls, maximum normalized position error is
0.1031–0.1383% at240Hz and0.03388–0.04484% at960Hz. All frozen C3 controls pass
and improve with refinement. Unmodified end-to-end tool errors at960Hz are
12.7575%,13.3680%,13.3448% against the requested critical response. This refutes
correct calibration in the tested cases, not the utility of waveform testing.

An extra, non-contract exact-array assertion in the first summary script failed
on corrected-UI versus analytic traces (first difference1.86e-9rad). It was
replaced by reporting differences, not by weakening C1/C2/C3. The largest such
difference over all cases is1.78814e-6rad /2.82693e-4rad/s. Sequential physics
resets are not a bit-exact cold replay claim; the raw traces remain retained.

## Evidence and reproduction

External directory `/home/kaifaty/NextEngine-training/gain-oracle-alC5vK`:
`contract.md`, `probe.py`, `summarize.py`, `240/`, `960/`, both launch logs,
`summary.json`, `comparison.png`. Each resolution contains the authored USD,
result JSON and12 complete NPZ trajectories (241/961 samples each).

Run the existing Isaac Python with `probe.py --hz 240` or `--hz 960`, using
process-local `LD_LIBRARY_PATH=/home/kaifaty/NextEngine-training/system-libs/root/usr/lib/x86_64-linux-gnu`.
Output directories must be fresh; existing evidence is not overwritten.
The script selects the working Isaac Lab headless experience. Then run
`summarize.py`. Do not repeat the known failing GUI startup.

SHA256: contract `b881a96aa489f4b6cbd817eb4644f46bb5f39192a51cefc0d62f7f8d5b5c8c60`;
probe `ba4b897cc1385cc6991a458813587a744cec83b5fd082aa2ab1c2d7412212664`;
240Hz result `c91cc16c8fec4322687e6834739496dca69fe6622c666401ed94b3e0c8f09c61`;
960Hz result `11a5972f29dda9fadfc697c8e6444906a934e8c70be49c5080be109a680edcbe`;
summary `7f08bc6d461476f606b6a7cdbf9396a39aa9ac9cbbbf2cd5342d36a982245037`.
The summary seals37 artifacts including vendor files, helper and all raw traces.

## Decision and remaining scope

Do not use this installed auto-frequency path to generate body gains. The
headless waveform test remains usable. The known-Ix plus unit-conversion control
is verified only on these single-axis fixtures, not a general replacement tuner.
No native gain sweep or anatomy changes follow from this result.

Current primary sources were checked on2026-09-05: the
[5.1 Gain Tuner documentation](https://docs.isaacsim.omniverse.nvidia.com/5.1.0/robot_setup/ext_isaacsim_robot_setup_gain_tuner.html)
defines frequency/damping tuning but now marks5.1 unsupported. The
[6.0 release notes](https://docs.isaacsim.omniverse.nvidia.com/6.0.0/overview/release_notes.html)
describe newer axis-projection and closed-form/oscillator tests. The directly
inspected [current upstream backend](https://github.com/isaac-sim/IsaacSim/blob/main/source/extensions/isaacsim.robot_setup.gain_tuner/isaacsim/robot_setup/gain_tuner/gains_tuner.py)
uses axis projection and distinguishes fixed branches; its
[new drive-math module](https://github.com/isaac-sim/IsaacSim/blob/main/source/extensions/isaacsim.robot_setup.gain_tuner/isaacsim/robot_setup/gain_tuner/gain_tuner_drive_math.py)
explicitly converts stored stiffness. These are mutable `main` source
observations, not a pinned installed6.0 result or proof of its damping boundary.
This supports
checking a supported upstream successor, **not claiming6.0 fixes our entire
case without executing it**. No upgrade, driver change or downloaded code was
executed. Next: compare the supported upstream implementation against this
fixed oracle, preferably in a separate compatible tool environment rather
than modifying the pinned training environment or writing a new full tuner.

Independent reviewer checked all37 sealed files and24 complete traces, verified
the vendor-to-fixture mapping and recomputed the ODE using real characteristic
roots (not SciPy's matrix exponential). Maximum oracle difference1.87e-14rad;
no load-bearing finding or simulator rerun. Review artifact:
`/home/kaifaty/NextEngine-training/gain-oracle-alC5vK/independent-review-r1.md`,
SHA256 `c3ff5cf620a362695464fec15a08960ec4eaa07e6d9c4d81e9f026f632e3cc12`.
Limits: after-trace gains are asserted but not separately serialized, no full
environment seal, no exact hidden solver reset or bitwise equality claim.
No repair/re-review needed for the bounded conclusion.

Verification: script compile and two bounded executions PASS; C3 controls PASS;
C1/C2 comparison FAIL; independent correspondence PASS; diff/local-links PASS.
No process remains running. Native/play/content/
host-check NOT RUN (external tool research, documentation-only repository diff).
