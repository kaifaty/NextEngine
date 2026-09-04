# PhysX water lane — spray and bulk from the fluid's own density (ADR-106 step 5)

| Field | Value |
| --- | --- |
| Research ID | `PHYSX-WATER-PRESENT-R4` (research report, not a product check) |
| Status | `RUN / G1-G5 PASS` (2026-09-04) |
| Parent | ADR-106 (Accepted 1.0); plans 25 and 26; ADR-102 (the spray layer of NGQ10 revision 3: neighbour and cluster thresholds) |
| Purpose | the particle pass tells bulk water from flying droplets by the fluid's own density: every published particle carries its neighbour count and the size of its connected cluster, computed on the host from the readback, so sheets render as a surface and sparse droplets as spray with streaks |

## Frozen scope

- **Analysis (lane, pure `neighbour_counts_and_clusters`).** After
  every readback: neighbours within `NEIGHBOUR_RADIUS = 2 spacings`
  (`0.1 m`) counted through a uniform hash grid of that cell size
  (`27` cells per particle), capped at `255`; connected components over
  the same neighbour links (union-find), sizes capped at `65,535`.
- **Profile.** Spray threshold `6` neighbours, cluster threshold `8`,
  spray disc radius `4 mm`, alpha `0.5`, `12` sub-droplets, streak
  `1/60 s`, bulk neighbours `20`, edge radius scale `0.5` (the NGQ10
  revision 3 values with the fluid's spacing); the surface radius stays
  `45 mm`.
- **Statistics.** `LaneStats` gains `analysis_cost_mean_us` /
  `analysis_cost_max_us` (the host analysis per frame) and
  `spray_fraction_max_permille` (particles below the spray threshold
  over the frame's count, at the frame where it peaks).
- **Not in scope.** Anisotropic kernels (revision 4), foam, the ring's
  blending under thick fluid.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots | `host-check`, `play`, `persistence-replay`, `water-present` PASS with unchanged roots and digest | pass |
| G2 analysis | unit test: on `2,000` pseudo-random particles in a `1 m` cube the grid's neighbour counts equal a brute-force count and the cluster sizes are consistent (every particle's cluster size equals the size of its component in the brute-force graph); an isolated particle has `0` neighbours and cluster size `1` | pass |
| G3 cost | analysis mean `<= 1,500 us` per frame over a `600`-frame pour session (peak about `5,300` particles); total lane cost mean `<= 4,000 us` | pass |
| G4 spray happens | in the pour session the spray fraction peaks above `5` percent (the splash) and the final frame's fraction is below the peak | pass |
| G5 look | frame `100` of the pour session: the splash shows streaked droplets while the block's body stays a surface | pass |

The radius, thresholds and caps are frozen; a change after the run is a
new revision.

## Result (2026-09-04)

Implementation: `neighbour_counts_and_clusters` in
`apps/game/src/physx_water.rs` (a dense linked-cell grid over the
particles' bounds, the forward half of the `27` neighbourhood so every
pair is visited once, union-find for the clusters), the revision 3 spray
profile values, `analysis_*` and `spray_fraction_*` statistics.

Apparatus note (recorded): the first implementation used a hash map per
cell and measured `2,253 us` mean analysis (`5.9 ms` at the splash's
peak), failing G3; the dense grid is the same rule (same radius, same
links, same counts — the unit test against brute force holds for both)
at `853 us` mean. The look was captured with the hash-map version; the
counts are identical by construction.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `host-check` PASS, `water-present` PASS (digest `88655c7b…` unchanged), `play` PASS (root `5f0c8bcd…`), `persistence-replay` PASS when run alone (a run concurrent with a game session writing saves returned an error once; an apparatus observation, not a root change) | pass |
| G2 analysis | unit test: `2,000` pseudo-random particles in a `1 m` cube plus one isolated particle — grid counts equal brute force for every particle, cluster sizes equal the flood-fill component sizes, the isolated particle has `0` neighbours and cluster `1` | pass |
| G3 cost | pour session of `600` frames: analysis `853 us` mean / `4,330 us` max; total lane cost `898 us` mean / `4,940 us` max | pass |
| G4 spray happens | spray fraction peak `865` permille (the block breaking up), last frame `735` permille with `87` sparse particles left — above `5` percent at the peak, below it at the end | pass |
| G5 look | frame `100` of the pour session: streaked spray droplets over the splash while the sheet on the level and around the crate renders as a surface | pass |

Captures stay outside Git.
