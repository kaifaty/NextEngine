# NSR3-B4E2D7R19R57 certificate-aware terminal refinement research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / CONTRACT NEXT / REPORT ONLY`.

## Evidence boundary

R56 closes the sign question at the exact R55 cycle-64 joint witness:

- 308 rows are strictly raw-positive in full binary128;
- 5692 rows are resolved negative and none is unresolved;
- fresh pair-once and captured directed binary64 are bit-identical;
- another 168 rows are positive only under the unchanged directed enclosure;
- contact box, normal ball and 494-row master membership remain closed.

The worst strict raw lower bound is about `3.820676e-20`. This is a real solver
residual, but solving the original minimum-norm projection ever closer to its
boundary is not by itself a terminal certificate strategy. At a true active
constraint the mathematical optimum has raw residual zero, while a sound
finite-precision interval around zero generally still has positive upper end.

## Literature check

Boyle and Dykstra prove that Dykstra's correction scheme computes the least
squares projection onto an intersection of translated closed convex sets. That
justifies the grouped halfspace/box reference, but it is an asymptotic
projection method, not a finite strict-interiority mechanism
([DOI](https://doi.org/10.1016/0378-3758(86)90111-4)).

Bauschke et al. give finite-convergence conditions but also construct cases in
which Dykstra stalls for an arbitrarily long time. Extra cycles are therefore a
useful diagnostic, not a production terminal architecture
([arXiv:2001.06747](https://arxiv.org/abs/2001.06747)).

Pang interprets Dykstra as alternating minimization on the dual and augments it
with supporting-halfspace QP steps. This makes SHQP a principled acceleration
candidate if the bounded reference below remains asymptotic
([arXiv:1601.01174](https://arxiv.org/abs/1601.01174)).

The July 2026 work by Ding, Feng and Li is especially relevant to our likely
fallback. It shows why a naive dual semismooth Newton step can remain singular
near a degenerate polyhedral projection, and constructs an extreme-point
projection-equivalent representative with a nonsingular generalized Jacobian,
plus a globalized Wolfe variant and local superlinear convergence
([arXiv:2607.12551](https://arxiv.org/abs/2607.12551)). This is a stronger R58
direction than adding an arbitrary diagonal regularizer if R57 does not close.

## Candidate comparison

### A. Continue the original cycle-64 Dykstra state

This retains the exact original minimum-norm QP and can show whether the 308
raw positives still contract. It cannot guarantee a negative certificate
margin at active constraints, and the literature permits arbitrarily long
stalling. It is rejected as the primary R57 experiment.

### B. Certificate-aware residual refinement

Let `w0` be the exact R55 cycle-64 witness and `u0` its full current directed
upper vector. Solve one new, zero-initialized minimum-norm correction:

```text
minimize    0.5 ||delta||^2
subject to  u0_i + A_i delta <= 0       for i in the 494-row master
            contact_low - w0 <= delta <= contact_high - w0
```

Then audit `w = w0 + delta` with the unchanged all-row directed certificate.
Unlike a fitted epsilon, `u0` is already the proved upper endpoint. The shifted
problem asks only for the displacement needed to cancel that endpoint. It
therefore creates a certificate-derived interior raw margin while preserving
the physical halfspaces, contact box and minimum-norm objective.

This is selected for R57. The first implementation remains grouped-box Dykstra
so that only the problem right-hand side changes; solver-family attribution is
not mixed with the certificate-refinement hypothesis.

### C. Face-aware SHQP or degenerate-polyhedral semismooth Newton

These methods can exploit the near-final active geometry and should converge
faster locally than row-action Dykstra. They require a separate generalized
Jacobian/representative-selection oracle, rank and globalization controls. They
are retained as R58, not mixed into R57.

## Selected experiment

R57 must replay exact R56 and retain the real cycle-64 vectors, audit, topology
and master. It then:

1. builds and releases one moved workspace;
2. derives correction-space box bounds from the exact cycle-64 witness;
3. starts `delta`, density multipliers and grouped-box Dykstra correction at
   exact zero for the shifted `u0 + A delta <= 0` problem;
4. executes exactly 64 cycles in stable 494-row order;
5. performs one fresh pair-once residual refresh after every box block;
6. records unchanged directed audits at cycles 8, 16, 32 and 64;
7. chooses the first fully certified checkpoint, otherwise retains cycle 64;
8. performs one R56-style binary64/binary128 sign decomposition on that chosen
   witness;
9. rolls back all state.

New work is exactly 64 density sweeps, 64 grouped box blocks, 64 residual JVPs,
four checkpoint directed audits and one terminal pair/binary128 decomposition:
69 pair passes and one quad row traversal. No row basis, Gram column,
projection call, nonlinear trial, outer, support audit or performance timing is
admitted.

## Outcome-independent classification

After structural gates and exact work/rollback, select in this order:

1. positive row outside the master: master expansion required;
2. normal-ball violation: ball integration required;
3. a certified checkpoint: restoration-next/TRQP-compatible candidate;
4. terminal full-binary128 resolved positive: stronger solver polish required;
5. terminal full-binary128 unresolved sign: sign oracle unresolved;
6. no true positive but directed-positive rows remain: enclosure fixed-point
   refinement required;
7. otherwise retain the reference without authority.

There is no residual tolerance and no gamma change. `PASS` means only that the
frozen refinement experiment and its classification are exact. It does not
apply the witness, exit restoration, authorize switching, mutate runtime state
or establish production readiness.

## Recommendation

Freeze and implement candidate B as R57. If it closes the directed certificate,
the next task is a transaction-level restoration/TRQP compatibility proof, not
performance. If it remains raw-positive or shows a fixed-point enclosure
defect, freeze R58 around extreme-point-corrected dual semismooth Newton/SHQP;
do not spend more research budget on unstructured Dykstra depth.
