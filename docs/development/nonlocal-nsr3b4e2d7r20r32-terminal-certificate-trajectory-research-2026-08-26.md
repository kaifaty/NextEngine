# NSR3-B4E2D7R20R32 terminal certificate trajectory research

Status: `RESEARCH COMPLETE / TERMINAL-SELECTION TRAJECTORY SELECTED`.

## Question

Does selecting an already evaluated, exactly KKT-certified trial only after
the inherited line search rejects close the R29 shear case and the full
12-case corpus without weakening ordinary globalization?

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| U1 | terminal KKT precedence is the missing completion rule | exact R29 iteration 22 selects power 0 and all 12 cases certify |
| U2 | the reported trial certificate does not correspond to returned state | selected primal/lambda/metrics roots differ or a fresh terminal audit fails |
| U3 | the hook changes an ordinary trajectory | a non-shear root or preterminal shear step differs from R29 |
| U4 | R31 depended on report-only observation order | the in-trajectory trial no longer has the exact R31 root/predicate |

## Selected causal probe

Replay the complete ordered R29 corpus under a separate default-false
candidate. Preserve the exact ordinary line search. Only after all inherited
trials reject may the candidate inspect those already computed trials in
existing power order. At the exact R29 shear iteration-22 boundary, select the
first trial whose full unchanged `metrics.certified` predicate is true.

Require exact R31 power-0 alpha, dual/metrics/trial roots, same projector face,
same ball state, failed Armijo and a fresh KKT audit of the state being
returned. Record this as terminal selection, not Armijo acceptance, and stop
without another nonlinear iteration. All other cases and all earlier steps
must retain R29 roots.

## Why this discriminator

Armijo controls whether a nonterminal step has sufficient model progress. A
complete KKT certificate answers the stronger terminal question: whether the
candidate already satisfies the constrained problem to the frozen tolerance.
Using the latter only to terminate does not admit an uncertified progress step
or change the merit model. It directly separates a missing termination order
from state-correspondence and trajectory-regression explanations.

## Ceiling

Success is a report-only 12-case candidate. It does not establish an
independent KKT oracle, generalization beyond the frozen corpus, binary64 or
runtime correspondence, production readiness, performance, GPU suitability
or permission to change the default solver.
