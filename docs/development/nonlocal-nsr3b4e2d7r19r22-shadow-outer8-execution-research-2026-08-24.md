# NSR3-B4E2D7R19R22 shadow outer-8 execution research

Date: `2026-08-24`

Status: `COMPLETE / PASS / SHADOW_OUTER8_EXECUTION_CANDIDATE`

## Question

Does exact continuation remain budget-offset invariant at outer 8, and does
the unchanged AL path keep reducing primal violation without exhausting the
remaining epoch-1 slice?

## Bound source and lanes

R22 binds R20 state/receipt plus the exact R21 grant chain:

```text
R20 state / receipt   1da2e0f54f4f15409ee29f8eb1be10798940635212c64448d12cc7e92982b84f
                     ec7a61ea0123865de888304c77ba2b90df17337dc8196e208745c5ae970c6bfd
R21 state / grant     64f90f969b76efcddb331705a653c8d31858e6d502555615d6eea2071c78093f
                     6df9d41926767b1cca9df69f576170253e2eb6682435850372bbf4420b7d0b43
position / dual       bda0ac4fa2be5c29484a17c520a968046849a49aa892a7c60b04f808a6199afd
                     dc5bcc6bb918c3336ebe4bd99e03d7b8e1c92a61ca0db73418303ed0a59484eb
history               7916f59936efde08993e6d94733e5e7254afa8d6d5b7ab0b26bc723d91978b9d
used                  8,2,27,1,102,625,41,25,602,23,2,25,0
```

Execute the same outer-update function from independent clones:

| Lane | Starting total | Maximum | New HVP available |
|---|---:|---:|---:|
| candidate | slice `102` | `512` | `410` |
| oracle | cumulative `625` | `8704` | `8079` |

Both execute outer index `8` exactly once. Candidate may not borrow oracle
capacity. Per-update bounds are 16 trials, 410 new candidate HVP, 247 new
workspaces and 39 new precision audits; the per-trust cap remains 34.

## Required proof

Require complete bit/work equivalence of update/trials, position/dual/support,
topology, update/work roots, resource deltas, failure and admissibility. After
equivalence, consume the R21 active owner (pre-derived consumed root
`08001aecf61cc43ee130187f626264aa093420830244e06293ee7824486c9721`)
and canonical-roundtrip one `NEALOER1` receipt and one `NEALOES1` state.

Every parent/source/owner/resource/candidate/oracle/mismatch/abort failure
preserves exact transaction and physical input bytes. Duplicate replay is
prework and idempotent.

## Interpretation

- exact and admissible: research a separate convergence/publication boundary;
- exact and non-admissible with capacity: research the next one-use grant;
- candidate slice exhaustion: stop and research epoch transition, never adopt
  oracle state;
- mismatch or solver failure: preserve R21 and diagnose the first exact
  boundary without coefficient/cap tuning.

## Scope

One private outer-8 candidate and one comparison oracle only. No following
outer, substep, macro, trajectory, timing, public/world commit, durable or
concurrent CAS, runtime integration or production authority.

The frozen implementation passes with exact candidate/oracle roots and remains
non-admissible after 80 new HVP. See the
[dated evidence](nonlocal-nsr3b4e2d7r19r22-shadow-outer8-execution-evidence-2026-08-24.md).
