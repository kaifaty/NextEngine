# Water look L6 + L7 — caustics under the surface, wake and splash of floating boxes

| Field | Value |
| --- | --- |
| Research ID | `WL6` |
| Status | `FROZEN / NOT_RUN` |
| Parent | plan `continuum-water/11` items L6 and L7; plan 13 (the water pass that shades the scene behind the surface); plan 09 (the stage that owns the ring and the jet); ADR-105 (the exact immersion the wake reads) |
| Purpose | the floor and the crate under the water carry moving caustic light, and a floating box leaves a depression in the surface and sheds droplets when its immersion changes fast; the stage stays a pure function of the checkpoint and the frame index |

## Frozen scope

- **L6 caustics (water pass).** For every water pixel the pass already
  has the scene point behind the surface and the path length `t`. It
  multiplies the transmitted scene colour by
  `1 + CAUSTIC_STRENGTH * c(p.xz, time)` where `c` is the product of two
  animated sine lattices (wavelengths `0.35` and `0.23 m`, speeds `0.25`
  and `0.4 m/s`, directions `30` and `120` degrees), raised to the fourth
  power and shaped to `0..1`, attenuated by `exp(-t)`; `CAUSTIC_STRENGTH =
  0.6`. Renderer-local constants; nothing new is read.
- **L7 wake and splash (stage).** `compute_water_presentation_frame`
  gains the committed dynamic boxes of the checkpoint (the box bounds and
  vertical velocity of every active `Dynamic` body with a box shape).
  For each box whose bounds intersect a volume horizontally and whose
  bottom lies below the level: (a) the ring gets a depression around the
  box's plan rectangle, `WATER_WAKE_DEPTH_MICROMETRES = 8_000` at the box
  edge falling to `0` over `0.35 m`, inside the unchanged `20 mm` cap;
  (b) when the box's vertical speed exceeds `0.3 m/s`, the frame carries
  splash droplets around the waterline: `1` droplet per `0.02 m/s` above
  the threshold, at most `64` per box, launched from the waterline
  outward and up at `0.8 m/s` with the same stateless age rule as the jet
  (age from the frame index, lifetime `1 s`), inside the jet's particle
  bound of `4,096` and bounds.
- **Verification.** `water-present` (roots, purity, capacities, cost)
  and `water-buoyancy` unchanged in law; release capture from
  `--start-at-water`.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots and purity | `water-present` PASS with the new stage inputs (roots identical, byte-identical recomputation, capacities and bounds) | pass |
| G2 cost | stage cost per frame in release from the `water-present` report | `<= 1.0 ms` |
| G3 look (human) | the `--start-at-water` capture shows caustic light on the floor under the water and a depression around the crate; droplets only while the crate moves fast (the first bounce) | human |

Do not change the caustic constants, the wake depth or the splash rule
after seeing the captures; a change is a new revision with its own
capture.
