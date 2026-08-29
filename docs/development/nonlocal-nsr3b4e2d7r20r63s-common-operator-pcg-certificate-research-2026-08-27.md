# NSR3-B4E2D7R20R63S common-operator PCG certificate research

Status: `FROZEN / IMPLEMENTATION_NEXT`.

## Question

With tangent Gram selected as finite operator representation and a two-sided
contractive common-operator inverse available, do the two frozen direct-PCG
lanes actually solve the immutable RHS under exact `H*` semantics?

## Independent certificate

For candidate iterate `x`, convert the immutable binary128 RHS and `x` to exact
dyadics and compute

```text
r_num = b d - H_num x,
r = r_num / d.
```

R63R provides

```text
A_num = ||Z||inf d,
A_den = d - ||I-ZH*||inf_num,
||Z||inf/(1-rho_left) = A_num/A_den.
```

The exact solution-error bound simplifies to

```text
error = ||Z||inf ||r_num||inf / A_den.
```

For each component, compare `abs(x_i) A_den` with the error numerator by exact
integer/exponent arithmetic. A strict positive or negative sign is derived
from the candidate interval itself; no dense solution or legacy sign vector is
consulted.

## Frozen replay

Reconstruct the two R63O candidate lanes:

1. retained-wide factor built from binary64 tangent storage;
2. one-time exported factor consumed in binary128.

Start from their immutable R63I solutions. Use original binary128 tangent
coefficients in direct `sigma T(T^T p)` products and the unchanged eight-update
PCG recurrence. Certify the initial state and every update `1..8` against exact
`H*`. Continue through all eight updates even after the first pass.

A lane passes only if it has at least one `102/102` sign certificate and every
later frozen certificate also passes. The two final sign vectors must agree.
Iteration zero is a valid first-pass location but does not authorize a runtime
stop rule.

## Interpretation

A two-lane pass establishes the first structured solution certificate for the
captured RHS, independent of stored dense `H/X`. It selects subsequent
arithmetic/storage qualification and sparse realization, not production.

A failure distinguishes PCG recurrence precision from the exact residual
floor. It does not restore dense `H` as oracle and cannot be repaired by
reusing R60 signs.

## Scope ceiling

No dense candidate product, dense inverse/sign oracle, defect transport,
state update, sparse realization, timing, runtime stopping, GPU or production
authority is admitted. R64 and R65 remain blocked.

