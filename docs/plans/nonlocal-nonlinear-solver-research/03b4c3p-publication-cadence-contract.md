# NSR3-B4C3P -- publication cadence discriminator

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / ADAPTIVE_REDESIGN_BLOCKED`

B4C3TR is the exact negative parent/control. Its isolated JSON-without-final-LF
SHA-256 is
`fbdce8bc1420b38c813f82a29bdae48cdb3e130d5d45c08d1077c56313d360`
and semantic SHA-256 is
`bf92622cb719574b5d13a5bdae06b18adefee95dd34da1781bada1091f27e02f`.
B4C3TAR2 remains the selected positive parent.

## Identities

```text
experiment       03d21501a481c9d0eddd3df53ff94bb199a3517b69d4bfc266fa6f6022047a8a
profile          e713a61649fc230b189fca9eda3628b69f9369c706df35f0a2b080a4bd189a70
macro ledger     39bac5938564c73daf7441d48aa8f58d5d9072e98234abcb0a528a800c096e21
P1 scenario      4128b11190366b45aa6511946cb23b46f254445e5a707ff9cdf7e67a2781aaa6
P2 scenario      013a83460fabbded819b7d5d9608747f8bc9548c798d71718603d785c238b3c1
levels           48, 96, 192 private substeps per macro frame
```

Experiment identity text is exactly:

```text
nextengine.nonlocal.publication-cadence|v1|private-substeps|balanced-macro-boundary|levels=48,96,192
```

The new profile binds unchanged integer microunits and aggregate-balanced
apportionment to macro-boundary-only publication. The new ledger policy binds
publication-compensated macro sum-scale residual at `1e-9`. Neither identity
is a runtime/schema commitment.

## Frozen candidate

For every P1/P2 macro frame and each fixed level:

1. copy the committed decoded position/velocity into a private transaction;
2. execute the unchanged joint-neighborhood constrained KKT interval for
   exactly `S` binary64 substeps, with every existing substep physical ledger;
3. balanced-publish only the final private state as canonical macro step
   `frame+1` using the new profile;
4. build and admit exactly one macro publication ledger entry;
5. atomically commit decoded state, frame, entry and cumulative totals.

The next macro frame consumes the decoded macro state. No private substep is a
canonical frame. Require exactly `macro_frames` committed frames/entries per
lane, contiguous steps and exact final decode.

## Macro publication ledger

For macro-start momentum `p0`, private binary result `pb`, decoded result `pq`
and aggregate interval impulses:

```text
Lsolver = pb - p0 - Jgravity + Jsupport + Jcontact
Iq      = pq - pb
Lraw    = pq - p0 - Jgravity + Jsupport + Jcontact
Lcomp   = Lraw - Iq
scale   = max(|pb-p0| + |Jgravity| + |Jsupport| + |Jcontact|, 1e-30)
```

Require finite fields, `Lcomp` equal to `Lsolver` within a computed binary64
forward bound, normalized compensated residual `<=1e-9`, explicit impulse and
center reports within their balanced-quantization bounds, and exact
kinetic/pressure/gravity mechanical-energy decomposition. Retain strict
max-scale residual as a finite diagnostic. Report macro legacy and policy
roots; the policy root binds the new profile/policy identities and all fields.

## Same-level binary tube

Use the independent fixed binary64 lane at the same `S`. At every macro frame
with publication prefix `P=frame+1` and elapsed `T=P/240 s`, require:

```text
position RMS <= min(0.05*dx, 8*P*1e-6*(1+T))
velocity RMS <= min(0.001*c, 32*P*1e-6)
```

Require terminal contact IDs exact and P2 first-contact time within
`1/(240*S)+64*epsilon`. Report maximum utilization and worst frame.

## Convergence and physics

The independent binary `48/96/192` reference must pass its inherited
order-or-binary64-floor rule. Candidate final position and velocity differences
must independently show positive ratio `[1.25,2.75]` or overlap pair floors
formed from the two macro-publication envelopes plus computed binary64 floor.
Label the selected branch exactly.

Retain B4C3TR/B4C3TAR2 count, capacity, contact, KKT, support, P1
center/density/speed, P2 precontact, canonical geometry, publication energy and
mechanical-energy gates. Only publication count replaces private substep count
in representation-derived bounds.

## Atomicity, work and execution

After one committed macro frame, inject failure after a complete private
interval but before publication commit. Public state, step, macro frame,
ledger, roots and cumulative totals remain exact; private work is reported.

Run the six candidate lanes as independent parallel jobs with deterministic
P1/P2 and `48/96/192` report order. Report all private substeps, KKT HVPs,
workspace work and eight/16 publication operations. Two isolated candidate
reports must be byte-identical.

## Decision boundary

PASS selects `MACRO_BOUNDARY_CANONICAL_PUBLICATION_CANDIDATE` and authorizes
only a new adaptive macro-transaction design. B4C3TC, nominal, B4C4/B4D, CUDA,
runtime/schema and production remain blocked. FAIL stops this repair line;
neither tolerance widening nor silent loss of durable roots is authorized.
