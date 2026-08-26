# NSR3-B4E2D7R20R31 terminal certificate precedence research

Status: `RESEARCH COMPLETE / CERTIFICATE-VERSUS-MERIT AUDIT SELECTED`.

## Question

Does any exact R30 iteration-22 trial already satisfy the complete frozen KKT
certificate even though bounded Armijo merit rejects it?

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| C1 | no trial is actually terminally certified | every `metrics.certified` is false and the first failed predicate is identified |
| C2 | a trial is terminally certified before Armijo | exact cone/face/KKT predicates pass while the same trial has negative Armijo margin |
| C3 | displayed residuals hide a gap-sign or finite failure | small scalar metrics coexist with false finite/gap-sign predicate |
| C4 | the dual-merit comparison is bound dominated | nominal merit change is smaller than the explicit old/new/RHS error budget |

## Selected discriminator

Replay exact R30/R29 shear and inspect only the existing 21 trials. Report each
complete `metrics.certified` predicate, metric root, finite/gap signs and ratios
to `2^-70`. Identify the first certified trial in inherited order.

For every trial decompose the R19 comparison into nominal dual change,
old/new dual bounds, certified slope term, RHS rounding bound, dual-increase
lower bound and final margin. Record whether the bound burden dominates the
nominal change. Add no trial and do not accept state.

## Ceiling

A positive result may authorize a separate opt-in terminal-certificate
trajectory: it does not prove the KKT oracle independent of all shared code,
change the ordinary Armijo policy, weaken the certificate or establish
production readiness.
