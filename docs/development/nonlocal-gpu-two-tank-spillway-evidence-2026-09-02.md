# Nonlocal GPU two-tank spillway evidence — 2026-09-02

## Result and claim ceiling

`NGQ8 REV 1-8 RUN / TELEPORT AND LIP STEP FIXED / LATE SPEED FOLLOWS HEAD / DISCHARGE 0.39-0.44 / FILM STALL REPORTED`: the first
interior-geometry presentation scene (upper tank on a `1 m` shelf, `0.2 m`
divider with a `0.3 x 0.5 m` opening, empty lower tank) runs on the accepted
`cap160.v6` game profile with the positional spill clamp and density-only
fixed solids. No fluid sample penetrates the shelf or the divider outside the
opening (`0 m` after contact over 241 emitted frames), water reaches the lower
floor before step 480, and the upper tank drains to `0.611` of its initial
count at step 960 against the frozen `<= 0.6` gate. The plan's own
interpretation applies: the opening throttles the flow (about `3x` below the
free-orifice Torricelli estimate); no opening, layer, spacing or contact value
is retuned. This is a presentation-candidate finding under Proposed
SPEC-38/ADR-100; campaign profiles and their roots are unchanged.

## Frozen inputs

```text
plan              docs/plans/nonlocal-gpu-full-step-performance/22-two-tank-spillway.md
lane              spill (box 5 x 2 x 1.5 m, 100 x 40 x 30 cells, fluid 40 x 10 x 30 = 12,000)
solids            shelf x 0..2, y 0..1; divider x 2.0..2.2; opening y 1.0..1.3, z 0.5..1.0
support           --boundary-layers 2 --boundary-support density, no lid, plus solids (35,472 fixed)
extractor binary  6b0d71fac81e1acc...
raw summary       3a2b43b6e36d6f64... (scratch, not tracked)
```

## Gates

| Gate | Value | Result |
| --- | ---: | --- |
| G1 finite state | 4 audits finite, degree `128 <= 160`, pairs within capacity | PASS |
| G2 penetration | `0 m` | PASS |
| G3 drainage | upper fraction at step 960 `0.611` | FAIL (`<= 0.6`) |
| G4 arrival | lower floor reached by step 480 | PASS |
| cost | physics `3.11 ms` per step, execute wall `4.29 ms` per step | report |

Upper-tank fraction per second: `0.897, 0.791, 0.697, 0.611` (about
`0.10` of `1.5 m^3` per second, `0.15 m^3/s`, against the free-orifice
estimate `0.47 m^3/s`).

## Live captures (debug xtask, closing surface)

| rendered frame | solver step | PNG SHA-256 |
| ---: | ---: | --- |
| `90` | `~40` | `1a82f4e8b4d5f663...` |
| `200` | `104` | `196af8d3d0e2de56...` |

Frame 200 shows the shelf water reaching the opening and the first jet
falling into the lower tank; the shelf, divider pieces and opening render
from the `spill` static mesh of `water-preview`.

## Decision

- The positional spill clamp plus density-only solids are usable for
  presentation scenes with axis-aligned interior walls and openings
  (H8A supported bounded: G2/G4).
- Drainage throughput is throttled relative to a free orifice; the
  presentation candidate may accept it or open a revision with a contact
  discriminator (opening lip, ghost layers inside the pipe). Not retuned.
- Watch it: `cargo run --release -p xtask --features desktop-sdl-ash -- water-preview --stream-binary <nonlocal-feasibility> --stream-lane spill --until-close`.

## Checks

- extractor Release build of the `nonlocal-feasibility` target: PASS
  (other research targets in the same tree fail on pre-existing
  unused-function warnings under `-Werror` and were not built);
- `cargo check -p xtask --features desktop-sdl-ash --all-targets`: PASS;
- one headless stream run and two captures: PASS, `0` dropped timing samples.

## Revisions 2-4: narrow opening, lip and iterations (user-directed)

Same frozen scene and gates; every run 960 steps, GPU extractor, closing
surface, two density-only layers. `Cd` is the discharge coefficient over
the first two seconds against a free orifice at the mean head `0.45 m`.

| lane | lip | iterations | Cd | penetration | upper at 4 s | physics per step |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| spill (0.15 m^2) | margin | 5 | `0.351` | `0 m` | `0.611` | `3.08 ms` |
| spill | flush | 5 | `0.404` | `0 m` | `0.568` | `3.11 ms` |
| spill | open | 5 | `0.187` | `0 m` | `0.814` | `3.28 ms` |
| spill | flush | 10 | `0.388` | `0 m` | `0.581` | `4.54 ms` |
| spill | flush | 20 | `0.331` | `0 m` | `0.630` | `7.73 ms` |
| spill-narrow (0.03 m^2) | margin | 5 | `0.387` | `0 m` | `0.907` | `2.98 ms` |
| spill-narrow | flush | 5 | `0.442` | `0 m` | `0.894` | `2.98 ms` |
| spill-narrow | open | 5 | `0.199` | `0 m` | `0.957` | `3.15 ms` |
| spill-narrow | flush | 10 | `0.379` | `0 m` | `0.911` | `4.36 ms` |
| spill-narrow | flush | 20 | `0.278` | `0 m` | `0.933` | `7.58 ms` |

