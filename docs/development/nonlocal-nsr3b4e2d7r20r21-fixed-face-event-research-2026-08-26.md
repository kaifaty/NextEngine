# NSR3-B4E2D7R20R21 fixed-face projector-event research

Status: `RESEARCH COMPLETE / ANALYTIC EVENT PREDICTOR SELECTED`.

## Derivation

Along one multiplier line the projector input is affine:

```text
z(alpha) = z0 + alpha dz.
```

On a fixed box face with clamped components `C`, free components `F` and an
active trust ball, the joint projector has

```text
y_C = b_C
y_F = R z_F / ||z_F||
R^2 = radius^2 - ||b_C||^2.
```

A component meets a lower or upper bound `b` when

```text
R^2 z_i(alpha)^2 - b^2 ||z_F(alpha)||^2 = 0,
```

plus the unsquared sign and face-side conditions. Since `z(alpha)` is affine,
this is a scalar quadratic. With an inactive ball the corresponding event is
the linear root `z_i(alpha)=b`. Fixed coordinates are excluded.

## Question and hypotheses

Can this fixed-face KKT equation identify the single R20 transition without a
64-point scan?

| ID | hypothesis | prediction |
|---|---|---|
| E1 | the fixed-face event equation owns the observed boundary | exactly one admissible event root lies in every R20 transition cell and its scalar is 168/189 as observed |
| E2 | trust-ball coupling makes the local event ambiguous | multiple admissible scalar roots occupy a transition cell or the selected root predicts the wrong scalar |
| E3 | finite root evaluation is insufficient | the polynomial root cannot be bracketed by the two exact R20 cell endpoints with the required sign/face checks |

## Selected discriminator

Replay R20 exactly. At each of its seven pre-step states, derive `z0`, `dz`,
`F`, `C` and `R` from the unchanged projector and affine multiplier direction.
Enumerate lower/upper event polynomials for every non-fixed scalar. Retain only
positive roots inside the frozen accepted/double bracket that satisfy the
unsquared sign, face-side and exact endpoint mask checks.

The selected event must be unique, name the observed changed scalar and fall
inside the exact R20 transition cell. Report coefficients, roots, polynomial
endpoint signs, ball/free/clamped dimensions and all rejected candidate
counts. Linear, quadratic-degenerate and zero-bound controls are mandatory.

This is formula validation only. It cannot generate or apply a solver line
trial, choose a post-boundary offset, change globalization/cap/tolerance, claim
continuous interval certification, or authorize runtime/GPU/production work.

