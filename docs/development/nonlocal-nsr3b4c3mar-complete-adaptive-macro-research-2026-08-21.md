# B4C3MAR complete adaptive macro replay research

Status: `COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Question

B4C3MAG proves one adaptive macro transaction. B4C3MAR asks whether that exact
transaction composes over the complete tiny P1/P2 horizons without schedule,
topology, physical, ledger, work or rollback drift.

It deliberately does not answer whether the adaptive trajectory is accurate
relative to fixed macro `48/96/192`. Combining composition and accuracy would
again make a failure ambiguous. A successful replay authorizes a separate
adaptive-versus-fixed discriminator.

## Controller ownership

For each macro frame:

1. start from the last committed decoded canonical state;
2. evaluate start/forecast spectrum read-only;
3. run up to four private binary64 levels from that exact state;
4. recover only exact typed `REJECT_LIMIT`, require an adjacent passing pair;
5. apply the embedded gate to private endpoints;
6. apply B4C3PE1 to selected publication versus its private coarse/fine pair;
7. require B4C3MAG topology and B4C3P macro ledger;
8. atomically append one frame and one ledger entry.

All trial work is accumulated, but only the selected fine private physics and
one publication contribute to durable physical/energy totals. Canonical step
numbers count macro frames, not KKT substeps.

## Long-horizon invariants

P1 runs eight and P2 sixteen macro frames. Preserve the independent P2 onset
schedule: frames 0--13 `INACTIVE_EXACT`, frame 14 `FORECAST_ACTIVE`, frame 15
`START_ACTIVE`. P1 frame zero remains `FORECAST_ACTIVE`.

Retain existing absolute physical gates: finite state, box feasibility,
pressure/contact ledger closure, P1 density/speed/center, P2 precontact rigid
flight, cumulative publication impulse and 1% pressure/mechanical energy
budgets. Every committed topology and macro ledger must pass independently.

The report records all adaptive schedules, recovery events and work without
freezing observed counts as new limits. Existing caps remain accepted fine
`<=192`, attempted level `<=768`, four levels and bounded neighborhood/workspace
lifecycle.

## Decision

Freeze the [B4C3MAR contract](../plans/nonlocal-nonlinear-solver-research/03b4c3mar-complete-adaptive-macro-replay-contract.md).
PASS selects a complete adaptive macro controller candidate and authorizes only
adaptive-versus-fixed macro comparison design.
