# NSR3-B4E2D7R19R55 contact-constrained Dykstra research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / GROUPED-BOX DYKSTRA SELECTED`.

## Question

Can the R52 density halfspaces and R54 contact box be solved as one
minimum-norm correction problem, so that contact feasibility no longer destroys
the density solution after the solve?

## Problem

With the R50 terminal witness as anchor and correction `p`:

```text
minimize    0.5 ||p||^2
subject to  upper(anchor) + A p <= 0
            lower - anchor <= p <= upper - anchor
```

The unchanged normal ball is audited but not included in the first reference:
R54 measures witness norm `5.04e-7` against radius `0.03125`, so it is
decisively inactive at this boundary.

## Algorithm research

Dykstra's algorithm computes the Euclidean projection onto an intersection of
closed convex sets and applies to translated convex sets
([Dykstra--Boyle](https://doi.org/10.1016/0378-3758(86)90111-4),
[Boyle--Dykstra](https://doi.org/10.1007/978-1-4613-9940-7_3)). When its sets
are halfspaces, it recovers Hildreth's quadratic-programming row action; the
primal-dual/coordinate connection is established explicitly by
[Bregman--Censor--Reich](https://ftp.gwdg.de/pub/misc/EMIS/journals/JCA/vol.6_no.2/5.html)
and revisited for block coordinate descent by
[Tibshirani](https://papers.nips.cc/paper_files/paper/2017/hash/5ef698cd9fe650923ea331c15af3b160-Abstract.html).

This permits a hybrid that preserves the useful R50/R51 structures:

1. retain one scalar Dykstra correction/dual coordinate per density halfspace;
2. process the 494 density sets in stable order using the captured row bases
   and dense Gram columns;
3. retain one vector Dykstra correction for the entire axis-aligned box;
4. after the density sweep, project `p + box_correction` component-wise into
   the correction box and update the box correction;
5. explicitly refresh `upper(anchor)+A p` after every box block because the box
   changes the primal vector outside the density Gram recurrence.

The box remains one separable set. It does not add 1125 coordinate rows to a
dense Gram. This is mathematically different from the rejected
solve-then-clamp sequence because the box correction persists and feeds back
into every following density sweep.

## Selected reference

- initialize `p=0`, density multipliers zero and box correction zero;
- run one deterministic 64-cycle continuation;
- each cycle is 494 stable density halfspace projections followed by one box
  projection and one fresh pair-once residual refresh;
- capture checkpoints 8, 16, 32 and 64;
- at each checkpoint run the unchanged directed certificate on
  `anchor+p`, audit all rows outside the persistent master, exact box
  feasibility and normal-ball norm;
- do not stop on a tolerance.

Within a density sweep, the existing recursion is valid:

```text
lambda_new = max(0, lambda + residual_i / ||a_i||^2)
delta      = lambda_new - lambda
p         -= delta * a_i
residual  -= delta * Gram[:, i]
```

The next cycle never trusts this recurrence across the box block; it starts
from one fresh `A p`.

## Frozen selection

1. first checkpoint with zero directed-positive rows selects existing
   compatibility;
2. otherwise any positive row outside the 494-row master selects master
   expansion before promotion;
3. otherwise any checkpoint outside the normal ball selects ball integration;
4. otherwise the smallest checkpoint simultaneously improving R52-64 `psi`,
   `h` and maximum upper selects a private contact-constrained candidate;
5. otherwise retain the R54/R52 reference.

No result is an infeasibility proof. Compatibility remains certificate-only.

## Alternatives not selected

- **Dense Gram with coordinate faces:** 1125 currently active coordinates
  would expand a 494-row dense model substantially, despite each face being a
  trivial one-component projection.
- **Naive alternating projection:** without the persistent box correction it
  need not return the closest point in the intersection.
- **Freeze outward components:** guarantees a restricted tangent cone but can
  exclude valid inward/cancelling solutions and is not the original box QP.
- **Box-eliminated dual semismooth coordinate solve:** attractive later, but
  requires a new piecewise coordinate root solver before this simpler exact
  reference establishes the attainable boundary.
- **Include the ball immediately:** rejected for this discriminator because it
  is inactive by four orders of magnitude; it remains a hard audit.

## Recommendation

Freeze and implement the grouped-box Dykstra reference. Admit 64 fresh
pair-once residual refreshes and four directed checkpoint certificates, but no
new row basis, Gram column or post-terminal projection. Use the result to decide
whether contact-aware convergence, master expansion or normal-ball integration
is next.
