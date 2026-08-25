# NSR3-B4E2D7R19R62 restoration-promotion research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / ATOMIC EXIT CANDIDATE SELECTED / CONTRACT NEXT`.

## The state transition is not `R43 + witness`

R61 closes the binary64 certificate for the exact R58 witness. The witness is
dimensionless and the corresponding physical vector would be
`SPACING * witness`, but applying it to the particle state would be the wrong
restoration transaction.

The frozen R48 problem is the compatibility problem relinearized *at* the R43
projected trial:

```text
y = exact R43 projected trial
A = SPACING * Jc(y)
c(y) + A*d <= 0
||d|| <= 0.03125
```

Thus `d` is the normal step for the next TRQP. It proves that `y` is a valid
restoration output; it is not the restoration displacement that produced
`y`.

This ownership is also required by Fletcher et al., Algorithm 2.1. On failed
compatibility the current iterate is inserted into the filter, restoration
produces an acceptable `x_k+r_k` whose next TRQP is compatible, and that point
becomes `x_(k+1)`. The normal step used to prove compatibility is retained as
`n_(k+1)` for the following iteration rather than immediately added to the
restoration point.

The exact promotion is therefore:

```text
iterate.position       <- R43 projected trial
filter                 <- old filter + pre-restoration source entry
trust.radius           <- 0.0625
cached_normal.witness  <- exact R58 dimensionless witness
cached_normal.operator <- fresh topology-owned R61 identity at R43
phase                  <- ordinary TRQP ready
```

The original R30 position, R43 trial and R58 witness remain immutable inputs
until every gate succeeds. No particle receives `SPACING*witness` in R62.

## Why R43 can be the restoration output

The required properties now exist separately but have never been joined in
one transaction:

1. R46 proves that the R43 point is acceptable against the filter containing
   the pre-restoration source. It passes the feasibility branch even for the
   strongest audited dyadic margin `gamma=1/2`.
2. R43/R46 prove fresh nonlinear violation reduction, source-to-trial contact
   non-worsening and stable-support coverage.
3. R61 proves, in binary64 and from topology-owned row degree, that the cached
   next normal witness satisfies every linearized density row at R43.
4. R48--R58 prove that the witness lies within the reserved half-radius and
   contact component box. R62 must recheck these geometric facts at the
   promotion boundary rather than inherit them implicitly.

Composite augmented-Lagrangian merit is deliberately not an exit gate. R45
already proved that the feasibility-improving R43 step raises that scalar
merit slightly; the filter exists precisely to keep physical objective and
constraint violation as separate coordinates. Fresh `f` and `h`, not the
rejected composite scalar, own promotion.

## Selected transaction architecture

Use a copy-on-write, report-only transaction object. Its candidate payload
contains the exact position, canonical one-entry filter, selected next trust
radius, cached dimensionless normal witness and identities of its nonlinear
workspace/topology/certificate. Publish the payload only after all gates pass;
otherwise publish no candidate and require all transitive source roots to be
unchanged.

R62 performs the following new checks:

1. replay exact R61 through a passive capture; do not duplicate or replace its
   audit owner;
2. rebuild the nonlinear workspace at the exact R43 point;
3. build a fresh superset anchored at R43 for ownership by the following
   iteration and prove its filtered support equals the workspace topology;
4. recompute nonlinear `f`, `h`, `psi` and the conservative `gamma=1/2`
   source-filter admission;
5. repeat all source-to-R43 contact tests;
6. derive contact intervals at R43, prove the cached witness is inside every
   interval and its global L2 norm is at most `0.03125`;
7. run the R61 topology-owned audit on that same workspace and require zero
   row-local positive uppers;
8. construct and root the filter, trust, cached-normal and complete candidate
   payload, then exercise fail-closed transaction controls.

R61 currently has no replay capture. Add a passive `impl(capture*)` wrapper,
requiring its public output to remain byte-exact. The capture owns references
needed by R62; it adds no work or authority to R61.

## Predeclared outcomes

```text
RESTORATION_EXIT_TRANSACTION_CANDIDATE
RESTORATION_EXIT_FILTER_REJECTED
RESTORATION_EXIT_TOPOLOGY_REJECTED
RESTORATION_EXIT_CONTACT_REJECTED
RESTORATION_EXIT_NORMAL_REJECTED
```

Harness/parent/source/work/rollback/atomic-publication defects remain separate
fail-closed routes. Only the candidate route may publish a private transaction
payload. Even that route does not mutate runtime state, execute the cached
normal step, compute a tangential step, update multipliers/Hessian, run the
next outer, admit timing or claim production readiness.

## Consequence for the next stage

If R62 passes, restoration is complete at the research transaction boundary.
R63 should consume the cached normal witness exactly once while constructing
the next tangential step and globalization decision. If any nonlinear,
topology, contact or certificate identity disagrees, restoration remains open
and R63 is forbidden.

## Primary source

- Fletcher, Gould, Leyffer, Toint and Waechter,
  [Global Convergence of a Trust-Region SQP-Filter Algorithm for General
  Nonlinear Programming](https://www.numerical.rl.ac.uk/media/people/nick-gould/FletGoulLeyfToinWach02_siopt.pdf),
  Algorithm 2.1 and the restoration discussion following it.

