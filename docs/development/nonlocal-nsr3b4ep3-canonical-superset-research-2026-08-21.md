# NSR3-B4EP3 canonical superset feasibility research -- 2026-08-21

Status: `COMPLETE / 227_STATE_AUDIT_PASS / B4EP3I_DESIGN_AUTHORIZED`

## Question

Can the exact nominal query sequence reuse a conservative neighbor superset
without repeating the earlier NP1-P4 reduction-order failure, exceeding the
current degree capacity or trading cell work for too many filtered pairs?

## Why the old negative does not disappear

NP1-P4 proved geometric coverage but failed exact GPU/f32 output after a
sample crossed a cell boundary. Its reused row retained anchor-cell order,
whereas the oracle rebuilt current-cell order. Equal membership was
insufficient because floating-point association changed.

The current CPU/f64 research root has a relevant structural difference:
`build_joint_neighborhood_with_static_support` explicitly sorts final pairs by
`(fluid, participant)` after cell discovery, then constructs flat CSR from
that canonical pair vector. A superset built in the same order and filtered
without reordering should be independent of cell crossing. This must be
proven on the actual nominal states; it is not inherited P4 credit.

## Frozen candidate geometry

Reuse the old untuned skin ratio as a discriminator:

```text
h       = 0.15 m
skin    = 0.04 h = 0.006 m
R_list  = h + skin = 0.156 m
```

The cell edge remains `h`. A superset build searches the fixed `[-2,+2]`
cell range, admits pairs under the exact `norm <= R_list` predicate and sorts
them lexicographically. Before reuse, compute binary64 maximum squared fluid
displacement from the anchor and require:

```text
4 * d_max_squared <= skin_squared * (1 - 2e-12)
```

This conservatively covers fluid-fluid pairs; it is stronger than needed for
fluid-static pairs. Each query filters the sorted superset using the unchanged
`norm <= h` predicate and builds the same flat CSR convention.

## Audit, not hot-path implementation

Run the exact B4EP1 macro once with state capture enabled. The 227 captured
states include the full-state parent and 226 work-only transaction queries.
Offline in the same process, compare for every state:

- filtered pair vector, fluid/support pair counts, maximum active degree,
  flat offsets and directed pair indices against a fresh canonical builder;
- evaluation and pressure tape bit-for-bit;
- certificate decision, anchor/rebuild state and superset capacity;
- canonical full cell-distance tests versus superset-build tests plus filtered
  candidate checks.

Require at least 114 reused states, maximum candidate degree at most 160,
candidate/active pair visits at most `1.25`, and total candidate construction
work below canonical cell-distance tests. These are feasibility gates, not a
wall-time claim.

Three negatives must prove that the audit rejects: displacement just beyond
the certificate, one removed active pair and one swapped active pair order.
Run the report twice in fresh processes and require byte equality.

## Routing

- PASS authorizes only a B4EP3I Release cache implementation/A-B contract;
- any exactness, capacity, overhead or reuse failure stops superset work and
  routes to deterministic HVP research;
- no skin/capacity tuning follows a failure under this identity.

B4E2, CUDA, runtime and production remain blocked.

The audit subsequently passes all 227 states with one rebuild, 226 reuses,
maximum candidate degree 122, `1.0689` candidate/active ratio and `0.1998`
construction-work ratio. See the
[dated evidence](nonlocal-nsr3b4ep3-canonical-superset-evidence-2026-08-21.md).
