# NSR3-B4E2D7R19R29 linearized-feasibility operator research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

## Decision

Close the dimensionless constraint-Jacobian operator before attempting a
range/nullspace or discretization-floor classification.

For fluid position increment `dx = SPACING * v`, define

```text
A v = J_c dx = SPACING * J_c v
```

where each constraint is `c_i = rho_i / rho0 - 1`. Static support particles
remain fixed and only the `3N` fluid coordinates are variables. R29 is a
read-only derivative/operator proof. It does not solve a least-squares
problem, form a correction, mutate state or execute another outer update.

## Why this prerequisite is required

R28 disproves a stable plateau inferred from outer 10 alone: outer 11 restores
`4.4290%` primal improvement. A causal discriminator is still useful, but a
claim about the component of `c` outside `Range(J_c)` is valid only after both
forward and adjoint actions of that exact operator are closed.

The current normalized AL lineage also does not contain the separate box-KKT
contact projection. Therefore R29 must not claim to measure a contact tangent
cone. The relevant frozen partitions are instead:

- all constraint rows;
- violated rows where binary64-owned `c_i > 0`;
- next-PHR-active rows where binary64-owned `u_i + c_i > 0`;
- rows coupled to at least one fixed static-support bond versus interior rows.

## Operator and independent checks

Use exact R28 position/dual, the frozen static-support index and binary64-owned
joint pair membership. Implement the candidate pair-once operator:

```text
(A v)_i = SPACING * sum_j g_ij dot (v_i - v_j)

(A^T w)_i = SPACING * sum_rows w_row * dc_row/dx_i
```

with `v_j = 0` for static support. Validate it by all of:

1. a separately folded directed-row JVP over the same frozen topology;
2. a centered finite difference of constraints over the frozen pair list at
   dimensionless epsilon `2^-16`, using a deterministic probe with RMS 1;
3. the bilinear adjoint identity `<A v,w> = <v,A^T w>` with deterministic
   row weights;
4. a translation probe that is exactly zero on rows without static-support
   bonds and whose nonzero image is confined to boundary-coupled rows.

The pair-once/directed difference must satisfy a componentwise binary64
forward bound based on maximum row degree and absolute term sums. Centered
difference relative L2 error must be at most `1e-6`; adjoint relative error
must be at most `1e-12`. These tolerances are frozen before observation.

## Bound source

```text
state / receipt  851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7
                 b145ce1af6d3ffca167f2264b55618b0bfccc55c340ff1eee17206569a239af6
history          fd838a046b2629f5d5e4d5aa0ab87160078b2e4b748b98101ac2da00bfa3a37d
position         e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828
dual             cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d
used             12,2,26,1,333,856,58,34,824,32,2,34,0
primal bits      0x3e53c2bf74000000
stationarity     0x3d2e9d0a6841b669
```

## Interpretation and continuation

- operator closure PASS: R30 may research/freeze a matrix-free LSQR range
  projection over frozen violated rows;
- derivative, adjoint or topology failure: preserve R28 and repair the first
  exact operator boundary; do not run LSQR;
- row partition anomaly: report exact counts/localization and diagnose before
  any floor claim.

R30, if authorized later, must report the projected residual and normal
residual without a post hoc floor threshold. A nonlinear/refinement test is a
separate later experiment.

## Scope

One parent replay, one diagnostic workspace and at most eight read-only pair
passes. No solver HVP, trial, precision audit, outer update, grant, state
commit, refinement, penalty/cap/policy change, substep, macro, trajectory,
timing, runtime integration or production authority.

