# NSR3-B4E2D7R20R23 event-side ULP-ladder contract

Status: `FROZEN / REPORT-ONLY 65-POINT EXPONENT AUDIT AUTHORIZED`.

## Parent

- R22 implementation `17dde13c`, semantic
  `3c39b724b35e097d2208d5f318d7680755360fff6d29923ce3d8c758a8229ad6`;
- exact R21/R20/shear roots;
- all seven analytic roots Armijo-positive, but only step 32 crosses at one
  alpha ULP.

## Frozen audit

For each unique R21 root compute its exact binary128 ULP and evaluate all 65
fixed powers `root + 2^k*ulp`, `k=0..64`, through the unchanged affine
multiplier, transpose, projector, dual and R19 Armijo formula. Require every
alpha to remain finite, positive, no larger than the frozen R20 transition-cell
high endpoint and multiplier-cone feasible.

The first changed-mask sample must change exactly the predicted scalar with
unchanged ball activity. Report its power, alpha displacement, mask, Armijo
margin, dual increase and KKT tuple. Count no-cross cases, wrong-face cases,
first-cross Armijo rejections and any later additional mask/ball event.

Classify all seven correct Armijo-positive crossings with no later event as
`ULP_LADDER_EVENT_SIDE_CANDIDATE`; otherwise prioritize wrong-face, no-cross,
Armijo rejection and later multi-event routes.

No early stop, fitted ULP count, solver trial/state update, cap/tolerance
change, timing, runtime/GPU or production authority.

