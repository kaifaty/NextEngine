# R5f base skinning and render rig — complete task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-17 |
| Task key | `r5f-base-skinning-render-rig` |
| Scope | Admit one content-addressed neutral humanoid render rig and bounded linear base-skinning route for the existing R5a player/NPC pose projection, while Physics and Physical Animation retain their accepted authority |
| Definition of done | The reference player and NPC consume the same exact surface profile through the production presentation/render handoff; malformed rig, mapping or weights fail before activation; sampled and bind-pose fallback frames remain visible; headless/game and save/replay parity remains exact with no new owner; focused plus mapped ProductChecks pass |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in content and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R5f is complete. One exact shared player/NPC surface profile now reaches production presentation, deterministic B0 LBS and the SDL/ash dynamic vertex route with a complete bind-pose fallback, while the accepted command/Physics/Motor/RPG/save/Replay owners remain unchanged.
- **Why:** Cook/activation reject incomplete mapping or weights, `PresentationSnapshotV3` atomically closes skinned scene records against complete joint poses, and sampled plus forced-fallback plans pass focused, product and hardware checks.
- **Next action:** Start bounded R5g: authored pose correctives plus cadence/LOD authority isolation. Do not admit injury/severity, active articulation or learned control in that cut.
- **Current blocker:** None.
- **Do not retry:** Reworking the accepted R5f base route to add active articulation, learned control, SPEC-36 injury deformation, neural/secondary deformation or presentation cache persistence; each remains a later cut. Pose correctives start only under the R5g boundary.
- **Reconsider when:** R5g demonstrates that its consumer-backed corrective/cadence/LOD data cannot extend the exact V3 projection without a bounded successor; do not reopen R5f pre-emptively.

## Locked boundary

1. One shared reference surface profile binds the exact humanoid mesh, neutral skeleton and R5d BodySchema asset.
2. Render joints carry stable IDs plus explicit animation-joint and body-semantic mappings; runtime name/nearest-bone/index matching is forbidden.
3. Every vertex has one to four canonical influences whose weights sum exactly to `u16::MAX`; malformed closure rejects the complete profile before activation.
4. The reference player and NPC use the same mesh/profile with different existing materials and committed body transforms.
5. Skinning reads only the immutable R5a local pose. CPU/GPU buffers, palettes and deformed vertices are reconstructible presentation data and never enter save/replay or gameplay hashes.
6. Missing optional pose sampling selects the existing bind-pose projection. A valid base profile must always render its undeformed surface; no partial palette is published.
7. R5f does not claim pose correctives, cadence/LOD isolation, injury/severity variants, active articulation, general retargeting or full R5 completion.

## Sequential implementation order

1. **Completed:** froze the minimal profile, joint mapping, vertex influences and resource bounds with canonical round-trip and negative tests.
2. **Completed:** authored the exact reference humanoid profile and closed mesh/skeleton/BodySchema dependencies through cook and activation.
3. **Completed:** published the player/NPC skinning pose atomically with the scene snapshot and consumed it in the B0 frame plan.
4. **Completed:** uploaded/used the deformed vertex stream on the real desktop renderer while retaining static bind-pose and optional no-shadow fallbacks.
5. **Completed:** proved sampled/fallback visibility, player/NPC reuse, malformed rejection and authoritative-root isolation; mapped checks pass and roadmap state names R5g next.

## Decisions

### D-001 — Skinning is a presentation successor, not a saved owner

- **Decision:** Publish exact local render-joint poses with the immutable presentation generation and reconstruct deformed vertices from locked content on demand.
- **Reason:** SPEC-28/30/37 place palettes, GPU buffers and vertex deformation after committed `RenderPose`; replay already preserves every future-affecting animation fact.
- **Consequence:** Presentation generation may advance, but checkpoint/Replay V10 owner arity and root construction remain unchanged; exact hashes advance only with the admitted project/content lock, never with GPU/deformation cache state.

### D-002 — One shared profile covers player and NPC

- **Decision:** Both physical-animation bindings use the same authored mesh and base-skinning profile; existing material selection preserves their visual role distinction.
- **Reason:** The R5 contract requires one humanoid archetype used by player and NPC, not parallel render stacks.
- **Consequence:** Production tests must observe two independently posed instances backed by one exact profile revision.

