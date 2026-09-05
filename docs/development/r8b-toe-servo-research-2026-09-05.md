# R8b toe servo research — contract revision 1

Research ID: `FOOT-SERVO-01`. Status: `INCONCLUSIVE` for the frozen contact-free
MTP-failure claim; finite native control/effort observations below.
Consumer: select the next correction to the articulated foot, without changing
mass, safety limits or admitting a training environment.
Architecture snapshot: `c2e14d42`, Accepted SPEC-35 and ADR-118/119; exact V8
body and CompiledV4 identities from the [standing report](r8b-articulated-standing-2026-09-05.md).

## Frozen experiment

Question: is ground contact necessary for an observed MTP velocity violation
under the following finite native control experiment? The negation is one
contact-free run that violates the unchanged MTP velocity bound. This is not
a theorem about all references or proof of the cause of the loaded-transfer run.

Six fresh native worlds: root translated upward by either 0 or 10 m, each with
input side none/left/right. Use the production state restore API; preserve root
orientation/velocities and every joint. Verify restored joint state, rotations,
velocities and rigid translation (canonical position tolerance 2 micrometres).
Use the exact existing standing V3 controller and safety controller. Modify
only the selected MTP target: ramp 0->50000 urad over ticks1..15, hold16..30,
ramp back31..45, zero46..60. Integer division truncates; no residual action.
Initial height is an experimental boundary condition, not altered body bytes.

Run at existing 240 Hz physics /60 Hz motor rate for at most240 substeps per
world, stopping on the first observed joint safety failure or controller error.
Retain initial state and all post-step poses, joint states, applied targets,
requested/applied efforts and clamp flags, raw contacts and failure. Existing
effort/rate/power/work and observed ROM/velocity tolerances remain unchanged.
This free-fall diagnostic does not apply a standing fall/timeout evaluator;
it cannot establish standing, contact-impact admissibility or recovery.

Controls: no toe input in both height conditions; confirm signal bounds and
return, exact repeated output, rigid initial translation and no contacts in the
raised trials. Any raised contact invalidates a contact-free interpretation.
If a non-MTP joint ends a raised run before the MTP witness, that run is censored,
not a stable-toe result. If the no-input case itself fails, input is not necessary;
the shared controller/body remains a possible cause. A grounded no-input prefix
must match the preserved nominal standing trace. No sampling or retries to green.

Competing explanations:

- H1: sampled toe control/effort-rate interaction can fail without ground contact.
  Witness: an MTP safety failure in a contact-free raised run.
- H2: ground constraints are needed for the observed failure in this experiment.
  Prediction: raised runs finish without MTP failure while grounded trials fail.
  A single raised MTP witness refutes necessity, not contact's contribution.
- H3: whole-body control and reference feasibility, rather than a local toe law,
  determine the failure. Inspect other-joint failures and full reference history;
  retain this explanation even if H1 has a witness.

Budget: six one-second worlds, one exact rerun, focused tests and at most two
independent reviews (one initial, one after a batched material repair). Freeze
source/output hashes before review. No gain sweep, longer rollout or body edit
under this contract. Inconclusive if controls fail or another joint censors the
necessary comparison. Native f32 physics and canonical integer measurements;
analysis may use f64, but actual termination is the production integer check.

## Prior art and applicability

