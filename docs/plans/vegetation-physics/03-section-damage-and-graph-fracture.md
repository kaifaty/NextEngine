# V3 — Section damage and graph fracture

## Outcome

Add physical cutting and one load-bearing graph split under prescribed
canonical tool/load inputs, before PhysX or gameplay integration. The result is
derived from section geometry and material response, not scalar HP.

## Section state

- Instantiate the exact V0 polar lattice only at the authored felling zone.
- Cell IDs, radial/angular bounds and closed states are canonical.
- A cut input carries expected structure/profile/topology revision, tick,
  target zone, tool/contact identity, direction and fixed-point work.
- Update affected cells in cell-ID order; derive remaining area, centroid,
  second moments and capacity from geometry.
- Preserve only V0-declared damage/history variables. Splinter meshes, bark
  tears, dust and tool marks are presentation.

This stage supports the directional notch and back cut only. Arbitrary-height
cuts, saw kerf, detailed strands, fatigue and secondary fragmentation remain
non-goals.

## Failure and topology

For every substep, compute section resultants and the frozen directional
failure criterion. Select at most one failing section using the complete V0
total key. Publish only a bounded `PendingFracture` fact and one stage-9
internal Outcome proposal. The Outcome command owns the
`PhysicsTopologyTransaction`, graph partition and stable derived-body ID; it
commits the anchored graph plus staged body atomically, and the body first
activates next substep. Pending material cannot split, downgrade or transfer.

Run progressive notch, back cut, mirrored/asymmetric cut, insufficient cut,
overload without cut, stale revision, duplicate command, capacity and forced
numeric failure cases. Repeated command traces and input-order permutations
must produce the same root and failure tick/direction.

## Exit

`VEGETATION-FRACTURE-P1 = PASS` requires V0 load-displacement, removed-section,
failure tick/direction and topology thresholds; one complete graph split or no
publication; exact same-target roots; and no scalar health/fell flag in the
authority path.
