# NSR3-B4E2D7R19R34 first-order reference extension research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

## Decision from R33

R33 rejects both immediate first-order saturation and immediate promotion to
Newton-CG. Its violation norm falls strictly at iterations `1/2/4/8`, reaching
`0.627875x` of the source, while projected stationarity is still absent and
the trust radius is effectively unused.

The next useful evidence is a deeper, fixed first-order work/reference curve.
Without it, a generalized-Hessian candidate could appear better merely by
using more operator work.

## Frozen experiment

Resume the exact R33 iterate and response after iteration eight. First
recompute `A v8` and require the exact frozen R33 response root. Then continue
the unchanged projected normalized-steepest recurrence through iterations 9
to 32, recording checkpoints 16 and 32.

Each additional iteration uses one VJP and one JVP. Prefix validation costs
one JVP; fresh terminal response and gradient checks cost one JVP and one VJP.
The predeclared new-work cap is therefore:

```text
prefix check          1 pair pass
iterations 9..32     48 pair passes
terminal checks       2 pair passes
total maximum        51 pair passes
```

This is a correctness/work-accounting bound, not a timing measurement.

## Convergence evidence

Report separately:

- `phi16 / phi8` and `phi32 / phi16`;
- violation-norm contractions over the same blocks;
- geometric per-step contraction for the 8-step and 16-step blocks;
- step, iterate, active-row and projected-mapping values;
- exact checkpoint and terminal operator roots.

No observed contraction threshold is a hard gate. Correct continuation can
classify as projected stationary, saturated at binary64 resolution, or a
strictly improving 32-step first-order reference. This prevents selecting a
convenient cutoff after the nominal result.

## Method-family boundary

Exact-line steepest descent is deliberately retained because R33 still makes
strict progress. Its checkpoint curve will expose conditioning and create a
fixed baseline. The next curvature method remains generalized-Hessian
TRON/Newton-CG for squared hinge, as motivated by:

- https://www.jmlr.org/papers/volume9/lin08b/lin08b.pdf
- https://jmlr.org/papers/volume13/ho12a/ho12a.pdf

R34 does not implement or claim equivalence to those methods. If R34 remains
nonstationary, a later stage must freeze a matrix-free generalized-Hessian
recurrence and compare it against the R34 residual at equal pair-pass work.

## Controls and authority

Use three independent dense controls: an ill-conditioned diagonal problem
with strict long-horizon contraction, an early-stationary problem and a
trust-boundary stationary problem. Preserve exact R33 parent bytes, source,
workspace, prefix response, every accepted line KKT, direct terminal operator
checks, work ledger and rollback.

R34 cannot apply the iterate, evaluate a nonlinear moved state, update duals,
change penalty/trust policy, execute another outer or claim floor, timing,
runtime or production authority.
