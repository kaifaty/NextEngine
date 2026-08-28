# NSR3-B4E2D7R20R63ZL admitted tangent work-boundary contract

Revision: `1`.

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZL` |
| Architecture snapshot | SPEC-38/ADR-076 `Proposed`; ADR-081 `Accepted`; R63ZJ/R63ZK `INCONCLUSIVE` |
| Engineering consumer | decide whether a one-time admitted tangent context can make the next recurrence checker work-complete |
| Claim class | fixed-fixture API and work-boundary candidate; no numerical-correspondence or production claim |
| Budget | one boundary implementation, six admitted products, six legacy reference products, negative controls, Dev plus two Release repeats, one initial review plus at most one batched-repair re-review |

## Frozen question

Can the fixed `102 x 315` tangent payload cross an untrusted boundary once and
then support six bit-exact products without repeated full-payload validation or
hashing, while every remaining validation, numerical and root-derivation path
is explicit and sealed?

R63ZL does not re-run or repair the R63ZK correspondence claim. It validates
only the boundary needed by a future new checker package.

## Trust boundary

The serialized parent fixture is untrusted. R63ZL may read only the tangent,
scale and six declared input-vector fields needed by this package. It must not
call `formula_probe_parent_fixture_valid`, the legacy
`formula_probe_tangent_product` wrapper, R63ZJ/R63ZK validators, schedulers,
classifiers or routes to construct the admitted candidate.

The admitted tangent type must have:

- a private constructor and no default construction;
- no public mutator and no public mutable reference/pointer to the payload;
- value-copy/move semantics that can only copy an already admitted value;
- a factory returning an empty/failed result when any admission predicate
  fails;
- product entry points that accept the admitted type, never a raw fixture.

The legacy wrapper is allowed only after the admitted candidate has completed,
as a separately accounted reference path.

## Admission semantics and exact work

On the valid frozen fixture, admission evaluates exactly these ten predicates
in order:

1. dimension is `102`;
2. tangent columns are `315`;
3. tangent size is `32130`;
4. declared tangent root equals the frozen root;
5. one complete tangent-payload hash equals the declared root;
6. inverse scale equals the frozen binary128 value;
7. projected scale equals the frozen binary128 value;
8. sigma is strictly positive;
9. sigma is finite;
10. sigma is the exact reciprocal of the frozen inverse scale.

The valid receipt owns `10` predicate checks, `1` tangent-payload hash,
`32130` copied tangent components, `1` context-root derivation, `1` receipt-root
derivation and `1` admission-result-root derivation. Failure stops at the first
failed predicate, never constructs a context, never performs work after that
stage, and still returns a sealed failure receipt/result.

## Product semantics and exact work

Use exactly six immutable fixture vectors, in this order:

1. projected RHS;
2. original RHS;
3. baseline solution 0;
4. baseline solution 1;
5. baseline solution 2;
6. common-projected solution 2.

Each must contain `102` binary128 values. For each input, the admitted path:

1. checks the input size once;
2. invokes exactly one frozen `T(T^T x) * sigma` binary128 numerical kernel
   without calling the legacy payload-validating wrapper;
3. owns the kernel's `315` inner dots, `102` outer dots, `32130` inner terms,
   `32130` outer terms and `102` scale products;
4. owns the kernel's four vector-root derivations and one result-root
   derivation;
5. derives one typed product-receipt root and one admitted-result root.

Across six admitted products the exact totals are `6` input guards, `6`
kernels, `1890` inner dots, `612` outer dots, `192780` inner terms, `192780`
outer terms, `612` scale products, `30` kernel root paths, `6` receipt roots
and `6` admitted-result roots. No tangent-payload validation or full tangent
hash may execute after admission.

Run the legacy wrapper on the same six inputs only as reference. Its distinct
reference receipt owns `6` targeted tangent/scale validations, `60` predicate
checks, `6` full tangent hashes, the same six numerical kernels and their `30`
kernel root paths. Compare exact/product guards, all `612` binary128 output
components, all numerical work fields and all six legacy product roots with
the admitted outputs. The candidate receipt and reference receipt must never
be merged.

## Checker and controls

The package-local checker owns component comparisons, receipt comparisons,
root derivations and controls separately from both candidate and reference
work. It must validate semantics before roots; opaque root equality alone is
not evidence.

Required controls:

- mutated dimension fails admission at predicate 1;
- mutated declared tangent root fails before the payload hash;
- mutated tangent component with the frozen declared root executes one payload
  hash and then fails admission;
- zero sigma fails the strict-positive predicate;
- a `101`-component input fails before any admitted numerical kernel;
- the public API exposes no constructor, mutator or mutable payload accessor
  capable of fabricating an admitted context;
- candidate/reference receipt or result-root mismatch selects fail-safe reject.

Every executable control has an exact separate control receipt. Compile-time
API inspection covers the unconstructible/unmodifiable control.

## Routes and allowed claim

If admission, all six products, component semantics, exact candidate/reference
receipts, controls, result sealing, Dev/Release determinism and legacy
regressions pass, select `ADMITTED_TANGENT_WORK_BOUNDARY_CANDIDATE`.

This permits only the claim that, for the frozen fixture and six frozen inputs,
the admitted type removes five repeated full tangent validations/hashes from
the candidate path while preserving bit-exact legacy products and exposing all
remaining declared work.

Admission failure selects `ADMITTED_TANGENT_ADMISSION_REJECTED`; product or
semantic mismatch selects `ADMITTED_TANGENT_PRODUCT_REJECTED`; work mismatch
selects `ADMITTED_TANGENT_WORK_REJECTED`; control or result-seal failure selects
`INCONCLUSIVE`. Unknown classifier input is fail-safe `INCONCLUSIVE`.

## Verification and stop rules

- Build and run the Dev target once and a clean Release target twice; stdout
  must be byte-identical.
- Re-run R63ZI, R63ZG, R63ZH, R63ZJ and R63ZK Release binaries; their frozen
  stdout hashes must remain exact.
- Freeze an author evidence packet, then request one independent review. At
  most one batched repair and one re-review are allowed.
- A review `GO` authorizes only a new recurrence-checker package to consume the
  admitted boundary. It does not convert R63ZL into physics correspondence or
  production evidence.
- Any remaining load-bearing work-ownership finding after the one repair
  closes R63ZL as `INCONCLUSIVE`.
- Portable representation, width three, dynamic building, corpus, timing,
  runtime/Rust/GPU integration and ProductChecks remain blocked.
