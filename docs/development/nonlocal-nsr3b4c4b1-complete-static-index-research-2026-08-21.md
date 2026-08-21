# B4C4B1 complete-lane static-support-index design

Status: `PASS / FLAT-ONLY CSR DESIGN AUTHORIZED`

Date: `2026-08-21`

## Scope

B4C4B proves one-macro correctness and B4C4BM observes a stable construction
speedup on every recorded query state. B4C4B1 changes only lifetime: one
immutable index is owned by an entire adaptive or macro-fixed lane instead of
one macro transaction.

The index is created from the unchanged fixture boundary before the first
workspace, bound once, passed read-only across all frames/substeps/trials and
destroyed after the lane. There is no process-global cache, cross-fixture
sharing or runtime authority.

## Predeclared complete-lane work

B4C4A1 already fixes retained workspace counts. With `544/1,216` P1/P2
support samples, exact support-sort work becomes:

| Lane | Workspaces | Legacy records | Candidate | Removed |
|---|---:|---:|---:|---:|
| P1 adaptive | `1,924` | `1,046,656` | `544` | `1,046,112` |
| P1 fixed 48 | `1,557` | `847,008` | `544` | `846,464` |
| P1 fixed 96 | `2,937` | `1,597,728` | `544` | `1,597,184` |
| P1 fixed 192 | `4,631` | `2,519,264` | `544` | `2,518,720` |
| P2 adaptive | `323` | `392,768` | `1,216` | `391,552` |
| P2 fixed 48 | `900` | `1,094,400` | `1,216` | `1,093,184` |
| P2 fixed 96 | `1,751` | `2,129,216` | `1,216` | `2,128,000` |
| P2 fixed 192 | `3,449` | `4,193,984` | `1,216` | `4,192,768` |

Legacy conceptual index builds equal workspace counts; candidate index builds
equal one per lane. Dynamic fluid records, workspaces, pairs, distance tests,
evaluations, tapes and retained ownership remain exact.

## Composition gates

For each lane run retained legacy, static-index candidate and repeated
candidate. Reuse all B4C4A1 schedule, physical, topology, publication, root,
query and rollback gates. Additionally require exact static-index identity,
predeclared work and separate complete-lane work receipt.

The adaptive forced-prepublication rollback uses the same lane-scoped index.
It must preserve the committed prefix, publish nothing from the failed macro,
release every retained workspace and leave no static ownership observable
outside the local lane object.

## Decision

The frozen
[B4C4B1 contract](../plans/nonlocal-nonlinear-solver-research/03b4c4b1-complete-static-index-contract.md)
passes all complete lanes and rollback; see the
[dated evidence](nonlocal-nsr3b4c4b1-complete-static-index-evidence-2026-08-21.md).
This completes static-index packaging only and authorizes B4C4C flat-only CSR
design. B4D, nominal corpus, CUDA, runtime and production remain blocked.
