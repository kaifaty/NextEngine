# NSR3-B4E2D7R19R24 shadow outer-9 execution research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

## Question

Does exact continuation remain budget-offset invariant at outer 9, and is the
unchanged per-trust cap 34 still sufficient after R22's slower primal progress
and 32-HVP last trial?

## Bound source and lanes

R24 binds exact R22 state/receipt plus the complete R23 grant chain:

```text
R22 state / receipt   dc983c93075040fc2f23080072807f8a1b691adade4f3d7a950be3c9f44b36d9
                     96975d4dda71d7490e91ccccfa95c51de480c1bcd3191367f527d5b001c2fa8e
R23 state / grant     b9b8a2d3f0827d9a66a54aeb24f70c674015b8d968d8aaae16189e4adc72ef82
                     4dd8744547f991b40e5871a4c1823c3e073952dc7b51d6c28acabd92cebf5d18
grant receipt/owner   16b216ea8c7748ee80a5551f3baf5ee8f3afedea13fe5b88c6283cf07f7de285
                     bd3b2356eb10bf422ae07c16ba9433a63a63da1ccd7e567dcdd3dd3bc4fcee81
position / dual       68d0b821be6f1bdd71f70bf570035be7713cd578fe08774644cbd7c53c672b0f
                     fbbe1973290706320cbb5f3f3280b6e62b493e3abad4d280ac2985951b7568f3
history               fa9e2db82287c5f3faac6d8c8031c55bab78230fd708479a569329152c74be7c
used                  9,3,32,1,182,705,46,28,679,26,2,28,0
```

Execute the same outer-update function from independent clones:

| Lane | Starting total | Maximum | New HVP available |
|---|---:|---:|---:|
| candidate | slice `182` | `512` | `330` |
| oracle | cumulative `705` | `8704` | `7999` |

Both execute outer index `9` exactly once. Candidate may not borrow oracle
capacity. Per-update bounds are 16 trials, 330 new candidate HVP, 242 new
workspaces and 36 new precision audits; the per-trust cap remains 34.

## Required proof

Require complete bit/work equivalence of update/trials, position/dual/support,
topology, update/work roots, resource deltas, failure and admissibility. After
equivalence, consume the R23 active owner at independently pre-derived root
`3f4b2fb5d84b3f27260e53e344d5141530b8105f577483aca7a435ec85372522`
and canonical-roundtrip one `NEALOER1` receipt and one `NEALOES1` state.

Every parent/source/owner/resource/candidate/oracle/mismatch/abort failure
preserves exact transaction and physical input bytes. Duplicate replay is
prework and idempotent.

## Interpretation

- exact and admissible: research a separate convergence/publication boundary;
- exact and non-admissible with capacity: research the next one-use grant;
- any trust solve reaching the unchanged cap without convergence: hard solver
  failure and a new narrow recurrence diagnosis, not a cap increase;
- candidate slice exhaustion: stop and research epoch transition, never adopt
  oracle state;
- mismatch or other solver failure: preserve R23 and diagnose the first exact
  boundary without coefficient or policy tuning.

## Scope

One private outer-9 candidate and one comparison oracle only. No following
outer, substep, macro, trajectory, timing, public/world commit, durable or
concurrent CAS, runtime integration or production authority.
