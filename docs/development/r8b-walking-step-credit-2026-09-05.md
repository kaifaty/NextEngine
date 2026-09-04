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
