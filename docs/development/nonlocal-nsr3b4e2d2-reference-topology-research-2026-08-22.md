# NSR3-B4E2D2 reference-binary64 topology research

Date: `2026-08-22`

Status: `COMPLETE / TOPOLOGY_RECLOSURE_SELECTED / NO_TRAJECTORY`

## Scope

B4E2D1 changes no physical coordinate in the external reference; it identifies
which exact binary64 encoding those coordinates already use. B4E2D2 therefore
does not alter formulas or tolerances. It re-runs the bounded B4E0 Dam
neighborhood preflight from the selected decoded frame zero and binds the new
pair/capacity facts.

The addition-built and decoded neighborhoods must both be constructed against
the same immutable support index. Their sorted participant-ID pair sets are
compared explicitly. Every pair present in only one set must have its admitted
radius within `64 * epsilon * h` of `h`; otherwise the discrepancy is not the
identified support-boundary rounding effect.

Both evaluations must be finite and pressure inactive: zero active centres,
zero compression energy and zero pressure gradient. This proves that the
initial pair-set delta contains no active pressure term, while preserving
density as a reported diagnostic. The selected decoded neighborhood must
remain below 960,000 pairs and degree 160.

A one-bit decoded-position mutation must change the topology or its exact
pair root. No transaction, KKT solve, publication or trajectory runs. PASS
authorizes only a new B4E2D pilot repair contract; it does not rewrite the
historical B4E0/SIRDI results.

