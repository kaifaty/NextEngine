# Continuum water W1 sealed pressure-solver research — 2026-08-18

Status: `ACCELERATED_PROJECTED_GRADIENT_SELECTED_FOR_W0H_RECLOSURE / NO_W1_CREDIT`.

## Problem and bounded hypotheses

The W0G-rooted Linux W1 run first reaches a real solver failure in
`CW-SEALED-001` step 2. The frozen projected active-set PCG stops after its
50th pressure-operator application at `194,545 ppb`, above the unchanged
`100,000 ppb` mean positive-compression gate. Step 1 passes at iteration 46.
No later W1 scenario was run from that failed lineage.

The bounded research cycle tested four competing explanations:

1. the 50-iteration ceiling is merely too small;
2. the global active-set restart destroys the conjugate-gradient recurrence;
3. a standard or fast-expansion MPRGP active-set solver can retain the same QP
   and close within 50 operator applications;
4. the same non-negative QP needs an acceleration method without global
   active-set restarts, or ultimately a multilevel preconditioner.

All tests start cold from the accepted canonical frame. They retain the W0F
pressure operator, right-hand side, non-negative multiplier, contact order,
canonical publication, 50-application production ceiling and W0G energy
semantics. Diagnostic execution roots are domain-separated and receive no W1
credit.

## Frozen PCG diagnosis

A two-step control with a diagnostic ceiling of 200 accepts sealed steps 1 and
2 at iterations `46` and `96`, with compression residuals `98,467` and
`97,816 ppb`. The corresponding mean projected-KKT residuals are still
`118,352` and `116,271 ppb` at the inherited compression-only acceptance
points.

The exact trace shows a projection restart and an active-set restart on every
iteration after the first. One changed row therefore clears the conjugate
direction for all 48,000 rows. The implementation named PCG degenerates to a
diagonally preconditioned steepest-descent sequence at product scale. Raising
the ceiling to 100 would preserve this bad scaling and is rejected.

## Active-set alternatives

The same QP was transformed by the positive diagonal change of coordinates
`k = S y`, where `S_i = sqrt(alpha_i / dt^2)`. Standard MPRGP with `Gamma=1`
and fixed expansion step `0.5` performs zero feasible CG steps in the first 50
iterations: 44 expansions and 6 proportioning steps. It ends sealed step 1 at
`154,641 ppb`.

The projected-CG expansion variant MPPCG also performs zero feasible CG steps
and ends at `161,927 ppb`. It improves the active-set expansion mechanism in
the cited algorithm family but does not close this matrix/profile boundary.
Both variants are rejected for W0H.

## Selected algorithmic candidate

The surviving candidate applies Nesterov-style accelerated projected gradient
to the same diagonally scaled convex QP:

```text
k = S y
y >= 0
step = 0.25
momentum(iteration) = (iteration - 1) / (iteration + 2)
next = max(0, extrapolated - step * scaled_gradient)
```

Rows whose frozen DFSPH inverse-diagonal factor is zero remain fixed at zero;
their unscaled compression and KKT residuals still participate in acceptance.
This is required by dam-break step 245, where one dispersed row reaches the
existing denominator cutoff. Treating that row as an error was a coordinate-
transform defect, not a physical non-convergence.

Every accepted iteration must satisfy all of:

- one exact pressure-operator application;
- directional quadratic curvature `<= 1 / step = 4` for the actual projected
  displacement; there is no hidden backtracking;
- minimum 2 and maximum 50 iterations;
- mean positive compression `<= 100,000 ppb`;
- mean projected-KKT residual `<= 100,000 ppb`.

The initial `0.5` candidate passed sealed steps 1/2 in `20/28` iterations but
failed its own majorization guard in hydro step 6 at curvature bits
`0x4000e195d40ccf30` (approximately `2.11`, above `1/0.5`). It is rejected.
The fixed `0.25` step passes the complete internal discriminator without
dynamic step selection.

## Complete internal discriminator

| Scenario | Steps | Maximum density iteration/error | Other blocking observations | Trajectory result |
| --- | ---: | ---: | --- | --- |
| Hydro | 1,200 | `37 / 99,178 ppb` | penetration `0`; energy drift `3,377,402 ppb`; momentum `26 ppb` | repeat exact; reference pending |
| Free fall | 96 | `2 / 0 ppb` | exact canonical recurrence; momentum `0` | pass |
| Dam break | 720 | `26 / 96,534 ppb` | excess `0`; deficit `633,740,030 ppb`; momentum `29 ppb` | repeat exact; reference pending |
| Still tank | 7,200 twice | `24 / 99,403 ppb` | penetration `0`; energy drift `4,558,215 ppb`; momentum `13 ppb` | both roots `504c0473682a3b1fe9dcf051ea6b506d6524ea842f822e1ed90532a80d47f3ce` |
| Orifice | 720 | `34 / 99,085 ppb` | penetration `0`; excess `0`; momentum `22 ppb` | repeat exact; reference pending |
| Sealed 48k | 480 twice | `40 / 91,168 ppb` | penetration `0`; excess `0`; momentum `9 ppb` | both roots `48956184c770c9db9943b1d49862336202b7bb8e32fa6604cb5099f29df06f45` |
| Storage order | 240 × 3 orders | `20 / 86,950 ppb` | penetration `0`; energy drift `3,821,035 ppb`; momentum `27 ppb` | identity/reverse/affine root `6667d9534e12d59d8f3d6682b3c30f94d1d9e8f1cc66f212b52c5b304727e992` |

