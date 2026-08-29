# NSR3-B4E2D7R19R52 fixed-master sweep-depth research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / THREE-CHECKPOINT SWEEP DEPTH SELECTED`.

## Question

Does R51 remain uncertified because eight cyclic Hildreth sweeps under-solve the
exact fixed 494-row master, or because a different active-QP method/nonlinear
step is required?

## Evidence

R51 closes external row migration: every one of its 481 terminal
directed-positive rows is already inside the persistent master. Yet the
eight-sweep linear predicted maximum is still positive at
`2.4062191742798642e-14`. Only two new bases were required, and the accepted
full step made substantial nonlinear progress. Capacity and globalization are
not the observed boundary.

Hildreth's original procedure is a cyclic dual-coordinate QP method. Modern
Dykstra analysis makes the dual coordinate-gradient equivalence explicit and
establishes eventual linear convergence for polyhedral sets, but not finite
exact termination. Primary sources:
[Hildreth](https://doi.org/10.1002/nav.3800040113) and
[Wang--Pong](https://doi.org/10.1137/23M1545781).

Perkins proposes a combined Dykstra--conjugate-gradient method for polyhedral
projection and is the principled next accelerator if plain coordinate depth is
the limiter: [Perkins](https://doi.org/10.1137/S0036142900367557). A
Goldfarb--Idnani active-set reference is also possible, but adds dense
factorization/rank policy before the cheaper causal discriminator has been run:
[Goldfarb--Idnani](https://doi.org/10.1007/BF02591962).

## Selected discriminator

Replay the exact R51 parent, but regenerate candidate corrections from the
unchanged R50 terminal anchor and exact R51 494-row union/cache:

1. solve the identical master independently from zero duals at exactly 16, 32
   and 64 sweeps in stable row order;
2. preserve the same Gram, upper vector, ball-box projection and source witness;
3. project each correction independently and run a fresh unchanged all-row
   directed audit;
4. evaluate all three checkpoints. Select the smallest-sweep certified
   candidate if one exists; otherwise select the smallest-sweep candidate that
   strictly improves `psi`, `h` and maximum directed upper versus R51;
5. if none dominates, retain R51. Do not fit a residual tolerance or select a
   checkpoint merely because its master-predicted maximum is smaller.

The independent solves intentionally spend `16+32+64=112` dense coordinate
sweeps. This is a reference discriminator, not the future implementation. They
reuse all 494 bases and Gram columns, need no new row operator work and require
exactly three candidate JVP audits. Parent replay work stays historical.

## Why multiplier transfer is not selected yet

Within an unchanged master and upper vector, continuing the eight-sweep dual
state is arithmetically the same sequence as running more cumulative sweeps. It
does not need a separate experiment.

After R51's nonlinear correction is applied, however, the new projection
problem is anchored at a new primal origin. Reusing old multipliers while also
adding their full `-A^T lambda` correction can double-count the already applied
step. A valid cross-outer warm start needs an absolute-anchor or correction-
delta derivation. R52 does not improvise that policy.

## Classification

- An all-row certificate selects the private next-TRQP compatibility candidate.
- Strict R51 dominance selects a persistent-master sweep-depth candidate.
- Monotone master residual improvement without nonlinear dominance selects a
  CG-polish research requirement.
- Nonmonotone/invalid Gram, projection, audit or work has its own exact route.

No master residual replaces the all-row certificate. No witness is applied and
no restoration transaction, timing, runtime or production authority is added.

## Recommendation

Freeze and implement the three-checkpoint rollback-only sweep-depth reference.
If 64 sweeps do not materially dominate R51, stop extending cyclic sweeps and
derive a Perkins-style active-face conjugate-gradient polish next. Do not widen
the master, run a second nonlinear outer or transfer multipliers first.
