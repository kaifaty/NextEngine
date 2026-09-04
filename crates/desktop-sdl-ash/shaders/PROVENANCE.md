# B0 visual-foundation shader provenance

The `b0_textured` vertex and fragment modules are engine-owned shader assets for
the minimal indexed rendering path. They are distributed under the repository
license.

The checked-in SPIR-V modules are generated offline from the adjacent GLSL 4.50
sources. Runtime shader compilation is not permitted.

`b0_textured` now implements `B0ShaderInterfaceV2`: position, UV and optional
SNORM16 normal input plus a 208-byte outdoor frame block. A zero normal selects
the derivative flat-normal fallback. The fragment stage applies a one-sided
directional sun, hemispheric ambient, world-distance fog and the sampled
2048² outdoor shadow map with 3x3 PCF. `shadow_depth` is the fixed depth-only
caster suite; `b0_textured_no_shadow` is the stable sampled-depth/allocation
fallback. `sky_gradient`
and `ui_overlay` are separate shader suites; UI no longer shares world-lighting
shader code. All values remain renderer-local and never enter gameplay,
persistence or replay authority.

## Pinned compiler invocation

Compiler:

```text
glslangValidator
Glslang Version: 10:11.7.0
GLSL Version: 4.60 glslang Khronos. 11.7.0
SPIR-V Version 0x00010500, Revision 4
Khronos Tool ID 8
SPIR-V Generator Version 10
```

Commands, executed with this directory as the current directory:

```text
glslangValidator --quiet -V --target-env vulkan1.2 -S vert -e main -o b0_textured.vert.spv b0_textured.vert
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o b0_textured.frag.spv b0_textured.frag
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o b0_textured_no_shadow.frag.spv b0_textured_no_shadow.frag
glslangValidator --quiet -V --target-env vulkan1.2 -S vert -e main -o shadow_depth.vert.spv shadow_depth.vert
glslangValidator --quiet -V --target-env vulkan1.2 -S vert -e main -o sky_gradient.vert.spv sky_gradient.vert
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o sky_gradient.frag.spv sky_gradient.frag
glslangValidator --quiet -V --target-env vulkan1.2 -S vert -e main -o ui_overlay.vert.spv ui_overlay.vert
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o ui_overlay.frag.spv ui_overlay.frag
```

Vulkan 1.2 is the highest Vulkan target accepted by this pinned compiler and
emits SPIR-V 1.5. The modules are valid inputs to the engine's Vulkan 1.3
baseline. The exact source and module hashes are recorded in `manifest.json`.

The modules in this revision were produced by the copy of
`glslangValidator.exe` shipped with the locally installed Android Emulator.
The absolute installation path is deliberately excluded because it is not a
portable build input. That compiler executable is 7,572,296 bytes and has
SHA-256
`0b7b2ea92af3f4b74f4bea9b4389dd2bbda3bed345cc8fece8a8d22bcf7b9d7f`.

## Fluid surface suite (ADR-102, presentation-only)

`fluid_splat`, `fluid_screen`, `fluid_filter`, `fluid_thickness`,
`fluid_composite` and `fluid_spray` form the separate `fluid_surface` suite: the screen-space particle surface pass that
runs after the opaque world pass and before the UI overlay. The suite does
not touch the closed `B0ShaderInterfaceV2` contract (`interface_contract_sha256`
is unchanged); its own descriptor and push-constant layout is recorded under
`fluid_suite_compiler.interface` in `manifest.json`. All values are
renderer-local and never enter gameplay, persistence or replay authority.

The suite was compiled with a different pinned compiler, the `glslang`
binary shipped by the `kf6-core24` snap on the Linux host:

```text
glslangValidator
Glslang Version: 11:15.1.0
GLSL Version: 4.60 glslang Khronos. 15.1.0
SPIR-V Version 0x00010600, Revision 1
Khronos Tool ID 8
SPIR-V Generator Version 11
```

That executable is 7,923,264 bytes and has SHA-256
`96ea85d4228d7065507cd58454628e0d3c9ec8bbd29a0cd20ee0f3cefaf6d026`. The
absolute snap path is deliberately excluded because it is not a portable build
input. Commands, executed with this directory as the current directory:

```text
glslangValidator --quiet -V --target-env vulkan1.2 -S vert -e main -o fluid_splat.vert.spv fluid_splat.vert
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o fluid_splat.frag.spv fluid_splat.frag
glslangValidator --quiet -V --target-env vulkan1.2 -S vert -e main -o fluid_screen.vert.spv fluid_screen.vert
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o fluid_filter.frag.spv fluid_filter.frag
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o fluid_thickness.frag.spv fluid_thickness.frag
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o fluid_composite.frag.spv fluid_composite.frag
glslangValidator --quiet -V --target-env vulkan1.2 -S vert -e main -o fluid_spray.vert.spv fluid_spray.vert
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o fluid_spray.frag.spv fluid_spray.frag
```

The `vulkan1.2` target emits SPIR-V 1.5 from this compiler as well, so the
fluid modules are valid inputs to the engine's Vulkan 1.3 baseline. Source and
module hashes are pinned in `manifest.json` next to the B0 entries.

## Water surface suite (plan `continuum-water/12`, presentation-only)

