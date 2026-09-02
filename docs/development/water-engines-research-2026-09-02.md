# How shipping engines and games compute water — research note (2026-09-02)

Working note for the ADR-100/103/104 water track. It surveys what
current engines and games actually run for water, which mechanics each
approach can express (communicating vessels, dams, gates, siphons,
whirlpools, floods, boats), and which ideas transfer to this engine's
tiered design (exact table and flow network as authority, height field or
lattice for large surfaces, a particle budget for active water). Claims
are limited to what the linked sources state; cost figures are the
vendors' or authors' own.

## 1. What the engines ship

| Engine / product | Water model | Mechanics it can carry | Notes |
| --- | --- | --- | --- |
| Unreal Engine 5 Water plugin | spline-driven ocean/lake/river bodies on landscape; Gerstner waves from a `WaterWaves` asset; flow maps; ripples and wakes from a small interaction simulation; physics volumes for buoyancy | swimming, buoyancy, decorative flow, ripples | no volume transfer between bodies; levels are authored |
| Unreal 5.6 "Water Advanced" / Shallow Water River, third-party Fluid Flux | 2D shallow-water equations (height plus 2D velocity) on a GPU grid per simulation area; advected foam, caustics, waves; Niagara readback of height/flow for gameplay | flow, waves, wet/dry fronts, pooling, wading ripples, boat wakes; eddies as a consequence of the velocity field | grid cost, bounded areas, coastline is non-simulated beyond the area |
| Unreal Niagara Fluids | 2D grids for games, 3D FLIP grids for hero moments or baked flipbooks | splashes, pours, local liquid | Epic's own guidance: 3D is for hero effects or cinematics |
| Unity 6 HDRP Water System | FFT spectrum (up to three bands: swell, agitation, ripples), deformers and foam generators from render textures, current maps, underwater rendering | swimming, boats on waves, decorative currents | Unity states it focuses on plane deformation, not fluid or splash simulation |
| Zibra Liquids (Unity/Unreal) | 3D particle liquid (physical solver plus learned SDF object representation), millions of particles on PC | pours, tanks, local liquid mechanics with forces | a budgeted hero system, not a world water model |

No shipping engine treats particle water as world authority. The common
shape is: authored levels or a height/shallow-water field for the body,
a small particle or 2D grid system for splashes, and rendering effort
(FFT, foam, caustics) for the look.

## 2. What the games do

| Game | Model | Mechanics |
| --- | --- | --- |
| Timberborn | water as a sheet over the terrain: per-tile depth, flow between tiles by head difference, flow measured in m^3/s, top-level-only flow calculation, waterfall flow cap; simulation at half speed; badwater and evaporation as tile fields | dams, floodgates, channels, irrigation, drought, flooding, pumps — the whole economy |
| Cities: Skylines II | water flows over the heightmap with sources and seasonal levels; dam power from the head difference; the simulation is known to be slow to settle and to stall after large terraforming | rivers, dams, floods, sea level |
| From Dust and hydraulic-erosion tools | virtual pipes (Mei, Decaudin, Hu 2007): each cell exchanges water with four neighbours through imaginary pipes by hydrostatic pressure difference; erosion and sediment ride the velocity field | flooding, lava/water flow, terrain change |
| Sea of Thieves | FFT ocean (Tessendorf) with Gerstner-style displacement; ships ride the surface | sailing, storms; the Kraken and whirlpools are authored displacement, not a simulated vortex |
| Uncharted 3/4, Horizon Forbidden West | ocean and river rendering with mesh LOD and flow shaders; gameplay volumes for swimming | look and traversal, no volume transfer |
| Noita | falling-sand cellular automaton: every pixel follows local rules, world in 64x64 dirty-rect chunks, bottom-up update order | 2D liquids, gases, pressure-free flow, mixing |

## 3. Which method gives which mechanic

| Mechanic | Height field / virtual pipes / our network | Shallow water equations (height + 2D velocity) | Particles (SPH/PBF/FLIP) |
| --- | --- | --- | --- |
| communicating vessels, siphons, pipes under floors | yes: a pipe edge carries head in either direction; "extended virtual pipes" (Dagenais et al. 2018) do exactly this for flooded passages | only through the surface; a duct under a floor needs an added edge model | yes, but expensive and not exact |
| dams, gates, channels, pumps | yes, natively (Timberborn) | yes | yes |
| floods over terrain, wet/dry fronts | yes with a lattice of cells | yes, better fronts and waves | yes, costliest |
| waves, ripples, wakes | no (levels only); add a presentation wave layer | yes | yes |
| whirlpools, eddies, vortices | no: the model moves volume, not momentum | yes: rotational flow is a consequence of the velocity field; lattice-Boltzmann shallow water captures it well | yes |
| breaking waves, spray, jets from the side | no | no (converted to particles in hybrids) | yes |
| exact save/replay, determinism | yes in integer arithmetic (ours) | only with a canonical float profile or fixed point | hardest |

The literature's answer to "large body plus small detail" is the hybrid
of Chentanez and Müller (2010, 2011): a height-field or tall-cell grid
carries the body, and regions the field cannot represent (breaking waves,
waterfalls, splashes) become particles that hand mass and momentum back
to the field. Water wave packets (Jeschke and Wojtan 2017) and water
surface wavelets (2018) add convincing dispersive waves on top of any
height field for the cost of a few thousand packets or a coarse grid.

## 4. What transfers to this engine

