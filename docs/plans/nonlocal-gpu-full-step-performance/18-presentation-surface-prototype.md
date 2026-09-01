# Original Nonlocal GPU presentation surface prototype — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NGQ4` revision 1 |
| Status | `FROZEN / PRESENTATION_ONLY / TOOL_ONLY` |
| Parent evidence | NGQ2 visual PASS and NGQ3 capacity-160 performance PASS |

## Question and claim ceiling

Can the accepted 4k/16k sphere-depth frames be converted into a coherent,
less particle-shaped visual surface by a small deterministic presentation
pass, without changing simulation state or claiming renderer readiness?

This prototype is not a fluid solver and is not part of the 4 ms physics
measurement. It reads immutable accepted depth frames. It may write images and
an OBJ mesh, but it cannot modify particles, contacts, velocities, density,
neighbor graphs or any simulation/result root.

## Frozen extraction

For every accepted frame, in this order:

1. reproduce its input root exactly;
2. label the raw wet mask with deterministic 8-connectivity;
3. retain only the largest component (earliest seed wins ties); other pixels
   remain available only as a culled-spray diagnostic;
4. perform one binary 3x3 close (dilate once, erode once); dilation treats
   pixels outside the frame as dry and erosion tests only in-frame neighbors,
   so basin-edge water is not artificially shaved;
5. assign newly closed pixels the mean depth of retained 8-neighbors;
6. apply one masked 3x3 binomial height pass with weights
   `[1,2,1] x [1,2,1]`;
7. clamp presentation height to the basin vertical extent;
8. triangulate every grid quad whose four presentation pixels are wet.

Pixel pitch remains `spacing/4 = 0.0125 m`. The rendered montage uses a
fixed directional height-gradient light. File output occurs after extraction
and is excluded from observer timing.

## Frozen lanes and gates

Use the exact version-6 capacity-160 `FALLING_DAM_4K` and
`FALLING_DAM_16K` trajectories and the five accepted steps
`0,24,48,72,96`.

Every extracted frame must satisfy:

```text
input raw frame root unchanged
finite presentation depths
presentation material components        = 1
presentation satellite area              = 0
presentation/raw wet area ratio          in [0.95, 1.05]
common wet-pixel coverage                 >= 0.95
common-pixel depth RMSE                   <= 0.025 m
common-pixel depth p95 absolute change    <= 0.050 m
mesh vertices and triangles               > 0
```

Maximum depth change is reported and root-bound but is not a rejection gate.
The lane passes only if its parent simulation lane remains apparatus/quality
PASS and all five presentation frames pass.

## Controls

- empty and non-finite input frames are rejected;
- changing one presentation mask bit changes the presentation root;
- changing one presentation depth value changes the presentation root;
- disabling file output preserves all simulation and presentation roots;
- the parent 4k/16k trace and result roots remain the exact NGQ2 values;
- extraction work/time is reported separately and never added to the CUDA
  physics distribution.

## Interpretation and stop rule

- If parent roots change, stop `APPARATUS_INCONCLUSIVE`; do not interpret the
  visual output.
- If extraction violates a shape gate, classify
  `PRESENTATION_SURFACE_REFUTED_BOUNDED`; do not retune the filter in this
  revision.
- A PASS is only `PRESENTATION_SURFACE_SUPPORTED_BOUNDED` for these frames.

CPU extraction timing is diagnostic. A later renderer/compute implementation
must be measured separately before it can consume the remaining frame budget.
SPEC-38 and ADR-076 remain Proposed; CPU DFSPH remains the fallback.
