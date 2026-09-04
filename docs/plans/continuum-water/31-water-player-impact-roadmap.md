# Water — the remaining work, ordered by the player's impression

| Field | Value |
| --- | --- |
| Status | `ROADMAP / NOT_FROZEN` (an ordering document; each item is frozen as its own plan with gates before anything runs) |
| Date | 2026-09-04 |
| Parent | task-state `water-volume-authority.md`; water V1 accepted 2026-09-03 (ADR-100/103/104/105); the PhysX presentation lane per ADR-106 (plans 23-30) |
| Purpose | one ordered list of what the engine still lacks for water, from the item a player feels most to the one they feel least, so the next plans are picked in that order and every item names its dependencies, scale and evidence |

## How the order was made

Impact is judged from the player's side only: what they do at the water
(walk in, look, listen, fall in, push things, solve a level) and what
breaks the illusion first. Cost is noted but does not move an item; a
cheap low-impact item stays below an expensive high-impact one. Ties go
to the item that unblocks others. Scale: `S` a plan of one increment,
`M` two to three increments, `L` an ADR-level change with several plans.

Everything here keeps the program invariants: the exact CPU water
authority (no GPU state as gameplay authority, no presentation read by
any command, query or root), roots unchanged unless a plan says so,
frozen gates before running, no tuning after the readings.

## The list

| # | Item | Why it is felt | Scale | Depends on |
| --- | --- | --- | --- | --- |
| 1 | The player in the water | today the player walks on the basin floor with a message on screen; water that does not carry, slow or wet the player is scenery | L | — |
| 2 | The camera under the surface | a player who goes in sees nothing special; the underwater view is the second half of item 1 | M | 1 (or the wading depth alone) |
| 3 | Water heard | jets, falls, splashes and the player's wading are silent; sound sells contact more than pixels do | M | stage records (done), SPEC-08 / SPEC-45 |
| 4 | The player's own wake and splashes | the boxes make wakes and splashes, the player does not; the water ignores the one body the player watches | M | 1 |
| 5 | Wetness | bodies and the player leave the water dry; the shoreline and the crate do not darken | M | 4 for the drips |
| 6 | Gates, pumps, sources and sinks as play | the flow network can only be watched; levels cannot ask the player to raise, drain or redirect water | M | flow network (done) |
| 7 | Floating bodies that tilt, drift and turn | boxes rise and fall on a vertical line; a crate that never lists or drifts with a current is a lift, not a float | M | buoyancy batch (done) |
| 8 | Waves that answer events | the wave spectrum is the same whether the water is still or a crate just fell in; the shallow-water presentation grid gives ripples from bodies, shores and the player | M | 4, 7 |
| 9 | Big water | the lattice bounds fit a basin and vessels; an open plane (a lake, the sea) with the exact level and no flow network, and larger bounds when a region needs them, with the quiescence clause of D-010 | L | D-010 revisit |
| 10 | The PhysX lane's finish | kernels smoothed over time, the neighbourhood analysis on the GPU, the fluid recreated after its failure, and the retention decision that D-012 waits for | M | ADR-106 evidence |
| 11 | Water and the ground | soaking, seepage and the ground-water level of SPEC-38's housekeeping; felt only in levels that use them | L | 9 |
| 12 | Far water and rendering order | LOD for distant water, shadows on and under the surface, ordering against other transparent objects | S-M | 9 |

## The items

### 1. The player in the water (L)

- **Now.** The reference character walks on the basin floor; the exact
  table classifies the capsule as in water and the HUD says so (plan 11).
  The buoyancy batch (ADR-105) lifts only the boxes.
- **What.** Buoyancy and drag for the capsule from the exact levels of
  its cells (the batch gains a capsule sample set next to the boxes),
  swimming as a locomotion mode of SPEC-37 (surface, dive, surface
  again), wading resistance by immersion depth, climbing out at a rim
  of bounded height, and the water's level entering the character
  controller's ground query. All authority, all exact, all in the
  checkpoint.
- **Evidence.** Roots move (a new recorded revision, `play` and
  `persistence-replay` re-recorded), the batch's cost gate as plan 08,
  a scripted session in the basin (walk in, float, swim across, climb
  out) with the frame-clock sample, and a human play gate.
- **Risk.** SPEC-37's locomotion state machine and the motor closure
  tests pin the biomechanics identity; a swim mode is an ADR (the
  controller reads water) rather than a plan.

### 2. The camera under the surface (M)

- **Now.** The water pass renders the surface from above only; below
  the level the player sees the basin walls through clear air.
- **What.** A submerged state of the water pass from the camera's exact
  depth: the surface seen from below with total internal reflection past
  the critical angle, depth fog by the absorption already in the
  composite, light shafts from the sun through the surface, the
  particle layer's droplets hidden, and an audio low-pass flag for item
  3. The state is presentation only, derived from the camera height and
  the level of the cell under it.
- **Evidence.** Captures above and below the level at the same frame,
  the pass cost gate of plan 13, the `water-present` digest unchanged.

### 3. Water heard (M)

- **Now.** The stage emits edge records (jet, fall, sill, mouth) and box
  splash records every tick; nothing turns them into sound.
