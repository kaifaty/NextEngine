# NSR3-B4E2D7R19R37 hybrid polish continuation research -- 2026-08-24

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Question

R36 selects curvature-plus-polish at equal work, but its projected mapping is
still `7.4709090069224081e-10`. Strict dominance over R34 is a comparison,
not a termination certificate. Before evaluating any moved nonlinear state we
need to know whether the unchanged polish recurrence continues to drive the
linearized trust-region problem toward projected stationarity or reaches its
first numerical/active-set plateau.

## Method basis

For minimization over a closed convex set, Birgin, Martinez and Raydan define
the scaled projected gradient as `P(v-t*g(v))-v` and prove that it vanishes if
and only if the point is constrained-stationary. Their algorithms use the
unit-scaled projected gradient as the termination statistic. See the authors'
[primary paper](https://ime.unicamp.br/~martinez/bmr.pdf), especially
Algorithm 2.1 and Lemma 2.1. Their later
[review](https://www.ime.usp.br/~egbirgin/publications/bmr5.pdf) retains the
same projected-gradient stopping structure.

This supports continuing to report the R36 mapping, but it does not provide a
problem-specific tolerance for this Nonlocal normalization. R37 therefore
must not fit one from the observed result. Exact binary64 zero remains the
only stationarity route in this discriminator; nonzero values form a
convergence curve for later tolerance/error-budget research.

## Selected experiment

Start from the exact private R36 endpoint and its maintained response. First
recompute a fresh JVP to attest that prefix. Continue only the unchanged R33
recurrence:

```text
positive residual
  -> pair-once VJP gradient
  -> P_ball(v - gradient / ||gradient||)
  -> normalized feasible chord
  -> pair-once JVP
  -> exact all-row hinge line minimum
  -> update private iterate and maintained response
```

Freeze additional-step checkpoints at `6`, `12` and `24`. These are doubling
blocks from the R36 endpoint rather than an observation-dependent horizon.
The gradient already computed at the start of step 7/13 supplies the mapping
for checkpoints 6/12; fresh terminal JVP/VJP supplies checkpoint 24. No extra
operator pass is hidden in checkpoint reporting.

The exact new-work ledger is:

```text
fresh R36-prefix JVP                  1
24 polish steps x (VJP + JVP)       48
fresh terminal JVP + VJP             2
total                               51 pair passes
generalized HVPs                     0
```

Every accepted step must preserve strict objective reduction, exact-line KKT
and global-L2 trust feasibility. Checkpoints report objective, violation,
active count, projected mapping and block contraction. A nonzero terminal
mapping may select continued convergence or a mapping-regression observation;
neither is a solver failure. Any parent/root/operator/work/lifecycle mismatch
fails before classification.

## Alternatives rejected now

- another curvature block: R37 isolates termination behavior of the selected
  R36 polish phase;
- spectral/Barzilai-Borwein scaling: promising later, but it changes the
  recurrence before the current curve exists;
- tolerance `1e-9` or `1e-10`: both would be fitted after seeing R36 and lack
  a physical/nonlinear error budget;
- applying the R36/R37 vector or evaluating a moved nonlinear state: the
  linearized subproblem does not yet have a frozen termination policy;
- wall-time comparison: the shared-host performance stop remains active.

## Expected decision

R37 can establish exact stationarity, continued nonzero contraction, or the
first mapping regression/plateau at a frozen horizon. All outcomes remain
report-only. A later stage must connect the mapping scale to nonlinear model
agreement and physical error before the normal step can be admitted.

Frozen contract:
[R37 hybrid polish continuation](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r37-hybrid-polish-continuation-contract.md).
