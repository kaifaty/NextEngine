# Physical sound V26 — off-vertex field-transfer research

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `BOUNDED_RESEARCH_COMPLETE / BARYCENTRIC_SURFACE_EVALUATION_SELECTED / PROTOCOL_NOT_YET_FROZEN` |
| Trigger | [M0a-E implementation-conformance reject](physical-sound-v25-m0a-official-evaluation-result-2026-09-02.md) |
| Scope | Deterministic evaluation of a per-vertex modal-gain field at the same physical surface query across coarse/refined meshes |
| Product effect | None; research-only preprocessing with authored fallback unchanged |

## Falsifiable problem

M0a treated every impact query as if it had to equal a vertex in both the
refined and coarse mesh. The official T0 row is a valid point on both surfaces,
but its fractional coordinate is absent from the coarse vertex lattice. The
question is whether a deterministic local field evaluator can compare the same
physical query on both meshes without changing the frozen model/value surface.

## Competing hypotheses

| Hypothesis | Evidence for | Evidence against | Conclusion |
| --- | --- | --- | --- |
| Source/hash corruption | The error appeared while reading official evidence | manifest, combined lineage and 204-artifact aggregate all pass exact hashes | Rejected |
| Numeric round-off near a shared vertex | Exact-binding tolerance is very small | miss is `1.67 mm`, over five million times the tolerance | Rejected |
| Valid surface point absent from coarse vertex set | refined mesh binds; coarse mesh fails; eighth fractions do not align with denominator `12` | none | Supported |
| Neural representation failure | official command failed | no model, weight, loss or metric was created | Rejected as unobserved |

## Prior art

- The [CGAL barycentric-coordinate manual](https://doc.cgal.org/latest/Barycentric_coordinates_2/index.html)
  defines triangle coordinates by signed sub-triangle areas and describes
  locating a containing element followed by linear interpolation inside it.
- The [CGAL triangle-coordinate reference](https://doc.cgal.org/latest/Barycentric_coordinates_2/group__PkgBarycentricCoordinates2RefFunctions.html)
  states that the query is reconstructed as the weighted sum of the three
  triangle vertices; a non-degenerate triangle is required.
- The [libigl API](https://libigl.github.io/libigl-python-bindings/api/igl/)
  exposes barycentric interpolation specifically for per-vertex data on a
  triangle mesh.
- The [libigl tutorial](https://libigl.github.io/tutorial/) separates closest
  point/element location from recovering barycentric coordinates, matching the
  two checks needed here: surface membership first, field evaluation second.

These sources support a local piecewise-linear field evaluator. They do not
justify global smoothing, nearest-neighbour snapping or a relaxed surface
membership test.

## Selected successor semantics

For a query point and a triangle mesh:

1. reject non-finite queries, degenerate triangles and a point farther from the
   surface than a scale-derived frozen tolerance;
2. locate every triangle whose closest-point residual and barycentric bounds
   satisfy the tolerance;
3. choose the lexicographically smallest canonical sorted vertex-index triple,
   making edge/vertex ties independent of triangle enumeration order;
4. compute float64 barycentric weights, verify finiteness, sum-to-one,
   reconstruction residual and bounded coordinates;
5. interpolate the ten per-vertex gain values with those weights;
6. evaluate the identical world/local query independently on coarse and
   refined meshes, then apply the already frozen remesh difference gate.

The successor changes only field sampling and its conformance fixture. Model
layers, inputs, seed, schedule, losses, controls, thresholds, evidence roles,
resource ceilings and stop rules remain frozen. No hidden analytic `u/v`,
object-family lookup or material default enters candidate code.

## Rejected alternatives

- **Loosen exact-vertex tolerance:** would snap to a spatially different query
  and make results depend on mesh density.
- **Nearest vertex:** has the same density-dependent discontinuity and cannot
  compare one physical point across remeshes.
- **Expose T0 analytic `u/v`:** would create a teacher-family shortcut absent
  from the public mesh/query representation.
- **Natural-neighbour or global smoothing:** expands the hypothesis and may
  hide genuine local-field disagreement.
- **Retriangulate at runtime:** changes source identity and creates a second
  mesh authority.

## Required conformance fixture

Before another official execution, a value-independent structural fixture must
exercise the exact official coarse/refined grid shapes and all twelve contact
fractions for both plate and beam geometry. It must additionally cover exact
vertex, shared edge, triangle interior, outside-surface, degenerate-face,
non-finite, triangle-permutation and tie-break cases.

Passing means every legal official query resolves to the same physical point
on both meshes; barycentric invariants and canonical triangle identity repeat
byte-for-byte; invalid/OOD queries fail closed; and no protected role or model
value is opened. Only then may a newly frozen M0b official A/B run occur.
