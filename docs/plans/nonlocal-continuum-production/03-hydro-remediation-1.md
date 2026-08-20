# NPR0-R1 — hydrostatic remediation

Status: EXECUTED / H3_SUPPORT_REMEDIATION_CANDIDATE / NPR1_BLOCKED

## First failure

The frozen NPR0-E corpus rejects both profiles only at TPH-1 mean positive
compression. The derived profile reaches 0.0078648680 against 0.0001 after
four iterations. Free fall, rigid translation reversal and static face/corner
contact pass.

This remediation separates nonlinear convergence, product support ratio and
the physical interpretation of kappa. It does not relax the gate or fit a
coefficient to the failed result.

## Primary-source interpretation

Equation 7 of the
[Nonlocal paper](https://peridynamics.com/publications/2026-Liu-NUV.pdf)
uses kappa times the squared dimensionless density-ratio error as a
per-particle bulk potential. The paper calls kappa the strength of the
incompressibility constraint and reports fixed values rather than a material
unit calibration. It also states that the current SISSM path has no line
search and no unconditional convergence guarantee.

The previous exact scale-law test used length ratio s=10 and time ratio
t=25/6. That algebraic similarity also requires gravity to change by s/t².
The product profile correctly retains 9.81 m/s², so using the cadence ratio as
a physical material-time ratio is invalid.

## Independent hydro-head hypothesis

Engineering inference from Equation 7: if particle volume is V, then kappa/V
acts as an effective bulk-modulus scale. The product particle volume is
0.000125 m³. Requiring relative compression epsilon at most 1e-4 under the
maximum initial product water head H=0.75 m gives the lower-bound estimate:

    pressure = rho g H
             = 1000 * 9.81 * 0.75
             = 7357.5 Pa

    effective bulk modulus >= pressure / epsilon
                           = 73,575,000 Pa

    kappa >= effective bulk modulus * V
          = 9196.875 J

This is a profile-level hydrostatic lower bound, not a claim that the discrete
solver exactly realizes K=kappa/V and not a calibration to the observed
failure. NPR1 must validate the mapping against external references.

Bulk viscosity remains lambda=360 during this discriminator so that only
kappa, iteration count and support ratio vary.

## Frozen diagnostic matrix

Run the unchanged TPH-1 geometry/contact/density metric for 24 substeps:

| Identity | kappa | h/dx | Iterations |
| --- | ---: | ---: | --- |
| algebraic-cadence-h2 | 576 | 2 | 4, 8, 16, 32, 50 |
| hydro-head-h2 | 9196.875 | 2 | 4, 8, 16, 32, 50 |
| hydro-head-h3 | 9196.875 | 3 | 4, 8, 16, 32, 50 |

The h2 rows use two rooted support layers; the h3 row uses three, so each
row covers its entire interaction horizon. In the tiny basin this means 272
and 624 boundary samples respectively. At full product scale, the exact h3
lattice complement would contain 38,856 boundary samples
(`86*26*46 - 80*20*40`) and exceed the current 32,768 boundary-sample
capacity. Therefore an h3 pass is evidence for a support-ratio redesign, not
a directly deployable profile.

Every row reports mean/maximum positive compression, maximum speed,
horizontal COM drift, penetration, fixed-support displacement and finite
status. The 1e-4 mean-positive-compression gate is unchanged.

No wall-clock value participates in the outcome.

## Disposition

1. If hydro-head-h2 passes within 50 iterations, issue one v4 physical
   coefficient candidate using the smallest passing count and rerun all NPR0-E
   cases. This matrix alone does not select the product profile.
2. If h2 fails but hydro-head-h3 passes, record a support-ratio remediation;
   do not silently amend SPEC-38.
3. If both physical rows fail, emit PROFILE_RECLOSURE_REMEDIATION_2. Under
   the NPR0 contract this is the final bounded remediation before a research
   stop or a newly authorized solver redesign.
4. The algebraic-cadence row is diagnosis only and cannot be selected.

Performance, adaptive stopping, line search and GPU timing remain blocked
until the full tiny corpus selects one physical profile.

## Execution result

The physical-head h2 row fails through 50 iterations, reaching
0.0003956033 mean positive compression. The h3 row first passes the unchanged
gate at 16 iterations with zero positive compression. The frozen disposition
is therefore `H3_SUPPORT_REMEDIATION_CANDIDATE`, not a selected product
profile. Exact results and capacity consequences are recorded in the
[dated evidence](../../development/nonlocal-continuum-npr0-hydro-remediation-evidence-2026-08-20.md).
