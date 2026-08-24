# NSR3-B4E2D7R19R30 first diagnostic

Date: 2026-08-24  
Scope: private CPU shadow diagnostic; no timing and no state mutation

The first nominal R30 execution reached LSMR stop `COMPATIBLE` after 388
iterations. It reproduced all parent/source/operator roots and passed all four
dense controls, but returned hard `FAIL / ORTHOGONALITY`.

Key observations:

```text
||b||                  8.1139951163476689e-08
||q||                  5.0778391645710737e-16
||q|| / ||b||          6.2581245018751602e-09
||C^T q||              1.3912509730218889e-16
raw angle cosine       4.2756282469787546e-05
Pythagorean defect     5.3514799548257377e-13
```

The failure exposed a verification defect rather than a solver or physics
failure. The original implementation normalized `|p^Tq|` by `||p||||q||`.
For this compatible solve, `q` is already roundoff-sized, so that denominator
makes the angle of numerical noise a hard gate. The independently computed
Pythagorean identity shows the cross-energy is approximately
`2.68e-13 * ||b||^2`, safely inside the frozen `1e-10` gate.

Repair: use `|p^Tq| / max(||b||^2, tiny)` as the hard dimensionless
orthogonality control and retain the angle cosine as a report-only observation.
Do not change LSMR tolerances, iteration cap, scaling, row selection or the
`1e-10` gate. The exact first failure remains recorded here so this unstable
normalization is not reintroduced.
