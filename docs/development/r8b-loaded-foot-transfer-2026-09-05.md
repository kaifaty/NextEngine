# R8b loaded forefoot transfer — bounded diagnostic v1

Status: `V1_AND_V2_LOADED_TRANSFER_FAILED / BOUNDED_SERVO_RESEARCH_NEXT`.
Consumer: determine the next physical/control correction after nominal V8 stance.
Authority: [ADR-119](../architecture/adr/119-articulated-foot-standing-diagnostics.md)
permits bounded native diagnostics; this adds a separate example, not a change
to its strict standing command, body bytes, shared controller or safety profile.

## Inputs, hypotheses and stopping rule

Exact canonical V8 / CompiledV4 / standingV3 / contactV2, unchanged gains and
normal production safety. Three15-second episodes: none, left, right. Only the
selected side's ankle-pitch and MTP reference targets receive the same additive
pulse: zero through tick300; linear0->100000 urad at ticks301..360; hold through480;
linear return at481..540; zero afterwards. Integer division truncates. This is
a single predefined functional perturbation, not a gain search, external push
or a claim of walking. Record all240-Hz physical poses, contacts and efforts.
Stop on old safety/terminal failure or900 motor ticks. No retry/amplitude sweep.

- H1: the articulated foot can transfer loaded support with this small input.
  Predict sustained raised heel with loaded forefoot, then normal rear contact.
- H2: contact geometry/control prevents that transfer (including toe lift,
  whole-body compensation or failure). Predict absent loaded-heel interval,
  excess impact, incomplete return or a safety termination. This does not by
  itself identify whether geometry or the reference law needs correction.

Success for each side: at least60 consecutive physical substeps during the
input window with both rear heel-edge corners above5 mm, the entire forefoot
sole within[-1,+2] mm of ground and absolute net vertical toe impulse >=0.05 Ns
per substep. Then at least240 consecutive substeps after tick540 with rear heel
edge within[-1,+1] mm, rear vertical impulse >=0.05 Ns, and no safety termination
before the15-second timeout. Both sides must pass independently; also report
unthresholded maxima, durations and load changes to expose near-misses.

Control: `none` must match the first900 motor ticks /3600 physical substeps of
the retained V8 nominal trace in targets, applied effort, joint state, physical
pose and raw contacts. Decoder/geometry must use actual descriptor collider
offsets and native rotations, not body-origin lines or inferred toe contact.
The oracle's scalar corner transform and signed contact sum get manufactured
flat/raised/reversed-input controls before interpreting native results.

No output here proves disturbance recovery or a new learning environment.
If H2 occurs, preserve its first failure and use actual contact/joint/effort
order to select the smallest next discriminator; do not relax5 mm or safety.

## V1 result and v2 input discriminator

The zero-input control exactly matches the retained V8 trace for3600 physical
steps (joints, effort, raw contacts and root pose/velocities), and900 whole-body
motor poses/targets. Left/right coupled-input runs stop at substeps1305/1324,
5.4375/5.5167 seconds, on MTP velocity8.063903/8.036823 rad/s (>8 rad/s).
Neither lifts its heel5 mm; instead toe sole corners rise7.20/7.96 mm. Re-contact
criterion is not reached. This rejects H1 for the declared coupled input, not
all possible heel-rise controllers or the complete body anatomy.

The final joint positions oscillate as well as reported velocities: e.g. left
steps1298..1304 go0.033759,0.067016,0.089072,0.079388,0.046141,0.013554,0.010839 rad,
with recurring toe contact loss. A velocity-label artifact alone cannot explain
that position series. Full cause (command, contact, sampled servo/rate cap)
remains unresolved; do not adjust the8 rad/s limit.

V2 changes exactly one causal input: retain the same ankle pulse, but leave
MTP targets at the shared standing controller's neutral zero. Its existing
K/D and all body/safety parameters remain unchanged. This is a neutral elastic
toe target, not zero torque or a passive-joint guarantee. Use the same left/right
episodes, window, amplitude, ceiling and success oracle, no search. Existing
v1 traces and command semantics remain unchanged; v2 has a separate probe ID.

Prediction: if active toe extension/release drives the observed failure, v2
should retain toe load and reduce/eliminate that position oscillation, possibly
achieving heel rise. If v2 also fails, command removal is insufficient and a
bounded contact-free/loaded toe-servo discriminator is needed before gain changes.
No future result may reinterpret the coupled v1 run as a successful transfer.

## V2 result and next decision