Read on2026-09-05: NVIDIA's [PhysX articulation documentation](https://nvidia-omniverse.github.io/PhysX/physx/5.6.1/docs/Articulations.html)
distinguishes implicit built-in drives from explicit externally computed PD
efforts. It describes constraint-order conflicts with contacts and limits;
it does not certify our external PD or pinned5.9 native execution.
The [ovphysx stability guide](https://nvidia-omniverse.github.io/PhysX/ovphysx/latest/guides/articulation_stability.html),
updated2026-08-21, recommends considering angular inertia and timestep together,
and warns that stiff control and contact constraints can interact. These are
reasons for the contact-free discriminator, not permission to change mass,
velocity limits or use an implicit-drive stability claim for our controller.

Next after the result: only if a raised witness exists, separate the local
sampled PD/rate limiter from coupled multibody response with an independently
checked scalar/replayed safety calculation. If raised runs pass, investigate
loaded contact/reference feasibility. Neither outcome alone authorizes a fix.

## Result — native observations, not a local-cause proof

The frozen claim remains inconclusive. All raised runs have self-contact
records from step1, so none meets the predeclared **no contact records** control.
Those records have no ground endpoint and every reported impulse component is
zero throughout all three raised runs. Do not silently redefine the contact
firewall to turn these into contact-free successes. The raised-right run also
ends on a different joint, censoring its MTP observation.

| Initial root lift / MTP input | Steps | Terminal observation |
| --- | ---: | --- |
| 0 m / none | 240 | Horizon; exact preserved nominal prefix |
| 0 m / left | 240 | Horizon; selected MTP peak8.000051rad/s |
| 0 m / right | 240 | Horizon; selected MTP peak8.000011rad/s |
| 10 m / none | 240 | Horizon; both ankle-roll and MTP approach8rad/s |
| 10 m / left | 240 | Horizon; both ankle-roll and MTP approach8rad/s |
| 10 m / right | 198 | Left ankle-pitch8.002205rad/s exceeds8.001 observed bound |

These horizons are not quiet-motion successes. In both grounded moving trials
the selected MTP position changes by as much as33.494/33.442mrad per substep.
The rate limiter engages101/100 times, including16 times per side where the
applied effort opposes the sign of the unconstrained PD request. This is the
declared effort intersection, not evidence of an implementation sign bug.

The raised **no-MTP-input** run is particularly informative about the next
scope: at step1 both MTP inputs, target, requested moment and applied moment
are zero. Their output positions are nevertheless-25,512urad and velocities
-7,999,975/-7,999,999urad/s. The only nonzero applied moments are bilateral
knees+14.583333Nm and ankle-pitch-9.166667Nm. Thus a toe's own PD moment is not
necessary for this first-step relative motion. Whole articulated response
must remain in the next model; this does not isolate knee versus ankle torque,
solver constraints or the later loaded-transfer failure. On step2 both
ankle-roll channels are rate limited; the unloaded problem is not toe-only.

## Correspondence and verification

Native initialization preserves all joint states, orientations and velocities;
the raised canonical rigid-translation discrepancy is at most1 micrometre
(frozen tolerance2). Grounded no-input control matches all240 old substeps in
joints, raw contacts, root pose/velocities and efforts, plus all60 full-body
motor samples and reference/applied targets. A fresh six-case native rerun is
byte-exact. The independent Python rational effort oracle reproduces all34,950
applied moments and flags, including rate/power/work intersection and work reset.
It uses integer divmod rather than Rust truncating division for ties-even.
Raw requested moments are reconstructed from the retained pre-state/target and
hash-bound descriptor gains; they are not a native force readback.

Independent review: accepted the inconclusive claim and bounded observations;
no load-bearing correspondence defect, no repair/re-review required. The reviewer
verified input/source/output hashes, independently reconstructed the V3 descriptor
to V4 outer hash, and used Fraction/built-in rounding plus median interval
projection (without importing the audit) for all34,950 reference/target/effort
channels, including flags and failure precedence. Initial translation, census,
no-ground/zero-canonical-impulse observations and first-step MTP response match.
The review's exact commands/results are in task tool evidence; no separate
reviewer script was authored or native rebuild consumed.

Non-load-bearing limitations for future tool reuse: the audit does not itself
reject a truncated horizon or inverse terminal mismatch; baseline zip can shorten
coverage; descriptor-to-outer-hash closure is not automatic. The reviewer checked
these on the complete captured inputs, so they do not alter this result. Also,
reference/begin-tick errors would exit the native tool before aggregate JSON;
none occurred. This audit is not a general admission verifier. Zero canonical
impulses do not establish absence of native constraint effects.

No source body,
shared control law, safety threshold, compiler, bridge or training selection
was changed. The probe and audit are intentional reproducible diagnostic tools.

Reproduction (existing pinned native SDK from task-state):

```sh
cargo run -p next_motor --features physx-sdk --example probe_toe_servo
python -m lab.scripts.audit_toe_servo DESCRIPTOR BASELINE TRACE OUTPUT
python -m unittest lab.tests.test_toe_servo
```

External root:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/toe-servo-01`.

| Artifact | SHA256 |
| --- | --- |
| native.json and repeat.json | `d56dc0c8950273bc5468af9f01d416a5a625a594f90ffbd425ba1887d9afc5e2` |
| report.json | `e1ebef6971efefd78cdbde4af7d0899904b5aa3574c97c793910f8af63ceac19` |
| Native example source | `d5085e61622fd3a3186f3c7274165cf88472474820eedd6b64f58d2dae39b1ea` |
| Python audit source | `14a0336fcc642642eae3d0c7077e3f390ade46cd7a4f4ff8bd19483114c13ce3` |
| Python tests | `95faa118b1fe8fd154529a342af8eeaf0e57d4855b11af65d2cb46b72a03eed5` |

Contract before result/status additions SHA256
`6f391878ac8db34b6312a4ab902eaf2ee1b6b9b7ff89135570a85ba5e29ba4c1`.
Input hashes are embedded in report.json; descriptor-r2 and standing baseline
are the exact artifacts from the prior standing report. Rust1.97.1,
Python3.11.15, Ruff0.16.6; native SDK profilef259d3da… unchanged.

PASS: one native example input test, three Python manufactured rounding/rate/
power/work tests, focused native Clippy, Ruff, formatting, boundary-scan and
content-package. Full motor suite, broad host-check, gameplay/replay, performance
and training: `NotRun(ExampleAndAnalysisOnly)`; no runtime library changed.

Claim ledger: `SUPPORTED_BOUNDED` (`EMPIRICAL`, `CORRESPONDENCE`) for these six
trials and exact controls; `INCONCLUSIVE` for the frozen contact-free MTP
necessity question. No analytic stability proof or mass-correction claim.

## Changed next action

Do not repeat FOOT-SERVO-01 with a larger signal/longer horizon or reinterpret
raw zero-impulse records as meeting its empty-contact criterion. The immediate
next discriminator is the **first-step coupled knee/ankle/toe response**, with
an all-zero-effort free-fall control and separately applied knee/ankle input.
Retain the same native body and inspect all joint states/contacts, not just MTP.
Then evaluate sampled ankle/toe PD and rate-limiter behavior against that
response. A fixed-rear scalar toe alone cannot explain the observed zero-own-
effort first step. This changes the investigation scope without moving mass,
weakening safety or declaring the requested body/standing/walking goal complete.
