# NSR3-B4C3TR -- fixed canonical reference

Status: `EXECUTED_FAIL / B4C3TC_BLOCKED / CADENCE_RECLOSURE_ONLY`

Parent B4C3TAR2 selects
`CANONICAL_BALANCED_ADAPTIVE_RECOVERY_KKT_LEDGER_CANDIDATE`; its
JSON-without-final-LF SHA-256 is
`5862a1c9a56414d1fb4809bcc08e3e5decfe059679defb974e05890909b8dc3d`
and semantic SHA-256 is
`ae52a97a6c0bd6fe5cacd7746d3131a7f45ebc1e9a79e231e665e1e87cb777f7`.

## Identity and unchanged inputs

```text
identity sha256  52bcd5bc908ea9b2afb36e15248bfe5a623e2b5b00f90e60e3fdddb2ec624b13
identity text    nextengine.nonlocal.fixed-canonical-reference|v1|balanced-kkt-ledger|levels=48,96,192|parallel-independent-lanes
representation  f57d88c222a8334206a962a72a814bdd530696ed2767c94fa88910281265579c
ledger policy   b1136c2c3dc7970cbee0ce67b0d129b849ed8957268bb7281063af9e60200e2f
P1              4128b11190366b45aa6511946cb23b46f254445e5a707ff9cdf7e67a2781aaa6
P2              013a83460fabbded819b7d5d9608747f8bc9548c798d71718603d785c238b3c1
levels          48, 96, 192 substeps per macro frame
```

Reuse all P1/P2 particles, coefficients, macro horizons, KKT solver,
single-pass joint neighborhood, compact pressure tape, B4C3Q balanced
publication and B4C3L KKT-scale ledger admission exactly. No recovery,
coefficient, tolerance, quantizer or solver change is authorized.

## Independent fixed lanes

Run all three fixed levels for all eight P1 and sixteen P2 macro frames. Each
lane starts from the immutable fixture. At every substep:

1. solve the unchanged constrained KKT step;
2. publish and decode one balanced canonical frame;
3. admit the publication using the KKT-scale ledger policy;
4. consume only that decoded state in the next substep.

Within a lane require global step `frame*S+local_step`, exactly
`macro_frames*S` committed frames and ledger entries, and final state equal to
the last decoded frame. Report canonical trajectory, legacy ledger and policy
ledger roots for each lane. A failed macro stage commits nothing from that
stage and stops only its lane.

## Per-lane binary tube

Run an independent binary64 fixed lane at the same `S`. At every macro-frame
boundary compare mass-weighted RMS position and velocity. With total committed
prefix `N=(frame+1)*S`, elapsed `T=(frame+1)/240 s`, `q=1e-6`, freeze:

```text
E_x(N,T) = min(0.05*dx, 8*N*q*(1+T))
E_v(N)   = min(0.001*c, 32*N*q)
```

Require position `<=E_x` and velocity `<=E_v`; report maximum utilization and
worst frame. Require terminal contact IDs equal to the corresponding binary
lane and P2 contact-time error `<=1/(240*S) + 64*epsilon`.

## Temporal convergence and representation floor

The binary64 `48/96/192` references must pass the inherited final-state rule:
each position and velocity pair has positive ratio `[1.25,2.75]`, or both
successive differences overlap the existing computed binary64 floors.

Compute canonical final RMS differences for `48->96` and `96->192`. For each
field independently accept either:

- positive ratio in `[1.25,2.75]`; or
- both differences no greater than their pair representation floors.

For levels `a,b` at final elapsed time `T`, the pair floors are:

```text
F_x(a,b,T) = E_x(a*frames,T) + E_x(b*frames,T)
             + b4b_rms_floor(C_a.position,C_b.position)
F_v(a,b)   = E_v(a*frames) + E_v(b*frames)
             + b4b_rms_floor(C_a.velocity,C_b.velocity)
```

Report order/floor branch and maximum floor utilization. Never label a floor
overlap as an observed convergence order.

## Unchanged physical and energy gates

Each canonical lane must satisfy:

- exact fluid count/mass, finite aggregates and inherited capacity limits;
- contact and pressure activation, decoded penetration
  `<=1e-6 m + 64*epsilon`, support closure `<=1e-10` and compensated KKT
  ledger residual `<=1e-9`; the underlying constrained solve retains its
  unchanged contact feasibility gate;
- finite strict residual diagnostic and exact publication correspondence;
- P1 lateral drift
  `<=committed_steps*0.5e-6/fluid_count + 1e-10 m`, vertical center change
  `<=0.05dx`, density strain `<=1e-3` and speed `<=0.01c`;
- P2 canonical precontact free-flight bounds `<1e-6 m` and `<1e-6 m/s`,
  velocity spread/support-reaction/pressure gates, plus terminal pressure and
  contact activation;
- no mechanical-energy creation above `1%` of the independent scale plus the
  explicitly measured cumulative absolute publication mechanical delta;
- cumulative absolute publication pressure and mechanical deltas each
  `<=1%` of the independently defined case energy scale.

No discarded/failed stage contributes physical or energy totals.

## Atomicity, work and scheduling

After one committed fixed-48 macro frame, inject failure after two private
substeps of the next frame. State, global count, cumulative totals and
canonical/legacy/policy roots must remain bit-exact.

Report all nonlinear HVPs, trials, rejects, pairs and committed/attempted
substeps per lane. The six lanes may run concurrently as fixed independent
jobs; report order is always P1 `48/96/192`, then P2 `48/96/192`. Concurrency
must not alter any lane field or hash. Maximum participants remain 1,280,
active pairs `160*fluid_count`, and fixed substeps exactly the declared level.

## Historical, repeatability and decision boundary

Reproduce the complete B4C3TAR2 report at its exact parent hash. Run isolated
B4C3TR lanes before two complete parent-gated reports; the two complete reports
must be byte-identical.

PASS selects `CANONICAL_FIXED_REFERENCE_CANDIDATE` and authorizes only B4C3TC
adaptive-versus-fixed comparison design. FAIL preserves B4C3TAR2 and all
negative evidence. Nominal, B4C4, B4D, CUDA, runtime/schema and production
authority remain blocked.

## Recorded decision

The isolated B4C3TR report fails twice byte-identically at raw JSON-with-LF
`b04a7c93e3b90c67950b1460c48c99236f5cdc2829be4aabc010afa664b351be`
and semantic result
`bf92622cb719574b5d13a5bdae06b18adefee95dd34da1781bada1091f27e02f`.
The KKT/ledger lanes pass, but per-substep canonical publication makes contact
phase and same-level velocity error grow under refinement. See the
[dated evidence](../../development/nonlocal-nsr3b4c3tr-fixed-reference-evidence-2026-08-21.md).
Preserve the FAIL and authorize only a separately frozen publication-cadence
discriminator. Full parent-gated B4C3TR replay was not executed.
