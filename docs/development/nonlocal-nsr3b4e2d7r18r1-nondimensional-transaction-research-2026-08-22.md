# NSR3-B4E2D7R18R1 nondimensional AL transaction research

Date: `2026-08-22`

Status: `RESEARCH_COMPLETE / CONTRACT_FREEZE_NEXT`

## Question

Can the exact `kappa*dt^2` scale candidate be expressed without the isolated
D7R18 dimensional-HVP roundoff, while preserving the legacy AL problem and
defining convergence state that is invariant to the algorithmic penalty
scale?

## Normalized formulation

The dimensional PHR objective is

```text
E = M/(2 dt^2) ||y-y_hat||^2
  + sum((max(0,lambda+kappa*c)^2-lambda^2)/(2*kappa)).
```

Define

```text
u     = lambda/kappa
theta = kappa*dt^2/M.
```

Multiplying the complete objective by the positive constant `dt^2/M` does
not change its minimizer, trust ratio or Newton step. It produces

```text
Ebar = 1/2 ||y-y_hat||^2
     + theta/2 sum(max(0,u+c)^2-u^2),

gbar = y-y_hat + theta sum(max(0,u+c) J),

Hbar p = p + theta sum((J p) J + max(0,u+c) H_c p),

u_next = max(0,u+c).
```

For the reference and aligned profiles,

```text
theta = 0.17031250000000001
bits  = 0x3fc5cccccccccccd
```

exactly. The normalized solver can therefore consume identical `{theta,u}`
instead of building terms `6084x` larger and normalizing them after their
floating-point sums.

## Why this is more than an HVP patch

D7R18 isolates the strict miss to dimensional HVP accumulation, but its
scaled candidate would also make raw multiplier-state gates inconsistent:

- `lambda` scales with the objective and therefore with `kappa` for an
  otherwise identical normalized state;
- the existing `abs(delta lambda)` and derived pressure-change diagnostics
  would scale by `6084`;
- `delta lambda/kappa`, `lambda*c/kappa`, normalized stationarity and the
  position/impulse consequences remain representation-invariant.

The correct candidate is therefore a complete private normalized transaction,
not a special HVP used beside dimensional energy, reduction and dual state.

## Admission mapping

At the reference profile the old gates map exactly to:

| Gate | Normalized form | binary64 limit |
|---|---|---:|
| primal | `max(max(c,0))` | `1e-8` |
| stationarity | `max(norm(gbar))/dx` | `1e-10` |
| dual change | `max(abs(delta u))` | `1e-8/1226.25 = 8.154943934760449e-12` (`0x3da1eed347666340`) |
| complementarity | `max(abs(u*c))` | `1e-9/1226.25 = 8.154943934760449e-13` (`0x3d6cb1520bd70533`) |
| position state | `rms(delta y)/dx` | `1e-8` |

The old `equivalent_pressure_change` remains a reconstructed diagnostic. It
cannot be a second unchanged gate across a penalty rescale because it is
algebraically the same raw `delta lambda` gate in different units. A nominal
stage must retain kinematic pressure impulse and full support/fluid ledger
checks; those constrain the published consequence rather than the internal
dual representation.

## Numerical certificate

D7R18's `64 epsilon` HVP comparison cannot be relaxed. R1 instead implements
the normalized formula directly and requires reference/substep outputs to be
byte-exact because both receive the same `theta`, `u`, geometry, prediction
and direction.

To prove equivalence with the existing dimensional formula, reconstruct the
dimensional HVP from the normalized result and compare componentwise. The
certificate is an absolute forward bound:

```text
gamma_n = n*epsilon/(1-n*epsilon)
n       = 96*maximum_directed_degree + 256
bound   = gamma_n * sum(abs(all reconstructed component terms)).
```

The deliberately conservative operation count covers both neighbor passes,
radial Hessian algebra, inertia, coefficient products and accumulation. An
absolute sum is required because a relative bound is invalid under component
cancellation. The report publishes degree, `n`, `gamma_n`, maximum observed
absolute error, maximum bound and margin for reference and aligned profiles.

## Smallest discriminator

D7R18R1 must:

1. add a separate normalized sparse workspace/energy/gradient/HVP/divided and
   long-double/binary128 oracle over the D7R18 tiny active fixture;
2. use dimensionless `u` as the only dual state and exact `theta` as the only
   penalty/time coefficient inside those formulas;
3. prove dense/sparse exactness and reference/substep byte equality;
4. certify dimensional reconstruction at both scales using the frozen
   componentwise absolute bound;
5. prove the reference legacy admission mapping and cross-scale normalized
   admission equality;
6. reject non-finite/invalid `dt`, `kappa`, `theta` or `u` before work and
   make a one-ULP `theta` mutation visible;
7. retain D7R18 bytes and directly regress D7R17 from each clean build;
8. execute no nonlinear solve, outer transaction, nominal substep or timing.

## Rejected alternatives

- Raise the D7R18 HVP tolerance: it changes a frozen gate after observation.
- Multiply the completed dimensional HVP by `dt^2/M`: the roundoff has already
  occurred and D7R18 shows that post-scaling does not prove invariance.
- Normalize only HVP: energy/reduction/trust ratio and dual state would use a
  different arithmetic identity.
- Keep absolute `lambda` admission: it is representation-dependent under the
  proposed objective scaling and would reject equivalent states.
- Treat reconstructed pressure as material pressure: this AL multiplier is a
  discrete constraint force state; kinematic impulse and ledger are the
  invariant physical consequences available at this stage.

## Decision

Freeze D7R18R1 as the complete tiny normalized-transaction formulation gate.
A pass may authorize a separately frozen bounded private normalized solve on
the tiny D7R13 fixture before any nominal retry. It does not authorize D7R19
directly: confirmation/holdout and precision acceptance must first reproduce
under the new representation.
