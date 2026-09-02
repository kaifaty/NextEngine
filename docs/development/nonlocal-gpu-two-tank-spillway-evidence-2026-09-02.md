# Nonlocal GPU two-tank spillway evidence — 2026-09-02

## Result and claim ceiling

`NGQ8 RUN / G1 G2 G4 PASS / G3 FAIL (0.611 vs 0.6) / NO_RETUNE`: the first
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
