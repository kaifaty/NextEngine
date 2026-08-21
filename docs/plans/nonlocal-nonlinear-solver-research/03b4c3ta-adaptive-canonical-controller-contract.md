# NSR3-B4C3TA -- complete adaptive balanced canonical controller

Status: `EXECUTED / FAIL_PRESERVED / B4C3TAR_REPAIR_ONLY`

Parent B4C3A1 selects `CANONICAL_BALANCED_STAGE_LEDGER_CANDIDATE`; semantic
SHA-256 is `5b50800cc5427e812cd5637b2952e6f4657803e430203214bf95de3667620367`
and JSON-without-final-LF SHA-256 must equal
`107e56e5be49ebb157b0662767c934ced6addf6f3a7054d696772d6699ee1e90`.

## Identity

```text
identity  joint-pressure-canonical-balanced-adaptive-r0
profile   f57d88c222a8334206a962a72a814bdd530696ed2767c94fa88910281265579c
P1        4128b11190366b45aa6511946cb23b46f254445e5a707ff9cdf7e67a2781aaa6
P2        013a83460fabbded819b7d5d9608747f8bc9548c798d71718603d785c238b3c1
```

The scenario roots are SHA-256 of the exact ASCII strings recorded in the split
research note and bind the full adaptive lane, not a one-frame control.

## Controller transaction

Run all 8 P1 and 16 P2 macro frames. At each committed decoded frame start:

1. evaluate frame-start/forecast spectrum read-only; publish neither;
2. run up to four balanced canonical candidate levels from the same committed
   state and global accepted-step offset;
3. stage global step numbers `committed_count + local_ordinal` and paired ledger
   entries privately;
4. apply the unchanged embedded gate to adjacent levels;
5. atomically append only the selected finer level and advance state/count;
6. destroy every lower/discarded frame and ledger entry.

Frame diagnostics consume the decoded committed fine state and publish nothing.
Require contiguous global steps, exact final decode, at most `192` accepted
substeps per frame and the existing `largest_level <=768` work gate. Forecast
and diagnostics must not mutate committed roots/state/ledger.

P1 frame zero remains `FORECAST_ACTIVE`. For P2 require `INACTIVE_EXACT` through
frame 13, `FORECAST_ACTIVE` at frame 14 and `START_ACTIVE` at frame 15, plus zero
precontact pressure violations. A deviation is a meaningful local-noise result,
not an automatically loosened schedule gate.

## Frozen binary envelope

Use the complete B4C2T binary adaptive lane as an independent oracle. At each
macro frame let `S` be the larger accepted-substep prefix and `T` elapsed time.
With `q=1e-6 m` or `m/s`, require:

```text
RMS position <= min(0.05*dx, 8*S*q*(1+T))
RMS velocity <= min(0.001*c, 32*S*q)
center        <= 0.05*dx
q99 height/front <= 0.10*dx
kinetic relative <= 0.15, with the existing near-zero floor rule
```

Terminal contacts remain exact. Contact-onset time differs by no more than the
larger accepted coarse step at the onset frame plus 64 epsilon. Retain finite,
mass, density/speed, support/contact closure and capacity gates. Canonical
precontact rigid-flight position/velocity and velocity-spread allowances use
the analytical accumulated local `<1`-unit bound; pressure activation remains
strictly forbidden before the forecast onset.

## Publication ledger and energy

Commit only fine B4C3A1 entries. Require every compensated residual `<=1e-9`,
all per-step aggregate/kinetic/gravity/decomposition gates, and cumulative
impulse within the exact sum of per-step `m*sqrt(3)*0.5e-6` bounds.

For each lane define the independent energy scale

```text
E_scale = max(abs(initial mechanical), N*m*abs(g_y)*dx, 1e-12 J).
```

Require committed cumulative absolute pressure publication delta and absolute
mechanical publication delta each `<=0.01*E_scale`. Report utilization; do not
apply this budget to discarded levels.

## Rollback and identity

After one real committed macro frame, inject a failure after two private
substeps of the next candidate. The committed state, frame/ledger roots,
global step count and cumulative ledger totals remain bit-exact, with no gap.

Two complete reports must be byte-identical. B4C3A1/B4C3Q/B4C3A/B4C2T raw
reports remain exact.

PASS selects `CANONICAL_BALANCED_ADAPTIVE_CONTROLLER_CANDIDATE` and authorizes
only B4C3TR fixed-reference design. FAIL preserves B4C3A1 and blocks complete
canonical continuation. No fixed canonical reference, nominal, CUDA, runtime,
schema or production authority is granted.

## Executed outcome

The first execution failed on P1 frame four because a 16-substep candidate hit
`KKT_SOLVE:REJECT_LIMIT`; see the
[dated evidence](../../development/nonlocal-nsr3b4c3ta-adaptive-canonical-evidence-2026-08-21.md).
The exact failed state passes at 32 and 64 substeps and their unchanged embedded
gate passes. P2 also exposed an implementation mismatch: this contract froze a
local canonical precontact allowance, while the harness required exact-zero
velocity error. These results do not change this contract or convert its FAIL
to PASS. They authorize only a separately frozen B4C3TAR repair discriminator.
