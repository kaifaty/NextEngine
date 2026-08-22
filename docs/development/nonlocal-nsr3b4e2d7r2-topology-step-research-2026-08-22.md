# NSR3-B4E2D7R2 topology/step discriminator research

Date: `2026-08-22`

Status: `RESEARCHED / CONTRACT_READY / REPLAY_ONLY`

## Problem reclosure

B4E2D7R1 rules out its original arithmetic-floor hypothesis. At the exact
failed AL inner state, both raw subtraction and an independently factorized
per-term difference say that the Newton proposal raises the objective:

```text
predicted reduction        =  1.3230255447006828e-15 J
raw actual reduction       = -4.5363018896793506e-16 J
factored actual reduction  = -4.5281433411232910e-16 J
```

The model therefore predicts descent that the objective does not have. At the
same time the aggregate active/pair topology changes, but D7R1 does not say
which set changed or whether that change caused the disagreement.

The trust policy also does not test a smaller step. The physical Newton norm
is approximately `2.479e-10 m`; after nine rejects the radius is still
`4.768e-8 m`. With quarter-radius rejection updates, the radius first binds
only after 13 shrinks, four beyond the frozen reject cap.

## Why membership alone is insufficient

The normalized cubic kernel is compactly supported and its value, first
radial derivative and second radial derivative are all zero at `r=h`. A pair
crossing the enumerated `r<=h` membership boundary is therefore an operational
topology change, but not automatically a discontinuity in the objective,
gradient or Hessian.

Conversely, the third radial derivative changes at the horizon. A sufficiently
large step can cross to a different polynomial branch and invalidate a
second-order model even though the kernel is C2. The unilateral PHR term has a
separate branch at `lambda + beta*c = 0` and must be distinguished from the
radial branch.

A useful counterfactual must consequently preserve the *current mathematical
branches*, not merely reuse the current pair container while still evaluating
the live compact-support branch.

## Selected diagnostic

Replay the exact D7R failed state and reconstruct its first rejected Newton
direction without accepting it. For the current point and full proposal:

1. enumerate pressure-active centres, unique fluid pairs and directed
   fluid-boundary pairs as sorted typed keys;
2. publish exact roots plus added/removed counts for each set;
3. for every changed pair publish current/trial radii, signed horizon margins
   in metres and `epsilon*h` units, and normalized `W/W'/W''` on both sides;
4. prove `W(h)=W'(h)=W''(h)=0` in the actual binary64 implementation;
5. derive the number of quarter-radius shrinks required before the trust
   radius binds the original direction.

Then evaluate a fixed alpha ladder `alpha=2^-e`, `e=0..20`. Each row records
the rounded position-step norm, live active/pair roots and deltas, quadratic
predicted reduction, raw and factored live actual reductions, and actual/model
ratio. A row whose rounded coordinates equal current is explicitly marked and
cannot establish descent.

## Current-branch continuation

For the same alpha rows, evaluate a diagnostic-only continuation of the branch
seen at the current point:

- every fluid/boundary relation uses the current radius branch (`q<1`,
  `1<=q<=2`, or zero-support `q>2`) at the trial radius without switching;
- every PHR centre uses its current active/inactive branch at the trial
  constraint without applying a new `max` decision;
- inertia is unchanged;
- energy difference uses the same factorized fixed-order formula as D7R1.

This continuation is not a new physical objective and may never accept or
publish state. It is only the local smooth function whose derivatives the
current Newton model approximates.

## Decision logic

Routes are evaluated in this order:

1. any pressure-active membership change selects
   `ACTIVE_CONSTRAINT_BRANCH_RESEARCH_REQUIRED`;
2. otherwise, a moved row where current-branch continuation has admissible
   descent but the live branch does not selects
   `HORIZON_TOPOLOGY_DERIVATIVE_RECLOSURE_REQUIRED`;
3. otherwise, the first smaller moved alpha with live positive model/direct
   reduction and direct ratio at least `0.1` selects
   `TRUST_REJECT_POLICY_RECLOSURE_REQUIRED`;
4. otherwise select `AL_HESSIAN_MODEL_RESEARCH_REQUIRED`, provided the exact
   replay, negative directional slope and at least one moved row remain valid.

The classification changes no kernel, PHR formula, beta, tolerance, trust
policy, reject cap or state. It authorizes research for exactly one selected
route and no trajectory.

## Rejected shortcuts

- Do not reinterpret pair-count drift as a nonsmooth physics failure without
  inspecting kernel branches and contributions.
- Do not increase reject limit merely to make the current radius bind.
- Do not replace raw subtraction: D7R1 already proved the direct reduction is
  negative too.
- Do not lower inner tolerance: a more accurate solve still needs a globally
  valid step acceptance mechanism.
- Do not use wall timing or resume the stopped performance A/B lane on the
  shared host.