- **What.** A sound event set from the stage records (flow rate to
  loudness and pitch for jets and falls, splash impulse from the boxes'
  immersion change and, after item 1, the player's entry) into SPEC-08's
  audio path, with SPEC-45's physical synthesis as the optional source.
  Sound is derived from presentation records, never from a root.
- **Evidence.** The stage's pure-function test set gains the event
  mapping, an audio-queued count on the session line, a listening gate
  by a human on the pour demo and the vessels.

### 4. The player's own wake and splashes (M)

- **Now.** `WaterFloatingBoxV1` records drive wakes and splashes for the
  boxes and the PhysX lane's emission; the player's capsule has no
  record.
- **What.** A capsule record in the stage (position, immersion, speed)
  from the same exact inputs, the ring's wake and splash rules extended
  to it, the lane emitting from it as from a box (plan 25's rule with a
  capsule waterline).
- **Evidence.** Stage tests for the capsule record, the lane's emission
  test, captures of a walk through the basin, the digest re-recorded.

### 5. Wetness (M)

- **Now.** Materials are dry at any distance from the water; a crate
  lifted out shows no water on it.
- **What.** A wetness term in the material suite (WL1) driven by the
  exact level for the shoreline band and by an immersion memory with a
  bounded decay for bodies and the player (presentation-only state on
  the render side), plus drips from item 4's records for bodies leaving
  the water.
- **Evidence.** Captures of the rim and the crate before, in and after
  the water; the material pass cost gate.

### 6. Gates, pumps, sources and sinks as play (M)

- **Now.** The flow network has fixed edges and sources; sinks exist
  for the vortex demo. Nothing in the world can change them.
- **What.** Level-authored gate edges with an exact open fraction
  driven by a command (the same command path as the level command of
  D-003), pumps as sources with bounded rate on a switch, interaction
  prompts on the actors. The network change is a command in the
  checkpoint's command log like any other.
- **Evidence.** Flow harness gates for a gate opening and closing
  (volume conservation, the head law), `persistence-replay` with a gate
  toggled mid-session, a human play gate.

### 7. Floating bodies that tilt, drift and turn (M)

- **Now.** Boxes are exact vertical free bodies (D-008): one axis, no
  rotation, no horizontal force.
- **What.** Horizontal drag and the network's flux as a current force,
  a righting torque from the immersed volume's centroid against the
  centre of mass (two rotational degrees, exact integer arithmetic as
  the batch), and drift.
- **Evidence.** Buoyancy harness cases (a listing crate rights itself,
  a crate drifts with a flow of known rate), the batch cost gate, the
  roots re-recorded.

### 8. Waves that answer events (M)

- **Now.** The surface has a wave spectrum, the boxes' wakes and the
  vortices; waves do not travel from an impact or reflect from the rim.
- **What.** The shallow-water presentation grid on the ring's plan
  (heights and velocities, explicit steps at the frame clock, bounded
  cells), excited by the stage's records (boxes, the player, jets,
  falls), reflecting at the rim, damped to the spectrum at rest. It
  displaces the ring's vertices and feeds the normal of the water pass;
  it is never read by the authority.
- **Evidence.** A grid harness (energy decays, a ring wave reaches the
  rim at the right speed), the ring's upload cost gate, captures.

### 9. Big water (L)

- **Now.** The lattice bounds fit the basin and the vessels; the always
  stepped network costs microseconds at this size.
- **What.** An open water body type: an exact level plane with a bound,
  no flow network, the same classification, buoyancy and presentation
  rules; larger lattice bounds for a river or a flooded street, and the
  quiescence clause of D-010 revisited then with its trade-offs.
- **Evidence.** A new recorded lattice revision with its cost gates, the
  roots of a scene with an open body, the classification harness.

### 10. The PhysX lane's finish (M)

- **Now.** Plans 25-30 done; the lane is opt-in on a run option, falls
  back to droplets, reports itself. Open: kernels flicker frame to frame
  when the neighbourhood changes, the analysis costs 2.2 ms on four
  threads at five thousand particles, a failed fluid stays failed for
  the session, AMD and CUDA-less hosts have never run the fallback.
- **What.** A temporal blend of kernels by particle identity, the
  analysis in a compute pass of the particle pass (positions are on the
  GPU anyway), recreation of the fluid after a bounded delay, a run on a
  host without the GPU library; then the retention decision of ADR-106
  and, if retained, the SPEC-18 preference of D-012.
- **Evidence.** Cost gates as plans 27 and 30, captures for the flicker,
  the fallback session's report on the other host.

### 11. Water and the ground (L)

- **Now.** The lattice is above the terrain; water neither soaks nor
  seeps.
- **What.** SPEC-38's housekeeping: a soak rate per surface material
  into a ground-water cell, seepage as a sink edge of the network, the
  ground-water level as a source for wells. Felt only in levels built
  around it.
- **Evidence.** Flow harness gates for conservation across the sink,
  the roots re-recorded.

### 12. Far water and rendering order (S-M)

- **Now.** One ring resolution, no shadows on the surface, transparency
  ordered by pass order.
- **What.** Distance LOD for the ring and the wave spectrum, shadow
  reception on the surface and shadow casting of the surface onto the
  floor, a depth-sorted order against other transparent objects.
- **Evidence.** Captures and the pass cost gates; no root touched.

## What is not on the list

- Weather (rain, ice, temperature): no product requirement names it.
- Water as a fluid simulation authority on the GPU: refused by ADR-100
  and D-011; the lane is presentation only.
- A continuous integration service: the program runs without one by
  decision; each plan's evidence is recorded by hand.
