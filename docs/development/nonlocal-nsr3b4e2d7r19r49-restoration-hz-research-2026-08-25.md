# NSR3-B4E2D7R19R49 restoration Hager--Zhang research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / CONTRACT FREEZE RECOMMENDED / ROLLBACK ONLY`.

## Question

Can the unchanged R48 next-TRQP compatibility problem reach an independently
certified primal witness with a short active-set-aware recurrence, rather than
spending more iterations in the already observed slow PDAL regime?

## Evidence carried forward

R48 establishes four facts:

1. the fresh moved-state linear problem, stable support superset and
   contact/trust geometry are exact and reproducible;
2. PDAL lowers `h` by `7.18997x`, so the problem responds to matrix-free first-
   order work;
3. its terminal witness consumes only `1.18594e-5` of the allowed normal
   radius, so neither the ball nor the contact box is the observed limiter;
4. after 128 main iterations and 322 JVP/VJP passes, the directed primal upper
   bound remains positive. Raising the same cap would extend a known mechanism,
   not test a new one.

R38--R39 provide a local algorithmic prior on the related all-inequality hinge
model. The guarded Hager--Zhang recurrence survives eight active-set-changing
steps without restart and, at equal lane work, reaches terminal objective,
violation and projected mapping ratios `3.87e-9`, `6.22e-5` and `1.56e-4`
relative to steepest descent. That is not direct evidence for the R48 moved
operator or its contact box, but it is strong enough to justify a bounded
transfer experiment.

Hager and Zhang derive the raw nonlinear-CG coefficient and prove a descent
property for their unconstrained continuously differentiable method. Their
result does not directly cover a direction subsequently projected onto an
intersection of a box and a ball. R49 must therefore treat the formula only as
a direction-memory proposal and independently guard both raw and projected-
chord descent. See the [primary paper](https://doi.org/10.1137/030601880).

## Selected experiment

Keep the exact R48 problem

```text
minimize  phi(d) = 0.5 ||max(c + A d, 0)||^2
subject to d in C = contact-box intersect normal-ball(0.03125)
```

and replace only its primal candidate generator:

1. start at `d0=0` with `g0=A^T max(c,0)`;
2. propose steepest on the first step and the inherited raw Hager--Zhang
   direction thereafter;
3. normalize the raw direction, form an exterior target and project that target
   with the exact R48 Euclidean `box ∩ ball` projector;
4. use the chord from the current feasible point to the projected target;
5. restart to projected steepest if beta is invalid/nonfinite, raw descent is
   not strict, the chord is empty, or projected-chord descent is not strict;
6. minimize the all-row squared hinge exactly on `alpha in [0,1]` along that
   chord using the R32 binary128 event order and long-double folds;
7. after every accepted step, run a fresh directed JVP and the unchanged R48
   rowwise primal enclosure. Only this enclosure can certify compatibility.

Because `C` is convex, the full chord between its two feasible endpoints stays
inside `C`; the scalar line search cannot escape the exact contact/trust domain.
This is a direct convexity argument, not a claimed extension of the original
Hager--Zhang convergence theorem.

Freeze a maximum of 16 accepted steps and checkpoints `1/2/4/8/16`. Sixteen is
the first outcome-independent doubling after the already validated eight-step
R39 recurrence. At the cap, the new generator owns at most 16 line JVPs, 16
fresh directed-certificate JVPs and 16 VJPs, or 48 pair passes. A compatibility
certificate may stop early; no fitted residual tolerance or iteration extension
is allowed.

## Classification

The independent R48 directed enclosure owns
`RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE` if every row upper bound is
nonpositive. Otherwise:

- exact projected stationarity with a positive certificate upper bound is a
  distinct unresolved stationary result, not an infeasibility proof;
- strict terminal improvement over R48 in `psi`, `h` and maximum directed row
  upper, within the 48-pass cap and with at least two consecutive non-restarted
  memory steps, selects only a guarded-HZ accelerator candidate;
- failure of that comparison retains R48 PDAL as the reference and stops this
  transfer experiment.

The exact R48 dual result remains authoritative but unchanged and negative. R49
does not manufacture a dual variable from a primal nonlinear-CG recurrence and
cannot claim radius infeasibility.

## Alternatives not selected

- **More PDAL iterations:** rejected because it does not isolate a new
  mechanism and would choose work after observing the endpoint.
- **Diagonal-preconditioned PDAL:** remains a valid later experiment, but it
  changes the prox metric and needs a separately frozen projection/certificate
  correspondence. The existing HZ path has stronger project-local evidence.
- **Semismooth Newton or active-set KKT:** potentially stronger if HZ stalls,
  but it adds generalized-Jacobian and linear-solve policy before testing the
  already available matrix-free accelerator.
- **Generic SCS/HSDE:** useful eventually for infeasibility authority, but too
  broad for this primal-only transfer and does not reuse the exact hinge line.
- **L-BFGS:** postponed because curvature-pair damping and projected-memory
  ownership add more policy than the one-vector HZ recurrence.

## Recommendation

Freeze and implement R49 as a report-only primal-accelerator discriminator.
Preserve R48 bytes, geometry, certificates and rollback; do not apply a witness,
exit restoration, update a filter/trust radius, run a following outer or admit
timing/runtime/production authority.
