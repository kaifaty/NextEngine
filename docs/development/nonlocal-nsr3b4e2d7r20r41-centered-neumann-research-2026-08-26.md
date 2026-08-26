# NSR3-B4E2D7R20R41 centered Neumann research

Status: `RESEARCH COMPLETE / CENTERED FIXED-POINT ENCLOSURE SELECTED`.

## Question

Can a report-only refined center resolve the six torsion passive signs, or is
the exact passive solution itself too close to the NNQP face?

## Why R40 is insufficient

R40 proves `C=I-XA` contractive with `||C||inf<=0.131768`, but encloses the
exact error `e` around zero:

```text
e = z + C*e,       z = X*(b-A*x).
```

Because `||z||inf` is `2.499e14`, every component receives the same
`2.878e14` radius even though the fixed point may have a precise, nonzero
center. The bound is mathematically valid; its center is the remaining loss.

## Centered certificate

Choose any represented shadow correction `y` and define

```text
g = z + C*y - y,
h = e - y.
```

Then `h=g+C*h`, hence

```text
||h||inf <= ||g||inf / (1-||C||inf).
```

Generate `y` only as a deterministic numerical accelerator using
`y_(k+1)=z_hat+C_hat*y_k`, `y_0=0`. Its generation need not be trusted: at
fixed depths `1,2,4,8,16,32`, independently recompute `g` with Dot2, explicit
input bounds and an exact dyadic oracle. The sign interval is centered at the
error-free sum of the original represented solution and `y`.

This is iterative refinement as an offline certificate, not a solver-state
update. The fixed depth 32 and all checkpoints are frozen before observing any
centered signs; there is no tolerance stop or fitted iteration count.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| N1 | the zero-centered enclosure is the only remaining loss | all 65 final centered intervals exclude zero |
| N2 | the represented inverse is contractive but the exact passive solution is face-near | at least one final interval still crosses zero |
| N3 | a centered correction contradicts an already certified R39 sign | any of the 59 R39-resolved signs changes |
| N4 | composed arithmetic is invalid | parent, underflow or exact-containment rejection |

N1 authorizes only a later opt-in refined-passive-solution trajectory. N2
selects componentwise interval Gauss--Seidel or higher-precision/direct
principal solve research. N3 rejects the apparatus because a valid tighter
enclosure cannot contradict an existing valid enclosure. R41 does not apply
`y`, change the passive set or continue torsion.
