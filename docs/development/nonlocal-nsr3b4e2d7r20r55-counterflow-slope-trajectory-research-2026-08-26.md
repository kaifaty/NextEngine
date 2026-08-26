# NSR3-B4E2D7R20R55 counterflow slope-refinement trajectory research

Status: `PASS / COUNTERFLOW ORDINARY CERTIFICATION RESTORED`.

## Question

When the R54 verified error is consumed at the exact counterflow slope
rejection, does the unchanged solver certify the case or expose a later
independent boundary?

## Candidate mechanism

Add a default-null, root-agnostic slope-error refiner. It is eligible only after
the natural face and NNQP solve are exact, NNQP is KKT-certified, its direct
solution corresponds to the final support, and the initial global slope alone
is uncertified. The refiner may replace only the scalar direction error and
recompute the existing slope bound. It may not move the solution.

For R55, cap successful refinement at one. The callback invokes only the
existing classical verified-inverse on the captured direct factor. If the new
slope certifies, execute the unchanged existing line search and subsequent
semismooth logic. A later refinement request fails closed without another
inverse build, preserving the first subsequent boundary.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| T1 | counterflow needs only verifier placement | one audit is consumed and the case certifies ordinarily |
| T2 | a later slope boundary repeats | one audit advances the trajectory; a second request hits the cap |
| T3 | globalization becomes the next blocker | refined slope passes, then unchanged Armijo/recovery rejects |
| T4 | trajectory apparatus is inconsistent | old prefix/slope/direct tuple or audit correspondence changes |

No result promotes the policy to default or to the full corpus. T1 would
authorize a separate five-case replay with the generic torsion verifier and
the slope fallback composed under independent caps.

## Outcome

T1 is confirmed. Exactly one iteration-10 refinement is consumed; unchanged
globalization certifies counterflow with the same 701 principal solves and
transitions as the rejected baseline. A composed five-case replay is now the
smallest remaining generalization test.
