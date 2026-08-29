# Nonlocal FCR3-B2 pressure acceleration research — 2026-08-20

Status: `DECISION / PRESSURE_ACTIVATED_CHEBYSHEV / REPORT_ONLY`

## Finding

The FCR3-B v1 pressure update is not a mistranscription. Enlarged visual
inspection of Nonlocal Eq. 20 and Eq. 26 confirms the same split as SISPH
Eq. 13/14 and the released CUDA implementation:

```text
a_ij = kappa*dt^2/rho0 * w(r_ij)/r_ij <= 0
A+_ij = -a_ij                         >= 0
A-_ij = J_i*a_ij                      <= 0
```

The conservative update sums the `ij` and `ji` contributions. The FCR3-B1
pressure masks accept full steps and monotonically reduce the exact
objective, but remain above the gradient gate after 80 iterations. The first
problem is therefore fixed-point convergence rate, not the pressure energy,
pair reaction or Armijo rejection.

## Candidate

SISPH explicitly describes SISSM as Jacobi-like with normally linear
convergence and gives a reduction-free Chebyshev recurrence:

```text
x_(k+1) = omega_(k+1) * (f_k - x_(k-1)) + x_(k-1)
omega_1 = 1
omega_2 = 2 / (2 - spectral_radius^2)
omega_(k+1) = 4 / (4 - spectral_radius^2 * omega_k)
```

The paper reports `spectral_radius=0.9` as converging comparably to Anderson
acceleration while avoiding dot products and global reductions. FCR3-B2
adopts that published value once; it does not sweep or estimate it from the
test corpus.

The Nonlocal paper does not include a global line search and explicitly
acknowledges the resulting lack of unconditional convergence. Our candidate
therefore retains the exact FCR2 objective and Armijo acceptance as the
correctness boundary. Chebyshev proposes a direction; it never becomes the
energy oracle.

## Decision

Run one pressure-activated Chebyshev candidate under the frozen
[FCR3-B2 contract](../plans/nonlocal-continuum-formula-reclosure/03b2-pressure-chebyshev-contract.md).
When `kappa=0`, the path remains byte-for-byte equivalent at the report level
to corrected SISSM v1. No pressure coefficient, tolerance, geometry,
iteration budget or material parameter changes.

Passing closes only the pressure-quality defect and authorizes a distinct
cost-localization stage: the v1 repulsive-surface case alone already exceeds
the original aggregate evaluation budget, so pressure-only work cannot make
that performance gate pass. Failure closes the current fast-SISSM branch.
Neither outcome authorizes a spectral-radius sweep, Anderson acceleration, a
higher iteration cap or profile/CUDA work.

## Primary sources

- Nonlocal Unified Variational Framework, DOI `10.1145/3799902.3811196`,
  Eq. 20 and Eq. 26.
- Semi-Implicit SPH, DOI `10.1111/cgf.70043`, Eq. 13/14 and Eq. 22/23.
- released PeriDyno `SemiImplicitDensitySolver.cu`, including the conservative
  pressure accumulation and Chebyshev blend kernel.
