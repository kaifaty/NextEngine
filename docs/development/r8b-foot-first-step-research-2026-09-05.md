# R8b first-step foot response — contract revision 1

Research ID `FOOT-RESPONSE-01`; status `SUPPORTED_BOUNDED` for the five-case
native input isolation below, not a stability/physical-correctness claim. Consumer: isolate
which initial proximal input generates the immediate unloaded MTP response
seen in [FOOT-SERVO-01](r8b-toe-servo-research-2026-09-05.md), before choosing a
control correction. Snapshot `b38ff9c8`, same Accepted SPEC-35/ADR-118/119
diagnostic boundary, canonical V8 / CompiledV4, existing safety and native SDK.

Five fresh worlds, each with the same production-restored initial pose lifted
10m. Exactly one240-Hz physics step per world, starting with zero velocity and
fresh safety history. No body/property/physics changes. Input cases are:

1. `zero`: every joint target zero.
2. `knees`: only bilateral knee targets +100000urad.
3. `ankles`: only bilateral ankle-pitch targets -140000urad.
4. `both`: those two groups together (previous raised-none first-step control).
5. `ankles-small`: only bilateral ankle-pitch targets -250urad, giving -0.1Nm
   each from K400Nm/rad before clipping; this tests whether a much smaller
   applied moment gives an unsaturated response, not an inverse-mass calibration.

All inputs pass production target/effort/rate/power/work intersection. Record
every pre-state/target/applied moment/flag and post-state joint/link/contact
observable. Validate observed safety after the one step and retain failures.
No standing terminal is used for free fall; no rollout stability claim follows.
Physics nativef32, measurements canonical integer, X-right/Y-up/Z-forward;
joint signs and rotation frames are unchanged. MTP is relative joint motion,
not necessarily the same as toe world angular velocity; inspect both and the
rear foot world velocity instead of calling one the other.

Controls and falsifiers: `zero` should have all25 joint positions/velocities
within1urad /1urad/s and no nonzero canonical contact impulses. Failure means
reset/gravity/constraint baseline needs investigation first. `both` must equal
the complete first raised-none state and efforts in retained FOOT-SERVO-01.
No ground contact endpoint in any world. Preserve all raw self-contact records;
zero reported impulse is not absence of native constraints. All initial states
must match exactly, and one repeat must reproduce the complete output.

Competing hypotheses: knees alone, ankle-pitch alone, or their combination is
needed for the large first-step MTP response. Quantified finite question: which
of these cases reaches absolute MTP velocity>=1000000urad/s on either side?
Also report all unthresholded positions and velocities, so a criterion does
not hide near misses. A single group's above-threshold response refutes the
other group's necessity for this first-step witness, not all later failures.
The small-ankle case can expose magnitude dependence but cannot prove linearity
from one point. Keep numerical constraints and coupled physical reaction as
alternative mechanisms; do not use this as a universal stability proof.

Budget: five single-step cases, one exact repeat, focused tests, and one fresh
independent review (at most one batched repair/re-review). Stop if controls fail;
do not alter thresholds or add a search under this contract. Next action must
follow the observed group separation; no toe-only gain sweep or arbitrary mass
shift. This is a materially different input-isolation experiment, not a retry
of the previous contact-free failure claim.

## Result and changed decision

Both groups independently exceed the frozen1rad/s MTP threshold. Ankle-pitch
alone is much stronger, but do not report knees as insufficient or ankle input
as necessary. None of the cases violates the production safety bound8.001rad/s.

| Case | MTP position L=R, urad | MTP velocity L/R, urad/s | Applied effort per side, Nm |
| --- | ---: | --- | --- |
| zero | 0 | 0 /0 | 0 |
| knees | -2423 | -1094778 /-1094778 | knee+14.583333 |
| ankles | -24838 | -7999977 /-8000002 | ankle-pitch-9.166667 |
| both | -25512 | -7999975 /-7999999 | both preceding efforts |
| ankles-small | -338 | -152740 /-152740 | ankle-pitch-0.1 |

Every MTP applied moment is zero. The large ankle target is first slew-clipped
to-80000urad, then its moment is rate-clipped. The small input is not clipped.
The zero control has exactly zero motion in all25 joints, not merely within
the1-unit tolerance. All five worlds retain eight self-contact records with
zero canonical impulses and no ground endpoint. This does not exclude native
constraint effects below canonical resolution or non-contact joint constraints.

