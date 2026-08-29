# Nonlocal formula reclosure research — 2026-08-20

Status: `DECISION / NEW_IDENTITY_REQUIRED / REPORT_ONLY`

## Outcome

The stopped source-shaped lineage must not be repaired in place, but a
smaller corrected research lineage is justified. Two discrepancies are
confirmed and one earlier diagnosis is narrowed:

1. The reference cubic implementation returns `dW/dq` while the paper and
   standard SPH convention require `dW/dr`. For `q=2r/h`, the missing chain
   factor is exactly `2/h`.
2. The reference viscosity path dispatches a zeroth-order kernel value `W`.
   The published incremental potential instead uses
   `omega=-dW/dr`; normal and tangential components also have distinct
   coefficients.
3. The earlier claim that surface force necessarily misses `1/r0` is not
   established. A dimensionally explicit definition
   `C(r)=r0*C_hat(r/r0)` gives `dC/dr=c(r/r0)` without that factor. What
   remains real is a parameter-unit ambiguity: Eq. 14 yields a position
   coefficient containing `gamma*m`, whereas the reference code exposes an
   untyped `strength` and uses it directly.

The new identity therefore freezes its own physical-distance surface
potential, mass convention and pair enumeration. It inherits no old
coefficients or timings as correctness evidence.

## Primary-source findings

### Kernel derivative

The Nonlocal paper defines `w(r)` as the first derivative of the SPH kernel.
The official SPlisHSPlasH cubic implementation differentiates its normalized
radius and explicitly divides the radial gradient by the support radius. The
pinned PeriDyno cubic implementation uses `q=2r/h` but omits `2/h` in its
`gradient` return. Its value has kernel, rather than kernel-gradient,
dimensions.

### Compression semantics

The literal printed Nonlocal Eq. 7 is quadratic in `rho/rho0-1`. The authors'
SISPH predecessor and both released solver paths clamp density with
`rho=max(rho,rho0)` before forming the update. The SISPH paper explicitly
connects this to clamping negative pressure to zero to prevent artificial
particle clumping. The formula-reclosure default is therefore the
compression-only potential `max(rho/rho0-1,0)^2`; the literal two-sided
variant remains a named FCR1 discriminator.

### Viscosity and directed pairs

Eq. 10 is a directed double sum. After multiplying dissipation by `dt` and
combining the two directions of one pair, the normal gradient coefficient is
`lambda` while the tangential coefficient is `2*mu`. If a directed-edge GPU
visit also writes the equal/opposite endpoint contribution, each edge must
use half of that undirected total: `lambda/2` for normal and `mu` for
tangent, matching Eq. 13. The released path visits neighbor edges and writes
both endpoints, so FCR1 must prove whether its actual graph contains one or
two directed visits before comparing coefficients.

### Surface normalization

The Nonlocal paper declares a physical-distance potential `C(r)` and force
spline `c(r)=dC/dr`, but publishes no closed form for `C`. Released code
contains a dimensionless primitive in `q=r/r0` and uses its derivative as the
force spline. Both of the following are mathematically possible:

```text
C(r) = C_hat(r/r0)       -> dC/dr = c(q)/r0
C(r) = r0*C_hat(r/r0)    -> dC/dr = c(q)
```

Only the second matches the force equation as printed and is adopted by the
new contract. An additive constant is selected so the compact potential is
continuous and zero at `3*r0`. This is a new explicit model definition, not
an erratum claim about the authors' implementation.

## Solver research consequences

- SISSM's positive/negative split does not by itself prove convergence of
  the changing nonlinear system. Its per-iteration linearized coefficients
  must be refreshed, and the predecessor paper adds an explicit bounded step
  adjustment for overshoot. The Nonlocal paper acknowledges the absence of
  unconditional convergence without a global line search.
- FCR2 must report both objective/residual progress and the accepted step.
  A fixed iteration count alone is not a correctness gate.
- Performance work remains closed until the corrected CPU identity passes
  the physical corpus; the old CUDA numbers quantify only the retained
  source-shaped arithmetic.

## Sources and frozen artifacts

| Source | Identifier / SHA-256 | Use |
|---|---|---|
| Nonlocal SIGGRAPH 2026 paper | DOI `10.1145/3799902.3811196`; PDF `610047ef32e895026c3661c57999f14ae550d8e2719ba2fcd95742371ad031c1` | total energy, Eq. 7–15 and Eq. 23–26 |
| released PeriDyno source | commit `1aa892bb296fe766d2f9249c881b8605af23a69b`; `Kernel.h` `0bf941606d13c27ac18d56bf92e92756dabe9c3a0ef9cc651e8d5be61d800ec2`; solver `23b78d69690eaf765598d3ccfe923051e5d411d6e94feb1152a86513ba80c516` | observable reference arithmetic and pair updates |
| Semi-Implicit SPH, CGF 2025 | DOI `10.1111/cgf.70043`; PDF `56946fec8f2f7a72c23db1f254137aefb9b220ed0c9ef04398fab62de18f6c82` | compression clamp, bulk energy, SISSM update |
| Projective Peridynamics, TVCG 2023 | DOI `10.1109/TVCG.2023.3271511`; PDF `f6b5669428d02d004763a5bea213ed61f9ef3f701fab802e3218eea8b8d8f46d` | SISSM coefficient refresh and overshoot treatment |
| Akinci surface tension, TOG 2013 | DOI `10.1145/2508363.2508395`; PDF `e03474d7ae278d7609c8026d0ca3b022fc755df1ae21c38102701ded8a6a0b66` | pairwise force convention and compact spline precedent |
| SPlisHSPlasH cubic kernel | official repository file snapshot `b4cc0631c171b4d25a298ae320828058fa52445a7c5bf7f4517119727f9c3819` | independent `dW/dr` convention |

No public erratum or reviewed upstream change resolving these ambiguities was
found as of 2026-08-20.

## Decision

Proceed with [the separate formula-reclosure roadmap](../plans/nonlocal-continuum-formula-reclosure/README.md)
and its strict-f64 FCR0 oracle. Do not modify the stopped profile or infer
production readiness from an algebra pass.
