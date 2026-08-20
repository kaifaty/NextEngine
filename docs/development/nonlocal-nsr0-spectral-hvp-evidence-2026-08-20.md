# Nonlocal NSR0 spectral/HVP evidence -- 2026-08-20

Status: `PASS / NSR_HVP_CANDIDATE / NSR1_AUTHORIZED / REPORT_ONLY`

## Outcome

The exact analytic matrix-free Hessian-vector product passes every frozen NSR0
control against both the verified FCR2 gradient and a dense tiny matrix. This
closes the curvature-transcription gate and materially changes the diagnosis:
the stopped local solvers omitted large global coupling, and the pressure
objective can be locally indefinite even after the positive inertial term is
included.

| Case | HVP FD error | Off-particle Hessian norm | Eigenvalue range | Negative modes | Positive condition estimate |
|---|---:|---:|---:|---:|---:|
| compressed pair | `7.05e-10` | `68.96%` | `-32586.27 .. 160239.56` | `2` | `22.26` |
| combined tetrahedron | `1.18e-10` | `51.45%` | `7200.00 .. 187930.33` | `0` | `26.10` |
| repulsive surface pair | `4.82e-11` | `8.40%` | `6637.50 .. 9200.00` | `0` | `1.39` |
| attractive surface pair | `1.98e-11` | `4.61%` | `7200.00 .. 7950.00` | `0` | `1.10` |

Maximum dense symmetry error is `3.97e-17`; maximum analytic-HVP/dense-product
error is `2.53e-16`. Every finite-difference probe retains its kernel, surface
and pressure-active branches. The report is byte-identical across two runs.

The compressed pair's negative eigenvectors are transverse geometric modes;
the radial pressure direction remains strongly positive. Therefore a method
that clamps all negative curvature may discard real local structure, while a
plain Newton linear solve is unsafe. This is direct evidence for a
negative-curvature-aware trust-region discriminator.

## Harness correction before accepted execution

The first run stopped with `NSR0_BRANCH_CROSSING:compressed_pair`: the inherited
`0.05 m` pair distance lay exactly on the surface spline knot even though the
pressure-only control had `gamma=0`. Its HVP error already passed, but it
violated the frozen no-branch-crossing precondition. The control separation was
set to `0.045 m`, with its rest density recomputed to preserve the exact `1.1`
density ratio. No formula, coefficient, threshold or output gate changed.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| accepted raw report, run 1 | `ab578edb362f849e21535c707109cd9187e9378058a8c0997ddf00f9e34ddba8` |
| accepted raw report, run 2 | `ab578edb362f849e21535c707109cd9187e9378058a8c0997ddf00f9e34ddba8` |
| semantic result | `ac715f59475b601d16af5bbb701ec5d4bc9d391f771743b1a392f5ea96ac143f` |

Frozen non-regression hashes after NSR0:

- FCR0: `996eff3d61126491a3c1c92b6147d1c1f3eec0487d4b9dee45caadc6588b4345`;
- FCR1: `ead18de38f7e5fa68602c99f69cd891d11034502fe935fd473978040f2672140`;
- FCR2: `10b98cb4d29935062d25a649994112c09598ece552023d88b22110340a92a2a8`;
- expected FCR3-B2 failure:
  `95c51f978953a784bdd9a7895fa825bd22c282253fd70403ade726393597cd06`.

## Decision

Select `NSR_HVP_CANDIDATE` and authorize NSR1. This is an algebraic/numerical
oracle only. It does not prove that Newton--CG is fast, physically valid at
scale or production-ready.

