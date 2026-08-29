# NSR3-B4E2D7R19R31 first diagnostic

Date: `2026-08-24`

The first R31 implementation stopped at the frozen `DENSE` hard gate before
the nominal interval was interpreted. Three controls passed; the zero-margin
inactive control produced binary64 `-0.0` for `alpha_upper` because unary
minus preserved the sign when converting `-c/r` from `c=+0.0`.

The repair canonicalizes an exact zero numerator to binary128 and binary64
`+0.0`. It does not change interval formulas, expected classifications,
source vectors, ratio ordering, repair cap or any physical/solver threshold.
The detailed four-control JSON remains in R31 so signed-zero regressions fail
at the same boundary instead of contaminating nominal evidence.

The initial parent/source/rollback roots were exact and no nominal interval
route was selected. This was an implementation diagnostic, not physics or
performance evidence.
