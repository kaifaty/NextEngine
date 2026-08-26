# NSR3-B4E2D7R20R63E projector nullspace research

Status: `FROZEN FOR IMPLEMENTATION`.

## Question

R63D proves that the selected source rows are algebraically independent and
numerically full-rank, while their projected Gram block develops one numerical
near-null direction. Which part of the projector creates it?

The current derivative has two distinct nullspace mechanisms. With `Q` the
diagonal mask that retains free coordinates, `y_f=Qy`, and an active ball,

```text
J = 1/(1+eta) * (Q - y_f y_f^T / ||y_f||^2).
```

Therefore:

- clamped coordinates lie in `null(Q)`;
- the free radial vector `y_f` lies in the nullspace of the ball-tangent
  projector;
- the scalar `1/(1+eta)` changes magnitude but not mathematical rank.

For projection onto an ordinary closed Euclidean ball, the exact Fréchet
derivative outside the ball removes the radial component, and its derivative
applied to the radial direction is zero. Our box-plus-ball formula is the same
tangent operation after restricting to the current free coordinates.

Source: [Li, Strict Fréchet Differentiability of the Metric Projection Operator in Hilbert Spaces, Theorem 3.3 / ball derivative](https://arxiv.org/pdf/2312.14362).

Generalized derivatives of convex projection are self-adjoint, positive
semidefinite and nonexpansive. That supports the range/nullspace energy
decomposition, but does not guarantee that a dual Gram matrix formed around
the projector is nonsingular.

Source: [Bello Cruz, Ferreira and Prudente, A semi-smooth Newton method for general projection equations, Theorem 2.1](https://optimization-online.org/wp-content/uploads/2024/01/genproj.pdf).

## Why this precedes QR or wider precision

If clamping creates the weak direction, the next architecture question is
active-set/dual-representative selection. If the active ball tangent creates
it, the natural formulation is instead in the projector range or tangent
space. A generic QR can reveal a small singular direction, but cannot tell
which physical constraint geometry owns it.

A recent result for degenerate *polyhedral* projection is instructive: rather
than blindly regularizing singular generalized Jacobians, Ding, Feng and Li
select an extreme point of a projection-equivalent lifted set where a
nonsingular generalized Jacobian is available. This does not directly solve
our nonpolyhedral active-ball case, but it is evidence that representative
geometry can be preferable to arbitrary regularization or row deletion.

Source: [Ding, Feng and Li, Semismooth Newton methods for degenerate polyhedral projection, 2026](https://arxiv.org/abs/2607.12551).

## Selected experiment

Capture the exact projector state at the same unique verified-inverse audit as
R63D. Reconstruct the selected 102 source rows `B` and three Gram blocks:

```text
G_Q = B Q B^T                       clamp-only
G_T = B (Q - y_f y_f^T/||y_f||^2) B^T   clamp + tangent
G_J = B J B^T                       full derivative
```

When the ball is inactive, `G_T=G_Q` and `G_J=G_Q`. The reconstructed `G_J`
must match the captured principal matrix bit-for-bit; otherwise the projector
state was captured at the wrong NNQP call and no geometric result is valid.

Apply the already frozen exact modular and normalized binary64/binary128 rank
profiles to each block. This localizes the first numerical rank change without
choosing a new threshold.

Then recover the deterministic R63D binary128 pivot witness `w` for the
normalized full block. If `D=sqrt(diag(G_J))`, the corresponding original row
combination is `alpha=D^-1 w`. Form

```text
v = B^T alpha
c = (I-Q)v
q = Qv
r = y_f (y_f^T q)/||y_f||^2
t = q-r
Jv = t/(1+eta).
```

The harness must close `v=c+r+t`, reproduce `Jv` through the production-form
JVP, reproduce `G_J alpha` through `B(Jv)`, and close the energy identity

```text
alpha^T G_J alpha = ||t||^2/(1+eta).
```

It reports clamped, radial and retained-tangent energies. These ratios explain
the observed witness but do not define a solver tolerance.

## Interpretation routes

After correspondence and exact-full modular gates:

- normalized `G_Q` already rank 101 in binary128:
  `CLAMP_METRIC_NUMERICAL_RANK_LOSS`;
- `G_Q` rank 102 but active-ball `G_T` rank 101:
  `BALL_TANGENT_NUMERICAL_RANK_LOSS`;
- `G_Q` and `G_T` rank 102 but full scaled `G_J` rank 101:
  `PROJECTOR_SCALE_ARITHMETIC_BOUNDARY`;
- reconstructed `G_J` does not reproduce the inherited 101 profile:
  apparatus failure, not a new physical route.

Binary64 ranks and energy dominance are corroboration. Binary128 is the route
selector because R63D already showed the same split in that profile; neither is
an exact/physical rank declaration.

## Candidate architecture after the result

- Clamp route: research projection-equivalent dual representatives and scaled
  RRQR of the active source operator, preserving the primal projection.
- Ball route: research a matrix-free range/tangent solve using `B T` (or an
  augmented primal-dual KKT system) rather than explicit inversion of the
  normal Gram block.
- Scale-only route: verify the normalized tangent identity and wider residual
  arithmetic before changing formulation.

These are research branches, not preselected implementations.

## Ceiling

R63E observes one immutable projector state and one near-null witness. It does
not choose rank 101, remove/reweight a row, solve the NNQP RHS, change the trust
ball, apply a center, run a trajectory or authorize runtime/GPU/production
work. R64 and R65 remain blocked.
