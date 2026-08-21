# NSR3-B4C4A1 -- complete-lane retained-workspace application

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / B4C4B_BLOCKED`

Parent B4C4A passes with JSON-without-final-LF SHA-256
`452245c1e1a8631541db57943c8aab441bfdbaf203eac141986820dba5899166`
and semantic SHA-256
`a1d35119d7620113a98c4a77359ac282a8a7a0ca7142f3b883150a30d8fbf3fc`.

## Identity

```text
sha256  ce2c25fcad1a4dbd747f626d47b31019db11bd6652ae2b0bdbaf73f4cea8fc67
text    nextengine.nonlocal.retained-workspace-complete-lanes|v1|adaptive=p1:8,p2:16|fixed=48,96,192|expected-removals=a:p1:444,p2:124;f:p1:384,768,1536;p2:768,1536,3072
```

## Candidate lanes

Run retained and legacy variants of:

- selected complete adaptive macro replay for P1/P2;
- macro-boundary fixed `48`, `96` and `192` references for P1/P2.

Do not modify or select the rejected per-substep publication reference. Do not
enable query-vector recording on complete lanes; scalar counters, per-frame
query roots and bounded retention receipts are sufficient.

## Exact work obligations

Legacy build count minus candidate build count must equal:

| Lane | P1 | P2 |
|---|---:|---:|
| adaptive | `444` | `124` |
| fixed 48 | `384` | `768` |
| fixed 96 | `768` | `1,536` |
| fixed 192 | `1,536` | `3,072` |

For every row, transfers, reads, releases and receipt sequence equal the same
delta. Candidate and legacy nonlinear HVP, trial-build and accepted/rejected
workspace counters remain exact; only joint evaluation, neighborhood and tape
build totals decrease. Maximum retained ownership is one, maximum total live
workspaces remains at most two and all final live counts are zero.

## Physical and durable correspondence

Require bit-exact adaptive schedules and attempts, selected private states,
fixed-lane runs, physical aggregates, contact identities and onset, canonical
frame roots, publication ledger entries, trajectory root, legacy-ledger root
and policy-ledger root. Candidate and legacy lane status must both pass.

The adaptive legacy roots must reproduce B4C3MAR. B4C4A parent execution must
already reproduce B4C4M0/B4C3MC1. No tolerance or accuracy policy changes.

## Transcript and rollback

Fold frame index plus each local receipt into one complete adaptive receipt;
the fixed lane uses one ordered receipt across all frames. Candidate receipt
and query-policy roots must repeat exactly and differ from their legacy query
roots.

Run the selected adaptive forced-prepublication rollback with retained
ownership. Require unchanged prefix state/frame/ledger roots, no publication
from the failed transaction and zero retained/total live workspaces.

## Gate and authority

Two full reports must be byte-identical and reproduce B4C4A at its exact
parent hash. PASS selects only
`COMPLETE_LANE_RETAINED_ACCEPTED_WORKSPACE_CANDIDATE`. It authorizes B4C4B
static-support-index design. Flat-only CSR, B4D, nominal corpus, CUDA,
runtime/schema and production remain blocked.