### D-003 — Linear blend skinning with bind-pose fallback is the complete baseline

- **Decision:** Admit bounded linear blend skinning and declare bind pose as the mandatory complete fallback. Pose correctives start only in the next cut.
- **Reason:** This is SPEC-37 implementation order step one and is sufficient to prove the content/presentation/renderer ownership route.
- **Consequence:** R5f may close the base route without claiming the broader `CHARACTER-EMBODIMENT-P1` matrix.

## Evidence log

| Evidence | Result | Consequence |
| --- | --- | --- |
| R5a task state and `PhysicalAnimationPoseV1` source inspection | `PASS`: player/NPC sampled and bind-pose projections already exist and are reconstructible | R5f can consume pose without another animation or replay owner |
| SPEC-30/37 boundary inspection | `PASS`: skinning is renderer-owned presentation data; invalid base content rejects before activation | The implementation must use an exact profile and retain static bind fallback |
| Contract/content implementation | `PASS`: `NeutralBaseSkinningProfileV1` canonical round-trip, exact mesh/skeleton/BodySchema closure, 1–4 positive influences and exact `u16::MAX` sums; malformed mapping/weights reject before publication | The 37-root/123-entry current package contains one required exact base profile |
| Presentation/recovery implementation | `PASS`: `PresentationSnapshotV3` carries exact player/NPC `CharacterSkinningPresentationRecordV1` records and recovery round-trips them byte-exact | Skinning remains a complete reconstructible snapshot subprojection, not a save owner |
| B0 and desktop implementation | `PASS`: fixed-point LBS produces exact hashed vertex streams; bind/sample failure retains the authored mesh; SDL/ash writes a per-frame-slot dynamic vertex ring and records exact indexed draws | Player/NPC share one mesh/profile with distinct materials; current skinned shadows use the declared optional no-shadow fallback |
| Focused Rust suites | `PASS`: contracts 221, presentation 46, render 8, desktop 61, application 24, project content pipeline 15, reference-game visual/live suites and R5f content-package vectors | Positive, malformed, sampled, fallback, recovery and real-consumer paths are covered |
| `cargo run -p xtask -- content-package` | `PASS`: 123 records, 64 chunks, 37 roots; composition lock `b215145aab66260d58e8392fd8bcea5e94f0d4b48fb1f61e6cdcc43fb3d9f296` | Exact R5f content cooks, reopens and activates atomically |
| `cargo run -p xtask -- play` | `PASS`: 32 ticks, 52 events, 23 RPG events; state root `d17d5b7461d2eefe43e5f4c82e879b6eb3eb62eb0c740bd66bd61fb9a04e8aea` | Existing offline gameplay remains complete under the new exact content lock |
| `cargo run -p xtask -- persistence-replay` | `PASS`: 20 ticks, two generations; final state root `f007c14ba8f2d49759cbf58abee83f025f77a387566770a2d500bfc7b0cdcbe7`, ledger `ad6234b7fbbb4d7fde733ac7c448bd2440297fe5aef46b039325b9c44084508b` | Ten-owner Save/Replay closure remains exact; no skinning owner/segment exists |
| `cargo run -p xtask --features desktop-sdl-ash -- platform` | `PASS`: portable contract and SDL/ash candidate; 8 rendered objects, snapshot `1574d2bf7bce23c6b3b0fd4598ec7e6fe41f3bce5b6cb960a50da54d4fc31283` | The actual affected Vulkan adapter consumes the production frame plan on this host |
| `cargo run -p xtask -- host-check` | `PASS`: fmt, workspace clippy/tests/doc-tests and boundary scan on `x86_64-unknown-linux-gnu`, Rust 1.97.1 | Cross-cutting public-contract promotion gate is green |

## Explicitly deferred

- Pose correctives and cadence/LOD authority-isolation permutations (next bounded R5 cut).
- SPEC-36 condition/injury views, severity profiles and retained/detached topology assets.
- DQS, neural residuals, load-aware muscle deformation, cloth/tissue secondary motion and contact deformation.
- Active articulated gameplay, learned policies, full general retarget/IK corpora and R5/Stage 0 promotion.
