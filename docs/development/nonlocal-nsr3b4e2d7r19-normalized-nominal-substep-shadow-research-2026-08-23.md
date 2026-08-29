# NSR3-B4E2D7R19 normalized nominal-substep shadow research

Date: `2026-08-23`

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / IMPLEMENTATION_NEXT`

## Question

Does the confirmed normalized, pairwise-precancelled transaction with explicit
dimensionless Krylov forcing resolve D7R17's first aligned nominal Dam substep
without violating the structural, boundary or impulse gates?

D7R19 reruns the same frame-zero-to-first-substep shadow. It does not advance
from D7R17, execute a second substep or publish state.

## Solver mapping

Use the aligned profile already proved by D7R18--R4R2:

```text
dt    = TIME_STEP / 78
kappa = 1226.25 * 78^2 = 7,460,505
theta = kappa * dt^2 / M
      = 0x3fc5cccccccccccd
u     = lambda / kappa
u_next = max(0, u + c)
```

The inner objective, gradient, HVP, pairwise-precancelled actual reduction,
long-double/binary128 certificates and dimensionless forcing are exactly the
R4R2 formulation. The static support index remains identity-bound and candidate
all-pair work remains forbidden.

## Frozen input and predictor

Reuse D7R17's exact decoded Dam frame zero:

```text
frame root      0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7
particles       6000
dt bits         0x3f0c01c01c01c01c
lower-y clamps  400
free samples    5600
```

Compute `v*=v+dt*g`, box-clamp the predictor displacement and keep predictor
contact impulse separate from the nonlinear pressure correction. The solver
starts from the clamped predictor and zero normalized dual.

## Structural watchdog

Retain D7R17's first-substep safety envelope:

```text
outer updates             <= 16, including holdout
inner trials per update   <= 16
HVP per trust solve       <= 32
HVP total                 <= 512
workspace builds          <= 288
precision audits          <= 64
maximum live workspaces   <= 2
candidate all-pair calls  = 0
```

Every expensive unit must consume its budget before work. Invalid profile,
dual, binding or budget rejects before workspace, HVP, forcing-policy and
precision work. Exhaustion is a classified private result with exact release
and rollback, not a reason to enlarge a cap.

## Normalized impulse reconstruction

D7R17's dimensional support ledger cannot be copied literally. If

```text
Ebar = (dt^2/M) * Ephysical,
```

then normalized support gradients obey

```text
gbar_support = (dt^2/M) * gphysical_support.
```

The physical fixed-support impulse is therefore

```text
I_support = -dt * sum(gphysical_support)
          = -(M/dt) * sum(gbar_support).
```

The kinematic pressure impulse is independently

```text
I_pressure = (M/dt) * sum(y - yhat).
```

The required ledgers remain:

```text
Delta P_fluid = I_gravity + I_contact + I_pressure
I_pressure + I_support = normalized stationarity residual mapped by M/dt.
```

Both scaled closures retain the `1e-10` limit. Reconstructed physical
`lambda=kappa*u` and pressure are diagnostics only; normalized admission is
authoritative.

## State, boundary and route precedence

Select only the confirmed update after two consecutive admissible outer states
and an admissible warm holdout. The holdout validates but does not replace the
selected state. Require finite state, exact mass, nonnegative `u`, primal
`<=1e-8`, stationarity `<=1e-10`, normalized complementarity within the R4R2
gate and position update `<=1e-8 dx`.

If confirmed, measure density and maximum closed-box penetration. Do not add
projected AL contact. Preserve D7R17's route precedence:

1. `NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED`.
2. `NORMALIZED_NOMINAL_SOLVER_NOT_CONFIRMED`.
3. `NORMALIZED_NOMINAL_BOUNDARY_PENETRATION`.
4. `NORMALIZED_NOMINAL_IMPULSE_LEDGER_MISMATCH`.
5. `NORMALIZED_NOMINAL_SUBSTEP_SHADOW_CONFIRMED`.

## Decision

Freeze D7R19 as one private aligned nominal-substep shadow from frame zero.
It must reproduce R4R2 and D7R17 bytes, execute no second substep, publish
nothing and collect no timing. Any route authorizes only research of the next
named boundary; no runtime or production authority follows.

