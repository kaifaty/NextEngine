# NSR3-B4E2D7R20R21 fixed-face projector-event contract

Status: `FROZEN / REPORT-ONLY ANALYTIC PREDICTOR AUTHORIZED`.

## Parent

- R20 implementation `4ca6de6e`, semantic
  `4ca89c1ffa825d8a09422f460c40886b7e5e60ac9992602726eb7d0a5dcc1de5`;
- exact R19 semantic `46f8d84a...a06f` and shear root
  `fdd57c9e...fbb9`;
- seven simple one-scalar breakpoint brackets, with observed scalars 168 for
  steps 25--27 and 189 for steps 29--32.

## Frozen audit

Use the fixed-face KKT derivation in the research document. For each R20
bracket, construct the affine projector input from the exact pre-step
multiplier and unchanged representative. Enumerate both box bounds for every
non-fixed scalar:

- inactive ball: solve the linear boundary equation;
- active ball: solve the derived quadratic and require the unsquared boundary
  relation, sign and face-side predicates;
- zero bound: use the unsquared linear `z_i(alpha)=0` equation;
- repeated/near-degenerate roots: fail closed unless the exact cell endpoint
  predicates select one root without tolerance fitting.

Require exact reproduction of R20 and its 14 endpoint roots. A successful
bracket has exactly one admissible event in its frozen transition cell, the
predicted scalar equals the observed scalar, and exact projector masks on the
cell endpoints differ only at that scalar. Classify all seven successes as
`FIXED_FACE_EVENT_PREDICTOR_CANDIDATE`; otherwise distinguish ambiguous,
wrong-scalar, unbracketed and algebra/control rejection.

No adaptive scan, fitted epsilon, solver trial/state update, post-boundary
offset, cap/tolerance change, timing, runtime/GPU or production authority.

