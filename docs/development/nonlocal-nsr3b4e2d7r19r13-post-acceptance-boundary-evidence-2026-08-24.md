# NSR3-B4E2D7R19R13 post-acceptance boundary evidence

Date: `2026-08-24`

Status: `PASS / POST_ACCEPTANCE_OUTER_COMPLETE_NOT_ADMISSIBLE / SHADOW ONLY`

## Outcome

The R12 acceptance candidate completes the current inner solve and outer 5
without another trust solve or any new HVP. Accepted-state stationarity is
`3.1346022438807793e-13`, about 319 times below the existing `1e-10` limit.

The resulting outer state is finite and primal-monotone but not yet globally
admissible. This is a successful boundary result: a future soft total-HVP cap
can finish the in-flight trial and outer update, stop at projected total `523`
and return a structurally complete outer-boundary state. It does not need an
inner-CG checkpoint ABI for this exact boundary.

## Parent and target closure

```text
R13 identity SHA-256  e7ec6101fa06454082ca70a6bf799850146f884076b285d14978206bead0ad34
R12 stdout SHA-256    e57ba96aca2c3114ea2c9c10fa2728d8c91dd7ac7969010f381a245219175cee
R12 semantic          3e08a082789053fb5209c4622b391db647328ee9f6582612fd936d8b9ec30f18
accepted trial root   58dadefd7426e9f80188b694557a02efcaf005dd67abfff5e337b90f214be5a8
outer-start root      1623b7fea06a28b24f11b76e2b54be8774d8278ed0a6dc17a0c14ffee7ccfacd
parent transaction    c6a03d43a04ba00cef80b618bb983ab23f59e27d79ad073f327f119b145660cf
```

R11/R10/R9/R8/R7/R6/R5/R2 remain transitively exact through R12. The
outer-start state is not inferred: it matches the recorded outer-5 trial-0
current position, and trial 0's result matches the R10/R11 captured current
position before trial 1.

## Inner boundary

```text
stationarity       3.1346022438807793e-13
stationarity bits  0x3d560ecc22573c55
limit              1e-10
fraction of limit  0.003134602243880779
gradient root      ba8d353879e7b9a01731c06ceae2ea5f1fd1eab35c90dff040c737c8ad7c31d6
new trust solves   0
```

The accepted workspace therefore returns through the existing inner
stationarity exit. R13 then releases it and performs the current implementation's
outer-final workspace rebuild. The two normalized evaluations are bit-exact.

## Outer-5 boundary

```text
previous primal       2.9913721055763176e-8
outer-5 primal        2.6313490275597928e-8
dual change           2.6313490275597928e-8
complementarity       6.8415291768609504e-15
position update / dx  1.1345542950030735e-8
minimum / maximum u   0 / 2.6000082486987708e-7
monotone              true
admissible             false
```

Outer-state root is
`5dd9a07d60cbfe851bdc4c383944a1eb8042f178a821f4a546da33101dd0a19a`;
updated-dual root is
`f4279bde29358f65b17b15ad7456e8c5743b44ec99fd3612d78cb5f905b00aca`.

The state is not admissible because primal and dual change still exceed their
limits, and position update is slightly above `1e-8`. Stationarity and
complementarity already pass. No threshold is changed or reinterpreted.

## Work and rollback

R13 builds/releases two sparse workspaces sequentially, so maximum live is
one. It consumes zero HVP, model HVP, trust solve, trial, divided reduction,
precision audit or all-pair candidate call. Static binding and both workspace
evaluations are exact.

R12 trial, parent transaction, current/predicted/dual/theta/static inputs,
projected HVP accounting and all parent roots are unchanged. The shadow outer
state and dual are not committed. No later outer update, substep, macro,
trajectory or timing runs.

## Reproducibility

Contract commit:
`d87a9e49` (`research: freeze post-acceptance boundary`).

Implementation commit:
`44b4fa13` (`research: classify post-acceptance boundary`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r13-a.LWfIoh`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r13-b.YJGWNW`.

Both binaries are `6,153,696` bytes, have SHA-256
`f2024b658cef001205db9addd58d70a4743294340fa3118822ff6efc9a17aeff`
and GNU build ID `b1c3c94e6b4f212f71204130c2e8563066146ecb`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r13-a.ABozyb`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r13-b.q1cbjq`.

Both exit `0`, emit empty stderr and reproduce byte-exact:

```text
stdout-with-LF bytes  2,334
stdout SHA-256        0f248c4506303055cbf7a967dd1328a36d8628341a04b6861aec210af17f57f9
semantic result       3a60f64ee09a85b4ea892f12341ff3b8c4e7dcb63bbe4eb4cbebd99c237e5e57
route                 POST_ACCEPTANCE_OUTER_COMPLETE_NOT_ADMISSIBLE
```

## Decision

Retain R13 as proof of a safe completed outer boundary at projected total HVP
`523`. An inner-recurrence checkpoint is unnecessary for this captured case.
Do not yet change the live cap.

Research D7R19R14 as a bounded soft-cap admission and suspension policy:

- `512` is a soft admission boundary for starting new trust solves;
- an already-admitted trial may use a separately bounded atomic reserve;
- the exact observed completion is `10 recurrence + 1 model HVP`, ending outer
  5 at total `523`;
- once the completed outer returns at or above the soft cap, no later outer or
  trust solve may start;
- the private solver must return an explicit resumable outer-boundary status
  and state, distinct from convergence, failure and public commit.

R14 must first freeze the reserve bound, admission points, accounting,
ownership and resume semantics. It may not implement a live policy, continue
outer 6, run a second substep or time performance before that contract exists.

