# NSR3-B4E2D7R20R2 global ADMM oracle evidence

Status: `PASS / GLOBAL_ADMM_ORACLE_UNRESOLVED / PARTIAL HYPOTHESIS SUPPORT`.

## Frozen execution

Implementation `bab26969` executed once against the exact v2 manifest,
preflight problem roots and unresolved Dykstra semantic
`885dc7bdb628e4d435b3cd6966c87b6866dd9ca33fec3d8b9718b8477328c60f`.
The binary128 dense controls, Cholesky reconstruction, every linear solve,
workspace lifecycle and finite gates passed. Candidate iterations and old
oracle reruns were zero. Result semantic:

```text
929a6676182dc7702a8950343833e56faad4ac46e4784fda1c67f169f6eff08c
```

## KKT observations

The four quiet cases again certified at cycle zero. The globally coupled
oracle certified the supported-column case at cycle 16,384:

```text
primal            1.444e-33
projected dual     1.291e-32
complementarity    8.502e-34
stationarity       1.986e-32
scaled gap          2.114e-30
```

The rejected cyclic Dykstra oracle had not certified this case after 262,144
cycles and still had stationarity `2.312e-17`. This is strong bounded evidence
that global coupling removes a real local-projection convergence obstruction.

Both filled development cases improved their dual/stationarity components by
orders of magnitude, but reached the fixed ADMM cap without a certificate:

| case | primal KKT | dual KKT | complementarity | stationarity | primal consensus | dual consensus |
|---|---:|---:|---:|---:|---:|---:|
| filled edge | `3.345e-5` | `3.376e-5` | `1.122e-6` | `1.626e-6` | `1.109e-5` | `1.626e-6` |
| filled corner | `3.609e-4` | `3.604e-4` | `2.305e-5` | `5.182e-7` | `1.093e-4` | `5.182e-7` |

The signed primal-dual gap is retained in each checkpoint root and its
nonnegative scaled display is zero for the two infeasible iterates. This does
not satisfy the certificate: the separate signed-gap and primal-feasibility
gates reject. Future reports must expose the signed gap explicitly to prevent a
zero clamped display from being mistaken for convergence.

## Interpretation

Hypothesis A1 is supported for the transfer case and mechanistically supported,
but not closed, on the filled cases. Their primal residuals exceed their dual
consensus residuals by about `6.8x` and `211x`. Fixed single-parameter `rho=1`
therefore over-solves stationarity relative to consensus/feasibility,
especially in the corner regime.

This matches the known sensitivity of ADMM to penalty selection. It also
matches newer multiconstraint results: density consensus and domain consensus
are distinct blocks and need not share an effective penalty. R20R2 is retained
as the first globally coupled reference, but fixed-rho depth extension is
closed.

Because numerical observations of the two R20 v2 holdouts are now known, they
become development cases. They cannot provide final generalization evidence
for any penalty/refinement policy designed from this result. A new source-
frozen v3 holdout pair is required after development convergence is achieved.

No wall timing, runtime, GPU or production authority is granted.