V2 zero input again matches the retained nominal trace. Neither side achieves
the loaded-heel or return criterion. Left input stops at2042 (8.5083s), on left
ankle pitch -8.011794 rad/s; right input stops at1810 (7.5417s), on the opposite
left MTP -8.001789 rad/s. V2 therefore refutes removal of the active toe target
as a sufficient repair. It changes the failure timing/location, not success.
The left run eventually lifts forefeet far from ground (maximum sole corner
about0.245m), so this is not a near-miss to relabel as loaded heel rise.

The first analysis labelled any velocity above the nominal8rad/s as a violation.
Source inspection and native stopping order show the existing observed-state
tolerance is1000urad/s: failure requires strictly more than8.001rad/s. Corrected
the reporting oracle (not the simulator/safety) and added both-sign threshold
tests. `loaded-foot-transfer-02/report.json` is the earlier uncorrected diagnostic;
`report-final.json` agrees with actual stop steps2042/1810. Preserve both.

Next is a bounded research cycle before any similar pulse/gain variant: separate
sampled toe-servo/effort-rate behavior from loaded contact response using a small
contact-free control and explicit successful baseline. Inspect the actual rate
limit (120Nm/s gives0.5Nm per240Hz substep), inertia/force schedule, contact state,
and position increments; do not infer mechanism from one velocity spectrum.
Use relevant primary-source research when choosing a controller intervention.
Whole-body weight transfer/reference feasibility remains a competing cause;
a stable unloaded toe alone would not prove the heel-rise command feasible.
No mass/gain/ROM/contact ceiling or shared standing controller was changed here.

## Reproduction, artifacts and checks

```sh
cargo run -p next_motor --features physx-sdk --example probe_articulated_foot_transfer -- none
cargo run -p next_motor --features physx-sdk --example probe_articulated_foot_transfer -- left
cargo run -p next_motor --features physx-sdk --example probe_articulated_foot_transfer -- right
# v2: same three commands with final argument neutral-toe
python -m lab.scripts.audit_articulated_foot_transfer DESCRIPTOR BASELINE EXTERNAL_DIRECTORY
python -m unittest lab.tests.test_articulated_foot_transfer
```

Use the existing pinned native SDK and external Python recorded in task-state.
External directories are under
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05`, named
`loaded-foot-transfer-01` and `loaded-foot-transfer-02`; the V8 descriptor is
`body-v8-01/descriptor-r2.json` and baseline `articulated-standing-01/standing.json`.

| Native artifact | SHA256 |
| --- | --- |
| v1 none | `50cfb71e55de6b4e9eee77b962f562c724a8dda24c6ba99781d3a1805298ed9f` |
| v1 left | `6e2e25da2c67c4e7797d23f3cb631773913aebbaddeb17148150733d5f1eea7f` |
| v1 right | `46bea86506b96612fbfa407fd45fdc980efddbb3bc521ebb1569a51eb4073ac6` |
| v2 none | `30956f2d83916250567278a93b9f07294bf6e3762eeeecdd3c84e7dadead4af6` |
| v2 left | `39abb822d4ab15991828485a2bd124882c1bb69576abaef020179eee69b1e12a` |
| v2 right | `9f055b2ec9c02fe9d704d7fba3c00f066336b871c77f22c7d011edaae3e5621c` |

Final probe source SHA256 `4df422aa9e4ba368e57b120b69205a1755a72945e0b2eec7077e2c7a55032d67`;
oracle `b6c666ca7f96b977cfbb9d1d179ca28682edce4dde99f7cd49e31d5d3f5a91b6`;
tests `57d1c316da93d5cfe42975cea675eb5f12f3c0843da52dff0fa2214190363cb6`.
V1 source before the v2 extension was7e845a3a80f85c7084834ee4e6272aefecfc038d60a8ba9dcbd0018e82406872.

PASS: pulse bounds/symmetry/slew native-example test; five Python manufactured
geometry/load/duration/complete-interval/velocity-boundary tests; focused Clippy,
Ruff, format, boundary-scan, content-package and diff/link checks. Initial Rust
compile failed on a missing `?` in schema-hash output, repaired before any native
trial (`none-build-failure.log`). Initial Ruff import-order warning repaired.
Dynamic loaded-transfer criterion: FAILED in all four moving cases. Full native
motor suite, host-check, play/replay and training: not run for example/analysis-only
changes; runtime library remains exactly at the prior verified commit. No admission
or completed-body claim follows from passing diagnostic-code tests.
