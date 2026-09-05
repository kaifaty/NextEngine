# BODY-BANDWIDTH-01 — frozen contract revision2

Base `e8ddb5b60dc5ea53d610459bf5c606aa845572ce`, Accepted SPEC-35,
ADR-069/115–121. Consumer: test whether a jointly bandwidth-limited K/D vector
can give usable native controlled responses, then preserve loaded support.
This does not close the full calibration goal by itself. All25-channel loaded
response, heel-rise/re-contact and disturbances remain required; no training.

Revision2 before native motion corrects only the unloaded fixture height:
10m free fall reaches the floor in about1.43s, invalidating the4s ground-free
contract. Use100m, preserving gravity and horizon (free-fall displacement about
78.5m). Revision1 and its one unchanged gain calculation remain external in
`body-bandwidth-lDjoJM/contract-r1.md` and `calculation.json`. No native r1 run,
new gain vector or relaxed contact criterion. Initial capture and all raw
contacts still verify the actual native boundary; no analytic assumption of
perfect translation invariance is used as native evidence.

## One fixed candidate, before calculation/native motion

Use both complete previously reviewed FOOT-CONTROL-01 response operators B
reconstructed from the raw102 one-step trials. SI units,25 semantic joint DOFs,
free root, ROM-interior knees/elbows, no ground endpoint/nonzero canonical
contact impulse. B is a local measured operator, not a global inverse mass.
Preserve its original adequacy checks and raw unsymmetrized operators.

Let b_i be the maximum diagonal B_ii over both operators and corresponding
left/right members (central torso channels use themselves). Normalize
C_ij=B_ij/sqrt(b_i*b_j); let R be the maximum absolute row sum across both C.
Set omega=min(60,180/R) rad/s, K_i=min(V8_K_i,omega²/b_i),
D_i=2*sqrt(K_i/b_i). This uses critical damping for the scalar local effective
inertia as a candidate construction, not a coupled damping-ratio certificate.
The row-sum limit bounds the normalized coupling and avoids choosing a scalar
frequency without considering the other joints. Calculate IEEEf64 once,
nearest-ties-even Q16 once, retain all values. No gain sweep/second candidate.
All anatomy, inertia, materials, physics timing/solver and safety remain V8.

Preflight both actual rounded K/D vectors in the existing alpha=17/32,
240Hz sampled-PD map. Require both spectral radii<1; report margins, eigenpair
residuals and finite960-step induced2-norm amplification in [q,h*v], not a
global stability claim. Independently compare with16 explicit kick/drift
propagations of50 basis states. Manufactured one-DOF stable/unstable controls
must be distinguished. Failure stops this vector; do not vary omega to pass.
The original V8 and failed V10 vectors are fixed comparison data, not candidates.

## Native discriminator after preflight and explicit admission

Use one new exact BodySchema identity before native motion, never edit V8–V10.
Native CompiledV4, SDKf259d3da…, Rust1.97.1, explicit PD240Hz/reference60Hz,
canonical integers and nativef32, original full target/effort/rate/power/work/
ROM/velocity safety. Initialize through production state import: root raised
100m, knees/elbows0.1rad, other coordinates0, velocities0; capture and use this
pose as the constant reference. No grounded reset repair or safety relaxation.
Gravity remains enabled; unloaded means no ground contact, not absent raw
zero-impulse self-contact constraints. Retain all links/joints/contacts/efforts.

For V8 and the one candidate, run25 isolated positive0.05rad step pulses plus
one simultaneous pulse and one simultaneous1Hz sine (amplitude0.025rad).
Four seconds/960 substeps per world: hold reset reference0–1s, pulse1–3s,
return3–4s. Sine is zero outside1–3s and uses tick-aligned samples. Stop each
world on first safety/active forbidden contact/ground contact failure, retaining
the full census including censored trials. No retries with smaller amplitudes.
Use normal safety target initialization to the captured reset reference, not
an effort bypass. Full selected/unselected channel observations are required.

Native response acceptance: all27 candidate worlds finish; selected step
position RMSE<=0.01rad over2.5–3s and final3.5–4s, simultaneous sine tracking
RMSE<=0.02rad over1–3s; all nonselected joint excursions<=0.15rad. Full velocity/
ROM/effort/contact rules always apply. No near-Nyquist peak concealed by60Hz
downsampling: analyze240Hz velocity, report power above60Hz for every channel.
Require final-half-second RMS speed<=0.05rad/s in every step channel. Sine
frequency is the requested test input, not a walking-policy frequency claim.

