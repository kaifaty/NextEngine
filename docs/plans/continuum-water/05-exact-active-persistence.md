# W5 — Exact active persistence

## Outcome

Save and restore the one pinned-active water region as part of one atomic
physical owner checkpoint, with exact continuation and no sleep conversion or
independent sidecar lifecycle.

## Composite checkpoint candidate

The W6 promotion introduces a current-only successor to
`PhysicsWorldCheckpointV2` only after W5 passes. Its logical closure contains:

```text
PhysicsWorldCheckpointV3 candidate {
  existing PhysicsWorldCheckpointV2 closure,
  sorted_continuum_owner_segments[],
  composite_physical_root,
}

ContinuumWaterOwnerSegmentV1 candidate {
  region_id/revision,
  profile/content/coupling hashes,
  completed physics tick/substep,
  exact sample count and conservation totals,
  sorted (SampleId, position_um[3], velocity_um_s[3])[],
  water_state_root,
}
```

The water segment is neither a separately committed file nor an optional
attachment to an already-published rigid checkpoint. The outer save validates
and publishes all physical owner segments in one generation. Neighbor grids,
DFSPH factors, render buffers, external curves and native PhysX data are not
persisted.

## Save and restore flow

The profile fixes a positive checkpoint-epoch length. Every epoch reconstructs
and validates a fresh PhysX scene whether or not a save was requested; save
requests wait for that scheduled boundary, and uninterrupted comparison runs
execute the same boundaries.

1. close the scheduled rehydration boundary after a complete water/PhysX substep;
2. validate exact region/profile/content and sample bounds;
3. serialize rigid and water segments into private staging;
4. recompute every segment and composite root;
5. atomically publish the complete save generation;
6. on load, validate the outer version and all hashes/bounds before building a
   fresh staged physical world;
7. reconstruct PhysX through its existing canonical route and water caches
   from exact samples;
8. verify the restored composite root before world replacement;
9. resume from the next declared substep.

Any failure retains the prior active world and prior complete save generation.
Unsupported alpha versions reject without migration/default filling under
ADR-046.

## Exact evidence

- save at each selected boundary `N`, resume to `M`, and compare every
  subsequent water, rigid, reaction and composite root with uninterrupted run;
- process restart in both `game` and deterministic `headless`;
- sample insertion/decode order permutations canonicalize to the same root;
- `N-1/N/N+1` sample/byte/depth bounds;
- corrupt length, duplicate/missing sample ID, stale profile/content/body
  revision, mismatched segment/composite root and unsupported version;
- failure during staging, rigid reconstruction, water reconstruction and final
  publication exposes no mixed physical generation.
- save/no-save permutations execute identical epoch barriers and roots;

`CONTINUUM-PERSISTENCE-P1 = PASS` requires exact owner-root and continuation
parity. It never compares exact active root with a future lossy sleep root.

## Region lifecycle

The region remains pinned active across the selected vertical. Unload,
streaming eviction, halo/ownership transfer, sleep candidate state and compact
field conversion are absent. A later sleep specification must define its own
representation, conversion receipt, material-specific error metrics and
repeated-cycle drift test; W5 cannot be cited as that evidence.

## Non-goals

Sleep/wake, lossy compression, region transfer, migration from old continuum
formats, background I/O framework, independent water save and terrain state.
