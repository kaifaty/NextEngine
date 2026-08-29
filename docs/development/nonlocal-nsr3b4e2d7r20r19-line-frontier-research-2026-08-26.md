# NSR3-B4E2D7R20R19 shear line-frontier research

Status: `RESEARCH COMPLETE / TRIAL FRONTIER AUDIT SELECTED`.

## Question

Are late steps small because the Newton model is invalid specifically across a
projector-mask boundary, or does Armijo reject trials even after the mask is
stable?

## Hypotheses

| ID | hypothesis | prediction |
|---|---|---|
| L1 | projector-mask crossing is the dominant acceptance frontier | in at least 6/8 late steps, the first mask-stable trial is exactly the first accepted trial, with no earlier stable rejection |
| L2 | the local direction/model is poor even within one projector face | at least one mask-stable trial rejects before a later mask-stable acceptance |
| L3 | a crossing trial can still satisfy rigorous Armijo | accepted trial retains mask changes; the next state changes mask/support and resets the sequence |

## Selected audit

Replay the exact candidate shear and expose every existing line trial for steps
25–32. Independently reconstruct the binary128 Armijo margin and require its
sign to match the stored acceptance. Record the first mask-stable power, first
accepted power, stable rejected trials, crossing accepted trials and dual/KKT
facts. No new trial, alpha, merit rule or state update is allowed.

Route priority: any stable rejection selects L2; otherwise at least six exact
frontier coincidences select L1; otherwise any crossing acceptance selects L3;
else unresolved. This audit decides whether the next research should target a
mask-aware piecewise path or the same-face Newton/globalization model.