`water_surface` is the material suite of dynamic rings declared as
`DynamicSurfaceShadingV1::WaterSurface` (ADR-101): the B0 vertex program
plus a fragment stage with Schlick Fresnel (`F0 = 0.02`) between the lit
water body and the reflected sky gradient, a Blinn-Phong sun glint
(exponent `240`) and the sampled shadow map, on the closed
`B0ShaderInterfaceV2` descriptor and push-constant layout. It was compiled
with the same pinned Linux `glslang` 15.1.0 as the fluid suite (SHA-256
`96ea85d4228d7065507cd58454628e0d3c9ec8bbd29a0cd20ee0f3cefaf6d026`):

```text
glslangValidator --quiet -V --target-env vulkan1.2 -S vert -e main -o water_surface.vert.spv water_surface.vert
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o water_surface.frag.spv water_surface.frag
```

Source and module hashes are pinned in `manifest.json`. All values are
renderer-local and never enter gameplay, persistence or replay authority.

## Water pass suite (plan `continuum-water/13`, presentation-only)

`water_scene` is the fragment stage of the water pass: the `water_surface`
vertex program with a fragment that samples the opaque scene colour copy
and the read-only scene depth (set 3 with a `128`-byte water uniform:
inverse view-projection, viewport, absorption `[1.2, 0.5, 0.25]` per metre
and refraction strength `0.08`, shore band `[0.12 m, 0.04 m, 0.85]`) for
refraction, depth absorption, a soft shoreline and foam on top of the WL1
material. Compiled with the same pinned Linux `glslang` 15.1.0:

```text
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o water_scene.frag.spv water_scene.frag
```

Source and module hash are pinned in `manifest.json`.

## Reflection suite (plan `continuum-water/15`, presentation-only)

`b0_reflect` is the fragment stage of the mirrored reflection pass: the
`b0_textured` fragment with a discard below the mirror plane, whose height
rides the `w` lane of the frame block's camera position for this pass
only; the `b0_textured` vertex program is reused. `water_scene` gained set
3 binding 3 (the reflection target). Both compiled with the same pinned
Linux `glslang` 15.1.0:

```text
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o b0_reflect.frag.spv b0_reflect.frag
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o water_scene.frag.spv water_scene.frag
```

## Water look L6 (plan `continuum-water/17`)

`water_scene` gained the caustic term: two animated sine lattices over the
scene point under the surface, sharpened and faded by the path length,
multiplied into the transmitted scene colour (renderer-local constants
only; no interface change). Revision 2 of the plan attenuates by the
vertical depth of the scene point instead of the path length and squares
the lattice product instead of raising it to the fourth power. Recompiled with the same pinned Linux
`glslang` 15.1.0:

```text
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o water_scene.frag.spv water_scene.frag
```

## G-buffer suite (water look L8, plan `continuum-water/18`)

`gbuffer.vert` and `gbuffer.frag` form the separate `gbuffer` suite: the
frame plan drawn again after the scene passes into four `32`-bit targets
(albedo + group mask, encoded normal + roughness, screen motion vectors,
linear depth). The suite has its own set 0 (a `160`-byte uniform with the
jittered view-projection, the previous frame's view-projection, the
viewport and the current/previous jitter; a storage buffer of previous
model matrices) and a `96`-byte push block (the B0 draw bytes plus
`meta`); it reuses the B0 texture set as set 1 and does not touch the
closed `B0ShaderInterfaceV2` contract (`interface_contract_sha256` is
unchanged). Compiled with the same pinned Linux `glslang` 15.1.0:

```text
glslangValidator --quiet -V --target-env vulkan1.2 -S vert -e main -o gbuffer.vert.spv gbuffer.vert
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o gbuffer.frag.spv gbuffer.frag
```

## Underwater suite (plan `continuum-water/33`)

`water_under.frag` on the particle pass's fullscreen vertex program
(`fluid_screen.vert`) forms the `water_under` suite: with the eye below a
water ring's level, a fullscreen pass inside the water pass (after the scene
copy, before the rings) attenuates every pixel by the path through the water
to the scene point behind it (clipped at the level plane) and adds the
in-scatter colour. It uses the water pass layout (set 0 the B0 frame block,
set 3 the water set, whose uniform grows by one `under` lane inside the
`128` bytes). `water_scene.frag` gains the back-face branch of the same plan
(Snell's window and the mirror beyond the critical angle). Compiled with the
same pinned Linux `glslang` 15.1.0:

```text
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o water_under.frag.spv water_under.frag
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o water_scene.frag.spv water_scene.frag
```

## Wet band suite (plan `continuum-water/35`)

`water_wet.frag` on the particle pass's fullscreen vertex program forms the
`water_wet` suite: with the eye above the water, a fullscreen pass inside
the water pass (after the scene copy, before the rings) darkens and glosses
every scene pixel inside a ring's widened plan within the band over that
ring's level, with the normal from the depth's world-space derivatives. The
water set gains binding 4 (a `272`-byte uniform of up to eight ring plans
and levels). Compiled with the same pinned Linux `glslang` 15.1.0:

```text
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o water_wet.frag.spv water_wet.frag
```

## Detail fade (plan `continuum-water/42`)

`water_scene.frag` scales its detail tilt by `12 / (12 + d)` with `d` the
distance from the camera in metres, so the `0.11` and `0.18 m` detail waves
no longer shimmer on far water. Compiled with the same pinned Linux
`glslang` 15.1.0:

```text
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o water_scene.frag.spv water_scene.frag
```
