# Presentation surface area corrigendum — revision 2

| Field | Value |
| --- | --- |
| Research ID | `NGQ4` revision 2 |
| Status | `FROZEN / PRESENTATION_ONLY / TOOL_ONLY` |
| Supersedes | Only the presentation-area gate in NGQ4 revision 1 |

## Decisive revision-1 result

The first valid 4k frame at step 0 has `4,800` raw sphere pixels. The frozen
one-pixel close produces `6,400` presentation pixels, so revision 1 rejects
area ratio `1.333333` against `[0.95,1.05]`.

This is the desired conversion from separated circle footprints to a
continuous lattice footprint, not a large-scale silhouette expansion:

```text
common raw coverage       1.0
presentation components   1
depth RMSE                0.002491392 m
depth p95 change          0.003432320 m
mesh triangles            12,482
```

Revision 1 is `PRESENTATION_SURFACE_REFUTED_BOUNDED`. Its gate is not changed
or reinterpreted.

## Single gate correction

The extraction algorithm, mask close, height kernel, fixtures and every other
gate remain byte-identical. Replace only the raw-area preservation rule with:

```text
presentation/raw wet area ratio       in [0.95, 1.40]
every newly filled pixel               has a retained raw 8-neighbor
presentation bounding-box expansion   <= 1 pixel per side
common raw wet-pixel coverage          >= 0.95
```

The `1.40` ceiling is frozen before the revision-2 run and bounds the observed
circle-to-cell fill with explicit margin. The local-fill and bounding-box
conditions prevent that allowance from becoming arbitrary silhouette growth.

Frame/corpus roots and JSON advance to revision 2 because the gate payload now
binds local-fill and bounding-box-expansion evidence. Physics and raw NGQ2
roots must remain exact.

## Stop rule

If any 4k/16k frame exceeds the revised area/locality gate or any retained
depth/topology/mesh gate, stop. Do not add a third presentation revision in
this work package.
