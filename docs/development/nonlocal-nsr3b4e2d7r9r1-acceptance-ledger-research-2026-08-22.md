# NSR3-B4E2D7R9R1 acceptance-ledger reclosure research

Date: `2026-08-22`

Status: `RECLOSURE_COMPLETE / CONTRACT_FROZEN / NOT_RUN`

D7R9's only hard failure is an impossible control: exact replay includes two
accepted intermediate trials inherited from D7R8, while the contract also
requires the inherited count to be zero. Candidate code never used its new
reduction to accept or commit a trial.

The smallest valid reclosure changes only the ledger assertion:

```text
inherited accepted trials = {(eta=1e-8, trial=7),
                             (eta=1e-9, trial=5)}
new accepted trials       = 0
public commits            = 0
```

D7R9R1 must first reproduce complete D7R9 failing bytes, then rerun the same
binary64 candidate and frozen `11/0/12` oracle scoring. It may not change
radius/kernel/density/PHR/inertia algebra, the `0.5` magnitude bound, route
precedence or any solver state. This reclosure corrects evidence ownership;
it does not relax a numerical gate.