Already aligned: the authority tier is the Timberborn model made exact
(cells and head-driven edges in integer arithmetic, ADR-103); pipes and
gates are the extended-virtual-pipes idea; particles stay presentation
(ADR-100/102), which is the same split the engines use.

Ideas to take, in order of value:

1. **Lattice cells as the large-body tier.** ADR-103 already admits a
   network whose cells are a regular lattice with open edges. That is the
   Timberborn/virtual-pipes sheet: floods, channels and terrain-following
   water at cost proportional to wet cells, exact, saved with the world.
   The height field the presentation solver produces today should read
   its levels from these cells rather than from particles.
2. **Momentum for eddies and whirlpools where they matter.** A volume
   network cannot rotate. Two admissible routes: a presentation-only
   shallow-water grid (height plus velocity, GPU, Fluid-Flux class) fed
   by the network's levels and fluxes, which yields eddies and flow lines
   for free; or a local sink edge plus a rotational velocity field driven
   by the sink flux for the particle pass (an authored whirlpool, which is
   what Sea of Thieves does). Gameplay pull toward a whirlpool reads the
   exact sink edge, never the visual.
3. **Hybrid conversion at edges.** Chentanez–Müller's rule, applied to
   our tiers: a gate, sill or waterfall edge spawns particles in
   proportion to its flux (already planned under ADR-102) and the
   particles' presentation mass returns to the destination cell's visual
   surface; the authoritative volume never leaves the network.
4. **Wave layer on top of the field.** Wave packets or surface wavelets
   for ripples, wakes and wind waves on the cell surfaces; presentation
   only, cheap, artist-controllable.
5. **Producer-side readback discipline.** Fluid Flux exposes height and
   flow to gameplay through an async readback; we already have the
   stronger form (exact queries on the network), and the presentation
   solvers should never become a readback source.
6. **Update order and dirty regions.** Noita's bottom-up order and
   64x64 dirty chunks, and Timberborn's top-level-only flow calculation,
   are the sleep/activity model we need for map-wide lattices: only
   cells with changed head step; resting water costs nothing.

Not worth taking: Sea-of-Thieves-class FFT oceans (no ocean in scope),
3D FLIP grids as a world model (Epic's own guidance limits them to hero
moments), learned solvers as authority (see the ML note in the task
state).

## Sources

- Unreal Water system: <https://dev.epicgames.com/documentation/unreal-engine/water-system-in-unreal-engine>; UE 5.6 river to Niagara 2D fluid: <https://dev.epicgames.com/community/learning/tutorials/OZMa/fab-new-in-unreal-engine-5-6-turn-your-water-body-river-into-a-game-ready-niagara-2d-fluid-simulation>; Niagara Fluids overview: <https://dev.epicgames.com/documentation/en-us/unreal-engine/fluid-simulation-in-unreal-engine---overview>
- Fluid Flux: <https://imaginaryblend.com/2025/01/10/fluid-flux-documentation/>, <https://80.lv/articles/fluid-flux-a-cool-water-simulation-system-for-unreal-engine>
- Unity HDRP water: <https://unity.com/blog/engine-platform/new-hdrp-water-system-in-2022-lts-and-2023-1>, <https://docs.unity3d.com/Packages/com.unity.render-pipelines.high-definition@14.0/manual/WaterSystem-Overview.html>
- Zibra Liquids: <https://80.lv/articles/real-time-water-simulation-made-in-unity-with-zibra-liquids>
- Timberborn: <https://www.gamedeveloper.com/design/deep-dive-timberborn-s-water-mechanics>, <https://timberborn.fandom.com/wiki/Water_(Flowing)>
- Cities: Skylines II water: <https://cs2.paradoxwikis.com/Map_Creation:_Water>, <https://www.paradoxinteractive.com/games/cities-skylines-ii/features/electricity-water>
- Virtual pipes and extensions: Mei, Decaudin, Hu 2007 <https://ieeexplore.ieee.org/document/4392715/>; Dagenais et al. 2018 extended virtual pipes <https://www.sciencedirect.com/science/article/abs/pii/S0097849318301341>; overview <https://lisyarus.github.io/blog/posts/simulating-water-over-terrain.html>
- Hybrid height field and particles: Chentanez, Müller 2010 <https://matthias-research.github.io/pages/publications/hfFluid.pdf>; tall-cell grid 2011 <https://dl.acm.org/doi/10.1145/2010324.1964977>; coupling 3D/height field/particles <https://matthias-research.github.io/pages/publications/hybridsim_preprinted.pdf>
- Shallow water in games and vorticity: <https://rke.abertay.ac.uk/en/studentTheses/shallow-water-equations-in-real-time-computer-graphics/>, breaking waves <https://matthias-research.github.io/pages/publications/breakingWaves.pdf>
- Waves: Jeschke, Wojtan 2017 water wave packets <https://research-explorer.ista.ac.at/download/470/7359/wavepackets_final.pdf>; water surface wavelets 2018 <https://dl.acm.org/doi/10.1145/3197517.3201336>
- Sea of Thieves: <https://playersforlife.com/2024/05/01/sea-of-thieves-how-rare-achieved-the-best-water-physics-in-video-games/>
- Uncharted water technology (GDC): <https://gdcvault.com/play/1015309/Water-Technology-of>; Horizon Forbidden West water rendering (SIGGRAPH 2022): <https://advances.realtimerendering.com/s2022/SIGGRAPH2022-Advances-Water-Malan.pdf>
- Noita: <https://80.lv/articles/noita-a-game-based-on-falling-sand-simulation>, <https://noita.wiki.gg/wiki/Falling_Sand_Game>
