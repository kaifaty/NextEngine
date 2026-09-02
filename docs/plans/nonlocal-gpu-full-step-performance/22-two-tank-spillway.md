# Two-tank spillway visual lane — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NGQ8` |
| Status | `RUN / G1 G2 G4 PASS / G3 FAIL 0.611 / NO_RETUNE` (evidence: `docs/development/nonlocal-gpu-two-tank-spillway-evidence-2026-09-02.md`) |
| Parent | D-048 density-only boundary support; ADR-100 presentation-only water |
| Purpose | first internal-geometry scene for the presentation candidate: an upper tank drains through a wall opening into a lower tank |

## Scene (frozen)

Outer box `5 x 2 x 1.5 m` (`100 x 40 x 30` cells at `50 mm`), `+Y` up.

- Shelf: solid block `x 0..2, y 0..1, z 0..1.5`; the upper tank floor is
  its top at `y = 1.0`.
- Divider: solid slab `x 2.0..2.2, y 0..2, z 0..1.5` (four cells thick so
  the two-layer density complement of each side stays inside the wall).
- Opening ("pipe"): the divider minus `y 1.0..1.3, z 0.5..1.0`
  (`0.3 x 0.5 m`, `0.15 m^2`), flush with the shelf floor.
- Fluid: the accepted visual lattice `40 x 10 x 30` (`12,000` samples)
  in the upper tank, `0.1 m` above the shelf, released at rest. The lower
  tank starts empty.
- Contact: the accepted analytic outer-box clamp, then the spill clamp:
  inside the wall slab a sample is pushed to the nearer wall face unless
  its `(y, z)` lies in the opening, where it is clamped into the opening
  window; a sample left of the wall below `shelf_top + radius` is lifted to
  the shelf. Positional only, like the box clamp.
- Density support: `--boundary-layers` layers on the outer box without a
  lid, plus fixed density-only samples filling the top two shelf layers
  and the whole divider minus the opening. Profile, iterations, kappa,
  lambda, horizon and capacity are unchanged (`cap160.v6`).

## Frozen gates (960 steps, surface every 4, one cycle)

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 finite state | every audited frame finite, degree within capacity, pair capacity respected | pass |
| G2 no solid penetration | maximum over emitted frames of fluid penetration into the shelf or the divider (outside the opening), measured after contact | `<= 1e-4 m` |
| G3 drainage | fluid samples with `x < 2.0` at step 960 divided by the initial count | `<= 0.6` |
| G4 arrival | at least one fluid sample with `x > 2.2` and `y < 0.2` by step 480 | pass |
| cost | physics per step, execute wall per step | report |

Torricelli estimate for the report only: head `0.5 m`, opening `0.15 m^2`,
`v ~ 3.1 m/s`, `Q ~ 0.47 m^3/s` against `1.5 m^3` of fluid, so the upper
tank should be mostly empty after `~4 s` (`960` steps) if the opening flows
freely. G3 fails if the opening throttles far below that; G2 fails if the
positional clamp lets samples tunnel through the slab.

## Stop and interpretation

- all pass: the internal-geometry clamp and support are usable for
  presentation scenes; record cost and open the visual review;
- G2 fails: the positional clamp is insufficient for interior walls; do not
  widen the tolerance, open a swept-contact revision;
- G3 fails with G2 passing: the opening or the density complement throttles
  the flow; report the drain curve and stop, no retuning of the opening;
- G4 fails: the shelf lift or divider push traps the sheet; inspect dumps.

Do not tune the opening size, layer count, spacing or contact after
seeing the results.
