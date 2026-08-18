# R5j physical gameplay conformance — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-18 |
| Task key | `r5j-physical-gameplay-conformance` |
| Scope | Close the remaining bounded procedural R5 `PHYS-P6` gameplay gap for trip hazards, a physically colliding carried load and contact-driven melee on the existing production capsule/player path |
| Definition of done | The production reference avatar carries one visible compound load through the ordinary Physics owner, a bounded trip/carry course proves exact contacts and recovery, melee still requires a committed physical contact fact, and direct/restarted runs converge without a new mutable owner or Windows claim |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in contracts and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R5j is complete. The fixed `physical-character`
  check repeats the production generation twice with identical evidence and
  closes the remaining mandatory procedural `PHYS-P6` trip/carry/melee gap.
- **Implementation boundary:** extend the current grounded-capsule profile to
  one capsule plus one fixed local box shape on the same Physics body. The box
  is an ordinary exact `PhysicsShapeDescriptorV1`, so catalog, snapshot,
  save/load and replay already carry it without a schema successor.
- **Product consumer:** the reference player owns one visible carried-load box.
  A dedicated collision-layer proxy is coincident with an existing visible
  course wall, so the load reaches its blocker while the capsule keeps positive
  clearance; the normal R5b slope/stair/push/sensor/fall and quest routes remain
  intact.
- **Trip boundary:** a named low-riser hazard must publish a physical contact,
  traverse without teleport and preserve the procedural recovery path. Ragdoll,
  get-up clips and active articulation remain outside the capsule baseline.
- **Melee boundary:** health may change only through the existing physical
  contact fact -> Mechanics proposal -> validated RPG command path. No direct
  health mutation or animation event is added.
- **Host boundary:** Linux is active under ADR-082. Windows, THOTH hard timing
  and PhysX Stage 0 readiness remain deferred and do not occupy this work item.
- **Roadmap result:** the mandatory R5 procedural fallback and B-08 are closed.
  R6 creator CLI/second project is the next bounded stage; broader articulation,
  retarget/IK and learned routes retain their own optional/post-baseline gates.

## Completion audit

| Candidate | Audit result | Decision |
| --- | --- | --- |
| Remaining `PHYS-P6` trip/carry/melee closure | R5b explicitly deferred trip/carry; the R5 success criteria still require them, while melee has a current production contact consumer | `SELECTED AS R5J` |
| General animation graph | The current bounded Idle/Locomotion owner is already a production consumer and exact; a creator/general graph descriptor has no additional current consumer under ADR-046 | `DEFER` |
| Non-identity retarget and physical IK | The reference project has one shared neutral skeleton; inventing a second morphology or authoritative IK consumer would be speculative | `DEFER` |
| Active articulation / learned route | Stage 0, Windows/THOTH and the stopped R141 lineage do not grant cutover authority; neither is a procedural-R5 hard blocker | `DEFER` |

## Locked boundary

1. Physics remains the only writer of the capsule body pose and every carried
   collision shape follows that same body transform.
2. The bounded profile accepts exactly one primary solid capsule and at most one
   fixed local solid box; ambiguous/multiple/rotated variants fail before world
   activation.
3. Compound sweeps use canonical shape and primitive-face IDs plus the existing
   collision filters; a carried-load hit publishes the carried shape and its
   contacted face rather than fabricating a capsule contact.
4. Step-up is available only when the primary capsule is the blocking shape.
   A carried load cannot silently step through geometry.
5. Save/load and replay use the unchanged `PhysicsWorldCheckpointV1` and Replay
   V10 owner closure; there is no attachment owner, hidden cursor or native
   backend blob.
6. The existing R5b route and normal reference gameplay must remain exact apart
   from expected project/physics/presentation roots caused by the new production
   shape and visible binding.
7. General grabs, attach/drop commands, multiple carried objects, mass/effort
   effects, ragdoll/get-up, articulation and learned carrying remain out of
   scope.

## Completed implementation

1. The grounded-capsule reference profile accepts one fixed-local box, includes
   it in static/dynamic sweeps, contacts, activation penetration checks,
   staging forks and exact checkpoint reconstruction, and rejects a second box.
2. Contact continuity uses the actual carried shape plus the canonical moving
   and obstacle box-face IDs; the capsule alone may establish ground or step up.
3. The production player body owns the compound shape and derives one visible
   presentation record from the committed body pose every tick. Its
   `200×300×200` mm half-extents match the reused visible push-box mesh. There
   is no second body, mutable attachment owner or durable schema.
4. An opt-in layer-1 proxy shares the visible tall-wall geometry and leaves all
   ordinary layer-0 R5b and quest-route collisions unchanged.
5. `physical-character` covers trip traversal, carried-load blocking with
   positive capsule clearance, exact blocked-contact restart, and the existing
   contact-driven melee outcome twice per invocation.

## Verification closure