Production and a separately written support-complete hydro first-step
calculator match exactly on accepted iteration, compression residual,
projected-KKT residual and maximum multiplier. The independent path does not
call the production accelerated loop or scaled operator wrapper.

External hydro/dam-break/orifice references remain missing. These runs prove
only that the candidate survives W0H reclosure inputs; they do not complete W1
or run any `CONTINUUM-*` ProductCheck.

## Evidence artifacts

Artifacts remain outside Git. SHA-256 binds the exact diagnostic JSON bytes:

| Evidence | SHA-256 |
| --- | --- |
| PCG-200/KKT two-step control | `7403d157a5898677c383614b0d22405ca87c436e19902873e83ee03f510dab16` |
| Standard MPRGP trace | `b528493cf4232213186a5158ec6b236122e046725a3d8f5303668d6103cecad6` |
| MPPCG trace | `2298613b3209ea5b01912d7dbc545e0f1498185a4c1c69321a3ce6055aa0cbb1` |
| Accelerated `0.5` sealed control | `b749cf3d0ea35ef691e22281dcd73aa09db5b4b6979b148a3f76fd340eedafe6` |
| Accelerated `0.25` hydro | `bc96f179296e53f91482046ebff3522c1f78ce4ab7e5401f5bad7233f5fc56d9` |
| Accelerated `0.25` free fall | `4aa52eecbb25c075f09e2493aa6731426cdf5077c1340577c1d6e4f589256689` |
| Accelerated `0.25` dam break | `c0b0c7e40f72ced5e3c7468df41b3f8baaea32931854c614d434a38aa833b11e` |
| Accelerated `0.25` still tank | `3de69a3d580be4c37a3bbfb692365e63871630a9213a22fcc8722f5b9c6cdf96` |
| Accelerated `0.25` orifice | `730dbc495b76952bc268af6cc053c96c27a69e7033591e857553033c7ab262cf` |
| Accelerated `0.25` sealed 48k | `89c77248642300f473568d7d15a199eef50d05a172bfb8bcbb4b81a098879f02` |
| Accelerated `0.25` storage order | `32153480447ce8c1d85464c66c51b155b41573188c02fe24d5c3f928ea5ac017` |

## External research and bounded claims

- Takahashi and Batty describe non-negative fluid pressure as an LCP or
  equivalent box-constrained convex QP and use MPRGP with an active-set-aware
  multilevel preconditioner: <https://doi.org/10.1145/3606939>.
- Their later primal-dual work explicitly identifies repeated active-set
  updates as a performance bound for MPRGP even with strong AMG:
  <https://cs.uwaterloo.ca/~c2batty/papers/Takahashi2024/Takahashi2024.pdf>.
- Kružík et al. show that standard MPRGP expansion may identify the active set
  too slowly and define projected-CG expansion; the local MPPCG negative result
  therefore tests published prior art rather than an invented restart tweak:
  <https://doi.org/10.1016/j.advengsoft.2020.102895>.
- Vollebregt identifies frequent active-set CG restarts as a convergence cost
  and proposes continuation via Polak–Ribière, but also supplies a general
  counterexample; BCCG is therefore not selected without the matrix-class proof
  its guarantee needs: <https://doi.org/10.1007/s10957-013-0499-x>.

## Decision and reconsideration condition

Select the fixed-step, diagonal-scaled accelerated projected-gradient solver
for a new W0H child profile. Keep the QP, geometry, support, capacity, contact,
canonical state and W0G energy semantics unchanged. Root the new algorithm,
zero-diagonal rule, majorization failure, compression/KKT gates and exact
iteration schedule before resuming W1.

If the independent check, W0H clean-repeat roots or subsequent full W1 corpus
fails, revert the candidate and move to an active-set-aware multilevel/AMG
preconditioner. Do not raise the iteration ceiling, reintroduce hidden warm
state, accept dynamic backtracking without a new operation budget, or retry the
rejected MPRGP/MPPCG variants.
