# NSR3-B4E2D7R20R23 event-side ULP-ladder research

Status: `RESEARCH COMPLETE / FIXED EXPONENT LADDER SELECTED`.

## Question

How many alpha ULPs are required for the actual multiplier/transpose/projector
graph to observe each R21 face crossing, and does the first observed crossing
remain Armijo-valid?

## Hypotheses

| ID | hypothesis | prediction |
|---|---|---|
| U1 | root-to-projector roundoff is a bounded local displacement | every event first changes exactly its predicted scalar within the fixed `2^0..2^64` ULP ladder and remains inside the R20 transition cell |
| U2 | the first observable crossing is still physically acceptable | every first-crossing trial has a strict positive rigorous Armijo margin |
| U3 | the analytic event is not robust through the evaluation graph | a ladder never crosses, crosses another scalar/ball mode, leaves the frozen cell or first crosses after Armijo becomes negative |

## Selected discriminator

Replay R22 exactly. For every root and `k=0..64`, evaluate

```text
alpha_k = root + 2^k * ulp(root).
```

All 65 points are fixed before observation. Record the first point whose
projector differs from current, require exactly the predicted scalar and
unchanged ball activity, and compare it with the frozen R20 mask/Armijo cells.
Audit the rigorous Armijo margin, dual increase and KKT tuple at that first
crossing. Also record whether later ladder points introduce another event.

The ladder is a report-only finite-precision diagnostic. It cannot be shortened
after observing a case, fitted into a constant epsilon, inserted into the
solver or used to apply a state. A positive result can authorize only a
separately frozen exponential-bracketing candidate.