| Check | Result | Boundary proved |
| --- | --- | --- |
| `cargo run --locked -p xtask -- physical-character` | `PASS`: trip contacts `9`, final pose `[1200000,1050000,-6000000]`; carry contacts `17`, final pose `[7200000,900000,-7000000]`, load pose `[6500000,1300000,-6500000]`, capsule clearance `200000` µm, restored contact ticks `4`; melee contacts `15`, NPC health `0`; repeated generation identical | The bounded procedural `PHYS-P6` closure uses production catalog, Physics and melee routes. |
| Focused compound/reference-game tests | `PASS`: actual carried shape and canonical `+Z`/`−Z` face IDs; Begin/Persist/End; second-box rejection; production trip/carry presentation and exact restart | Compound collision and visibility are exact without breaking the existing capsule course. |
| `cargo run --locked -p xtask -- play` | `PASS`: `32` ticks; state root `1e1498bd6c240f4efba0d71e43e5ee1be7ffdafc1d7052464a95ac9914fdbf4e`; ledger `e66f0788511d48f75698a4efc3ef0b874d5013ecef1b4ba60c0d6fed1c2c3167` | Ordinary production gameplay remains intact. |
| `cargo run --locked -p xtask -- persistence-replay` | `PASS`: `20` ticks/two generations; state root `62013d24b7f4fba5463416e288ee764666fb5232c7378e354f76ce70ea601da6`; ledger root `ad6234b7fbbb4d7fde733ac7c448bd2440297fe5aef46b039325b9c44084508b` | Replay V10 and the unchanged owner closure reconstruct exactly. |
| `cargo run --locked -p xtask -- content-package` | `PASS`: `123` records, `64` chunks; manifest `70cef7998c07a2224b11f7712dab1085cda6dea89d698c37534be6f5e5c70fc7`; composition lock `2c5b466d95ed6e6cb636a7850e984a98d4e444627f6a04b9f2b71be670f689b4` | No new durable content/schema boundary was introduced. |
| `cargo run --locked -p xtask --features desktop-sdl-ash -- platform` | `PASS`: portable and SDL/Ash candidates; `9` rendered objects; presentation hash `773df60856dcd57a046550698ef0839fb9e1571a2179ea614200a12f6fab9a5a` | The visible load reaches the current Linux Vulkan path without changing authority. |
| Release `long-session-soak` performance report | `PASS / REPORT_ONLY`: `3,600` live ticks; windows `18731525/19371373/20329530` µs; final state `d51e98412021a89bac7ef45a64ec7976beafbc8d169e39d88afe3db282f2ef10`; `9/9` visible objects/indexed draws | The changed physics/render hot path is observable on Linux; this is not B-12 or release evidence. |
| Full `cargo run --locked -p xtask -- host-check` | `PASS` on `x86_64-unknown-linux-gnu`, Rust `1.97.1` | Formatting, workspace strict Clippy, all Rust/doc tests and boundary scan are green. |

Final `physical-character` roots are:

- Runtime state: `258def98968f9209f1df38fe3818b5b01cb987445a155ab1fcf4670efb4ea25a`;
- command ledger: `f07dcdc6cd2652fb3191bef9213efec24d261e3bb50cc8d8fb9e6193b51ec4b5`;
- carried-contact Physics checkpoint: `2a32537e346ddd50cd6ae040c7cfaf0599ae3e635312a0062dcc8233ca00a2e9`;
- matrix digest: `667d2462cd178be6ee314ee41bcacb477c5a96b7f28f047a1d5b5cd06dbc21c7`.

Windows/THOTH and PhysX Stage 0 are intentionally
`NOT_RUN (WindowsHostDeferred)` under ADR-082.

## Decisions

### D-001 — Carry is a bounded compound shape, not another owner

- **Decision:** Keep one capsule body pose and attach at most one identity-
  rotation local box through the existing V1 descriptor/checkpoint bytes.
- **Reason:** The gameplay need is silhouette clearance, not an independent
  grab/drop object lifecycle.
- **Consequence:** Save, replay and presentation derive the load from Physics;
  general attachments require a future consumer-backed contract.

### D-002 — Clearance collision is explicitly opt-in

- **Decision:** Give the production load and a coincident visible-wall proxy a
  dedicated bilateral collision layer.
- **Reason:** A permanently wider silhouette would otherwise invalidate the
  existing quest and R5b traversal routes everywhere in the world.
- **Consequence:** The authored carry lane is physical and visible while
  unrelated world geometry keeps its prior collision behavior.

### D-003 — Contact identity names the actual shape and feature pair

- **Decision:** Carry sweeps and continuity publish the attached box ID and its
  canonical face opposite the obstacle face; only capsule hits may ground or
  step up.
- **Reason:** Re-labeling a load hit as a capsule hit would be false evidence and
  could silently grant capsule-only locomotion behavior.
- **Consequence:** Begin/Persist/End and restored contact IDs remain exact for
  the complete compound silhouette.

## Do not retry

- Do not widen this cut into attach/drop commands, multiple loads, mass/effort
  coupling, ragdoll/get-up or active articulation.
- Do not reopen the stopped R141 learned lineage as a substitute for the
  completed procedural fallback.
- Do not run Windows/THOTH until the explicit pre-R7 host bring-up changes the
  ADR-082 host policy.
