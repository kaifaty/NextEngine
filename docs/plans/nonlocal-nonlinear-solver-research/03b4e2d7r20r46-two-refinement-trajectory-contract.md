# NSR3-B4E2D7R20R46 two-refinement trajectory contract

Status: `FROZEN / DEFAULT-OFF TWO-REPLACEMENT TRAJECTORY AUTHORIZED`.

## Parent

- R45 implementation `6278cc01`, semantic
  `4263411e0abf582cd45044635d26efa1f83e4d8d3be52170c18c480e87d46080`;
- second residual/correction/left/z roots `612b7268...08b5` /
  `6c25184c...8325` / `68dd63bf...c104` / `11734bce...2863`;
- second depth-16 root `85bffd4d...de64`, radius `3.1658780995e-14`,
  signs `29/36/0` and minimum separation `240.0319`;
- original practical depth-eight root `0d3e1914...6849` and exact R43/R44
  target roots.

## Frozen trajectory

Preserve R42/R43 default behavior. Generalize the practical helper only to
accept a frozen depth and optional target-specific value check; existing depth-
four/eight roots must remain exact. Install one R46 hook for a single torsion
replay. It may replace the original tuple once at depth eight and the exact
second tuple once at depth 16. Require both practical certificates finite,
no-underflow, contractive and fully signed; require the second radius/counts
equal R45 and uniform separation positive.

Report hook attempt/match/replacement cardinalities, every resulting principal
solve/transition, final case/step/failure roots and the first unrecognized
inverse tuple if present. Do not retry a later boundary. Clear the hook and
regress R45/R44/R43/R42.

Routes in precedence:

1. `TWO_REFINEMENT_PARENT_REJECTED`;
2. `TWO_REFINEMENT_FIRST_REJECTED`;
3. `TWO_REFINEMENT_SECOND_REJECTED`;
4. `TWO_REFINEMENT_CARDINALITY_REJECTED`;
5. `TWO_REFINEMENT_NOT_CONSUMED`;
6. `TWO_REFINEMENT_LATER_INVERSE_BOUNDARY`;
7. `TWO_REFINEMENT_LATER_BOUNDARY`;
8. `TWO_REFINEMENT_TORSION_CANDIDATE`.

R46 adds only the second centered arithmetic and one local passive-vector
replacement. It changes no factorization, inverse-column policy, principal
solve, ratio formula, tolerance, cap, globalization, trial set, counterflow,
production state or timing.
