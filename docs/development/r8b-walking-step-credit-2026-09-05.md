# R8b — foot-return safety and step credit

## Exact native discriminator

The unchanged nine 600-tick alternation tapes reproduce all terminal ticks,
reasons and joint facts from `alternation-probe-03.json` in -04. The new native
example replays the same Q1.30 actions and shard identity through the production
vector runner. Its diagnostic fields do not change physics, safety, rewards,
the wire protocol or replay roots. Only failing safety frames copy a checkpoint.

External evidence root:
`/home/kaifaty/NextEngine-training/r8b-canonical-walking-v1/`.

| Artifact | SHA-256 |
| --- | --- |
| `alternation-tapes-01.json` | `0987882cd141de289739587e8096e4e2ab9f37fe5adb4a6be55109e70a7f9608` |
| `alternation-safety-02.json` | `a5058d7c5c8aa27443cfa168e9d29118b396f0923a3d8681d469be10adad6f15` |

The report closes its diagnostic executable and input tape hashes. It is a
development-tree experiment, not a clean-commit training generation.

Competing explanations for duration=1, amplitude=1 at tick 115 were observed
hard ROM/velocity, positive-work depletion, and power/rate incompatibility.
The terminal is `MOTOR_SAFETY_EFFORT_ENVELOPE_EMPTY` **before the first physical
substep**, with zero work charged in the new motor tick. Right ankle roll has
velocity -4,160,966 µrad/s and previous moment 133,526,893 µN·m. Its maximum
moment change is 5,833,333 µN·m/substep: the rate interval starts at
127,693,560 µN·m, while the 500 W power constraint caps magnitude at
120,164,404 µN·m. The intersection is genuinely empty. The work cap is
480,657,597 µN·m and is not limiting. This is not an angle-reporting defect.

Five other cases fail observed hard ROM; two fail self-contact; one falls.
The hard-ROM observation tolerance is **10 µrad**, not 1,000 (the latter is
the velocity tolerance in µrad/s). No limit is relaxed. This finite manual
controller drives an unsafe trajectory; it does not prove the body cannot walk
or that the safety algorithm is wrong. Speculative effort-controller repair is
rejected without a demonstrated invariant violation.

## Smallest next learning change

The closed learner never releases a foot. Its binary one-contact reward gives
no graded credit for unloading a foot before contact disappears, and no timing
information for alternating sides. Lower action noise did not change this.
Test a separately identified, observable periodic load-transfer lesson, retaining
the existing body/actions/safety and independent walking-quality gates. Do not
imitate the failed hand-authored joint trajectory.

Primary research, accessed 2026-09-05:

- [Siekmann et al., 2021, sections III–V](https://arxiv.org/html/2011.01387v2):
  periodic swing-force/stance-speed costs with phase inputs learn bipedal gaits
  without reference joint trajectories. Their Cassie setup and 150 million
  samples are not evidence that our morphology or one-million-sample budget
  should succeed. Use the phase-observability principle, not an equivalence claim.
- [Legged Gym's native reward implementation](https://raw.githubusercontent.com/leggedrobotics/legged_gym/master/legged_gym/envs/base/legged_robot.py):
  airtime reward is contact-transition dependent and command gated. It does not
  establish that copying a quadruped reward solves this humanoid's foot release.

Remaining uncertainty: whether graded, phase-observable unloading discovers a
safe step under the unchanged plant. A numerical reward oracle must distinguish
balanced standing, correctly phased unloading, wrong-side unloading and flight
before a new learning run. Physical quality, not shaped return, decides success.

## Implemented V6 discriminator

[ADR-108](../architecture/adr/108-observable-periodic-walking-credit.md) freezes
the new lesson. Pure clock/reward controls and identical-action native V5/V6
physics/safety tests pass. The first external adapter control correctly fails
at tick 0: `MotorLabClient.step_normalized` did not yet recognize V6 and converted
actions using the legacy microradian scale rather than Q1.30. The training
adapter used Q1.30, so the independent control exposed the missing profile
registration. Add V6 to that explicit dispatch and retain a direct regression
test; do not add V6 to unsupported Isaac mirror dispatch tables. The failed
`adapter-check-01` stays immutable, and no optimization occurred in it.

V6 descriptor file SHA-256:
`fb5276f5a3a847497db1d46aa23ade5f216202b8ed1b17005b73d04035310dca`.
External root is `/home/kaifaty/NextEngine-training/r8b-canonical-walking-v2/`.
The previous native executable is preserved in `evidence/next_headless-v5-e3c967db`
with full SHA-256 `e3c967db3e5c92fcd43e130ca768ee6b1c86958e26a9f75f3bf8c38aca9a55e0`;
old manifests are not rewritten when the workspace build output changes.

Preflight: `adapter-check-02` passes 5,120 exact raw transitions and 399 resets
(step-root digest `f0b5384cf070a08a8267fbe5b1a0cffc740273c470a4083b661db15b95d8a13e`).
All 120 native motor tests and five headless protocol tests pass. V5 descriptor
is byte-exact with its previous `0f4610fe…db03` file hash. Focused Python tests
cover clock scaling, final/pre-reset clocks, timeout masking and V6 Q1.30
dispatch. Broad host-check will be reported separately before handoff.
