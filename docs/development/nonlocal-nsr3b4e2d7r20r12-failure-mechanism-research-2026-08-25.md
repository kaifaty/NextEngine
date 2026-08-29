# NSR3-B4E2D7R20R12 v3 failure-mechanism research

Status: `RESEARCH COMPLETE / DIAGNOSTIC REPLAY SELECTED`.

## Observation boundary

R11 has two strict successes and two deterministic active-set rejections. The
first unknown is not whether more iterations help: both rejected solves stop
before the outer semismooth cap. The unknown is the first exact predicate that
stops the inner natural-face/NNQP computation.

## Competing hypotheses

| ID | causal hypothesis | observable prediction | falsifier |
|---|---|---|---|
| H1 | natural-face membership becomes sign-ambiguous | final step has unresolved face or nonzero ambiguous rows before an NNQP solve | resolved face and named NNQP failure |
| H2 | inactive reduced-gradient enclosure straddles zero | `INACTIVE_SIGN_UNRESOLVED` | any other first failure |
| H3 | a principal system is singular or its solve cannot be certified | `PRINCIPAL_SOLVE_REJECTED` | factor/solve succeeds and a later predicate rejects |
| H4 | the active-set transition has an unresolved ratio, boundary update or finite transition budget | one of the three corresponding failure strings plus its transition ledger | a failure before transition logic |

`PASSIVE_SIGN_UNRESOLVED` and `VERIFIED_INVERSE_AUDIT_REJECTED` remain explicit
negative controls, but R11's zero inverse-audit counts are evidence against
them. Increasing the iteration cap is irrelevant unless the postmortem finds
`TRANSITION_CAP`.

## Selected discriminator

Replay only the two now-public failed cases through the unchanged R8 routine.
Require exact reproduction of their R11 case roots, then expose only the final
step's already-computed face, factor, enclosure and active-set ledger. No
arithmetic, factorization, bound, tolerance or branch changes are permitted.

This diagnostic can identify the first failing predicate. It cannot establish
the mathematical cause of that predicate or authorize a remedy; the result
will freeze the smallest subsequent derivation/counterfactual.

