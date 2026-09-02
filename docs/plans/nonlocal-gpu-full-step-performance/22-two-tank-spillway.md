# Two-tank spillway visual lane — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NGQ8` |
| Status | `REV 1-4 RUN / G1 G2 G4 PASS / G3 FAIL 0.611 / CD 0.40-0.44 FLUSH / H8C H8D REFUTED` (evidence: `docs/development/nonlocal-gpu-two-tank-spillway-evidence-2026-09-02.md`) |
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

## Revision 2 (frozen before running): narrow opening, user-directed

The user asked for a smaller pipe. Lane `spill-narrow` keeps every other
value and narrows the opening to `y 1.0..1.15, z 0.65..0.85`
(`0.15 x 0.2 m`, `0.03 m^2`, one fifth of revision 1). This is a scene
variant, not a repair of G3: the revision-1 result stands.

| Gate | Definition | Pass |
| --- | --- | --- |
| G1, G2, G4 | as revision 1 | pass |
| G3' drainage | upper fraction at step 960 and per second | report only; expected slower than revision 1 (`0.611`) |
| cost | physics and execute wall per step | report |

## Revision 3 (frozen before running): realistic discharge

The user asked for a real-world outflow speed. Observable: the discharge
coefficient over the first two seconds,
`Cd = drained_volume / 2 s / (A * sqrt(2 g * 0.45 m))`, reported by the
tool as `discharge_coefficient_2s`. A sharp-edged orifice in reality has
`Cd ~ 0.6..0.65`; revision 1 reads about `0.35`.

| ID | Causal hypothesis | Frozen change (`--spill-lip`) | Prediction |
| --- | --- | --- | --- |
| H8B | the one-radius clamp margin inside the opening shrinks the passable area (`0.25 x 0.45` instead of `0.3 x 0.5`; `0.1 x 0.15` instead of `0.15 x 0.2`) | `flush`: clamp sample centres to the opening faces themselves | `Cd` rises toward the geometric ratio, `~0.45` wide, larger relative gain narrow |
| H8C | fixed density samples in the divider cells adjacent to the opening raise density inside the pipe and push fluid away from its walls | `open`: `flush` plus no fixed samples in the one-cell ring around the opening | `Cd` in `0.5..0.8` |
| H8D | the throttle is the five-iteration incompressibility itself | neither change moves `Cd` by more than `0.05` | stop; solver contract, not geometry |

Gate: `Cd` within `0.5..0.8` on both `spill` and `spill-narrow` selects the
corresponding lip as the presentation default; G1, G2 and G4 must still
pass. No opening, layer, spacing, iteration or contact value beyond the two
frozen switches changes after seeing the results.

## Revision 3 result

`spill`: `Cd` `0.351` (margin), `0.404` (flush), `0.187` (open).
`spill-narrow`: `0.387`, `0.442`, `0.199`. Penetration `0 m` and arrival
pass everywhere. H8B is supported bounded (`+0.05`, just above the H8D
threshold); H8C is refuted (removing the ring halves the flow: the pipe
loses its density support and the fluid inside it decompresses). No lip
reaches `0.5..0.8`, so no presentation default changes by this revision's
rule; `flush` is the better of the two admissible lips on both lanes.

## Revision 4 (frozen before running): the solver iteration count

H8D says the remaining throttle is the five-iteration incompressibility of
the accepted `cap160.v6` profile. Frozen change: `--iterations 10` and
`--iterations 20` on `spill` and `spill-narrow` with `--spill-lip flush`;
everything else unchanged. Prediction under H8D: `Cd` rises with the
iteration count toward `0.5..0.8`; under not-H8D it stays within `0.05` of
revision 3. Gate as revision 3 (`Cd` in `0.5..0.8`, G1/G2/G4 pass); cost
per step is reported and is expected to scale with the iterations. An
iteration count other than the accepted profile is a research override:
the accepted profile, its corpus and roots are unchanged.

## Revision 4 result

`spill` flush: `Cd` `0.404` (5), `0.388` (10), `0.331` (20).
`spill-narrow` flush: `0.442`, `0.379`, `0.278`. H8D is refuted: more
iterations lower the discharge. The remaining gap to a real orifice is not
in the opening geometry, the lip, the ring support or the iteration count;
it belongs to the profile (viscosity, release lift, shelf support) and is
outside this plan. Stop; no further switch is added here.
