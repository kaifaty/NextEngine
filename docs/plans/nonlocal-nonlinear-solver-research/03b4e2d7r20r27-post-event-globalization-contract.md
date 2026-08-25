# NSR3-B4E2D7R20R27 post-event globalization contract

Status: `FROZEN / REPORT-ONLY REJECTED-STEP AUDIT AUTHORIZED`.

## Parent

- R26 implementation `4b0f58ba`, semantic
  `4203c7ce2b86014a3099eb440bb0d011f8b6e31856ef142c3acbd93ed807a8d2`;
- exact shear candidate root `1c9a0bf3...cf91`;
- 19 accepted steps, three event replacements, final primal/dual mapping
  `4.343128927689235e-9`, then `GLOBALIZATION_REJECTED`.

## Frozen audit

Replay only the R26 shear opt-in at cap 32 and reproduce its complete root.
Recompute every existing Armijo margin with the exact R19 formula. Report all
accepted steps and all final rejected trials, including face/support roots,
mask/ball deltas, slope/bound, dual increase and KKT metrics.

Classify in priority:

1. committed event does not produce its exact one-scalar face/support lineage:
   `POST_EVENT_FACE_INCONSISTENCY`;
2. any mask-stable final trial has strict negative margin:
   `POST_EVENT_SAME_FACE_REJECTION`;
3. all final trials cross masks/ball:
   `POST_EVENT_LINE_ENVELOPE_EXHAUSTED`;
4. a final margin is not rigorously signed:
   `POST_EVENT_PRECISION_BOUNDARY`;
5. otherwise `POST_EVENT_GLOBALIZATION_UNRESOLVED`.

No new line point, cap/tolerance change, event application, timing,
runtime/GPU, production or generalization authority.

