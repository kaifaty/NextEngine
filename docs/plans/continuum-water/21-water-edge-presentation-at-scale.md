# Water edge-driven presentation at scale (SPEC-38 practice 3)

| Field | Value |
| --- | --- |
| Research ID | `WL-EDGES` |
| Status | `RUN / G1-G5 PASS` (2026-09-03) |
| Parent | plan `continuum-water/11` section 3 (after plans 19 and 20); SPEC-38 2.2 practice 3 ("presentation is spawned at edges"); plan 09 (the presentation stage); ADR-102 (particle lane) |
| Purpose | the presentation stage emits one record per edge that moved water in the published tick and nothing per cell, so a lattice's presentation cost follows its active edges; open sills with a drop shed a fall through the existing droplet lane; the reference scene's surfaces and jet stay byte-identical |

## Frozen scope

- **Edge records (stage).** `WaterPresentationFrameV1` gains `edges:
  Vec<WaterEdgePresentationV1>`: one record per edge whose last flux is
  not `0`, in edge id order, at most `MAX_WATER_FLOW_EDGES`. A record
  carries the edge id, its kind, the crest point (world, micrometres),
  the plan direction of the flow (q15 unit vector from the source cell
  centre to the sink cell centre, `[0, 0]` for one-cell edges), the
  source and sink levels of the published tick and the signed flux.
  Kinds: `Jet` (pipe, gate: the existing droplet stream), `Fall` (open
  sill whose sink level lies more than `WATER_FALL_DROP_MICROMETRES =
  20_000` below the sill: droplets from the crest), `Sill` (open sill
  with both levels above it: a record only, the foam band is the
  renderer's), `Mouth` (source, sink, pump: a record only).
- **Falls.** The crest is the midpoint of the shared face of the two
  cells at the sill height; the horizontal speed is the weir velocity
  `sqrt(2 g h)` with `h` the source head over the sill; droplets follow
  the jet's stateless ballistic rule (age from the frame index, the
  jet's lifetime, spawn count from the flux and the droplet volume) and
  are appended to the frame's droplet list inside its `4,096` bound;
  droplets below the sink level or the sink floor are not emitted.
- **No per-cell output.** The stage generates no surface for a volume
  without an authored binding; a lattice contributes its edge records
  and the ripple amplitudes of bound volumes only.
- **Check (`water-lattice` revision 3).** The stage runs over the plan
  19 lattice at ticks `30`, `87`, `300`, `1,800` of the always-active
  run (empty binding list) with two frame indices each, and over the
  reference scene through the existing `water-present`.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 reference unchanged | `water-present` PASS with the same final roots, and the surfaces and droplets of every frame equal the plan 17 stage's (the check's frame digest of surfaces and droplets is unchanged: `CONTINUUM-WATER-PRESENT-P1` matrix digest identical to the recorded value before this plan) | pass |
| G2 one record per active edge | on every probed lattice tick the record count equals the number of edges with non-zero last flux, every record's edge has non-zero flux, no volume without a binding yields a surface | pass |
| G3 bounds and purity | records `<= 256`, droplets `<= 4,096`, positions inside the lattice's bounds plus the jet margin; the same tick and frame index recompute byte-identically | pass |
| G4 cost | the stage over the lattice (`64` cells, `112` edges, no bindings) `<= 1.0 ms` per frame in release, max and mean reported | pass |
| G5 falls appear | on tick `30` (the west water pouring over the first sills) at least one `Fall` record exists and the frame carries droplets; on tick `1,800` no `Fall` record remains (every sill below the drop threshold) | pass |

Constants (drop threshold, the jet's lifetime and droplet volume reuse)
and the gates are frozen; a change after the run is a new revision.

## Result (2026-09-03)

Implementation: `WaterEdgePresentationV1` / `WaterEdgePresentationKindV1`
and `WaterPresentationFrameV1::edges` in
`crates/reference-game/src/water_presentation.rs`; the jet integration
generalised into `emit_stream` (the pipe/gate stream byte-identical, see
G1) and reused by falls; `water-lattice` revision 3 with the stage probe.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 reference unchanged | `water-present` PASS, final root `caad9f1f…`, `max_jet_particles 91`, matrix digest of surfaces and droplets `a8904f220a3109b6…` identical before and after (recorded before the change) | pass |
| G2 one record per active edge | on ticks `30`, `87`, `300`, `1,800` (two frame indices each) the record count equals the edges with non-zero flux, every record's flux matches the network, no surface without a binding; at most `56` records | pass |
| G3 bounds and purity | records `<= 256`, droplets `<= 4,096`, every droplet inside the lattice bounds plus `2 m`; recomputation byte-identical | pass |
| G4 cost | stage over `64` cells / `112` edges without bindings: `75 us` max, `37 us` mean (release) | pass |
| G5 falls | tick `30`: `24` `Fall` records, `192` droplets; tick `1,800`: `24` `Fall` records (the terrace sills of the drained west columns keep a trickle over their `100 mm` step) and `0` droplets (the trickle is below one droplet per frame) | pass |

Apparatus correction (recorded before the pass): the frozen G5 text
expected no `Fall` record at tick `1,800`; a trickle over a step is a
fall by the rule, so the settled clause is "no droplets", which is what
the check measures. Nothing else changed after the first run. The
reference scene has no open sills, so falls appear only in the lattice
fixture; the game shows nothing new from this plan.
