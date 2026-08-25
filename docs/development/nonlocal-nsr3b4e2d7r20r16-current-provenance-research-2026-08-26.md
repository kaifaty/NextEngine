# NSR3-B4E2D7R20R16 current-direction provenance research

Status: `RESEARCH COMPLETE / DUAL INVERSE COUNTERFACTUAL SELECTED`.

## Question

Does the current direction at the corner ratio collision still have direct
principal-solve provenance, so its error can be independently refined rather
than propagated or weakened?

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| P1 | current `x` is the most recent complete passive solution | provenance flag is direct; stored factor/solution reproduces current support and a verified inverse makes the two-error ratio ordering strict |
| P2 | current `x` has been interpolated since its last solve | provenance flag is invalidated; a current inverse audit is inapplicable |
| P3 | direct provenance exists but its factor cannot be verified or refinement remains insufficient | audit rejects or dual-refined ratio still overlaps |
| P4 | retaining provenance perturbs the original solver | any R13–R15 root changes |

## Selected discriminator

When an unchanged NNQP iteration accepts a complete nonnegative passive
solution, retain a report-only copy of its factor, solve residual/bound,
solution and passive indices; mark it direct. Any boundary interpolation
invalidates it. At the already-preserved ratio collision, audit the retained
current solve if and only if provenance remains direct. Re-evaluate with both
current and candidate inverse-refined scalar errors, without consuming the
result.

If P1 resolves both cases, the smallest candidate implementation is on-demand
dual inverse refinement at ratio ambiguity. It still needs complete replay and
new holdouts before any generalization claim.

