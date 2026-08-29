# NSR3-B4E2D7R20R63ZN fixed-artifact initial recurrence contract — revision 2

Revision: `FROZEN / AUTHOR_PACKAGE_COMPLETE / INDEPENDENT_REVIEW_PENDING / NO_ENDPOINT_AUTHORITY`.

This is the first consumer of the formally reviewed R63ZM fixed binary
boundary. It does not modify, rebuild or reinterpret R63ZM. It freezes one
smaller question than a complete recurrence because the parent artifact does
not transport the two later direction products.

## Frozen question

Can a standalone fixed-capacity package:

1. admit the exact R63ZM cache, product artifact and checker audit without
   invoking the R63ZM producer or checker;
2. independently parse the original-RHS factor fixture and solve the initial
   preconditioned state `x0`;
3. prove that the computed `x0` is byte-identical to parent role 2;
4. consume the parent role-2 value as the only `H*x0` operator result;
5. independently compute `r0 = b - H*x0`, solve `z0 = M^-1*r0`, compute
   `rho0 = r0^T*z0` with the frozen Dot2 enclosure, and require its finite
   exact lower bound to be positive; and
6. publish a fixed receipt whose semantic roots, ordered events, work ledger,
   route and terminal seal are independently reconstructed by a separate
   checker?

Success admits only this initial binary128 recurrence prefix on one frozen
cache. It does not admit `H*p0`, `H*p1`, either PCG update, any state
certificate or a complete recurrence.

## Parent closure and immutable inputs

The only valid baseline inputs are:

| Input | Size | SHA-256 |
|---|---:|---|
| parent cache | `1033625` | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| R63ZM artifact v3 | `12916` | `ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87` |
| R63ZM checker audit v3 | `420` | `fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80` |

The R63ZM artifact must additionally expose magic `NER63ZM1`, version `3`,
route `CANDIDATE`, total size `12916`, terminal result
`7eaedbd3775bac423c4e5dfadc25c18f0ffe2ffe181e5080675ff0a457a37da3`,
product-set root
`6ac66c134a3b8eea010de7e66bab209aa995071089eb8b96bd14804e2722a06b`
and exactly six product records. The audit must expose its frozen accepted
route and checker root
`b4a7969c6676b30e87fff470f95e9c098f8951b2a477314213c359c6164c80b4`.

Exact whole-file hashes authenticate every parent byte. The consumer still
parses and checks the named fields before using role 2; the audit is closure
evidence, never arithmetic authority. No R63ZM executable is launched and no
R63ZM route decision is imported dynamically.

The cache is untrusted until the consumer's independent cursor has checked
the fixed cache header, every traversed length/boolean, exact EOF and the
whole-file hash. The selected recurrence payload is:

- dimension `102`;
- original binary128 RHS, `102` components;
- binary64 exported upper factor, `102 * 102` components;
- permutation, `102` indices;
- original binary128 inverse scale;
- baseline solution 0, `102` components, only as a correspondence witness.

The legacy declared roots are not parsing or arithmetic authority. Canonical
raw roots are recomputed from the selected bytes using typed big-endian TLV
material. The factor and permutation roots bind every component/index and
their exact dimensions. The input bundle root binds cache identity, factor,
permutation, inverse scale, RHS, baseline-0 and the exact R63ZM parent result.

## Why only the initial prefix is admissible

R63ZM roles are, in order, projected RHS, original RHS, baseline solutions
0/1/2 and common-projected solution 2. The original-input PCG operator sites
are `H*x0`, `H*p0` and `H*p1`. Role 2 can serve `H*x0` after an independently
solved `x0` matches the role-2 input. No parent role carries `p0` or `p1`.

The consumer must not synthesize a direction product from
`(H*x1 - H*x0) / alpha`. Binary128 state updates are rounded, so linearity of
the mathematical operator does not make that reconstruction bit-exact. The
frozen scalar counterexample is `x0=1`, `p=2^-113`, `alpha=1`: binary128 gives
`fl(x0 + alpha*p) == x0`, hence the state difference is zero while
`alpha*p` is nonzero. The exact observation and source identities are recorded
in the companion research note.

Any implementation that reads parent roles 3 or 4 to fabricate `H*p0` or
`H*p1`, imports cached certificates, calls a formula/core recurrence helper,
or classifies a complete recurrence violates this contract.

## Revision-2 pre-implementation correction

