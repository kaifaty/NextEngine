# NSR3-B4C4B -- immutable static-support index

Status: `PASS / ONE_MACRO_CANDIDATE_SELECTED / TIMING_DESIGN_AUTHORIZED`

Parent B4C4A1 passes with JSON-without-final-LF SHA-256
`243989062c55bccfbbf59aaa9119645c305cd5b2b233fb37bb7b3db8ce23cf39`
and semantic SHA-256
`79847936f886b938b71adfb0b15780a465c31acd89ccd6e564204dc28d3c4e93`.

## Identity

```text
sha256  4a456806bf82fd2d2319d4adfe151d955e9d902f35b198e1cdef5a8e79fe561b
text    nextengine.nonlocal.immutable-static-support-index|v1|parent=complete-retention-a1|scope=one-macro-p1,p2|split-order=cell:fluid-then-support|expected-index-builds=legacy:p1:264,p2:9;candidate:p1:1,p2:1|expected-support-records=legacy:p1:143616,p2:10944;candidate:p1:544,p2:1216|thresholds=none
```

## Candidate

Build one immutable support index at the start of each P1/P2 one-macro
transaction. It canonicalizes support by id, validates capacity/finite values,
computes cell coordinates, sorts support-local records, constructs cell ranges
and binds exact point/horizon/format identity.

For every workspace, canonicalize and cell-sort only dynamic fluid records.
For each legacy `dx/dy/dz` cell, visit the fluid range and then the support
range. Convert support-local participant `b` to `F+b`. Preserve all legacy
skip, distance, capacity, pair-sort, adjacency-sort and tape operations.

The existing `JointNeighborhood` continues to own its support vector. Flat
CSR, support views and nested-row removal are forbidden in this stage.

## Correspondence gates

Run B4C4A-retained legacy, candidate and repeated candidate one-macro P1/P2
transactions. Require bit-exact:

- canonical fluid/support arrays, discovered/final pairs and pair hash;
- adjacency rows, degrees, fluid/support pair counts and distance tests;
- joint evaluation, density, gradient, pressure tape and workspace hash;
- every KKT trial/acceptance, HVP, physical diagnostic and aggregate;
- selected schedule, contact/topology state, publication and durable roots;
- retained-workspace transfers, reads, releases, receipts and zero final live
  ownership;
- legacy query-chain identity between variants, because workspace-visible work
  fields do not change.

Candidate static-index identity and its separate work receipt must repeat
exactly. The historical B4C4A isolated hashes and B4C3MAR probe hashes remain
mandatory regression gates.

## Exact work obligations

| Case | Legacy index builds | Candidate | Legacy support records | Candidate | Removed |
|---|---:|---:|---:|---:|---:|
| P1 | `264` | `1` | `143,616` | `544` | `143,072` |
| P2 | `9` | `1` | `10,944` | `1,216` | `9,728` |

Workspace/neighborhood/tape builds remain `264/9`; dynamic fluid sort records
remain `12,672/243`; pair distance tests and directed records remain exact.
Report both legacy combined and candidate split range lookups. Apply no
wall-time threshold and make no speedup claim.

## Failure and invalidation controls

Require failure before any pair/adjacency publication for:

1. absent or failed index: `STATIC_SUPPORT_INDEX_SOURCE`;
2. expected/index identity mismatch: `STATIC_SUPPORT_INDEX_IDENTITY`;
3. corrupted record/range ordering or bounds: `STATIC_SUPPORT_INDEX_LAYOUT`;
4. support capacity, duplicate id, non-finite coordinate and invalid cell
   coordinate with the legacy failure classification.

A one-bit support-position mutation must change identity. Reusing the old
index against the new expected identity must fail; rebuilding must reproduce
the matching legacy neighborhood/evaluation/tape exactly. Permuted support
input must canonicalize to the same identity and result.

## Gate and authority

Two full reports must be byte-identical and reproduce B4C4A1 at its exact
parent hash. PASS selects only
`ONE_MACRO_IMMUTABLE_STATIC_SUPPORT_INDEX_CANDIDATE` and may authorize a
separate complete-lane application design. B4C4C flat-only CSR, B4D, nominal
corpus, CUDA, runtime/schema and production remain blocked.
