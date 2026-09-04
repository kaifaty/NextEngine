# ADR-108: Observable periodic walking credit

| Field | Value |
| --- | --- |
| ID | ADR-108 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-107](107-canonical-cpu-walking-learner.md) |
| Supersedes | ADR-107's one-run optimizer restriction for the one diagnosed V6 successor; no frozen V5 semantics, safety or promotion gate |
| Superseded by | none |

## Evidence and scope

The V5 canonical run learns forward motion without foot release and fails all
five walking episodes. Reduced-noise controls do not produce single support.
The finite manual alternation probe violates actual safety limits; tick 115
is a genuine ankle power/rate incompatibility, not a corrupt observation or
work-accounting bug. See [exact evidence and primary research](../../development/r8b-walking-step-credit-2026-09-05.md).

Admit one reference-free, canonical-only V6 lesson. It replaces binary support
credit with graded periodic unloading and supplies the period to the policy.
It does not prescribe joint angles or imitate the failed manual tape. The
research motivates phase observability and swing-force/stance-speed credit;
this integer load-fraction implementation is a separate hypothesis, not a
replication or a proof of learnability. It intentionally changes the learning
objective, without claiming potential-based policy invariance.

## Frozen environment

`nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v6` retains V5
BodySchema V4, compiled physics, 23 Q1.30 residual actions, fourfold residual,
translation-invariant reference, PD, safety, reset and 1,200-tick command
schedule. Old descriptors remain byte-exact. No Isaac implementation is admitted.

Append two signed Q1.30 quadrature triangle clocks to the existing 84 raw
observations. Period is 72 motor ticks, quadrature offset 18 ticks, phase is
`max(motor_tick - 120, 0) % 72`. Both channels are zero for exact zero command.
The pair distinguishes all cycle phases; reward uses the phase visible to the
action, and next observation uses the next tick/command. Phase derives from
existing replay-owned state; there is no trainer clock or hidden counter.

Replace reward component 9 with `reward.periodic-load-transfer.v1`, coefficient
1.0 Q16. Other ten components and coefficients are unchanged. Desired left
load is linear between `(tick,Q16)` knots `(0,32768), (6,0), (30,0),
`(42,65536), (66,65536), (72,32768)`, periodically. Each foot therefore has
24 swing ticks and each transition has 12 ticks of permitted double support.
Measure per-foot absolute world-Y ground-contact impulses over actual physical
substeps only. Left impulse / total impulse gives actual left load. If total
impulse is zero, moving credit is zero. Otherwise:

`max(0, 1 - 2*abs(actual_left - desired_left)) * (1 - min(1, weighted_stance_speed / 1 m/s))`

Stance speed is the target-load-weighted final-foot world-XZ L1 speed. Fixed
integer divisions define the Q16 implementation. At zero command retain the
old requirement of both sole-contact flags, now with the new coefficient.
The reward hash closes the previous ten terms and exact new formula/constants;
observation and correspondence identities are distinct. This load metric is
not an impact detector, contact authority or proof of foot clearance.

## Bounded run and checks

`lab/profiles/canonical-rsl-rl-walking.v2.json` admits one fresh-weight run:
seed 44, 128 slots / eight shards, 32 steps/update, 1,000 updates (4,096,000
samples), 3,600 s maximum optimizer wall time, save every 100 updates. The
network architecture and PPO settings are unchanged except input width. The
larger declared budget allows a first periodic skill to develop; V5/V6 returns
or unequal-budget outcomes do not establish a causal reward ablation.

Require pure reward/clock controls, native V5/V6 identical-action physics and
safety non-regression, old descriptor equality, focused protocol/adapter tests
including 86-channel resets/timeouts, exact native adapter control and the
normal clean-commit generation closure before optimization. Report broad
host-check for the cross-boundary implementation. Record moving-phase support
counts and credit separately from total return; no early checkpoint selection.

Evaluate the predeclared final `model_999.pt` with the unchanged five-seed
ADR-107 matrix: safe full horizon, >=3 m forward, velocity MAE <=0.2 m/s,
180 exact stopped ticks with speed MAE <=0.1 m/s, >=8 continuous support ticks
on each side and >=2 qualified side switches. Shaped credit or completed
training is not walking. On failure close the artifacts and diagnose before
any successor; do not retry unchanged, expand tolerance or silently resume.
Rollback retires V6; V5 and standing identities remain available. Mirror,
runtime/export, terrain, recovery and wider commands remain outside admission.