Frame check: left rear-foot/toe world angular velocity X is respectively
-3.472154/+4.527822rad/s for ankles, and-0.043610/+0.109129rad/s for small ankles.
Both MTP axes are-X, so the negative relative joint speed combines opposite
rotations of the two foot segments. It is not an8rad/s toe world rotation.

This rejects spontaneous first-step reset/gravity motion at zero effort and
the necessity of either one input group for the declared threshold. It exposes
strong proximal actuation and magnitude dependence, not a bad mass estimate,
inverse-mass matrix, or a reason to change an isolated toe gain immediately.
The controls remove the missing-zero-input ambiguity from FOOT-SERVO-01;
its original stricter contact-free claim remains inconclusive.

Next: inspect the small-signal coupled ankle/rear/MTP response and sampled PD
feedback before choosing a new gain profile. In particular, do not assume that
an external constant-per-step moment has the same discrete stability limits as
a built-in implicit PhysX drive, or as a single semi-implicit Euler update.
The pinned TGS source applies external efforts and records motion in each of
the16 position iterations; its controller still reads state once per240-Hz
step. A scalar model must state which parent/body motions and constraints it
excludes, and cannot certify the unloaded humanoid by itself. No more unchanged
first-step input variants under this contract; retain this result as control
when evaluating an actual controller correction. Full loaded transfer and
disturbance recovery remain open, and the user goal remains active.

## Independent verification and exact evidence

One fresh reviewer verified all contract/source/raw hashes and independently
recomputed all125 target/flag/DOF/effort channels with rational ties-even
arithmetic; all125 post-joint states pass production ROM/velocity checks.
Initial states and reset roots are exact across the five trials. The `both`
initial state, reset root and **entire first frame** equal prior raised-none.
One native rerun is byte-exact. The unchanged no-argument mode reproduces the
entire prior six-case output byte-for-byte. No repair/re-review was needed.

Descriptor-r2 contains base CompiledV3 hash5b7bc412…; the native trace carries
outer CompiledV4 hash4b8de3a8…. The reviewer independently reconstructed that
outer hash with the force-schedule domain/version/tag. It is not a hash mismatch
or permission to compare arbitrary compiled profiles. Exact descriptor SHA:
`0fe0252fe8e1c63caaf323bc4c555b4efacf81765db0a542ac851e116e07cf7f`.

External directory:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/foot-first-step-01`.

| Artifact | SHA256 |
| --- | --- |
| native.json /repeat.json | `eae43bc0af5c5b3aa070a5419e52fdbfa2ec3ad4fcc1113a01bd65907ab2f0d8` |
| report.json | `4a6e639adedf55817374439fee151dd246bd478a5cd2c652d7080d536dba7bf0` |
| Native example source | `36e01d9935d7f6327c8f7d771a01b3a4b474ed37a107aea4fffaeddcf403bd63` |
| Frozen contract before result/status additions | `f1aafcd53cb78f882d1ec86ab050e8706ac78bf253578c0cbb9731d701fdda1f` |
| old-mode.json /prior toe-servo-01 native.json | `d56dc0c8950273bc5468af9f01d416a5a625a594f90ffbd425ba1887d9afc5e2` |

The retained external `inspect.py` writes report.json, sealing itself, the
reused audit helper, descriptor, old/new inputs and native example source.
The independent reviewer used raw inputs and its own arithmetic, not that helper.
No external script or generated trace is distributed in the repository.

Reproduction: `cargo run -p next_motor --features physx-sdk --example
probe_toe_servo -- --first-step`, with the exact SDK from task-state.
Toolchain unchanged: Rust1.97.1, Python3.11.15, pinned native SDKf259d3da….
PASS: two native example tests, focused Clippy, formatter/diff checks and both
exact native output comparisons. Body/library code remains unchanged; full
motor suite, host-check, play/replay, performance and training were not run for
this example-only extension. Boundary-scan passes all six checks;
content-package passes123 records /64 chunks. No broad ProductCheck or body/
training admission is inferred from this diagnostic success.
