# R8b standing startup reference — contract revision 1

Research ID `STANDING-STARTUP-01`, status `REFUTED` for sufficiency of this
exact ramp; `INCONCLUSIVE` about the old velocity failure's dominant cause.
Consumer: distinguish whether the cold target transition contributes to the
V9 regression before changing gains/control architecture again. Body/mass and
all safety remain frozen; this is not a second BODY-DAMPING-01 damping candidate.
Authority: SPEC-35/ADR-119/120 diagnostic-only scope.

H1: abrupt cold reference transition contributes materially to the early
observed velocity violation. Prediction: a fixed startup ramp delays/removes
the first18-step failure. H2: sampled/contact/coupled dynamics still violate
safety even with smooth target startup. Prediction: the ramp also fails within
the30-second horizon. Neither outcome alone proves the native causal mechanism.

One fixed input change, no search: for motor ticks1..60 multiply the complete
existing standing reference by tick/60, with signed integer ties-to-even;
after60 use the original reference unchanged. All canonical reset joint angles
are zero. No imported warm pose, mass/gain change, impulse injection or altered
safety. This ramps the reference, including its feedback component, not just
the ankle bias; keep that distinction explicit.

Finite budget: one original30-second standing world each for V8 and V9 with
the same60-tick ramp, one exact repeat of the V9 outcome. Existing unmodified
standing outputs are controls. Keep all substep/state/effort/target evidence,
stop each world at the first original safety/terminal failure or1800 ticks.
No alternate ramp durations, quintic variant, reset pose or gains under this ID.

Primary criterion:1800 ticks/7200 steps and original terminal.timeout without
safety failure. Delay without timeout is not a standing repair. A timeout is
nominal support only, not uprightness, heel-rise or disturbance recovery.
Always report body identity and exact first failure; inspect torso and feet if
timeout succeeds. Test zero/midpoint/end/outside and signed ties in the ramp,
preserve no-ramp bytes, and independently review input/effort/stop correspondence
before interpreting native outcome. Max one initial plus one batched rereview.

Stop on failure, retain it. If V9 still fails, reject startup ramp as sufficient
and compare first failing mode/contact/safety clipping against the old startup.
Do not extend training or adjust tolerances. Any selected reference/profile
implementation requires separate Accepted identity/admission; this harness
only applies a labeled diagnostic target input through production safety.

## Result and independent correspondence

Both V8 and V9 stop at physics substep1: left knee-14urad then right-13urad,
versus minimum-10urad including the original observed tolerance. No velocity
violation. All recorded samples/substep witnesses are identical between bodies:
their plant, stiffness and reset match, and zero starting velocity eliminates
the damping term in the only applied frame. V9 repeat is byte-exact.

Actual reference: knees1667urad, ankle-pitch-2333urad bilaterally, other channels0.
Targets are unclipped. Independently reconstructed efforts are knees833500uNm,
ankles-933200uNm, others0; no target/effort rate, magnitude, power or work clipping.
This exact smooth-start input fails an earlier ROM boundary, so it cannot
establish whether abrupt startup causes the old18-step velocity failure.

Fresh independent reviewer verified all6 packet hashes,22995 signed ramp
cases against Fraction rounding, exact input/effort/DOF mapping and ordered
first failure. Largest numerator<=60*2^63 fits i128. Reconstructed anatomical
foot impulse norms1339939 /1339808uNs are below6Ns. These are independent raw
measurements, not classifier outputs: joint safety exits before classification.
No load-bearing finding or rereview; no independent native rerun required for
the complete identical first-step witness. V9 no-ramp control remains byte-exact
to its frozen18-step predecessor. V8 no-ramp ancestor completes7200 steps.

External packet:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/standing-startup-01`.
Frozen contract SHA `a014fe3d4431d279ae2f8ca4a88d5876fbe6edfc6e5f78e8d271d67681f1b88c`;
harness SHA `3f6617c06cd16f753fca2693ddb9d4457f3cc38126a6f43fcd3ea8d84dc44134`;
V8 JSON SHA `e73c59a15a21586a51c22390101d75f7e7d496570d5eaca928c0f233872bc6f4`;
V9/repeat JSON SHA `111a55500d23bc361f94a5b1bb0d3680f7411ae8bbb529dd8cdafeb313bade19`.

Actual first invocation, with exact SDKf259d3da… from task-state:
`cargo run -p next_motor --features physx-sdk --example probe_biomechanics_body_standing -- 9 0 0 sampled-v4 per-iteration unchanged startup-ramp`.
V8 invocation used the resulting `target/debug/examples/probe_biomechanics_body_standing`
with `8 0 0 articulated-v3 per-iteration unchanged startup-ramp`; V9 repeat
used the same binary and original V9 arguments. No-ramp control omitted only
the final argument. At handoff that unchanged native executable SHA is
`4636a83ffd64c3e8d81571034affcf350c69d55e21765cfe06f517844efab7b4`.
Rust1.97.1(8bab26f4f), native SDK/source same as BODY-DAMPING-01. The executable
hash was captured after the runs, not a pre-execution seal; no intervening
example rebuild changed it. This records the review's provenance limitation.

PASS:6 standing-example tests (including signed/end/boundary ramp vectors),
native all-target Clippy, format/diff. Parent V9 change passed152 native motor
tests,11 Python tests/Ruff, boundary/content/play/replay; no library changes
followed those checks. Broad host/performance/training not run.

Decision: no selected startup/profile/body change. Do not sweep ramp duration
or loosen ROM tolerance. Next separately bound a production-restored initial
stance with knees inside ROM and measured ground sole positions; this is an
initial-condition discriminator, not a claim it repairs later foot control.
If that succeeds, it still needs the original transfer/disturbance criteria.
