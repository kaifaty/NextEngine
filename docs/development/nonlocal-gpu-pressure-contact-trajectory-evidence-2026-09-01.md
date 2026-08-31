# NCGP13 Nonlocal pressure/contact trajectory evidence

Date: `2026-09-01`

Final result: `PRESSURE_CONTACT_TRAJECTORY_REFUTED / INDEPENDENT_GO`

Claim ceiling: `CPU LONG-DOUBLE PRESSURE/CONTACT COMPOSITION ONLY`

## Question and answer

NCGP13 tested whether the corrected Nonlocal density constraint pressure,
alternated with a separate frictionless analytical box contact, can admit one
step and then hold a tiny 128-particle state for 240 accepted steps.

The one-step 512-particle phase succeeds. Two independently assembled pressure
rounds close the nonlinear density, exact inset, balance, identity, mass and
topology gates. The 128-particle trajectory then commits only step 1. Trial 2
is rejected transactionally because its velocity RMS is
`0.076470122842192428 m/s`, above the frozen `0.05 m/s` limit. The failing
trial remains separately sealed and is not published as accepted state.

This is a valid bounded refutation of the frozen Phase-B fixture, not correct
water, GPU feasibility or performance evidence. A post-review geometry audit
also shows that Phase B is a freestanding open column in a much wider basin,
not a laterally confined hydrostatic column. The result therefore does not
establish that the pressure/contact formulation fails in a correctly confined
tank.

## Frozen identity

- revision-2 contract commit: `ebbfcf771ffec809bc07973653fc918e16b25419`;
- initial implementation commit: `6577f97424be9d858b3d714f10200ccb379d777f`;
- reviewed repair commit: `b1fdcd59fa105e48039bd4c1b5dda72bf7401a64`;
- reviewed repair tree: `c541413c73fee10186c1f9652264b7b64dd473cf`;
- NCGP13 source file SHA-256:
  `ca62174997aaeee928efef9622c9869f7e4f3cf0909fd6f562005684d17e3873`;
- aggregate source root:
  `fe71ebecca1bbac5232927931a65186d145d6f2b093ee48d28f24c505ae28af7`;
- contract file SHA-256:
  `0b40265867d2a619db7928c6de02ba372ec2d4c00c20e422b4a6b35a83341025`;
- aggregate contract-chain root:
  `3c26ed0b806308113384e54198935828bf61fb4a5447cc7311b78285e6b3c904`;
- Phase-A input root:
  `82cf83cbfc5839026c8dff77c55b81be2e5597982c31493a7016a9f5fca2a8fa`;
- Phase-B input root:
  `f9dbf235d5e176efe29468eba26551a1958106a5eb9d6cffbcb74ee57b4b4d1e`;
- profile root:
  `a73091a47fa059c6de5103420b6bac1d11c78eb22d4d68205fdba3bd5efc3816`;
- clean Release binary A/B SHA-256:
  `877c0e99f6984772cb8c74d2105a90e4f15162fd00df8d774c10c69b516662a4`;
- byte-identical 53,407-byte stdout A/B SHA-256:
  `496c450fe80b5bfd3f20b042a4874e69afef8d2604286447ad35f50f6e72246e`;
- final result root:
  `053a6a924a331153a72673d9a5d3809b7a0ea014799c36a3c6cba77ef088c6aa`.

Clean author build directories:

- `/tmp/nextengine-ncgp13-repair-a.pbqzoS`;
- `/tmp/nextengine-ncgp13-repair-b.zENuEO`.

The retained raw author report is outside Git at
`/tmp/nextengine-ncgp13-repair-build1.HFtdVX/run.json`.

## Exact command

```text
cmake -S crates/continuum-water/tools/nonlocal-feasibility \
  -B <fresh> -DCMAKE_BUILD_TYPE=Release
cmake --build <fresh> \
  --target nonlocal-corrected-cpu-pressure-contact -j2
<fresh>/nonlocal-corrected-cpu-pressure-contact \
  --pressure-contact-trajectory
```

Both processes exit zero because they reach a valid scientific
classification. `APPARATUS_INCONCLUSIVE` exits nonzero.

## Phase A — admitted pressure/contact step

The fixture contains `512` dynamic particles and `43,056` immutable basin
ghosts. Surface, viscosity and the retained finite `kappa` penalty do no work;
pressure is an explicit nonnegative constraint solve and contact is a separate
componentwise box projection.

| Observable | Frozen limit | Result |
| --- | ---: | ---: |
| projection rounds | `2..8` | `2` |
| maximum positive density strain | `<=1e-3` | `7.4034751e-5` |
| RMS positive density strain | `<=2.5e-4` | `1.0976352e-5` |
| exact inset penetration | `0` | `0` |
| normalized algebraic balance | `<=1e-8` | `3.3011751e-16` |
| bottom pressure proxy | `> top` | `363.225691 Pa` |
| top pressure proxy | diagnostic | `0 Pa` |
| connected components | `1` | `1` |

Canonical and permuted state, velocity, pressure, contact, work and result
roots are exact. The canonical Phase-A work root is
`bdef981f21f40612b5498707696acc321cff80813ac1e40851dc7a46dbd210e8`;
its result root is
`262c000150447ba4f7229a5450bd139899da7978e2e2a3959ab9e5c3f1d27b6a`.