If responses pass, run original30s free standing and its unchanged V8 control;
same native standing gates as BODY-GAIN-NATIVE-01. Only then execute bilateral
loaded-transfer/re-contact tests. A soft unloaded success that cannot support
gravity is a failure of sufficiency, not full calibration. It would motivate
a separate controller/load-compensation discriminator, not weaker load gates.

## Hypotheses, limits and stop rule

H1: excessive native sampled bandwidth is sufficient to explain the response
failures; K+D correction yields the declared responses. H2: clipping/contact/
pose-dependent coupling still defeats this local design; native responses fail
despite local preflight. H3: controller/measurement mismatch invalidates the
experiment; exact gains, effort/order/reset/contact checks fail. The preserved
local manufactured controls and V8 motion are implementation/regression controls,
not independent physical ground truth. No result proves unique causality.

Budget: one calculation,54 unloaded worlds plus one exact candidate repeat;
conditional standing/control and original loaded cases only on preceding pass.
Freeze source/diff/commands/raw hashes before one fresh independent executable
review. Reviewer independently reconstructs gains/model and native effort/
failure precedence; at most one batched apparatus repair/re-review, otherwise
INCONCLUSIVE. No further geometry, D-only tuning, solver iteration sweep or
changed failure tolerance under this ID. Every unresolved full-calibration
requirement remains open regardless of a bounded positive result here.

## Captured results (revision2)

The fixed calculation gives R=4.38603843, omega=41.03931210rad/s. Exact rounded
V11 K/D are stored in the factory;20 K and25 D differ from V8. Both model
spectral radii are0.99508149/0.99515064 (residuals<4.1e-15; alternate16-kick/drift
map difference<8.9e-16). Finite960-step2-norm amplification peaks27.15/27.39:
these non-normal coordinate transients preclude treating poles as an energy
or native-robustness certificate. V8 comparison radii53.8525/53.8495; V10
2.88846/2.88833. Stable/unstable manufactured controls are distinguished.
An initial dimension mismatch in the one-DOF manufactured check was fixed
before emitting the calculation; no gain was changed or selected after a retry.

Native V11 passes25/25 isolated step responses and the simultaneous sine.
Across isolated cases: maximum speed1.455307rad/s, selected plateau RMSE
<=0.001733673rad, selected return RMSE<=0.006347937rad, largest unselected
excursion0.033073rad, worst all-channel final RMS speed0.048396018rad/s.
The simultaneous sine's maximum joint RMSE is0.012045269rad. These pass their
predeclared criteria. Full240Hz per-channel spectra/absolute RMS are retained;
a high power fraction with vanishing absolute motion is not a large oscillation.

But simultaneous positive step case25 fails at substep268 on forbidden
self-contact between rear feet (body tokens1006/1013, shapes10005/10011).
This makes the all27-world conjunction FAIL; it is not26/26 after censoring.
V8 completes14/27 horizons and passes0/27 response criteria;11 trials stop on
velocity violations,2 on forbidden contact. Its completed trials retain
large near-limit oscillation. Full V11 repeat and historical V8 standing control
are byte-exact. Conditional standing/load trials are NOT_RUN by the frozen
response stop rule. V11 remains diagnostic, not a selected calibrated body.

External root `/home/kaifaty/NextEngine-training/body-bandwidth-lDjoJM`.
Pre-result `contract-r2.md`, full tracked/untracked source diff, commands,
calculation/controls,27+27 native cases, repeat and descriptor identities are
sealed by `manifest.json` SHA256
`61e757acfdb6c8dbd15ac33e266b00245472b31849fc1caf4f7de20ba32651f4`.
Calculation SHA256 `5dece9d3940a2cd8c00e4a61281346a98a47746d117badd590b56aff20b59272`;
native V11 SHA256 `4dd053fc5a3fb2d68aadf7b9301dd8555f9ca64870ce0f786f7974e303cdfbaf`;
summary SHA256 `ba08c49b1d389252953667062cb3ef5966d83d111a35991f0fcdb87e17e7038d`.
Python3.12/NumPy2.5.2; calculation uses the existing reviewed operator extraction
and map helper as author consistency machinery, not an independent oracle.

## Separate post-result target-geometry triage

