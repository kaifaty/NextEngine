# Water presentation in the game root — first increment

| Field | Value |
| --- | --- |
| Research ID | `WP1` |
| Status | `RUN / G1-G5 PASS / G6 HUMAN` (2026-09-03); the ripple function revised by plan 14 (WL4: ambient spectrum plus the flux ripple, same cap) |
| Parent | ADR-100 check `CONTINUUM-WATER-PRESENT-P1`; ADR-101 dynamic surface ring; ADR-102 particle surface pass; SPEC-38 2.2 practices 3, 5 and 6 |
| Purpose | the first presentation stage inside the game root: surfaces that follow the exact levels of the table and the network, a wave layer and a jet spawned from the exact gate flux, with gameplay roots untouched |

## Frozen scope

- **Stage.** An engine-owned CPU stage `WaterPresentationStageV1`
  (reference game crate) that is a pure function of the committed physics
  checkpoint at the published tick and of a presentation frame index. It
  reads `effective_level` of every water volume and `edge_flux` of every
  edge; it reads nothing else and writes nothing back.
- **Surfaces.** The basin quad and two new vessel quads (authored meshes,
  the existing water material) are declared as ADR-101 dynamic surfaces
  in the desktop run options. Each frame the stage publishes one update
  per quad: a `32 x 16` vertex grid over the volume's plan whose height
  is the exact level plus a wave layer. The wave layer is a deterministic
  ripple function of the frame index and the vertex position, with
  amplitude proportional to the volume's incoming edge flux of the
  published tick (still water is flat) and capped at `20 mm`; it never
  moves a vertex outside the volume's extent.
- **Jet.** A ballistic particle emitter at every `Gate`/`Pipe` mouth with
  positive flux: per frame it spawns particles in proportion to the flux
  (frozen: one particle per `0.5 L`, at most `256` per frame), launches
  them from the invert level with the Torricelli speed of the edge's
  head, integrates them under gravity in presentation time, and retires
  them below the destination level or after `2 s`. Bounded to `4,096`
  live particles; published as one ADR-102 particle set (kernel-less
  isotropic, no neighbour data) with velocities.
- **Fallback.** With the stage disabled every quad renders as today: the
  still surface at the exact level through the environment binding.
- **Verification** `xtask water-present` (`CONTINUUM-WATER-PRESENT-P1`,
  headless): run the reference session `600` ticks with the flow network
  and compute the stage for every tick on one runtime; run a second
  runtime without the stage; compare roots each tick; check every update
  against the declared capacities and bounds; recompute the stage for the
  same checkpoint twice and compare the update hashes; report the stage
  cost. With `--capture` on a host with a display, run the desktop adapter
  through `apps/game` bindings for a bounded window and write PNGs
  (diagnostic only).

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots | state roots and physics checkpoint hashes identical every tick with and without the stage | pass |
| G2 capacity | every published update within the declared vertex/index capacity, particle capacity and bounds; zero rejections from the adapter contract validation | pass |
| G3 purity | the stage recomputed for the same checkpoint and frame index yields byte-identical updates | pass |
| G4 cost | stage CPU time per frame for three quads and up to `4,096` particles on the reference host, release build | `<= 1.0 ms` |
| G5 one plan | the desktop report of the capture run shows one catalog, one snapshot root per published frame and one frame plan (`catalog_rebuilds = 0`) | pass |
| G6 look (human) | captures show the vessel surfaces at their exact levels (A draining, B filling) and the jet from the gate mouth reaching B | human |

Do not change the grid size, the ripple amplitude cap, the spawn rate or
the particle bound after seeing the results.

## Result (WP1, 2026-09-03)

`cargo run -p xtask -- water-present` PASS (`CONTINUUM-WATER-PRESENT-P1`),
`600` ticks, `1,200` frames (two frame indices per `30 Hz` tick), three
surfaces per frame:

| Gate | Result |
| --- | --- |
| G1 roots | physics checkpoint hash, events and state root identical every tick between the runtime whose checkpoint feeds the stage and the runtime that never sees it; final root `ab1b0139...260586`, physics `46265d6d...ea7ab0` (the same in the debug and the release build) — PASS |
| G2 capacity | `0` capacity violations (`512` vertices, `2,790` indices per quad; `84` live jet particles at most against `4,096`) and `0` bounds violations (every quad vertex inside its authored mesh bounds, ripple at most `19,999 um` against the `20 mm` cap, every droplet inside the union of the volumes plus `1 m`) — PASS |
| G3 purity | every frame recomputed from the same checkpoint and frame index is identical; the repeated generation reproduces the frames digest `1bf937d5...e331` and the matrix digest `67e33e68...9053` — PASS |
| G4 cost | release build: `98 us` maximum, `75 us` mean per frame (debug: `756 us` / `512 us`) — PASS against `1.0 ms` |
| G5 one plan | `apps/game --interactive --maximum-frames 362 --capture-frame 360 --capture-png ...`: `546` dynamic surface publications (three per published snapshot), `182` particle publications, `3` dynamic draws in the last frame, `183` frame-plan cache misses (one per published snapshot root), `0` explicit invalidations, `14` rendered objects — PASS |
| G6 look (human) | captures at frames `90`, `360` and `720` from the spawn camera show the basin surface at its level; the vessels lie outside the spawn view, so the look at A draining, B filling and the jet is left to a human walk with the capture flags — HUMAN |

Apparatus (recorded, not tuned): the jet emitter is stateless — every
frame re-integrates the droplets spawned during the last `2 s` from the
edge flux history implied by the committed tick, so the stage stays a
pure function of checkpoint and frame index without a particle pool; the
capture is one frame per run (`MAX_FRAME_CAPTURE_BURST` applies).