The retained NCGP12 witness also remains exact: `726` sweeps, `43,612`
coordinate updates, `60` positive multipliers, multiplier root
`e3a62beac5ce4ebb2d39e9baf8ce8a28d95f43ab9291c11f415ba20cb6975b6a`,
trial root
`00f5d0ff3bc77716e4a3da4a209bef8ab1382de6c317710058495e9ecd27b0c5`
and inset crossing `0.1787810703 mm`.

## Phase B — transactional trajectory failure

| Observable | Frozen limit | Committed step 1 | Failing trial 2 |
| --- | ---: | ---: | ---: |
| velocity RMS | `<=0.05 m/s` | `0.0382350614` | `0.0764701228` — FAIL |
| maximum speed | `<=0.10 m/s` | below limit | `0.08175` |
| position RMSE | `<=2.5 mm` | below limit | `0.477938 mm` |
| maximum displacement | `<=5 mm` | below limit | `0.510938 mm` |
| positive energy excess | `<=1%` | `0` | `0` |
| normalized momentum residual | `<=1%` | below limit | `1.2112e-17` |
| connected components | `1` | `1` | `1` |

The receipt publishes `accepted_steps=1`, `failure_step=2` and
`trial_committed=[true,false]`. The accepted state root is
`2f72e520fa3531a9fe38d08cbc81f15fc37b9c831ad43c8f4b7941b3231ba546`.
The separate failing-trial state/work/result roots are respectively:

- `11bc7f2491a9e75c3dc6d97b95ab04d61b62837d322f5ebe1f42d393ec660289`;
- `154b27043e8b22a4b9e33956c64e3db2338dc298544343b7b0c1a9a4b09380f5`;
- `5cf5a73a9efde72bd059d8c792f8951ea697356def2d3abf1e063744c4368c6e`.

The complete Phase-B work root is
`eb016f522cd42b458100ea9851dc04f1b45f53ee89038128f5691f220fec559a`;
the trajectory root is
`b20980d9d3abc2e1ac11c64e347fafc768e8f7443edffa227925fd45646f38f0`.

## Controls and work closure

All 11 frozen controls pass. Their exact logical root-derivation counts are
`[7,1,4,10,3,16,2,9,1,2,16]`, totaling `71`. Admission controls `5a/5b` have
their own sealed subreceipts. Every projection round publishes raw work;
analytic `Jv`, topology traversal, graph candidates, accepted pairs, six-plane
contact tests and independent-density work are counted and root-bound.

The contact corpus contains 27 manufactured cases and performs exactly
`27*6=162` candidate plane tests. Its independent componentwise oracle covers
all six single faces, outward and inward starts on each plane, corner ties,
tangential retention, impulse signs, non-expansiveness, outside and nonfinite
rejection.

The aggregate controls work/result roots are:

- work: `aee1a2c63241abb6ddb69c1adb07389233787eee14b02e7c68014becc202f4a1`;
- result: `befd1b4a3ce39200139f9be4f3acd19ad7e6557f1bd3237f9d1dbd32d163e6af`.

## Verification

- two clean Release configure/build/runs are byte-identical;
- an ASan+UBSan build/run exits zero with empty stderr;
- the independent reviewer rebuilt and reran NCGP12 and NCGP13 from a fresh
  detached worktree;
- an independently written serializer recomputed 72
  profile/work/receipt/round/step/trajectory/control/final nodes for both
  reports; every root matched;
- candidate/permuted routes are exact;
- `git diff --check` passes and the reviewed worktree remained clean.

The independent re-review returned `GO` and found no remaining load-bearing
F1--F7 apparatus defect. The single repair/re-review allowance is exhausted.

## Interpretation and next action

The pressure constraint and frictionless analytical contact are both viable
ingredients: the difficult 512-particle one-step phase passes with large
numerical margin. The Phase-B failure does not yet refute that composition for
confined water.

Phase B places a `4x4x8` block at approximately `x,y=0.275..0.425 m` inside a
`3.0x2.5 m` basin. Its nearest lateral wall support is outside the
`0.15 m` horizon. On the second trial, 112 particles are therefore in nearly
ballistic free fall while only the bottom 16 are clamped. The analytic RMS
`sqrt(112/128) * 2 * 9.81 / 240` is exactly the observed
`0.0764701228 m/s` to the published precision.

The smallest successor is a newly frozen CPU long-double geometry
discriminator, not immediate surface tuning or CUDA timing:

1. retain this open 128-particle state as a negative control;
2. add an open 512-particle size control;
3. put the same 128-particle state in a tight analytical tank whose side and
   bottom walls are inside the support horizon while the top remains free;
4. distinguish the frozen QP work ceiling from physics before interpreting a
   trajectory result.

Surface and free-surface correction remain separate later hypotheses. No GPU
or performance stage resumes until a properly confined pressure/contact
trajectory passes its independently reviewed physical gates. CPU DFSPH remains
the product fallback; SPEC-38 and ADR-076 remain Proposed.
