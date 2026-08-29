# NSR3-B4E2D7R20R49 three-refinement trajectory contract

Status: `FROZEN / DEFAULT-OFF THREE-REPLACEMENT TRAJECTORY AUTHORIZED`.

## Parent

- R48 implementation `2de0791b`, semantic
  `7d4d7a9e3765274338707855df4953d61c44b6a3d15c9362bf12b70eccbe7802`;
- third residual/correction/left/z roots `67753295...54b7` /
  `64111479...ae2a` / `46c04691...0fd5` / `33bcef2c...a3a7`;
- third depth-16 checkpoint root `f0eade1d...0441`, radius
  `6.492670105723917989289204994889832681e-14`, signs `32/33/0` and positive
  minimum separation;
- first/second practical roots and all three target tuples remain frozen.

## Frozen trajectory

Install one private default-off hook for one torsion replay. It may replace the
original target at depth eight, the second target at depth 16 and the third
target at depth 16, exactly once each and in order. Require every practical
certificate exact, finite, no-underflow, contractive, fully signed and equal to
its frozen radius/counts. Preserve the first later failed inverse tuple without
retry and report all hook cardinalities, solve/transition counts, final case/
step/failure roots and unknown tuple roots.

Routes in precedence:

1. `THREE_REFINEMENT_PARENT_REJECTED`;
2. `THREE_REFINEMENT_FIRST_REJECTED`;
3. `THREE_REFINEMENT_SECOND_REJECTED`;
4. `THREE_REFINEMENT_THIRD_REJECTED`;
5. `THREE_REFINEMENT_CARDINALITY_REJECTED`;
6. `THREE_REFINEMENT_NOT_CONSUMED`;
7. `THREE_REFINEMENT_LATER_INVERSE_BOUNDARY`;
8. `THREE_REFINEMENT_LATER_BOUNDARY`;
9. `THREE_REFINEMENT_TORSION_CANDIDATE`.

Regress R48/R47/R46. R49 changes no factorization, inverse policy, ratio,
tolerance, cap, globalization, trial, counterflow, production state or timing.
If a fourth inverse boundary appears, freeze it and stop target-specific
refinement chaining.
