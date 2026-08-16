# W4 — Basin, crate and debug presentation

## Outcome

Integrate the passing water/coupling candidate into one reference-game basin
vertical using production commands and immutable debug presentation, without
yet claiming the continuum schemas as Accepted architecture.

## Product path

- Add one content-defined sealed basin region and a separately authored dry
  fallback variant.
- Spawn the exact `0.5 m`, `50 kg` crate through the normal reference-game
  content/runtime path and bind its complete stable physics identity.
- Use the existing player action → input mapping → validated command path to
  push and re-contact the crate. No scenario-only force injection or mutable
  ECS/PhysX handle is permitted.
- Deterministic headless replays the frozen command trace with the same tick
  assignments and checks the same physical outcomes.
- Activation validates capability, region/profile/coupling-material hashes and
  all capacities before the first active water step. Failure selects the dry
  variant before world mutation.

## Minimal consumer-backed candidate boundary

W4 defines, but does not add to `crates/contracts` until the W6 Accepted
promotion change, the exact fields consumed by:

- `ContinuumRegionDefinitionV1`: stable region identity, sealed analytical
  geometry/content revision, water profile reference, capacity and authored
  dry fallback reference;
- `ContinuumWaterProfileV1`: W0 numerical constants and exact profile hash;
- `ContinuumWaterCanonicalStateV1`: region/tick/substep/root plus bounded
  sorted canonical sample states;
- `ContinuumBodyReactionBatchV1`: the W3 complete revision-bound batch;
- `ContinuumPresentationSnapshotV1`: source physical root/tick and bounded
  sorted debug samples/diagnostics.

The draft field tables live in the W4 evidence report until W6 proves the
consumer. No generic solver trait, raw neighbor/grid record, plugin ABI,
`RuntimeEntityId`, PhysX type or broad fluid query is introduced.

## Presentation

Extraction occurs only after the composite PhysicalStep commit. The immutable
snapshot contains current/previous fixed-point sample positions, stable IDs,
source tick/substep/root and bounded diagnostic classes. Presentation converts
to private floats and renders points/spheres plus optional density, iteration
and boundary overlays.

Checks run the same recorded physical trace with:

- extraction disabled;
- renderer disabled in headless;
- render cadence `30/60/144 Hz`;
- injected presentation cache/device loss;
- reordered render-buffer construction.

All authoritative water/rigid roots, reaction batches and command outcomes
must remain identical. Pixel equality is not a physics oracle.

## Product acceptance

- the crate reaches immersion `0.20 ± 0.05 m` after the declared stabilization
  interval;
- player and headless command traces produce the same exact physical roots;
- no particle crosses the sealed region and all W0/W3 residuals stay bounded;
- the debug view exposes sample distribution, density/error and failure state
  sufficiently to diagnose the corpus;
- disabling continuum before activation loads the playable dry variant;
- an active failure stops the physical run and cannot switch variants.

## Exit

W4 exits only after focused `play` and headless scenario evidence exercises
the production command path and presentation-independence cases. It remains a
branch-local candidate: public schemas and shipped status wait for W5/W6.

## Non-goals

Surface mesh, screen-space fluid, foam/spray, wetness, fluid gameplay queries,
multiple basins, streaming, save/restart and GPU presentation authority.
