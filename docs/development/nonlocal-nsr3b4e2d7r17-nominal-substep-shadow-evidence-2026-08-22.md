# NSR3-B4E2D7R17 nominal substep shadow evidence

Date: `2026-08-22`

Status: `PASS_CLASSIFICATION / NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED / SOLVER_NOT_CONFIRMED`

Implementation commit: `ae81c21d`.

## Result

D7R17 executes exactly one aligned private Dam substep and deterministically
selects:

```text
NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED
```

This is a successful fail-closed classification, not a nominal solver pass.
No state is selected or published, no holdout is attempted and rollback is
exact. The command executes no second substep, macro, trajectory or timing
lane.

## Solver boundary

The frozen outer-update budget is exhausted after update `15`:

| Fact | Outer 0 | Outer 15 |
|---|---:|---:|
| primal violation | `2.3320475972532506e-7` | `2.2852256487126965e-7` |
| stationarity | `2.200975383919953e-12` | `1.4931556059733052e-12` |
| complementarity | `6.668894402416776e-11` | `1.0350332475771004e-9` |
| absolute dual change | `2.8596733661317986e-4` | `2.802257951733944e-4` |
| equivalent pressure change, Pa | `2.287738692905439` | `2.2418063613871553` |
| position update, `dx` | `3.9641433229216844e-10` | `3.7690688071557756e-10` |

Every outer update takes one accepted trial, zero rejected trials and two
HVPs. Final stationarity is already about `67x` below its `1e-10` gate, while
the primal violation remains about `22.85x` above its `1e-8` gate. Therefore
the bounded observation identifies slow outer multiplier evolution, not inner
trust-region convergence, as the active mechanism.

The run uses:

| Work | Used | Frozen cap |
|---|---:|---:|
| outer updates | `16` | `16` |
| accepted / rejected trials | `16 / 0` | `16` trials per update |
| HVP calls | `32` | `512` total, `32` per trust step |
| workspace builds / releases | `48 / 48` | `288` builds |
| accepted precision audits | `16` long double, `0` binary128 | `64` |
| candidate all-pair calls | `0` | `0` |
| maximum live workspaces | `2` | `2` |

The invalid-budget control rejects before nonlinear work.

## Observed-only physics

Because the solver never confirms, these values describe only the final
private observed state and cannot become physical output:

| Fact | Value |
|---|---:|
| minimum density | `536.891174621109 kg/m^3` |
| maximum density | `1000.0002285225648 kg/m^3` |
| maximum closed-box penetration | `7.11347463572265e-10 m` |
| gravity impulse, `y` | `-0.3930288461538462 N*s` |
| predictor contact impulse, `y` | `0.026201923076923076 N*s` |

The penetration exceeds the frozen `1e-12 m` boundary gate, but route
precedence correctly stops first at the structural watchdog. Pressure/support
and full momentum ledgers remain unavailable because no state is selected.

## Reproducibility

Raw evidence:
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r17-final.egQdnB`.

Two independent clean GCC 15.2 Release builds produce identical
`5,309,568`-byte executables:

```text
SHA-256  e816b5b5646d6fb104c87923919b8372aa7a6925e39479f864d41bb285ed389d
Build ID 4fa3f365fa967df37dc9a8d681ac22502a2ab968
```

Each executable runs in a fresh process, exits zero and emits empty stderr.
Both produce the same `7,549`-byte stdout:

```text
stdout SHA-256  a2a8de930d41d8e49c255a3fcf987a8794b4da067ddadf658732946fca0ffced
semantic result 1f368c86ec760a232e0314875d7f61010ecdf37f3a2b80023274855c8c71913b
```

Direct regressions preserve:

```text
D7R16 stdout 4c537f706dee3941808f0c44c1b2db30dd79254bcbbd42c9ac0dc92aee3f8cd5
D7R13 stdout 514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de
```

## Decision

Do not increase the outer cap or tune the pressure-state gates. D7R17 proves
that the inner sparse Newton--Krylov transaction is not the observed
bottleneck: every outer state is solved to roughly `1.5e-12` stationarity in
two HVPs, while the multiplier removes primal violation only slowly.

The nominal substep uses `dt_ref / 78`, so its inertia curvature is `78^2 =
6084` times stronger than at the reference step. The current AL penalty stays
at `kappa = 1226.25`. Research/freeze an explicit-`kappa` and
dimensionless-scaling prerequisite next, preserving all legacy bytes and
running no nominal solve. Only after that prerequisite may a bounded scaled
nominal discriminator run.

No solver confirmation, boundary/contact redesign, macro, trajectory, timing,
parallel/GPU, public-state, runtime or production authority is granted.
