# B4C3P publication cadence reclosure research

Status: `COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Hypothesis

B4C3TR shows that a fixed canonical quantum applied after every solver substep
becomes part of the time-integration model. Doubling substeps doubles the number
of perturbation injections per physical second, so a nominally finer reference
can move farther from the binary64 solution.

The narrow candidate is:

```text
durable macro state
  -> S private binary64 KKT substeps
  -> one balanced canonical publish/decode transaction
  -> next durable macro state
```

The physical solver, `S=48/96/192`, macro duration, quantization quantum and
balanced apportionment do not change. Only publication cadence and therefore
durable step identity change.

## State ownership

Private substeps own no canonical roots and no publication ledger entries.
They retain all existing KKT/contact/momentum checks and exact work counters.
The macro transaction owns:

- one frame step `macro_index+1`;
- one balanced canonical root;
- one explicit publication impulse and center shift;
- one macro publication-energy decomposition;
- one legacy-style diagnostic ledger root and one new macro-policy root.

A solve or publication failure commits none of the macro state, roots, ledger
or cumulative publication totals.

## Macro ledger semantics

Let `p0` be committed macro-start momentum, `pb` the private binary64 result,
`pq` the decoded publication and `Jext` the aggregate gravity, pressure-wall and
contact-wall impulses reported by all private substeps:

```text
L_solver = pb - p0 - Jext
I_q      = pq - pb
L_raw    = pq - p0 - Jext
L_comp   = L_raw - I_q = L_solver
```

Gate every private substep with the unchanged KKT ledger. Gate the compensated
macro ledger using the sum scale
`|pb-p0| + |Jgravity| + |Jsupport| + |Jcontact|`, threshold `1e-9`. The strict
max-scale value remains a finite diagnostic. Because there is no single macro
KKT solve, the policy gets a new identity rather than pretending to be B4C3L's
per-substep correspondence check.

## Discriminator

The current B4C3TR report is the exact negative control. Run macro-boundary
candidate P1/P2 fixed `48/96/192` lanes and the same independent binary64
references. Require:

1. complete private KKT physics and exact macro transaction/rollback;
2. candidate versus same-level binary state inside the frozen representation
   envelope using publication count, not private-substep count;
3. terminal contacts and one-fixed-step event-time agreement;
4. binary64 first-order convergence plus canonical order or an explicitly
   bounded macro-publication floor;
5. publication energy budgets, physical gates and byte repeatability.

For macro prefix `P=frame+1`, reuse the pre-frozen form:

```text
E_x(P,T) = min(0.05*dx, 8*P*q*(1+T))
E_v(P)   = min(0.001*c, 32*P*q)
```

Using publication count is a semantic correction, not a fitted result: the
only representation transition now occurs once per macro frame.

## Decision

PASS may select macro-boundary durable publication and authorize only redesign
of the adaptive canonical transaction under that cadence. It cannot revive
B4C3TC directly because the current adaptive trajectory uses per-substep
publication. FAIL preserves B4C3TAR2 as the last positive boundary and stops
canonical trajectory research for a broader representation redesign.