Revision 1 was frozen before code, then a direct trace of the reviewed R63ZC
endpoint exposed an underspecification: R63ZC does not form residuals or
`rho0` with ordinary binary128 subtraction/summation. Residual components are
two-term Dot2 evaluations; `rho0` is a 102-term Dot2 evaluation; and
positivity means `value - bound > 0`. Its triangular solves also use the
frozen compensated `Binary128Accumulator` rather than naive accumulation.

No R63ZN implementation existed when this was found. Revision 2 replaces the
incorrect ordinary-arithmetic wording with the exact inherited numerical
schedule. Inputs, parent identities, role-2 consumption, fixed-prefix scope,
trust boundary and claim ceiling are unchanged.

## Independent arithmetic schedule

Both candidate and checker implement the following schedule separately:

1. convert original RHS and inverse scale from canonical 16-byte binary128 and
   convert every binary64 factor entry exactly to binary128;
2. execute the frozen permuted exported-factor solve for `x0`;
3. compare all `102` canonical `x0` components with cache baseline solution 0;
4. read the `102` role-2 value components at the frozen R63ZM product-record
   offset and recompute their canonical vector root;
5. require role `2`, count `102`, exact/no-underflow flags and frozen role-2
   value root
   `8b6373db6132ee119eff020cb53c01c7287d3d49e70a2d6ad7c387b7dd37dcce`;
6. compute each `r0[i]` in increasing index order as the frozen two-term Dot2
   over `{rhs[i], -hx0[i]}` and `{1, 1}`, retaining value, bound,
   absolute-products and no-underflow result;
7. solve the same factor for `z0`;
8. compute `rho0` with the frozen 102-term Dot2 schedule: two-product for each
   pair, two-sum accumulation in increasing index order, separate correction,
   outward absolute-product accumulation and the frozen binary128 gamma bound;
9. require every input, intermediate, value and bound finite, every Dot2
   `no_underflow`, and require `rho0.value - rho0.bound > 0` exactly, with no
   epsilon, fitted radius or adaptive retry; and
10. seal canonical roots for `x0`, `hx0`, `r0`, `z0`, `rho0`, work, events,
    trace and result before writing the receipt.

The factor solve must use the same frozen mathematical ordering in both
implementations but different source routines. Each row accumulates its
forward or backward dot with the frozen compensated accumulator
`next=sum+term; correction+=(abs(sum)>=abs(term) ? (sum-next)+term :
(term-next)+sum); value=sum+correction`. Per solve the exact structural ledger
is `10302` factor terms and `204` divisions. The accepted prefix therefore
owns exactly:

```text
parent operator products consumed       1
new operator products executed          0
parent product components read        102
factor solves                            2
factor terms                         20604
factor divisions                       408
baseline-x0 component comparisons      102
residual updates                       102
residual Dot2 calls                    102
residual Dot2 terms                    204
rho Dot2 calls                           1
rho Dot2 terms                         102
Dot2 two-products                      306
Dot2 two-sums                          203
certified positivity checks              1
adaptive stops                           0
```

Input reads, parser checks/skips, scalar/component loads, finite checks,
canonical root calls/bytes, comparisons, receipt writes, event construction
and terminal seals are separate named counters. Their exact accepted and
rejection-route values must be frozen in the implementation evidence before
review. Candidate work and checker replay work have different receipts and
must never be added into one shared ledger.

## Fixed-capacity and trust boundary

Package-owned storage is static or fixed-capacity for this exact profile:

- one `1033625`-byte cache slab;
- one `12916`-byte parent-artifact slab;
- one `420`-byte parent-audit slab;
- fixed arrays for factor `10404`, permutation `102`, and five binary128
  vectors of `102` components;
- a fixed event array and one fixed receipt buffer.

No package-owned `vector`, `string`, stream, JSON value, exception-driven
parser or runtime-sized allocation is allowed in the claimed path. POSIX
read/write, binary128 primitives, one fixed byte-oriented SHA-256 primitive
and terminal process exit mapping form the finite TCB. Candidate and checker
may share only format constants, canonical byte encodings and the fixed SHA
primitive; they must not share parser, factor solve, update, dot, route,
work-root, trace-root or result-root implementations.

The consumer may include the reviewed R63ZM fixed SHA header as a cryptographic
primitive. It may not include or link any formula-reclosure core, R63ZJ/K/L
implementation, R63ZM producer/checker source, cache reader, validator,
classifier or work model.

## Receipt and ordered transcript

