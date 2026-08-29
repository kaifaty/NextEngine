# B4C4C1 complete-lane flat-adjacency research

Status: `DESIGN FROZEN / EXECUTION AUTHORIZED`

Date: `2026-08-21`

## Question

Does the B4C4C single-owner CSR transfer remain exact across every complete
adaptive and macro-fixed lane, including refinement recovery, retained
diagnostics, canonical publication and forced rollback?

B4C4CM establishes a consistent local construction improvement, but only the
one-macro query sequence has used the new ownership path. Complete lanes have
different workspace volumes and cross-frame move/destruction schedules.

## Candidate lifetime

Compose the already selected mechanisms without changing either:

1. one immutable B4C4B index per complete lane;
2. retained accepted B4C4A workspaces for equal-state diagnostics;
3. one B4C4C flat CSR construction and ownership transfer per workspace.

Every query workspace owns its CSR first in the neighborhood, then in the
pressure tape. Accepted workspace moves carry the tape owner. Rejected and
failed private workspaces destroy it. No process-global cache, borrowed view,
solver arithmetic, pair order or tape format changes.

## Predeclared work

The B4C4B1 workspace table and fixed fluid counts derive all row/offset/
transfer counts before execution:

| Lane | Workspaces / transfers | Removed nested rows and each row-sort pass | Flat offsets |
|---|---:|---:|---:|
| P1 adaptive | `1,924` | `92,352` | `94,276` |
| P1 fixed 48 | `1,557` | `74,736` | `76,293` |
| P1 fixed 96 | `2,937` | `140,976` | `143,913` |
| P1 fixed 192 | `4,631` | `222,288` | `226,919` |
| P2 adaptive | `323` | `8,721` | `9,044` |
| P2 fixed 48 | `900` | `24,300` | `25,200` |
| P2 fixed 96 | `1,751` | `47,277` | `49,028` |
| P2 fixed 192 | `3,449` | `93,123` | `96,572` |

Forced P1 prepublication rollback has `540` workspaces/transfers, `25,920`
removed nested rows per sort pass and `26,460` flat offsets. It must retain
the B4C4B1 one-index/544-support-record result and finish with zero ownership.

Directed record counts are state-derived. Each lane reports the legacy count
and requires exact equality with candidate flat records/final tape records;
legacy builds that count twice and candidate once.

## Decision

Execute the frozen
[B4C4C1 contract](../plans/nonlocal-nonlinear-solver-research/03b4c4c1-complete-flat-adjacency-contract.md).
Require two byte-identical complete reports. B4D and all production authority
remain blocked.
