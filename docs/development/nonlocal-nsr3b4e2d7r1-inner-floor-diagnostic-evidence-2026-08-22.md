# NSR3-B4E2D7R1 inner-floor diagnostic evidence

Date: `2026-08-22`

Status: `FAIL / TOPOLOGY_GATE / REPLAY_ONLY`

## Reproducibility and stop boundary

Two independently configured Release build directories produce byte-identical
4,701,544-byte executables at SHA
`dd59c8edb2434a1fc75e856c0bfb03c6f2f3d4af51790a6eadbd4564c9c0c802`
and Build ID `d9b5d29e780ecccf0bc7f16667977ddc75f9eb9c`.

Both fresh processes exit one with empty stderr and byte-identical 6,367-byte
stdout reports at SHA
`9a9582d09897f63ed7cd9953cc2fb6153b4266813fb6797ba71a1de98140cbfe`.
The semantic result is
`3ba8ab9c999a63b72fa42ca4dad98062aea976977945fa16a60ba49bfcc6c51a`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r1.7XHdvU`.

The same final executable preserves both parent reports exactly:

```text
D7  stdout SHA = e0542abc4e0c7ff0b38acc0fe38095e270dec030617ad5183665414ce9b3db11
D7R stdout SHA = 4b0272df2d46bb53ec09175ea7095b0e8c699ac9e0e4e093c6e20dfd00240801
```

## Passing controls

- identity, exact D7 prefix roots and post-outer-9 forced-private root match;
- the parent failure reproduces as `INNER:REJECT_LIMIT` with ten outer
  records;
- all initial and trial diagnostics are finite;
- exactly nine replay-only trust trials reject, with zero accept, zero commit
  and exact public rollback;
- both builds and both process reports are byte-identical;
- no trajectory, timing, runtime or production authority is admitted.

## Direct merit result

The failing inner state is only marginally outside the frozen inner threshold:

```text
scaled stationarity = 1.0481187440427683e-8
gradient norm       = 1.0672297723371429e-5
total energy        = 0.0053999999999956002 J
energy ULP          = 8.6736173798840355e-19 J
```

All nine rejected trials repeat the same unconstrained Newton step because the
shrinking trust radius does not bind it:

```text
step norm / dx             = 4.9587280227513481e-9
predicted reduction        = 1.3230255447006828e-15 J
predicted reduction / ULP  = 1525.3446016296045
raw actual reduction       = -4.5363018896793506e-16 J
factored actual reduction  = -4.5281433411232910e-16 J
raw ratio                  = -0.34287334117238299
```

The independently factorized per-term difference agrees with the raw total
subtraction that the trial increases the objective. Therefore loss of descent
through cancellation in `E(current)-E(trial)` is falsified for this state.
Replacing the subtraction or merely tightening inner accuracy is not
authorized by this evidence.

## Failing boundary

Every trial reports `topology_exact=false`. The frozen contract also contains
an explicit gate requiring every reported active set and topology to match the
replay initial state. The first implementation candidate classified this as a
research route, but review caught that it had not enforced the literal gate.
The final executable fails closed with `first_failure=TOPOLOGY_GATE` and grants
no remediation authority.

The useful observation remains non-authoritative evidence for the next
contract: a positive local quadratic model predicts descent while both direct
and raw objective differences show ascent across a topology-changing step.
Moreover, the final radius after nine rejects is
`4.7683715820312503e-8 m`, still far above the approximately
`2.4793640113756742e-10 m` physical step norm. The reject cap therefore repeats
the same non-descent proposal before the radius can constrain it.

## Decision

Preserve D7R1 as hard FAIL rather than weakening its frozen gate after seeing
the data. Freeze one new replay-only topology/step discriminator. It must name
the changed active-center, fluid-pair and boundary-pair sets, inspect distances
to the compact-support horizon, evaluate a fixed alpha ladder along the same
Newton direction, and compare live-topology with fixed-current-topology
objective differences. Only that experiment may decide whether the next work
belongs to horizon derivative reclosure, trust rejection policy, active-set
semantics or the AL Hessian model.
