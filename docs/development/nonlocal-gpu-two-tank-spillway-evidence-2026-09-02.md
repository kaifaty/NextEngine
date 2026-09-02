# Nonlocal GPU two-tank spillway evidence — 2026-09-02

## Result and claim ceiling

`NGQ8 REV 1-4 RUN / G1 G2 G4 PASS / G3 FAIL (0.611 vs 0.6) / DISCHARGE 0.40-0.44 WITH FLUSH LIP / H8C H8D REFUTED`: the first
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
