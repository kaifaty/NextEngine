# NSR3-B4EP3 canonical superset feasibility evidence -- 2026-08-21

Status: `PASS / CANONICAL_SUPERSET_FEASIBLE / B4EP3I_DESIGN_AUTHORIZED`

## Result

The fixed `0.04h` superset passes every exactness and feasibility gate across
all 227 nominal query states. One initial superset build covers the complete
sequence; the remaining 226 states reuse it under the conservative
displacement certificate. Filtered pair vectors, flat CSR, evaluation and
pressure tape are bit-exact against a fresh canonical rebuild at every state.

This proves feasibility, not wall-time speedup. The audit intentionally runs
the original macro and then both canonical/candidate analyses; its 31-second
process time is not a candidate benchmark. Only a separate B4EP3I hot-path A/B
may claim performance.

## Build and deterministic report

| Field | Value |
|---|---|
| B4EP3 identity | `19c2c1f27c1bec7444743c1d67ce1bb5d69b8acc22804513ef5e8a6cc21e2552` |
| implementation commit | `42b068fbaf366a74136f27b86f29106e1395e807` |
| Release executable | 3,734,560 bytes; `712fedc22f7e448459284a8209c844f55aed47ba9728ba41961b84039390d3c0` |
| GNU Build ID | `b52e6323edb752e835564d8d0cd9726ab51f7487` |
| stdout | 6,363 bytes; `4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095` |
| semantic result | `d59a5fbe86e52bedc9d27bc41f7047a5e294311ce73ea28aa16b81b70fb83312` |
| captured corpus root | `6c80e661bb2a1624a3526eab8080a6beaf491f203514d134ca3e612ca8e0d972` |
| topology root | `2fb2a770a89dd36604f0d0f40bdc9537ce632d5f9ec617e05b8f0875b9b62f15` |
| audit work receipt | `b9f60b1cac9854a11adac7222fa11068827b817b73423d4d65daa22c8c3eef26` |

Two independent Release builds produce the same executable SHA-256. Two fresh
final-code processes produce byte-identical stdout. The existing B4EP1
candidate report remains exactly
`4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112`;
base self-test, B4E0 and B4E1S also retain their exact PASS reports.

## Feasibility facts

| Fact | Result | Gate |
|---|---:|---:|
| states compared | 227 | 227 |
| superset rebuilds / certified reuses | `1 / 226` | reuses `>=114` |
| certificate failures | 0 | 0 required for observed full reuse |
| maximum anchor displacement | `9.1231139421887875e-5 m` | conservative limit about `0.003 m` |
| maximum candidate / active degree | `122 / 122` | candidate `<=160` |
| candidate / active pair visits | `1.0688973656332614` | `<=1.25` |
| canonical cell tests | 503,855,556 | reference |
| superset-build cell tests | 8,673,618 | candidate work |
| filtered candidate checks | 92,000,830 | candidate work |
| candidate construction-work ratio | `0.19980815295405813` | `<1.0` |

The work proxy predicts an 80.02% reduction in neighbor-construction tests,
but it is not a wall-time projection. Exact CSR materialization,
evaluation/tape refresh and all HVP work still remain per query.

Every filtered pair vector/order, flat offset/index vector, evaluation and tape
matches bit-for-bit. Certificate-overflow, removed-active-pair and swapped-pair
order mutations all reject through their intended gates.

## Relation to the old P4 stop

The old GPU/f32 P4 negative remains valid for its numerical root: anchor-cell
row order changed association after cell crossing. B4EP3 does not retry that
representation. The current CPU/f64 builder and candidate both use a final
lexicographic `(fluid, participant)` pair order; filtering the sorted superset
therefore preserves exact order on the observed sequence. The all-state audit
is new evidence for this root only.

## Decision

Select `CANONICAL_SUPERSET_FEASIBLE` and authorize only B4EP3I hot-path cache
design/A-B. The cache must remain internal and optional, keep the full-state
parent/oracle unchanged, fail closed on certificate/capacity error, and
reproduce all B4EP1 physical roots/counters before timing. B4E2, CUDA, runtime
and production remain blocked.