The candidate publishes one fixed-size binary receipt. Before implementation,
the source must freeze and compile-time assert the complete offset chain for:

- magic/version/size/route and exact/finite/positive flags;
- cache, parent artifact and parent audit sizes/roots;
- input-bundle, factor, permutation, inverse, RHS and baseline-0 roots;
- computed `x0`, consumed `hx0`, derived `r0` values/bounds, derived `z0` and
  scalar `rho0` value/bound/absolute-products roots;
- the complete candidate work tuple;
- ordered event count and fixed event-root slots;
- trace root and one terminal result root.

The successful event order is exactly:

1. `ParentAdmission` — exact identities of cache/artifact/audit;
2. `InputBundle` — selected factor/RHS/baseline semantics;
3. `InitialSolve` — independently derived `x0` and its correspondence;
4. `ParentProduct` — role-2 `hx0` consumption;
5. `InitialResidual` — derived `r0`;
6. `InitialPreconditioner` — derived `z0`;
7. `InitialRho` — derived certified-positive `rho0` enclosure and
   candidate-work root.

The terminal result binds receipt version, route, all three input roots, trace
root, work root and total receipt size. No semantic decision follows the
terminal seal. Failure routes retain the same fixed receipt size and zero all
unavailable root/event slots.

## Routes and first-failure order

Candidate routes are checked in this order:

1. `CACHE_READ_REJECTED`;
2. `PARENT_ARTIFACT_REJECTED`;
3. `PARENT_AUDIT_REJECTED`;
4. `CACHE_SEMANTIC_REJECTED`;
5. `X0_CORRESPONDENCE_REJECTED`;
6. `PREFIX_ARITHMETIC_REJECTED`;
7. `RHO0_NONPOSITIVE_REJECTED`;
8. `INITIAL_PREFIX_CANDIDATE`.

The checker independently reconstructs the first applicable route before
consulting the candidate route. It accepts a candidate or a verified rejection
only when every available semantic, component, work, event and terminal field
matches. Malformed receipt, semantic drift, work drift, event drift and seal
drift are distinct checker failures.

## Required controls

The package is not reviewable until all controls use the real public
candidate/checker entrypoints and each invocation emits a distinct sealed
checker audit:

1. missing, short, trailing and same-size-mutated cache/artifact/audit inputs;
2. malformed cache header, selected collection length and invalid boolean;
3. factor, permutation, inverse-scale, original-RHS and baseline-0 mutations;
4. R63ZM route, role-2 flags/role/count/value/root and terminal-result drift;
5. candidate `x0`, `hx0`, `r0`, `z0`, `rho0` root drift;
6. each candidate work counter mutated independently and fully resealed;
7. deleted, duplicated and reordered events plus trace/result reseals;
8. candidate route mutation and every first-failure classifier branch;
9. an independently constructed small valid triangular solve;
10. reachable nonfinite, underflow or nonpositive-lower-bound `rho0` fixtures
    that stop before publication of later events;
11. the frozen binary128 state-difference counterexample, proving no later
    direction product is inferred from state products; and
12. baseline R63ZM artifact/audit hashes and checker acceptance remain exact.

Mutation tooling is external test infrastructure. It may reseal public receipt
fields, but it is never candidate or checker authority.

## Success, evidence and stop rules

Success requires:

- separate Dev and clean Release builds of candidate and checker;
- two byte-identical Release candidate receipts and checker audits;
- exact candidate/checker source, binary, input, receipt and audit hashes;
- zero package-owned allocations in candidate and checker for baseline and all
  applicable controls;
- all controls passing with the declared first-specific route;
- direct source audit proving no forbidden helper or hidden oracle; and
- fresh independent review before interpreting the prefix.

Any shared recurrence authority, unchecked selected byte, unowned domain
operation, post-seal decision, self-derived tolerance, ambiguous route or
accepted resealed drift closes R63ZN as `INCONCLUSIVE`.

A reviewed `GO` permits only the statement that the exact R63ZM role-2 product
can drive the independently reconstructed initial binary128
`x0/r0/z0/rho0` prefix with the frozen Dot2 enclosure on this one cache. It
grants no authority for `p0`,
`H*p0`, `x1`, `p1`, `H*p1`, `x2`, certificates, a full solver, a portable
representation, dynamic builder, corpus, timing, Rust, runtime, GPU,
cross-target determinism or production use. SPEC-38 and ADR-076 remain
`Proposed`; ADR-081 and later ProductChecks remain in force.
