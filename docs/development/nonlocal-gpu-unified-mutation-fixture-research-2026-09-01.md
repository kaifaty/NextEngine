# NCGP15 pressure-mutation fixture research — 2026-09-01

## Question

After two coherent NCGP15 apparatus revisions, why do the omitted kernel
derivative `2/h` and finite-pressure-penalty controls still fail to admit the
physical `P/PV/PS/PVS` corpus, and what is the smallest non-post-hoc
discriminator?

## Frozen observations

- Revision-3 source commit: `b4fb0f1809985827ec7da57a1daa9e37b9288bf9`.
- Release binary SHA-256:
  `96c22e483f41012118590d9b28bd51b5e2edf4411a673d49058fcf8a65a6ecd8`.
- Raw stdout SHA-256:
  `d8f245fa7c7dd004d0b02f105af185fc6eedf10ab6ff4a23fa75797eacd9075e`.
- The exact TIGHT-128 pressure-only corrected CSR and independent all-pairs
  routes both stop at `64` outer updates. Their final active multiplier count
  is `96`, maximum/RMS positive density strain is
  `3.8860627938404769e-7 / 9.8278377932749008e-8`, and multiplier fixed-point
  residual is `4.0826452996374767e-4`, above the frozen `1e-8` gate.
- The missing-`2/h` route reaches `LINE_SEARCH_EXHAUSTED` after one or two
  accepted inner iterations depending on the exact fixture; the finite
  penalty route reaches a finite root/work-exact work ceiling. The mutation
  hooks therefore execute and materially change the solve.

The primary paper describes a unified nonlinear position optimization and a
semi-implicit successive-substitution solve; it does not justify treating an
unsolved corrected control as evidence against a mutation. The conventional
DFSPH reference likewise separates density-error pressure correction from
non-pressure prediction and supports retaining an independently converged
corrected control before attribution:

- [A Nonlocal Unified Variational Framework for Free Surface Flows](https://doi.org/10.1145/3799902.3811196)
- [Divergence-Free SPH for Incompressible and Viscous Fluids](https://discovery.ucl.ac.uk/id/eprint/10056699/1/BK17.pdf)

## Competing hypotheses and updates

| Hypothesis | Prediction | Observation | Update |
| --- | --- | --- | --- |
| mutation hook is dead or shadowed | corrected and mutated paths/results are identical | missing chain exhausts line search; penalty produces millimetre-scale state change and distinct roots | falsified |
| pressure is inactive in the fixture | corrected multiplier signature is empty | TIGHT-128 has 96 active multipliers | falsified |
| the state observable is insensitive because multiplier scaling absorbs `2/h` | corrected and missing-chain positions remain close while multipliers rescale | missing chain changes state by centimetres and exhausts line search | falsified for these executions |
| TIGHT-128 is an invalid admission fixture for the unified optimizer | corrected CSR and oracle stop at the same work cap before mutation attribution | both stop at outer update 64 with identical finite residuals | selected |
| a smaller active-density fixture can discriminate without changing solver policy | corrected CSR/oracle converge within frozen caps; mutations produce distinct typed non-commit | observed below | selected |

## Minimal counterfactual experiment

A temporary read-only research driver called the unchanged public candidate and
independent all-pairs APIs. It used zero gravity, no ghosts, unbounded contact,
the pressure-only mask and binary32-canonical positions on cubic lattices. No
coefficient, tolerance, optimizer rule or work cap changed.

The smallest healthy witness found is `3x3x3`, origin `(0.5,0.5,0.5) m`,
spacing `0.04 m`, stable IDs `100..126`, reference equal to position and zero
velocity:

| Route | Outcome | Outer / inner | Active | Density max | KKT max | Multiplier fixed-point |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| corrected CSR | `PASS` | `7 / 1540` | `1` | `1.7697964946848666e-11` | `3.6400948429865710e-10 m` | `7.7145865066803472e-10` |
| corrected all-pairs | `PASS` | `7 / 1540` | `1` | same | same | same |
| missing `2/h` | `LINE_SEARCH_EXHAUSTED` | `0 / 1` | `0` | typed partial | typed partial | typed partial |
| finite penalty | `SOLVER_WORK_CEILING_INCONCLUSIVE` | `64 / 313` | `0` | `2.2495747377797141e-2` | `2.7425703850471727e-2 m` | `27.585410222023744` |

Candidate/oracle corrected positions are byte-identical in the printed
long-double result. Maximum corrected/mutated position differences are
`0.030445466374070296 m` for missing `2/h` and
`0.0030197806118905586 m` for finite penalty. A `3x3x3` lattice at `0.045 m`
and `4x4x4` at `0.04 m` independently show the same route separation; they
are retained only as margin checks, not additional frozen fixtures.

## Conclusion and decision

Revision 3 is `APPARATUS_INCONCLUSIVE`, not a physics result. The TIGHT-128
pressure-only control is active but not converged under the unchanged unified
optimizer ceiling, so it cannot own mutation attribution. Freeze one
Revision-4 apparatus correction that replaces only the two pressure-mutation
fixtures with the binary32 `3x3x3` witness above. Retain all physical
`P/PV/PS/PVS` inputs, equations, coefficients, tolerances, optimizer rules and
work caps byte-for-byte.

Inside a mutation control, a finite root-closed work-exact typed non-commit is
expected mutation evidence after corrected CSR/oracle PASS even when the same
outcome would be apparatus-invalid on a production physics route. The enclosing
mutation receipt must not require the deliberately mutated child itself to set
the production `apparatus_valid` bit; it must require the declared finite
prefix, closed roots and exact work instead.

## Revision-4 transaction follow-up

The first integrated Revision-4 execution preserved the numerical research
result but exposed one adjacent gate mismatch. Both corrected routes report
internal `PASS` at `7 / 1540` with one active multiplier, yet their closed
transaction does not commit because the common
`manufactured_active_empty` gate intentionally applies to every
`UNBOUNDED_MANUFACTURED` term fixture. This is an apparatus failure, not a
pressure result, and the gate must not be weakened.

A bounded follow-up retained the same 27 records, zero gravity, zero ghosts,
pressure-only mask and all solver settings, but placed the cube strictly inside
the existing analytical box at binary32 coordinates
`0.055 + 0.045*index`. Corrected CSR/all-pairs both pass in `14 / 204` with
one active multiplier; missing `2/h` line-search-exhausts after one inner
iteration; finite penalty reaches the outer cap after 41 accepted inner
iterations with maximum density strain `6.3342642397639207e-3`, KKT maximum
`1.3469598262338257e-3 m` and multiplier fixed-point residual
`7.7673915240105077`. No contact occurs.

Revision 5 therefore changes only the two mutation-fixture coordinates and
boundary tag. The generic manufactured empty-pressure gate, physical
TIGHT-128 corpus, equations, tolerances and work ceilings remain unchanged.

## Smallest next action

Freeze Revision 5 in the existing NCGP15 contract, delete the temporary
research driver, implement the analytical-box fixture and conditional mutation
admission, then run the focused Phase-A route. Only a fully admitted Phase A
authorizes the unchanged Phase B/C execution. CUDA and performance remain
`NOT_RUN`.
