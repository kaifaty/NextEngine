# NSR3-B4E2D7R19R38 direction-memory research -- 2026-08-24

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Problem

R37 establishes a stable linear regime: objective factors approach `0.92` per
step and projected-mapping factors approach `0.959`, while the active set
continues to change. Extending the same steepest direction tests no new
hypothesis. The next bounded question is whether one-step direction memory can
improve exact-line progress without weakening projection, descent or rollback.

## Primary-source basis

The Hager--Zhang
[survey](https://www.math.lsu.edu/~hozhang/papers/cgsurvey.pdf) defines the
classical HS, FR, PRP, DY and HZ parameters and emphasizes two independent
requirements: a descent direction and a suitable line search. It records that
PRP/HS have an implicit restart tendency as gradient differences vanish,
PRP+ truncates negative beta to zero, and the parameter-free
`max(0,min(beta_HS,beta_DY))` hybrid combines the practical HS direction with
the DY convergence bound. The Hager--Zhang
[primary method](https://doi.org/10.1137/030601880) provides a direction with
guaranteed sufficient descent in the unconstrained formulation and reduces to
a nonlinear HS form under exact line search. Dai and Yuan's
[primary method](https://doi.org/10.1137/S1052623497318992) provides the DY
denominator and global-convergence result under Wolfe conditions.

Those theorems do not directly authorize our projected chord: the raw NCG
direction is normalized, added to the current point, projected onto the global
L2 trust ball and only then used for exact piecewise hinge minimization. R38
must therefore test raw descent, projected-chord descent and line KKT
independently. Theory chooses the candidates and safety conditions, not the
nominal winner.

## Selected replay

Capture exact R37 private states after additional steps `6`, `12` and `24`,
together with each state's current gradient and the immediately preceding
reference gradient/direction. Rebuild one exact workspace. At each state:

1. recompute a fresh response JVP and require maintained/direct defect
   `<=1e-12`;
2. form four one-step lanes from the same state:
   - `STEEPEST`: `s=-g`;
   - `PRP_PLUS`: `beta=max(0, g^T(g-g_prev)/||g_prev||^2)`;
   - `DY_HS_PLUS`: `beta=max(0,min(beta_HS,beta_DY))`;
   - `HAGER_ZHANG`: the parameter-free HZ/CG_DESCENT raw beta before its
     problem-dependent lower truncation;
3. for a finite valid beta set `s=-g+beta*d_prev`; otherwise restart to
   steepest;
4. normalize `s`, project `v+s/||s||` onto the frozen ball and form the
   normalized feasible chord;
5. restart to steepest if raw or projected descent is nonnegative;
6. evaluate one fresh JVP and the unchanged exact piecewise line minimum;
7. record beta, restart reason, raw/projected descent, line root, terminal
   objective/violation/active set and direction root without updating state.

Each state spends one source JVP plus four equal one-JVP lanes. The fixed work
is `3*(1+4)=15` pair passes, zero VJP/HVP and zero accepted update.

## Frozen selection rule

Precedence is theoretical and fixed before observation:

```text
HAGER_ZHANG
DY_HS_PLUS
PRP_PLUS
STEEPEST
```

A memory lane is selectable only if, at all three states, its beta is valid,
no restart occurs, raw and projected directions are strict descent, exact-line
KKT passes, and final objective is strictly below the corresponding steepest
lane. Otherwise proceed to the next formula. If none qualifies, retain
steepest. This all-state rule prevents choosing a formula from one favorable
active-set snapshot.

The replay does not execute a memory recurrence and supplies no performance
claim. A winning one-step formula would authorize a separately frozen bounded
trajectory with the same restart rules.

## Rejected now

- scalar spectral scaling: exact line already optimizes along the unchanged
  steepest direction;
- an HZ truncation constant or Powell restart threshold: neither has a frozen
  problem-specific scale here;
- unguarded fixed-active-set Newton/CG: R37 active roots still move;
- selecting a lane by average or best checkpoint improvement;
- applying a candidate step, evaluating a nonlinear moved state or timing.

Frozen contract:
[R38 direction-memory replay](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r38-direction-memory-contract.md).
