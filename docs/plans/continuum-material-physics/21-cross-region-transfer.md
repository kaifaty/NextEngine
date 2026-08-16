# Package 21X — Optional cross-region transfer

## Outcome

Specify particle/material ownership transfer only after one-region production
evidence and a concrete world-streaming consumer require it. No first water or
terrain package may implement a private halo/transfer protocol.

## Required future contract

Before code, a new consumer-backed specification must freeze:

- adjacent region identity, exact ownership surface and canonical tie-break;
- halo width/content and whether halo values are cache or owner state;
- transfer tick, source/destination revisions and closed capacity reservation;
- stable sample identity/provenance across ownership change;
- interaction/reduction order at the seam;
- mass/momentum/history closure and duplicate/lost-sample diagnostics;
- atomic save/restart while transfer is prepared or committed;
- fallback when destination capacity or required region admission fails.

Source state remains authoritative until the destination and complete transfer
validate. Publication removes from source and adds to destination in one
composite generation; a sample cannot be owned twice or by neither region.

## Evidence

Seam crossing, reverse crossing, simultaneous opposite transfers,
capacity/stale/collision faults, save/restart at every transition state and
worker/completion permutations must preserve exact roots and material closure.
Streaming cannot discard dirty authoritative material state.

## Non-goals

Global fluid domain, camera-driven ownership, unbounded halo, approximate
duplicate suppression, lossy sleep and automatic expansion of the first
sealed-region product scope.
