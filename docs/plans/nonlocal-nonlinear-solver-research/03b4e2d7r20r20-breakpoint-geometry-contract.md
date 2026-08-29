# NSR3-B4E2D7R20R20 projector-breakpoint geometry contract

Status: `FROZEN / REPORT-ONLY SUBDIVISION AUDIT AUTHORIZED`.

## Parent

- R19 implementation `38b788c8`, semantic
  `46f8d84a950e3987609d32858f1c972d60e1cf1d937f1c3fd5b20d1f058ca06f`;
- exact shear root `fdd57c9e...fbb9`;
- seven first-stable/first-accepted coincidences, zero stable rejections and
  one crossing acceptance.

## Frozen audit

Replay only the exact shear candidate. Select exactly the seven steps for which
R19's first stable power equals its first accepted power. Reconstruct the
representative from the pre-step multiplier, accepted multiplier and accepted
alpha. For each selected step evaluate exactly 65 uniform binary128 points in
`[alpha_accept,2*alpha_accept]`.

Each sample must use the unchanged affine multiplier path, joint box-ball
projector, dual evaluator and R19 Armijo formula. Require:

- both endpoint dual/projector roots, masks, powers and Armijo decisions equal
  the existing R19 trials;
- multiplier nonnegativity and finite/exact projector/dual evidence;
- unchanged R19 semantic and shear candidate root;
- no solver state update and no generated solver line trial.

Report per bracket the mask-root run count, transition count, distinct changed
scalars, mask returns, ball changes, stable negative-margin samples, Armijo
sign transitions and the cells containing the first mask and sign transitions.
Use the classification priority frozen in the research document.

No adaptive subdivision, boundary fitting, timing, cap/tolerance change,
runtime/GPU or production authority. A positive result can authorize only a
separately frozen breakpoint-aware solver candidate.

