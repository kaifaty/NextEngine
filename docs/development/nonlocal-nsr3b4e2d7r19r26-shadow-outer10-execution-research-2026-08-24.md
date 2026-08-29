# NSR3-B4E2D7R19R26 shadow outer-10 execution research

Date: `2026-08-24`

Status: `COMPLETE / PASS / SHADOW_OUTER10_EXECUTION_CANDIDATE`

## Question

Does exact continuation remain budget-offset invariant at outer 10, does
primal violation continue to decrease, and is R24's stationarity rise
transient, stable or increasing under the unchanged solver policy?

Stationarity is an observation, not a newly invented acceptance gate. All
existing validity/admissibility predicates and cap 34 remain unchanged.

## Bound source and lanes

R26 binds exact R24 state/receipt plus the complete R25 grant chain:

```text
R24 state / receipt   749f0805dc2c4b80e2207ea456071271d9c16521515f0554a13dd7203da32d98
                     9c618bd10b6aeaed9ae9e5fddd9f7287e4df6d7adf3e76e467543c70e8674e28
R25 state / grant     b82cd881e7b50794de029e6829e8575b86630d50b0f4caaf74a7526a3bb66153
                     f825ede7b3fa52c93b47806f92e40f34eefb93daca54d0b2d3092bf0870eede7
grant receipt/owner   cc39098c863c2fa614086e3702029b0c04182bdfd501ffbf94753d9bd410a8fa
                     2f6aea4168bdc513f21099f363a9aa28cc9171620f4a7c558b2cc340d6a6f6c6
position / dual       b59fdcd410ab2612246324deb5417aa4cec908c644e7ba0d970c348ca088861a
                     69c062bb1ce89250d978dce462b8f733bc88bfd48c1fe880e57290be0e07550a
history               d39b98c23d63217b0145229b593154fa3b49e2ea2b701ba708c51f5f8eb0b28d
used                  10,2,24,1,231,754,50,30,726,28,2,30,0
```

Execute the same outer-update function from independent clones:

| Lane | Starting total | Maximum | New HVP available |
|---|---:|---:|---:|
| candidate | slice `231` | `512` | `281` |
| oracle | cumulative `754` | `8704` | `7950` |

Both execute outer index `10` exactly once. Candidate may not borrow oracle
capacity. Per-update bounds are 16 trials, 281 new candidate HVP, 238 new
workspaces and 34 new precision audits; the per-trust cap remains 34.

## Required proof

Require complete bit/work equivalence of update/trials, position/dual/support,
topology, update/work roots, resource deltas, failure and admissibility. Report
the resulting stationarity bits and compare them descriptively with R24; do
not alter pass/fail based on a post hoc trend threshold.

After equivalence, consume the R25 active owner at independently pre-derived
root `3e956008c7c633d5ab74e249af21f1a4503ae9f5709603cdc763b0047d45e28d`
and canonical-roundtrip one `NEALOER1` receipt and one `NEALOES1` state.
Every failure preserves exact transaction and physical input bytes; duplicate
replay is prework and idempotent.

## Interpretation

- exact and admissible: research a separate convergence/publication boundary;
- exact and non-admissible with capacity: preserve the trend and research the
  next one-use grant;
- cap or candidate slice exhaustion: hard failure, never adopt oracle state;
- mismatch or other solver failure: preserve R25 and diagnose the first exact
  boundary without coefficient, cap or policy tuning.

## Scope

One private outer-10 candidate and one comparison oracle only. No following
outer, substep, macro, trajectory, timing, public/world commit, durable or
concurrent CAS, runtime integration or production authority.

The frozen implementation passes with exact candidate/oracle roots. R24's
stationarity spike is transient; primal improves only 1.0032% and remains
non-admissible. See the
[dated evidence](nonlocal-nsr3b4e2d7r19r26-shadow-outer10-execution-evidence-2026-08-24.md).
