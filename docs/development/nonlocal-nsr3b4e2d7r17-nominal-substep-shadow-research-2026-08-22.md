# NSR3-B4E2D7R17 nominal substep shadow research

Date: `2026-08-22`

Status: `FROZEN / IMPLEMENTATION_NEXT / SHARED_HOST_PERFORMANCE_STOP`

## Question

What must be bounded and accounted before the first real aligned nominal Dam
AL substep can run without becoming another long, ambiguous experiment?

## Finding 1: sparse work is necessary but not a watchdog

D7R16 removes candidate all-pair traversal, but the sparse Steihaug solve still
uses a size-derived upper bound of `3N` HVPs. At nominal `N=6000`, one trust
step could therefore request up to `18,000` HVPs. Each nominal HVP traverses a
neighborhood near the observed `342,502` pairs. The existing `64` outer by
`64` inner limits are finite but are not an acceptable first-substep work
contract.

D7R17 must enforce deterministic internal limits before each expensive unit:

```text
outer updates            <= 16, including holdout
inner trials per update  <= 16
HVPs per trust step      <= 32
HVPs total               <= 512
workspace builds         <= 288
accepted precision audits <= 64
```

Exhaustion must return a named private failure with every workspace released.
No partially solved state may become the selected shadow state.

These are safety/work bounds, not tuned production limits and not timing
evidence.

## Finding 2: predictor contact and AL pressure are different impulses

The aligned free-flight predictor gives

```text
v* = v0 + dt g
dx_free = dt v*
dx_box = clamp_box(dx_free)
```

Exactly `400` bottom samples are clamped. Their kinematic contact impulse is

```text
I_contact = sum m (dx_box / dt - v*)
```

and must remain separate from the AL correction

```text
I_pressure = sum m (y - x_predictor) / dt.
```

For the selected confirmed outer update, the stored support gradient uses the
same multiplier with which its inner position was solved. The impulse on fixed
support is therefore

```text
I_support = -dt sum_boundary grad(E_AL).
```

At converged stationarity, `I_pressure + I_support` closes to the summed inner
residual. The fluid momentum identity is independently

```text
Delta P_fluid = I_gravity + I_contact + I_pressure.
```

Both closures must be reported. Contact may not be folded into pressure, and
the warm holdout may validate confirmation but may not replace the selected
confirmed state.

## Finding 3: boundary feasibility must be observed, not assumed

The predictor is box-clamped, but AL trial positions are not projected through
the Box-KKT path. D7R17 must measure the final maximum lower/upper penetration
against all three closed-box axes. A pressure/residual PASS cannot waive a
penetration failure.

Do not add projected AL contact yet. If the first bounded shadow penetrates,
that exact result selects the following projected-contact research rather than
silently changing solver mechanics in D7R17.

## Frozen experiment

1. Reproduce complete D7R16 stdout bytes.
2. Decode the exact nominal frame-zero state and rebuild the same identity-
   bound static support once.
3. Form the aligned clamped predictor and preserve its contact ledger.
4. Run exactly one private AL substep with the internal structural budgets.
5. Select only a doubly-admissible confirmed state with its successful warm
   holdout.
6. Report nonlinear residuals, work, density range, maximum penetration,
   private state root and both impulse closures.
7. Leave decoded/public state byte-exact and run no second substep, macro,
   trajectory or timing lane.

Use two clean Release builds with one process from each build only. This keeps
the result reproducible without repeating the old four-process pattern for a
potentially expensive nominal experiment.

## Decision routes

1. `NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED`.
2. `NOMINAL_SOLVER_NOT_CONFIRMED`.
3. `NOMINAL_BOUNDARY_PENETRATION`.
4. `NOMINAL_IMPULSE_LEDGER_MISMATCH`.
5. `NOMINAL_SUBSTEP_SHADOW_CONFIRMED`.

A confirmed result authorizes research of the next single-substep performance
bottleneck only. It does not authorize a macro frame, public commit, runtime
integration, GPU work or a production claim.

