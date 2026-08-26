# NSR3-B4E2D7R20R50 generic centered-verifier research

Status: `RESEARCH COMPLETE / ROOT-AGNOSTIC SHADOW POLICY SELECTED`.

## Observation and conclusion

Four different torsion active sets fail the legacy gamma inverse audit. The
first three represented inverses are independently contractive and their
centered solution certificates resolve all signs; consuming them merely moves
the same failure to a fourth tuple. Another tuple-specific audit would no
longer discriminate a serious hypothesis.

The defect is architectural: accurately computing the cancellation-dominated
residual must be part of the verification policy, not an exception keyed by
matrix hashes.

## Numerical basis

Ogita, Rump and Oishi prove the error-free-transform `Dot2` residual bound used
by R38--R49: https://doi.org/10.1137/030601818. Rump's verified-linear-system
overview gives the standard approximate-inverse iteration `C=I-RA`,
`Z=R(b-Ax)` and interval inclusion loop (Algorithm 10.7/Theorem 10.8):
https://www.tuhh.de/ti3/rump/intlab/ActaNumerica2010.pdf.

Current INTLAB 14 treats accurate residuals as the central primitive and uses
`prodK`/`spProdK` inclusions, superseding older Dot2/DotK implementations:
https://www.tuhh.de/ti3/rump/intlab/demos/html/dprodK.html. That is a later
performance/design candidate. R50 keeps the already validated binary128 Dot2
apparatus so that arithmetic choice and policy generalization are not changed
simultaneously.

## Selected shadow policy

On every failed legacy inverse audit in one torsion replay, without inspecting
object roots:

1. require dimension 65 and remaining global budget;
2. build the existing practical left-defect/residual/center certificate at
   fixed depth 16;
3. replace only if it is exact, no-underflow, contractive, fully signed and has
   positive separation;
4. otherwise fail closed at that same tuple.

The global certificate/replacement cap is the inherited semismooth accepted-
iteration cap, 32, rather than an observed target count. One 65-row depth-16
certificate executes 5525 compensated dots and 360,490 dot input pairs; the
frozen worst-case ledger is therefore 176,800 dots and 11,535,680 input pairs.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| G1 | the generic certificate closes torsion | root-agnostic replacements end in ordinary torsion certification within cap |
| G2 | a genuinely uncertifiable inverse/solution appears | first failed generic certificate is preserved with its complete bounds |
| G3 | another NNQP/globalization boundary dominates | generic certificates pass, then a non-inverse fail-closed route appears |
| G4 | work policy is structurally excessive | cap or exact work ledger rejects before a readiness claim |

R50 is still a default-off research trajectory. Even G1 does not authorize a
production policy; it first establishes generalization on one holdout case.
