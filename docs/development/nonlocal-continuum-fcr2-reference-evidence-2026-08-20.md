# Nonlocal FCR2 variational-reference evidence — 2026-08-20

Status: `PASS / CPU_VARIATIONAL_REFERENCE / FCR3_AUTHORIZED / REPORT_ONLY`

## Outcome

The corrected `nuv-variational-fcr1` objective now has an executable strict
binary64 reference optimizer. Its full analytical gradient agrees with a
central directional derivative at `1.11310e-8` against the frozen `1e-7`
gate. All seven tiny cases are finite, monotonically lower the declared
objective when active, preserve internal momentum and produce the expected
physical direction. Two reports are byte-identical.

This closes a slow CPU correctness reference only. It does not validate
SISSM, product coefficients, basin physics, CUDA or production performance.

## Solver boundary

Commit `829f044eafe7a5d5c8acc6edb0ec25ae2c12d068` implements:

- prediction `y_star=x+dt*(v+dt*g)`;
- candidate-position density/surface graphs and reference-position viscosity
  graph;
- the full FCR0 inertia, compression-only, viscosity and surface objective;
- its independently accumulated full gradient under the FCR1 unique-pair
  convention;
- inertial-diagonal preconditioned gradient descent;
- deterministic Armijo backtracking (`c=1e-4`, at most `40` halvings per
  iteration).

It is deliberately a reference optimizer, not a fast solver proposal.

## Tiny corpus

| Case | Initial → final objective | Initial → final gradient norm | Observable result | Result |
|---|---:|---:|---|---|
| isolated free fall | `0 → 0` | `0 → 0` | max ballistic error `5.44e-15` | PASS |
| compressed pair | `5.0000 → 0.200022` | `1333.40 → 5.86e-6` | density excess `0.1 → 0.004114`; separation `+0.010316 m` | PASS |
| normal viscosity | `0.0855813 → 0.0508006` | `29.0472 → 8.08e-8` | relative speed `2 → 1.18719` | PASS |
| shear viscosity | `0.171163 → 0.0722418` | `58.0945 → 7.48e-11` | relative speed `2 → 0.844131` | PASS |
| surface repulsion | `-2.025 → -2.03870` | `15.9099 → 1.76e-8` | separation `+0.00242961 m` | PASS |
| surface attraction | `-1.49635 → -1.59660` | `40.2167 → 8.65e-11` | separation `-0.007 m` | PASS |
| combined tetrahedron | `4.46616 → 0.756793` | `1155.83 → 3.31e-6` | all three terms active | PASS |

Maximum center-of-mass error is `1.10e-15`; maximum final normalized internal
momentum residual is `4.52e-16`.

## Conditioning finding

The reference passes but exposes severe stiffness:

| Case | Iterations | Backtracks | Minimum accepted alpha |
|---|---:|---:|---:|
| compressed pair | `80` | `1735` | `1.86265e-9` |
| combined tetrahedron | `80` | `815` | `4.65661e-10` |
| surface repulsion | `80` | `1238` | `9.53674e-7` |

This is not a formula failure; Armijo still accepts monotonically decreasing
steps and the gradient drops by many orders of magnitude. It is decisive
architecture evidence that FCR3 must treat nonlinear conditioning,
preconditioning and accepted-step behavior as first-class gates. A profile
sweep without solver/reference correspondence would be premature.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| FCR2 raw report, run 1 | `10b98cb4d29935062d25a649994112c09598ece552023d88b22110340a92a2a8` |
| FCR2 raw report, run 2 | `10b98cb4d29935062d25a649994112c09598ece552023d88b22110340a92a2a8` |
| result root in report | `35e282ba8f7f2be60862b76412d6244004a7382cec005ea7fa2510b80a7d25ef` |
| executable | `04828cd9a56f364e60c63a565f57eedac22bfc08ec799132ba6889155713a1f3` |
| implementation `.cpp` | `de1c2319866a0a9def2a80fe01699b5d640343540a063776ffdb04ba3b3dda1c` |
| unchanged FCR0 report | `996eff3d61126491a3c1c92b6147d1c1f3eec0487d4b9dee45caadc6588b4345` |
| unchanged FCR1 report | `ead18de38f7e5fa68602c99f69cd891d11034502fe935fd473978040f2672140` |
| unchanged NPR1-A report | `464a55bc33741f18ddc4b3c1cda6b46b5248bb653789a5e31585482f871ee207` |
| unchanged NPR1-B failing report | `e8ed6950e33fb9388887c40e304c74b517761698c0c7accbaff9a99899bf2c3a` |

## Decision

Select the f64 variational reference and proceed to FCR3. FCR3 must first
close corrected SISSM/reference correspondence and conditioning before
selecting a product-scale coefficient/support profile.
