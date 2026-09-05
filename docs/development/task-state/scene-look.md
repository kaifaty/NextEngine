# Task state: scene look (the reference scene's modern look)

| Field | Value |
| --- | --- |
| Task | Bring the reference scene from script-generated boxes under Lambert light to a modern look, presentation only |
| Status | `ACTIVE / ENGINE_FIRST (D-L01) / PLAN_01_DONE / PLAN_02_NEXT` |
| Branch | `codex/water-research` |
| Last updated | 2026-09-05 |

## Resume in 60 seconds

- **The roadmap** is [`docs/plans/look/00-scene-look-roadmap.md`](../../plans/look/00-scene-look-roadmap.md):
  seven items ordered by effect per unit of work (HDR chain and physical
  lighting; shadows; ambient occlusion; TAA; PBR materials; environment;
  post). Items 1 to 4 are adapter and shader work without content.
- **Plan 01 done (2026-09-05):** the HDR chain (`R16G16B16A16_SFLOAT`
  scene target, ACES tone map), the `LightingUniforms` block (set 0,
  binding 1) from the Preetham sky (SH2 irradiance, horizon fog, middle-grey
  exposure), `sky_analytic`, GGX with the catalog's metallic and roughness,
  the interface contract at `b0.v3`. Gates G2-G6 pass; G1 fails as frozen: the contract hash rides the render content profile into the project lock, so every root moved (re-pinned, creator-smoke scenario refreshed); GPU frame time
  `1.10 x`. Finding: the reference materials' albedos are `0.08..0.12`,
  so the calibrated scene reads dark until item 5 relights the content.
- **Next:** item 2 (`look/02`, cascaded shadows with PCF), then 3 (GTAO)
  and 4 (TAA).
- **Invariants.** Presentation only: no state root, checkpoint or
  command changes; captures from fixed cameras, the frame cost at
  `960 x 540`, `host-check`, and a fallback to the current look when a
  feature cannot be created.

## Decisions

- **2026-09-05 (user):** the reference scene reads as "a bare Minecraft";
  the look work is recorded as its own roadmap so it is not lost behind
  the water series. Order of the first plans: 1, 2, 3, 4.

- **2026-09-05 (user, D-L01):** the engine systems come first: items 1 to 4
  (HDR chain and physical lighting, shadows, ambient occlusion, TAA) on
  the current scene before any content work; the scene itself (items 5
  and 6) waits for them.

## Related

- Water look series: task-state [`water-volume-authority.md`](water-volume-authority.md), plans `continuum-water/12` to `18`, `33`, `35`, `42`.
