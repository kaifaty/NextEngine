# NSR3-B4EP10D -- owner-computes dataflow audit contract

Status: `CLOSED / PASS / B4EP10I_CONTRACT_RESEARCH_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10d-owner-computes-dataflow-audit|v1|parent=a75c1db690e26805b7f5d900a053be789316688c9ac033fd7db902f26e37cad6:b557d20b0e67755228c94b67be11a4371e2a9bd355a70a4bd176e2ecaa62872b:44e93e6e4ee24dcc623fe36d4c99ad4456f482fe1b47d18410ffb08204f9cd72|implementation=706f071d4f7331e64b722143707ae491e59f6030|topology=active-flags;exclusive-prefix;canonical-compact;row-owner-csr;oracle=selected-cached|evaluation=pair-local-density-scalar;row-owner-density;canonical-center-energy-fold;active-directed-values;target-owner-transpose-gather|hvp=row-owner-compression-direction;active-directed-values;target-owner-transpose-gather|order=global-pair-restriction;global-center-slot-restriction;exact-serial-arithmetic|returned=existing-oracle|audits=topology226;evaluation226;hvp459;zero-mismatch|memory=nominal-added-peak<=67108864|runs=2-byte-exact|regressions=b4ep7i-byte-exact;b4ep9-result-exact|timing=none|threads=none|reference=closed|credit=b4ep10i-parallel-contract-only
```

Identity SHA-256:
`db02821e280df90a285fbbebeea8cc2ec87630891cef105e0d2a4b9f6c3bdc88`.

## Command and returned authority

Add one dedicated command:

```text
--nominal-hydro-owner-computes-dataflow-audit
```

It executes the exact B4EP9 nominal topology cache, coefficient tape and fused
transaction. Existing serial scatter results remain the returned oracle and
the only state consumed by the nonlinear solver. Alternative owner-computes
values are audit-only and must never be promoted on mismatch.

No threads, OpenMP linkage or timing is allowed in B4EP10D.

## Topology alternative

For every cached query, recompute the selected topology from the immutable
superset using:

1. one active flag per superset pair;
2. a checked canonical exclusive prefix;
3. active-pair compaction in original pair order;
4. centre-owned CSR reconstruction from superset rows and the pair map.

Require exact fluid/support points, pair bytes, CSR offsets/indices, pair
class counts, maximum degree and capacity metadata against the selected cached
result. Audit exactly 226 queries.

## Evaluation/tape alternative

For every fused workspace:

- compute one density contribution per pair from the already selected radius;
- gather density per centre in its original CSR order;
- derive compression, branch margin and centre energy;
- fold total energy serially by ascending centre index;
- compute one vector per active directed source slot;
- construct target transpose rows by scanning global source centre/slot order;
- gather each target in that stored order.

Require exact evaluation fields and validate transpose source, target,
monotonic order, two-target coverage and active-slot coverage. The selected
tape remains unchanged except for an audit-only immutable transpose plan.
Audit exactly 226 workspaces.

## HVP alternative

For every selected HVP call, compute compression-direction per centre in
original row order, then one value per active directed slot and one
target-owned gather in transpose order. Require exact vector components
against `apply_joint_pressure_tape` before the oracle result is returned.
Audit exactly 459 calls.

Counters are derived outside arithmetic loops where possible. Require zero
topology/evaluation/HVP/order/coverage mismatch and zero fallback. The maximum
combined added nominal plan plus scratch payload is 67,108,864 bytes.

## Execution and exit

Two fresh Release processes must exit zero with empty stderr and byte-identical
stdout. The final binary must retain exact B4EP7I stdout and exact B4EP9
semantic result. B4EP10D has no timing claim.

PASS authorizes only a separately frozen B4EP10I OpenMP implementation/A-B
contract. Failure preserves the B4EP9 serial candidate and routes back to
dataflow research. B4E2, GPU/runtime/schema, PhysX and production remain
blocked.

Observed PASS: all `226/226/459` topology/evaluation/HVP alternatives are
bit-exact, zero mismatches/fallbacks occur and maximum added nominal payload is
29,557,700 bytes. See the
[dated evidence](../../development/nonlocal-nsr3b4ep10d-owner-computes-dataflow-evidence-2026-08-22.md).