Without changing r2 inputs or its FAIL, inspect the exact requested q0+0.05
plateau with full forward kinematics and all15 separating axes for each of
the four left/right foot OBB pairs. All four pairs overlap in this f64 geometric
model: maximum separating-axis gaps are-39.1794,-13.2455,-13.2455,-33.9969mm.
The initial pose instead has positive gaps47.83–57.83mm. Thus an exact tracker
would be asked to bring its feet into one another. ROM-valid individual angles
are not automatically a collision-free combined pose.

FK reconstruction compared with native initial/last-failure frames has maximum
component position error7.28um and rotation-matrix error8.03e-7. This is an
empirical correspondence check at those two states, not a universal bound.
The last actual foot gap is about-1um, below that correspondence resolution;
do not infer physical penetration depth from it. The recorded active impulse
is the native self-contact witness. The large requested-pose overlap is a
separate static infeasibility observation, not proof of the entire transient's
cause. Manufactured separated/overlapping boxes test the sign convention.

`target_geometry.py` SHA256
`995d5760c5d43e70cb25c4dbd853f90ec5f72c3e8e686559d81f671d7d0ac7e2`;
`target-geometry.json` SHA256
`24a7d052fa515193a11127506f5529d541c2ae85c6e6719551e5c90b35c8d0e8`.
These post-result files are separately sealed, not retroactively inserted into
the original manifest. Their geometry review is reported with the independent
review below. No source geometry or collision filtering was edited.

Next action: preserve r2's failure and the fixed V11 gains. Check a physically
collision-free simultaneous target/path before a separately versioned input
test (for example, paired hip abduction instead of simultaneous adduction).
Do not choose new gains to track an intersecting pose, silently replace case25,
discard failed data or select V11 for standing/training from26 positive cases.
Loaded support, relevant loaded poses and disturbances remain open.

Primary-source context rechecked2026-09-06:
[ovphysx stability guide](https://nvidia-omniverse.github.io/PhysX/ovphysx/latest/guides/articulation_stability.html)
(updated2026-08-21) relates gains to frequency/inertia and cautions about coupled
constraints and drive limits. It motivates the bandwidth question, not our
particular row-sum rule or its native success. The
[Isaac Sim6 gain tutorial](https://docs.isaacsim.omniverse.nvidia.com/6.0.0/openusd_tuning_tutorials/tutorial_06_joint_gains_tuning.html)
(updated2026-06-05) calls for isolated/parallel step and sine tests and avoiding
problematic poses that create unrealistic contact. That supports checking the
combined input's geometry, not retroactively turning the failed r2 trial green.

## Checks

PASS:156 native motor library tests, one new native probe input test,
all-target native Clippy with `-D warnings`, formatting, six boundary checks,
content-package, exact candidate repeat and historical V8 standing output.
Diff whitespace and753 local documentation links pass.
Full host/performance/game/play/replay and training: NOT_RUN (isolated new
unloaded diagnostic, unchanged game/standing consumers). Native all27 response
acceptance: FAIL. Independent executable review: PASS for the bounded evidence
and implementation checks; the full27 response claim remains REFUTED.

## Independent executable review

One fresh review pass closed without a candidate repair or native rerun.
The independent checker imports no author/repository helpers and reconstructs
both raw B operators, rounded gains, coupled maps, all54 stopping boundaries
and1,138,350 applied effort channels/flags. Every sealed input matches. It
confirms25 isolated responses and the sine pass, while case25's first material
rear-foot contact has impulse0.39917769Ns at268/240s; later metrics are censored.
No load-bearing apparatus defect was found. Allowed transient self-contact in
cases1/8/26 means these are unloaded, not constraint-free response tests.

A separate fresh homogeneous-transform/vertex-projection SAT implementation
confirms all four exact-target foot overlaps and native FK correspondence.
This establishes endpoint overlap only, not infeasibility of every trajectory
inside the0.01rad RMSE tolerance region or unique dynamic causality. Neither
review changes the failed r2 conjunction or admits standing/load/training.

External `independent-review.md` SHA256
`1726a9f05b0cc3a502a3cd5981bcd1effe2db5d2b2561cab741c948051153594`;
companion `independent-review.sha256` SHA256
`6fade887cc8cde14f92734a64eefad337eec2d6cf6a43df72855769216807dc8`.
All five sealed review files verify. Both independent checkers/results remain
beside the frozen native evidence, outside Git.
