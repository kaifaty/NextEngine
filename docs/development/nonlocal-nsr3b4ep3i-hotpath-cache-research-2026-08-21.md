# NSR3-B4EP3I hot-path superset-cache research -- 2026-08-21

Status: `COMPLETE / TRACE_OWNED_CACHE_AB_SELECTED / NO_RUNTIME_AUTHORITY`

## Objective

Measure whether the exact B4EP3 superset can reduce the 16.15-second B4EP1
work-only macro when it actually replaces per-query cell construction, while
preserving every solver decision and durable physical root.

## Ownership design

The cache is an optional internal object owned by one
`MacroAdaptiveTransactionCase`. `JointQueryTrace` carries a non-owning pointer
only during that transaction, so every existing command remains uncached and
byte-identical. The B4EP3I parent preflight also remains canonical/full-state;
the cache begins empty at the first work-only transaction query and is
destroyed after the macro.

This avoids threading a cache through the complete KKT/controller call tree
while keeping lifetime explicit and bounded. It is not global, thread-local,
persisted, public or reconstructible gameplay state.

## Query behavior

For each cached query:

1. validate the static-support binding and flat-CSR mode;
2. build the frozen `0.04h` superset on the first query or after certificate
   failure; never fall back silently;
3. otherwise reuse the anchor list after the exact B4EP3 certificate;
4. filter by unchanged `norm <= HORIZON` order, materialize exact flat CSR,
   then execute unchanged evaluation/tape/HVP paths;
5. count query, rebuild/reuse, certificate, superset/filter work and maximum
   candidate degree separately.

The nominal gate expects 226 transaction queries, one rebuild, 225 reuses,
zero certificate failure and maximum candidate degree 122. A capacity or
certificate implementation error fails the command; it cannot become an
uncounted canonical rebuild.

## A/B design

Add only a dedicated cached command. Require two byte-identical candidate
reports, exact B4EP1 physical roots/counters, and exact old B4EP1 plus B4EP3
report regression. Then run six fresh Release processes in order:

```text
UNCACHED, CACHED, CACHED, UNCACHED, UNCACHED, CACHED
```

Pair `(1,2)`, `(4,3)` and `(5,6)`. Keep timing/RSS external. Retain the
candidate only if it wins all pairs and median uncached/cached speedup is at
least `1.10x`. Otherwise preserve B4EP3 feasibility evidence but route next to
HVP design.

## Decision boundary

PASS authorizes only B4EP4 residual profiling/design. It does not enable this
cache in runtime, open B4E2 or imply GPU/production performance.
