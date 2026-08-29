# NSR3-B4E2D7R20R8 verified inverse-residual research

Status: `RESEARCH COMPLETE / ON-DEMAND ENCLOSURE SELECTED`.

## Problem with the cheap bound

R7 bounds `||H^{-1}||` indirectly through recursive absolute row sums of a
Cholesky inverse. The construction is safe but loses cancellation and then
squares the pessimism in the eigenvalue lower bound. On the 60-row corner
principal system it produces an error interval wider than a visibly positive
coefficient.

## Verified right-inverse enclosure

For the already frozen binary128 matrix `H`, compute approximate inverse
columns `Q[:,j]` with the same audited Cholesky factor. Form the residual

```text
R = I-HQ.
```

If an outward infinity-norm bound `rho>=||R||_inf` satisfies `rho<1`, the
Neumann series gives

```text
H^-1 = Q*(I-R)^-1,
||H^-1||_inf <= ||Q||_inf/(1-rho).
```

For the original solve residual `e=b-Hx`, this yields the componentwise-safe
uniform enclosure

```text
||x*-x||_inf <= ||H^-1||_inf * ||e||_inf.
```

This is an a posteriori verification of the existing solution, not a new
direction or regularization. It is closely related to standard verified
linear-system residual bounds and preserves the exact local NNQP.

## Hypotheses

| hypothesis | discriminator |
|---|---|
| V1: R7 failed only because of pessimistic conditioning | `rho<1`, row 53 becomes strictly positive under the refined bound, the unchanged active set continues and corner certifies |
| V2: the system is verifiably invertible but the sign remains unresolved | inverse enclosure passes, yet the refined interval still crosses zero |
| V3: the principal system itself cannot be verified | `rho>=1`, non-finite inverse audit, or audited inverse-column solve failure |

The inverse audit is expensive (`n` extra triangular solves), so it runs only
on a cheap-bound ambiguity and remains oracle-only.
