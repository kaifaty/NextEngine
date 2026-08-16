# V2 — Rooted tree graph and analytical wind

## Outcome

Build the exact V0 rooted trunk/major-branch graph on the V1 solver, apply
gravity and immutable analytical wind, and publish debug structural
projections. There is no rigid collision, cutting, fracture, foliage contact,
LOD transition or persistence schema.

## Definition path

- A private authoring fixture compiles the V0 tree into stable node, segment,
  section and proxy IDs.
- Validation proves a rooted acyclic graph, positive geometry/mass, complete
  material/anchor/drag references and source-order-independent bytes.
- The solver uses material SoA and graph adjacency derived in canonical ID
  order. Vector indices and worker order are never durable identity.
- Small twigs/leaves contribute only V0 bounded mass/drag clusters.

No neutral/public content type is added. The fixture becomes a production
content consumer only at V4/V7.

## Wind and loads

Evaluate the V0 wind function from tick/substep and fixed coefficients. Sample
at declared segment/cluster points and reduce load in a complete stable order.
Gravity, wind and prescribed pull work are recorded separately. Renderer leaf
motion, camera and wall time cannot change a load.

Run calm settling, steady wind, gust, pull/hold/release and branch-junction
fixtures. Compare tip/node curves, damping envelope, first modes and external-
work-aware residuals with V0 references. Debug rods, directors, forces,
curvature/stress and residual overlays consume only the accepted state.

## Failure

Invalid graph/profile, nonfinite/overflow, non-convergence, capacity excess or
wind evaluation mismatch publishes no candidate. The serial tool stops and
does not freeze the last pose while continuing wind ticks.

## Exit

`VEGETATION-TREE-P1 = PASS` requires all V0 tree/wind curves and roots, stable
gravity equilibrium, no visible/discrete junction discontinuity under the
numeric metric, and presentation-independent results. V3 may begin only from
that exact checkpoint.
