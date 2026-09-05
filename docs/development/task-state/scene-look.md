# Task state: scene look (the reference scene's modern look)

| Field | Value |
| --- | --- |
| Task | Bring the reference scene from script-generated boxes under Lambert light to a modern look, presentation only |
| Status | `PLANNED / ROADMAP_RECORDED_2026-09-05` |
| Branch | `codex/water-research` |
| Last updated | 2026-09-05 |

## Resume in 60 seconds

- **The roadmap** is [`docs/plans/look/00-scene-look-roadmap.md`](../../plans/look/00-scene-look-roadmap.md):
  seven items ordered by effect per unit of work (HDR chain and physical
  lighting; shadows; ambient occlusion; TAA; PBR materials; environment;
  post). Items 1 to 4 are adapter and shader work without content.
- **Nothing is frozen yet.** The first plan to freeze is item 1
  (`look/01-hdr-chain-and-physical-lighting.md`), gates before running,
  by the water-series pattern (plans `continuum-water/12` to `18`).
- **Invariants.** Presentation only: no state root, checkpoint or
  command changes; captures from fixed cameras, the frame cost at
  `960 x 540`, `host-check`, and a fallback to the current look when a
  feature cannot be created.

## Decisions

- **2026-09-05 (user):** the reference scene reads as "a bare Minecraft";
  the look work is recorded as its own roadmap so it is not lost behind
  the water series. Order of the first plans: 1, 2, 3, 4.

## Related

- Water look series: task-state [`water-volume-authority.md`](water-volume-authority.md), plans `continuum-water/12` to `18`, `33`, `35`, `42`.
