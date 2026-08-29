# NSR3-B4C3T full canonical controller split -- 2026-08-21

Status: `COMPLETE / B4C3TA_FIRST / B4C3TR_AND_B4C3TC_BLOCKED`

## Why the full stage is split

A monolithic B4C3T would combine three independent hypotheses:

1. adaptive frame/level transactions remain deterministic under balanced
   canonical continuation;
2. canonical fixed `48/96/192` lanes remain convergent;
3. canonical adaptive lanes remain physically close to their canonical fine
   references.

A failure would not say whether the cause was adaptive schedule drift, fixed
temporal convergence or reference comparison. It would also repeat every long
lane during diagnosis. The selected decomposition is:

```text
B4C3TA  complete adaptive P1/P2 lanes, global roots, rollback, binary envelope
B4C3TR  complete canonical fixed 48/96/192 lanes and convergence
B4C3TC  adaptive-versus-fixed canonical comparison and final case gates
```

Each stage has a separate result hash and can preserve negative evidence.

## Pre-frozen adaptive error envelope

B4C3A1 gives independent one-frame balanced-versus-binary evidence. P1 fine
uses 42 accepted substeps and differs by `5.30e-6 m` and `7.66e-4 m/s`.
Relative to `S*q` with `q=1e-6`, observed amplification is below one for
position and below 19 for velocity. Before any full-lane execution, freeze
power-of-two safety factors `8/32`:

```text
B_x(S,T) = min(0.05*dx, 8*S*q*(1+T))
B_v(S)   = min(0.001*c, 32*S*q)
```

`S` is the larger canonical/binary accepted-step prefix and `T` is elapsed
physical time. These bounds reuse existing B4B physical ceilings and B4C3A1
evidence; they are not derived from the future B4C3TA result.

The cumulative absolute pressure and mechanical publication-energy budgets are
each `1%` of the existing B4B physical energy scale. This is deliberately broad
but independent; the report must expose actual utilization.

## Decision

Freeze and execute B4C3TA first. B4C3TR, B4C3TC, B4C4, nominal, runtime and
production remain blocked.
