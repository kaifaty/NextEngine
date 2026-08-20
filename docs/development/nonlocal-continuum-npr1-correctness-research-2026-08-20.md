# Nonlocal NPR1 correctness research — 2026-08-20

Status: `REPORT_ONLY / NPR1_CONTRACT_INPUT`

## Question

What evidence can falsify the selected h3/16 Nonlocal profile as a water
solver before any authority or runtime integration decision?

## Primary-source findings

The [Nonlocal paper](https://doi.org/10.1145/3799902.3811196) establishes the
unified energy formulation and demonstrates stable coupled behavior, but it
does not provide a machine-readable quantitative water corpus. Its dam-break
resolution study keeps `h=3dx`, uses qualitative behavior, and its main
viscosity/surface-tension comparisons are image-based. The paper's
convergence error uses iteration 200 as the reference solution; this is useful
diagnosis, not an independent physical residual.

The paper does provide one analytical macroscopic control: two-dimensional
Poiseuille flow compared with the parabolic steady-state profile. It also
shows rotating spheres losing kinetic energy as shear viscosity increases and
explicitly identifies angular-momentum dissipation as a limitation. It states
that SISSM has no line search and therefore no unconditional convergence
guarantee.

The existing Next Engine DFSPH work already owns a stronger water reference
package: independently generated, hard-contact
[SPlisHSPlasH](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/tree/eccce86155776f6ac52d5080b1f720a52bf29450)
hydro, dam-break and orifice outputs, exact analytical invariants and frozen
aggregate metrics. The files are present in the configured research store and
their hashes match the accepted
[W1 evidence](continuum-water-w1-hard-clearance-reference-reclosure-2026-08-18.md).

## Consequences

1. Reuse the exact external water curves and metric formulas. A solver-family
   change is not a reason to move physical thresholds after seeing results.
2. Do not compare particle identity or exact trajectories to DFSPH. Compare
   canonical aggregate front, height, transfer, mass, COM, penetration,
   momentum and energy observables.
3. Do not claim surface-tension material validity from a visual coalescence
   scene. The selected water profile has gamma=0; NPR1 requires only an
   independent force/energy term control for the dormant surface path.
4. Use analytical Poiseuille flow for a separate viscosity profile. Report
   the known shear/angular-momentum loss rather than hiding it.
5. Treat kappa=9196.875 and lambda=360 as one selected numerical water-profile
   hypothesis. NPR0 calibrated neither as a universal material constant.
6. Canonical publication must be implemented independently of the GPU path.
   Quantizing final f32 output cannot reconstruct a canonical history.

## Reused external artifacts

| Scenario | SHA-256 | Role |
| --- | --- | --- |
| hydro | `84ae867f5b336cd0bd51be6f29a6a2a1f27f702c424f1dbd0a8f735b9f4bb435` | aggregate density/COM/work reference |
| dam break | `853d965489a40082a024aeee5a19f98aef054417014af8556fa212687d88d12c` | front and height curves |
| orifice | `60e9b3538d621ef1a3f1ae77569640740df471fbe4e5ef8eaa813d3751930849` | chamber-transfer curve |

These references use the same MKS scale, 0.05 m spacing, 0.125 kg mass,
1/240 s cadence, hard analytical contact and scenario geometry. Their DFSPH
support rule differs from the Nonlocal h3 kernel, which is acceptable for an
aggregate independent comparison and forbids particle-level equality.

## Architecture gaps exposed

- The current f64 gather oracle has no exact micrometre publication boundary,
  frame/trajectory roots or decode-next-step rule.
- Its neighbor builder is quadratic and cannot execute the nominal corpus
  efficiently.
- The full static profile supports only an outer box. NPR1 orifice requires a
  rooted internal plane with a closed aperture plus plane/edge/corner swept
  contact.
- Fixed 16 iterations have tiny hydro evidence but no broad iteration-doubling
  stability evidence.
- The existing CUDA implementation is f32 and exact only against its retained
  f32 arithmetic. NPR2, not NPR1, must decide whether CPU, deterministic GPU
  or research-only authority is defensible.
- The water profile disables shear viscosity and surface tension. Enabling
  those terms later creates separate material profiles and cannot inherit the
  water-corpus conclusion.

## Recommended NPR1 decomposition

```text
NPR1-A canonical binary64 publication + roots
  -> NPR1-B independent term/convergence controls
      -> NPR1-C deterministic O(NK) f64 trajectory runner + geometry
          -> NPR1-D smoke analogues and reference attestation
              -> NPR1-E nominal water corpus + iteration doubling
                  -> NPR1-F Poiseuille and dormant surface-term controls
```

Long nominal runs are forbidden before the corresponding smoke geometry,
capacity, canonical repeat and reference preflights pass. Timing remains
diagnostic; NPR7 owns the integrated production budget.
