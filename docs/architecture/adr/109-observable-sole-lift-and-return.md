# ADR-109: Observable sole lift and return

| Field | Value |
| --- | --- |
| ID | ADR-109 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-108](108-observable-periodic-walking-credit.md) |
| Supersedes | ADR-108's one-run restriction for one diagnosed V7 successor; no frozen V6 semantics or quality/safety gates |
| Superseded by | none |

## Evidence and decision

V6 completes 4.096 million transitions but never lifts a complete sole 5 mm
in its final evaluation. Its load credit cannot distinguish unloaded grounded
feet from lifted feet at equal load/speed inputs. The
[native geometric discriminator](../../development/r8b-lift-return-discriminator-2026-09-05.md)
distinguishes real release from heel rocking; wrong side and persistent lift
lose its fixed counterfactuals. These controls do not prove learnability or safe
return. Admit one canonical V7 lesson and longer fresh run to test that claim.

## Environment V7

`nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v7` preserves V6's
body, actions, fourfold residual, controller, safety, reset, command schedule,
first 86 observations and all eleven reward components. No corpus, joint-angle
teacher, Isaac implementation or runtime policy is introduced.

Append left/right complete-box minimum world Y in integer micrometres, indices
86/87; Python normalization is 100,000 um. Derive box offset and half extents
from the frozen BodySchema, require exactly one identity-local-rotation box
per foot, and measure final native link poses. Project the offset and support
radius using the existing Q30 rotation-row calculation (ties-even), then floor
the final projected height to um. Do not use a contact bit or body origin as
clearance. Two cached immutable sole descriptors add constant space; each
measurement uses fixed-size arithmetic and a bounded native-link lookup.

Append two Q16 error components, `reward.periodic-left-sole-height-error.v1`
and `reward.periodic-right-sole-height-error.v1`, each range [0,65536] and
coefficient -65536. Separate components preserve the existing protocol's
per-component [0,1] bounds without widening public contracts. Their sum is
the discriminator's exact cost, not a changed objective or averaged variant.

Use V6's pre-action phase `max(tick-120,0)%72`. Left target is zero outside
phase 6..30, rises to 60,000 um at 18 and returns to zero at 30; right is
offset 36 ticks. Each half is 12 ticks with quintic smoothstep
`60000*(10u^3-15u^4+6u^5)`, rational integer evaluation floored to um. Each
component is `floor(min(abs(actual-target),60000)*65536/60000)`. Compare
post-step geometry to the phase visible to its action. At exact zero command
both new costs are zero; the old stopping terms remain. This deliberately
changes the objective and makes no potential-based invariance claim.

## Run and checks

`lab/profiles/canonical-rsl-rl-walking.v3.json`: one fresh run, seed 44,
128 slots/eight shards, 32 steps/update, 10,000 updates = 40,960,000 transitions,
14,400 s wall ceiling, checkpoints every 100 updates. PPO/network settings
remain V6 except the two inputs. No weight initialization, resume or sweep.
The larger budget follows the prior short failure and substantially larger
sample budgets in the linked primary research; it is not a convergence promise.
V6's observed ~924 s per 1,000 updates motivates the bounded wall allowance,
not an exclusive-host throughput guarantee or an equal-budget causal ablation.

Predeclare report-only evaluations after updates 999 and 3999, with the same
five-seed matrix. They cannot select weights, modify the lesson, stop for a
better score or advance a stage. Restore training mode afterward; nested
milestone artifacts join the final manifest's hash closure. Final acceptance
uses only `model_9999.pt`: all five 1,200-tick episodes safe, >=3 m travel,
velocity MAE <=0.2 m/s, 180 exact zero-command ticks and stop speed MAE <=0.1
m/s, >=8 continuous single-support ticks per side and >=2 support switches.

Before optimization require pure phase/geometry controls, native V6/V7
identical-action physics/safety through terminal, old descriptor byte identity,
native geometric agreement and Python cost oracle on actual action tapes,
88-channel reset/terminal/bootstrap adapter tests and exact multi-slot native
control. Run broad Linux host-check for this cross-boundary implementation and
the usual clean-commit generation freeze. Finite-value/wall/protocol failures
close the run; a poor diagnostic score is not permission to retry unchanged.
Rollback retires V7; all previous descriptors remain available. Learned gait,
held-out robustness, Isaac correspondence and runtime/export admission remain
separate unpassed claims.
