# NSR3-B4E2D7R20R35 v4 generalization contract

Status: `FROZEN / ONE-SHOT FIVE-CASE SOLVER EXECUTION AUTHORIZED`.

## Parent

- R34 implementation `ad6191ad`, semantic
  `811ac23180956ddf02efae5d52c949bc77619ce33907817d92974e41be4802ab`;
- exact problem roots `5584d517...d3ad`, `ba002346...b21f`,
  `62b1dd7d...7a03`, `fa59af8d...7364`, `192adf04...0e7`;
- R32 semantic `e7b9acaf...0f73`, R31 certificate and all earlier parents;
- no solver observation has occurred for v4.

## Frozen generalized candidate

Execute the five v4 problems exactly once in manifest order with verified
inverse, dual-ratio refinement and the R26 certified-event replacement enabled.
Keep exact binary128 arithmetic, `2^-70`, 21 dyadic trials and cap 32.

After an ordinary complete line rejection, first scan the existing trials in
power order. A trial may terminate only if its full current
`metrics.certified` predicate is true; reconstruct its multiplier/dual/primal
state and require fresh root-identical KKT certification before return. This
is terminal selection, not Armijo acceptance.

If no terminal trial exists, at most one exhausted-line recovery per case is
permitted, and only when all 21 exact trials reject, every trial changes a
component mask, none changes ball activity and the endpoint is exactly
`alpha=2^-20`. Enumerate all fixed-face component and ball event polynomials in
`(0,2^-20]`. Require exactly one nearest admissible event, no equal-alpha
multiplicity, no prior admissible event, and specifically a zero-bound release
whose current mask is `-1` or `+1` and whose new mask is `0`.

Derive the componentwise R25 forward bound from the target component and every
nonzero sparse transpose entry touching that particle/axis. Set
`alpha_new=nextafter(root+Bz/abs(dz),+infinity)` and require it remains inside
`(root,2^-20]`, changes exactly the predicted scalar to `0`, preserves ball
activity, keeps multipliers nonnegative and has positive rigorous Armijo.
Apply it once and continue unchanged. A second exhaustion fails closed.

Report routes for all-certified, active-set, direction, globalization,
unsupported/ambiguous event, terminal correspondence, second exhaustion and
cap outcomes. Preserve every failure and first boundary.

No hardcoded v4 iteration/scalar/root, nonzero-bound or ball recovery, second
recovery, added line trial, tolerance/cap/Armijo change, retry, timing,
runtime/GPU, generalization beyond this corpus or production authority.
