# NSR3-B4E2D7R20R42 centered-refinement trajectory research

Status: `RESEARCH COMPLETE / TARGET-BOUND OPT-IN TRAJECTORY SELECTED`.

## Question

When the exact R41 four-step center replaces only the rejected torsion passive
solution, does the unchanged NNQP correctly process its six newly certified
negative components and reach a later meaningful boundary?

## Candidate boundary

Introduce a default-null verified-solution refinement hook. It may run only
inside the existing verified-inverse fallback and only when matrix, inverse,
RHS and represented-solution roots equal the frozen R41 target. The hook uses
the practical R38/R41 Dot2 bounds, not exact big integers, to:

1. enclose `C=I-XA`, `r=b-Ax` and `z=Xr`;
2. build exactly four shadow centers `y<-z+C*y`;
3. certify `g=z+C*y-y` and `||e-y||inf`;
4. return the compensated center `x+y` plus one uniform error only when all
   65 strict signs resolve without underflow.

R41's exact oracle has already validated this arithmetic construction. R42
must nevertheless reproduce the practical certificate root and all parent
roots before the hook may replace the local passive vector.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| T1 | sign ambiguity was the only torsion boundary | one certified replacement is consumed and torsion terminates/certifies |
| T2 | refinement is valid but exposes a later solver boundary | replacement is consumed, then a different exact failure occurs |
| T3 | integration does not reproduce the offline certificate | target/root/bound/sign rejection before replacement |
| T4 | hook changes unrelated behavior | invocation outside the unique frozen target or parent regression |

T1 or T2 may authorize a later case-agnostic development-corpus candidate.
T3 returns to the integration apparatus. R42 does not touch counterflow, change
the active-set ratio/globalization rules or promote the hook to a default path.
