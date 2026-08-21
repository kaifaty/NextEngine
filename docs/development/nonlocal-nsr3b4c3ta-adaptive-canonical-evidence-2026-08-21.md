# NSR3-B4C3TA adaptive balanced canonical controller evidence

Status: `FAIL PRESERVED / REFINEMENT RECOVERY ESTABLISHED / REPAIR REQUIRED`

Date: `2026-08-21`

This stage executed the frozen complete adaptive P1/P2 canonical-controller
contract. It does not authorize the fixed canonical reference, nominal corpus,
runtime integration or production use.

## Reproducible result

Command:

```text
nonlocal-formula-reclosure --canonical-adaptive-self-test
```

The first report is intentionally preserved as a negative result:

```text
status                 FAIL
first failure          p1-supported-adaptive-balanced:ADAPTIVE_LANE
candidate failure      FRAME_CANDIDATE:KKT_SOLVE:REJECT_LIMIT
raw JSON + LF          e16d24f4e0028c624a4f241d571d65859e59655ab148193a08adfb9a697ddc65
raw JSON without LF    52d9c1c9cce975927c7230ee5f68e7871ab6a345a3b227f8467d6f8ce2d8c377
semantic result        ed95592f3c493089676ae09ffb7d39a2dc3c735ffcf4a114b57b2b190ca4e3c9
wall time              72.73 s
maximum RSS            8,696 KiB
```

P1 committed four complete macro frames and 152 canonical substeps before the
next frame-start candidate at 16 substeps exhausted the unchanged nonlinear
reject limit. No part of that failed candidate was committed. The committed
frames used `21/42`, `21/42`, `18/36` and `16/32`; their maximum compensated
publication-ledger residual was `4.7364e-10`. Cumulative absolute pressure and
mechanical publication deltas were `2.9958e-4 J` and `3.0121e-4 J`.

P2 completed all 16 frames and retained the exact pressure-onset schedule,
terminal contacts, contact time, binary envelope and publication-energy
budgets. It accepted/executed `82/124` substeps; position and velocity bound
utilizations were `0.0331` and `0.1456`, while pressure and mechanical budget
utilizations were `0.0215` and `0.0220`. Post-commit rollback also passed with
all 42 committed frame and ledger roots unchanged after two private staged
entries.

## Failed-level refinement probe

The P1 failure was replayed from the exact committed frame-four state without
the parent-chain cost:

```text
command                 --canonical-adaptive-failure-probe
status                  PASS (diagnostic only)
raw JSON + LF           30fde05d9e331d73c4899317f7153d9d30e2411830ee09410d6662ce19ddc158
raw JSON without LF     522aa97ecbb3016bb9ff70d7c6675044e8c2de3dcb15ee76d607f15018295078
semantic result         c6767c992f489ec75d022a258c07eb03bd44bb406dcc57cefac5541b8c003c83
wall time               4.57 s
maximum RSS             7,056 KiB
```

| Level | Substeps | Solve | Outer trials | HVP calls |
|---:|---:|---|---:|---:|
| 0 | 16 | `REJECT_LIMIT` | 50 | 90 |
| 1 | 32 | PASS | 152 | 266 |
| 2 | 64 | PASS | 256 | 386 |
| 3 | 128 | PASS | 500 | 744 |

The unchanged 32/64 embedded gate passed at `0.000364 dx`,
`8.16e-6 c` and `9.67e-4` relative kinetic error. The 64/128 gate also passed.
Therefore the failed coarse solve is a recoverable discretization/refinement
signal, not evidence that the pressure formula or KKT conditions are invalid.
The original controller nevertheless aborted immediately, so B4C3TA correctly
remains FAIL under its implemented transaction policy.

## P2 physical-gate probe

An isolated report made every previously aggregated precontact quantity
observable:

```text
command                 --canonical-adaptive-p2-probe
status                  PASS (diagnostic complete; case remains FAIL)
raw JSON + LF           b117fa6b81bf951c1bd0282a8499590ddb02cb688238809cb85340f7d66cdf0b
raw JSON without LF     2cb51b5eae3d02dbfb783c0f672600a922eb4ce6ba55ae1baca26981ca51a3d9
semantic result         c4d47cd5a84adf6b5e1bee5373d37b4ce1237f7abb87480cb2ac3540b435f665
wall time               0.85 s
maximum RSS             4,960 KiB
```

The only exposed mismatch with the released-block case gate is a
`1.9246e-7 m/s` precontact velocity error. Position error and support reaction
are exactly zero; velocity spread is `6.667e-6 m/s`, below its accumulated
`44e-6 m/s` bound. The contract required the analytical local canonical
allowance, but the implementation accidentally required velocity error to be
exactly zero. A repair may restore the already-frozen `<1e-6 m/s` local bound;
it may not loosen pressure activation, contact, ledger or energy predicates.

## Decision

Preserve the B4C3TA FAIL and introduce a separately frozen B4C3TAR repair:

1. only `KKT_SOLVE:REJECT_LIMIT` may turn a failed candidate level into a
   refinement signal;
2. failed levels remain private and contribute exact attempted-work evidence;
3. selection still requires two adjacent passing levels and the unchanged
   embedded gate;
4. the released precontact position/velocity comparisons use the frozen local
   `<1` canonical-unit bounds;
5. every other stage, publication, capacity, identity and physical failure
   remains fatal.
