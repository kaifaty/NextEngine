# Nonlocal NSR3-B1S1 pressure-spectrum evidence -- 2026-08-20

Status: `PASS / SPECTRAL_POLICY_PREMISE_VALID / REPORT_ONLY`

## Outcome

The pressure-only finite-state Hessian has a deterministic, bounded maximum
eigenfrequency across the B1S matrix. Its stiffness scaling is exact, while
its nondimensional spectral amplification increases with compression
amplitude. This validates the premise for a later `dt*omega_max` policy.

B1S1 selects no trajectory target and grants no runtime authority.

## Independent dense oracle

The active normalized tetrahedron provides a `12x12` dense check:

| Observable | Result |
|---|---:|
| dense `lambda_max` | `95404.82463062560` |
| Lanczos `lambda_max` | `95404.82463062553` |
| relative difference | `7.63e-16` |
| symmetry error | `9.52e-17` |
| dense/HVP product error | `1.40e-16` |
| Lanczos iterations / calls | `12 / 12` |

The dense and matrix-free operators are therefore the same pressure curvature
to well below every frozen threshold.

## Six-state spectrum

| Spacing | `kappa` factor | Active | `lambda_max` | `omega_max` | Amplification | Ritz residual |
|---:|---:|---:|---:|---:|---:|---:|
| `0.99` | `0.25` | 81 | `60795.7` | `697.399` | `0.7041203` | `3.02e-15` |
| `0.99` | `1` | 81 | `243182.7` | `1394.798` | `0.7041203` | `4.64e-10` |
| `0.99` | `4` | 81 | `972730.9` | `2789.596` | `0.7041203` | `8.23e-13` |
| `0.98` | `0.25` | 117 | `74421.6` | `771.604` | `0.7790405` | `3.41e-12` |
| `0.98` | `1` | 117 | `297686.4` | `1543.208` | `0.7790405` | `1.08e-10` |
| `0.98` | `4` | 117 | `1190745.7` | `3086.416` | `0.7790405` | `6.51e-10` |

Every lattice uses 12,111 unique pairs and at most 122 neighbors. All runs use
the frozen 48 operator calls, maintain orthogonality below `2.45e-15`, and have
normalized translation-mode residual below `1.5e-17`.

At fixed amplitude, adjacent eigenvalues scale by exactly four within the
declared tolerance, eigenfrequencies by two, and amplification is invariant.
Mean amplification grows by about 10.6% from `0.70412` to `0.77904`. The active
count simultaneously grows from 81 to 117, exposing the finite-state effect
that the linear B1S policy omitted.

The uncompressed control has zero active centers, zero pressure HVP and zero
maximum eigenvalue.

## Implementation correction

The first pilot compared an absolute translation HVP residual with the
contract's normalized threshold, causing a stiffness-dependent false failure.
The one authorized implementation correction divides by
`max(|lambda_max|,1)*||translation||`. No operator, state, threshold or
eigenvalue changed.

## Cost and interpretation

A full estimate costs 48 pressure-HVP calls for 343 particles. That is cheap
for this report but not free in a frame loop; for comparison, the selected
nonlinear solves in B1S used only a few HVP calls per active step. A future
policy must either amortize/warm-start the spectral estimate or derive a
certified cheaper bound. B1S1 does not hide this cost.

## Repeatability and lineage

- B1S1 semantic result SHA-256:
  `5836107d2e9b90d3638ed9775bf966e0c0b2c88bcdf7ca90e99cd02b311651c0`;
- two byte-identical raw reports:
  `3b8dcccc49303e7fb27d1c4791352e9b03d8dc5ab201fec9dbb3d1d56225d64d`;
- B1S remains byte-identical FAIL at
  `4d57744c4c7ff393d156d2c86510570783bd41982b421a00a36e3d6e8c6ff62c`.

## Decision

Select `SPECTRAL_POLICY_PREMISE_VALID`. Freeze B1S2 to test a
`dt*omega_max` policy against the unchanged six trajectory cases, with
predeclared cost and accuracy gates. B1R/B2, CUDA and runtime remain blocked.

