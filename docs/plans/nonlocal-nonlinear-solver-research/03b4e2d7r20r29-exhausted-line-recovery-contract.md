# NSR3-B4E2D7R20R29 exhausted-line recovery contract

Status: `FROZEN / ONE-SHOT 12-CASE TRAJECTORY AUTHORIZED`.

## Parent

- R28 implementation `36f67e97`, semantic
  `a95370b4d4df0159b6cfd569fc85f714766772c702fc35a4f7e2ae617124e52d`;
- R28 new-side trial root
  `d5536891930aead154c3d0f29f10045807d01755759afecdf5c723cb5c2453c1`;
- R26 semantic `4203c7ce...a8d2`, shear root `1c9a0bf3...cf91` and
  exact 19-step accepted prefix.

## Frozen candidate

Execute the complete ordered 12-case R26 corpus under a separate report-only
candidate. Preserve every non-shear R26 root. For shear, permit exactly one
exhausted-line recovery only at iteration 20 after the exact R26 prefix and
only when all 21 inherited trials reject while crossing masks with unchanged
ball activity.

Re-derive, do not hardcode, the unique scalar-168 zero-lower root and R25
forward bound. Construct `alpha_new=nextafter(root+Bz/abs(dz),+infinity)` and
require the exact R28 alpha/trial root, nonnegative multipliers, one predicted
`-1 -> 0` mask change, unchanged ball, positive rigorous Armijo and finite KKT.
Commit that candidate once and continue the unchanged solver through cap 32.
No later globalization rejection may invoke another recovery.

Report all case roots, certifications, accepted steps, event replacements,
the one recovery, later line/NNQP work and final KKT. PASS of the harness does
not imply solver success; the candidate route distinguishes 12/12
certification, later structural rejection, cap unresolved and regression.

No second recovery, old-side trial, cap/tolerance change, proactive generic
search, timing, runtime/GPU, generalization or production authority.