G1, G2 and G4 pass in every run. Findings:

- H8B supported bounded: clamping sample centres to the opening faces
  (`flush`) raises `Cd` by `0.05` on both openings.
- H8C refuted: removing the fixed samples around the opening halves the
  flow; the pipe needs its density support.
- H8D refuted: more incompressibility iterations lower `Cd`, so the
  throttle is not the five-iteration solve.
- No variant reaches the frozen `0.5..0.8` window. A real sharp-edged
  orifice sits near `0.62` and a short tube near `0.8`; the candidate
  delivers about two thirds of the former with `flush`. The remaining gap
  lives in the profile itself (viscosity terms, the `0.1 m` release lift,
  shelf support), which is outside this plan's frozen switches.
- Presentation default stays `margin` by the revision-3 rule; `flush` is
  the better admissible lip and is exposed as `--stream-spill-lip flush`.

Narrow capture: rendered frame `200`, solver step `104`, PNG
`450424bb43f756ce...` (trickle through the small opening).

## Revisions 5-6: end-phase ejection (user observation)

The user saw single samples shoot across the lower tank near the end of
the narrow drain. The frozen diagnostic (five fastest samples per second
with their neighbourhoods, `24,000` steps) found them in the air just past
the divider at pipe height with `0..5` fluid neighbours and no fixed
neighbours, reaching `13..28 m/s` after `60 s`; the solver terms floor
density at rest, so the kick was not a term. The spill clamp was: a sample
inside the divider slab whose `(y, z)` left the opening window was pushed
to the nearer slab face along `x`, up to `0.2 m` in one `1/240 s` step.

Revision 6 keeps a sample that is inside the slab in the opening window
and pushes along `x` only in the radius-wide approach bands. Same lane,
same run length, extractor binary `09b90b1d646521b6...`:

| quantity | revision 5 (before) | revision 6 |
| --- | ---: | ---: |
| maximum sample speed after 15 s | `28 m/s` | `7.0 m/s` |
| exit ratio while >= 5 samples exit | up to `2.4` | `<= 1.0` |
| upper fraction at 100 s | `0.18` (stalled from 45 s) | `0.120` (still draining) |
| penetration (inclusive window, `1e-5 m` allowance) | `0 m` | `0 m` |
| `Cd` over the first 2 s | `0.441` | `0.441` |

`spill` (wide) flush after revision 6: `6.9 m/s`, `Cd 0.404`, sheet
stalls at `0.118` from `30 s`. The remaining `6..7 m/s` droplets are single
samples with one fluid neighbour, about `1.3x` the free-fall speed from
the release height; the residual sheet stall is the D-047 monolayer effect
on the shelf (fluid degree `29..45` against `92` in the bulk) and is a
profile limitation, not a geometry one. The frozen revision-6 gates
therefore fail narrowly and are recorded as such.

## Revisions 7-8: late droplet speed (user observation)

After revision 6 the user still saw the last droplets leave faster than
the trickle should. Frozen comparison on `spill-narrow`, `24,000` steps:

| lip | exit speed at 40 / 50 / 60 / 70 s (m/s) | free fall for the head | max sample speed after 15 s |
| --- | --- | --- | ---: |
| `flush` (rev 6) | `1.90 / 3.00 / 4.44 / 4.32` | `1.67 / 1.54 / 1.32 / 1.17` | `7.0` |
| `margin` | `1.05 / 0.92 / 0.82 / 0.32` | `1.77 / 1.59 / 1.42 / 1.19` | `3.0` |
| `flush` (rev 8, level floor) | `0.60 / 0.54 / 0.49 / 0.20` | `1.66 / 1.53 / 1.31 / 1.16` | `4.06` |

H8H holds: the `flush` lip had put the pipe floor for sample centres
`25 mm` below the shelf lift, so a sample crossing the lip was thrown by
that step; H8I (pipe over-density) is refuted by the `margin` run. Revision
8 keeps the opening floor level with the shelf and applies the zero margin
only to the top and side faces: late speed follows the head, `Cd` keeps
its `0.443`, penetration `0 m`. The `4 m/s` gate misses by `0.06 m/s` and
is recorded as such. Extractor binary after revision 8:
`ff0141d0ac534623...`.
