# NSR3-B4C4B1 -- complete-lane immutable static-support index

Status: `PASS / COMPLETE_STATIC_INDEX_SELECTED / B4C4C_DESIGN_AUTHORIZED`

Parent B4C4BM passes with deterministic SHA-256
`66cccf12ffebff98a0ef904907bb04aa015a8e57102f1f8dd7bf1ef5a7663465`.
B4C4B isolated correctness remains exact at JSON-without-final-LF SHA-256
`187cff865ea739f96c1440f3041e7fd1f6de1a8c95436ae810f604850d80c90e`.

## Identity

```text
sha256  4c5ad41ebafc9ee15a44ce64fed04ed0684f87d02f3b611297a65fa505fe7599
text    nextengine.nonlocal.complete-static-support-index|v1|parent=b4c4bm|adaptive=p1:1924,p2:323|fixed=p1:1557,2937,4631;p2:900,1751,3449|index-builds=1-per-lane|thresholds=none
```

## Candidate lifetime

At entry to each complete adaptive or macro-fixed P1/P2 lane:

1. build one immutable index from that lane's fixture support;
2. validate and bind its exact identity once;
3. pass the binding read-only through every macro frame, private interval,
   nonlinear current/trial query, retained diagnostic and macro ledger;
4. destroy the lane-local binding/index after all publications or any failure.

Do not introduce a global/static cache, support views, flat CSR or runtime API.

## Exact work obligations

| Lane | Legacy/candidate workspace builds | Legacy/candidate index builds | Removed support records |
|---|---:|---:|---:|
| P1 adaptive | `1,924` | `1,924 / 1` | `1,046,112` |
| P1 fixed 48 | `1,557` | `1,557 / 1` | `846,464` |
| P1 fixed 96 | `2,937` | `2,937 / 1` | `1,597,184` |
| P1 fixed 192 | `4,631` | `4,631 / 1` | `2,518,720` |
| P2 adaptive | `323` | `323 / 1` | `391,552` |
| P2 fixed 48 | `900` | `900 / 1` | `1,093,184` |
| P2 fixed 96 | `1,751` | `1,751 / 1` | `2,128,000` |
| P2 fixed 192 | `3,449` | `3,449 / 1` | `4,192,768` |

Candidate support records sorted equal `544` for every P1 lane and `1,216`
for every P2 lane. Dynamic records, workspace/neighborhood/tape builds, pairs,
directed records, distance tests and retained transfers/reads/releases remain
exact. Candidate split lookup counts must equal two copies of the legacy
combined lookup count.

## Physical, durable and failure correspondence

Require all B4C4A1 complete-lane physical, schedule, attempt, fixed-reference,
contact/onset, aggregate, energy, canonical frame, publication ledger,
trajectory and query-chain results bit-exact between legacy, candidate and
repeat. Retention receipts remain exact; a separate work receipt binds index
identity and all structural counters.

Repeat the complete adaptive forced-prepublication rollback through the static
binding. Require exact committed prefix/roots/cumulative totals, no failed
publication and zero retained/total live workspaces. B4C4B stale/corrupt/
permutation negatives remain mandatory regression probes.

## Gate and authority

The full gate must reproduce B4C4BM's deterministic parent and B4C4B's exact
correctness parent. Two B4C4B1 reports must be byte-identical; no wall-time
threshold applies. PASS selects only
`COMPLETE_LANE_IMMUTABLE_STATIC_SUPPORT_INDEX_CANDIDATE` and authorizes B4C4C
flat-only CSR design. B4D, nominal corpus, CUDA, runtime/schema and production
remain blocked.
