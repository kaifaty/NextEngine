# NSR3-B4E2D7R19R35 first diagnostic

Date: `2026-08-24`

The first R35 implementation stopped at the independent `DENSE` hard gate
before building the nominal workspace. Exact R34 parent/source, route and
rollback gates passed; new nominal pair/HVP work was zero.

An independent binary64 probe isolated two expectation errors. The 2D SPD CG
control reaches the exact step `(-1,-1)` after its mathematical two-step
termination, but a roundoff residual makes the contract's exact-zero
recurrence continue harmlessly to the frozen five-iteration cap; terminal
residual norm is `3.7982270983039182e-65`. The trust projection control maps
`10.9` to the nearest interior binary64 value
`0.99999999999999989`, while the implementation required bitwise `1.0`.

The repair accepts the already frozen SPD residual tolerance `1e-14` without
requiring an exact iteration count and accepts projection absolute error
`<=1e-15`. It changes no nominal state, generalized-Hessian/CG formula,
work cap, globalization, comparison rule or physical/solver threshold. This
was an implementation diagnostic, not curvature or performance evidence.
