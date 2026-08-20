# NSR3-B4A -- closed-box/free-surface eligibility contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / PHYSICAL_TRAJECTORIES_BLOCKED`

Parent B3R must pass exactly with JSON-without-final-LF SHA-256
`792dceb540d7998d62f9c290b4e4215832ec9e1a6ba6ecf9e16b3200f1e30da9`.

## Identity and scope

```text
free-surface-closed-box-eligibility-r0
```

B4A tests static geometry generation, free-surface separation, analytical
box contact and the cost boundary of the current oracle. It executes no time
trajectory, selects no water coefficient and compares no visual result.

Constants remain `rho0=1000 kg/m^3`, `dx=0.05 m`, `R=0.025 m`, `H=3dx`,
`m=0.125 kg` and the normalized cubic selected by B0R. Pressure uses the B2
split support formula. Viscosity and surface tension are exactly disabled.

## Box-owned support fixture

Use the inner axis-aligned box `[0,0,0]..[0.3,0.3,0.3] m`, whose lattice has
`6 x 6 x 6` centre cells. A layer-`L` shell contains lattice coordinates with
integer indices in `[-L, 6+L)` excluding `[0,6)^3`; the coordinates are
`R + index*dx`. The shell belongs to the box and never depends on the fluid
bounding box.

Required exact counts:

- two layers: `10^3 - 6^3 = 784` support samples;
- three layers: `12^3 - 6^3 = 1512` support samples.

Every support point must be outside the open inner box. Every missing-air
lattice point inside the box must remain absent from support.

## Topology controls

1. **Filled-box layer control:** evaluate a `6 x 6 x 6` fluid lattice and its
   uniformly `0.99` compressed copy with two and three box-owned layers.
   Density, pressure energy, fluid gradient, fluid HVP and virtual support
   reaction must agree at relative error `<=2e-12`. The third layer is admissible only as a
   zero-contribution oracle because `W(H)=W'(H)=W''(H)=0`.
2. **Free-surface control:** retain only the bottom `6 x 3 x 6` fluid cells
   while keeping the same complete two-layer box shell. The `6 x 3 x 6`
   missing upper-air cells must not be support. With the exact rest lattice,
   every density must be `<=rho0+1e-12*rho0`, pressure must have zero active
   centres, the bottom layer must reconstruct rest density within `1e-12`,
   and the top fluid layer must contain at least one density below `rho0`.
3. **Contact control:** an owned-displacement analytical sweep uses the exact
   centre bounds `[R,R,R]..[0.3-R,0.3-R,0.3-R]`. Test inward crossings of all
   six faces, lower and upper three-axis corners, an exact graze, moving away
   and one displacement that crosses the complete box. Required feature IDs
   are `x-/x+/y-/y+/z-/z+ = 0/1/2/3/4/5`. Penetration and fluid-impulse plus
   reaction closure are each `<=1e-12`; every active TOI is finite in `[0,1]`.

No contact point may move a support sample. Contact does not contribute to
the smooth pressure energy.

## Nominal cost projection

Publish exact outer-shell counts and brute-force candidate checks per
objective evaluation for the frozen pressure-corpus geometries:

- hydro `N=6000`, box `20x20x20`: `B=5824`, checks `52,941,000`;
- dam break `N=6000`, box `80x20x20`: `B=16384`, checks `116,301,000`;
- orifice outer box only `N=6000`, box `40x20x20`: `B=9344`, checks
  `74,061,000`;
- sealed `N=48000`, box `80x20x40`: `B=24704`, checks `2,337,768,000`.

The check formula is `N(N-1)/2 + N*B`. These counts must select
`JOINT_CELL_NEIGHBORHOOD_REQUIRED_BEFORE_NOMINAL`, not trigger a nominal run.

## Exit and non-regression

The report schema is `nextengine.nonlocal.nsr3b4a_eligibility.v1`. Two runs
must be byte-identical. B3R, D5, original B3 and B2 raw reports remain exact.

PASS selects `CLOSED_BOX_FREE_SURFACE_ELIGIBLE` and authorizes only B4B tiny
pressure-corpus contract design. Failure preserves its first exact gate; no
coefficient, threshold or support layer may be changed in place.

No hydrostatic/dam-break correctness, internal aperture, viscosity, surface
tension, canonical trajectory, CUDA, performance, runtime or production
authority is granted.
