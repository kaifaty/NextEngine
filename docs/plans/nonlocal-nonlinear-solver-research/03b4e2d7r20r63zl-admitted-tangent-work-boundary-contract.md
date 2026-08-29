# NSR3-B4E2D7R20R63ZL admitted tangent work-boundary contract

Revision: `3`.

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZL` |
| Architecture snapshot | SPEC-38/ADR-076 `Proposed`; ADR-081 `Accepted`; R63ZJ/R63ZK `INCONCLUSIVE` |
| Engineering consumer | decide whether a one-time admitted tangent context can make the next recurrence checker work-complete |
| Claim class | fixed-fixture API and work-boundary candidate; no numerical-correspondence or production claim |
| Budget | one boundary implementation, six admitted products, six legacy reference products, negative controls, Dev plus two Release repeats, one initial review plus one batched-repair re-review; revision 3 consumes the repair |

## Frozen question

Can the fixed `102 x 315` tangent payload cross an untrusted boundary once and
then support six bit-exact products without repeated full-payload validation or
hashing, while every remaining validation, numerical and root-derivation path
is explicit and sealed?

R63ZL does not re-run or repair the R63ZK correspondence claim. It validates
only the boundary needed by a future new checker package.

## Trust boundary

The serialized parent fixture is untrusted. R63ZL may materialize only the
tangent, scale, declared tangent/RHS identities and six input-vector fields
needed by this package. A package-local selective reader must skip every other
field structurally, without constructing its DTO, deriving its semantic root
or invoking its validator. The reader must never call
`formula_probe_parent_fixture_valid`; its bounded length reads, selected-field
copies, structural skips, header/EOF checks and receipt/result roots are an
explicit sealed apparatus receipt. Malformed/truncated input returns a sealed
failure object rather than escaping to an stderr-only exception path.

The admitted candidate must not call the parent-fixture validator, the legacy
`formula_probe_tangent_product(FormulaProbeParentFixture, ...)` wrapper,
R63ZJ/R63ZK validators, schedulers, classifiers or routes. A reference overload
which accepts only the selective boundary projection may reproduce the frozen
legacy targeted tangent validation and numerical kernel after the complete
candidate phase.

The admitted tangent type must have:

- a private constructor and no default construction;
- no public mutator and no public mutable reference/pointer to the payload;
- value-copy/move semantics that can only copy an already admitted value;
- a factory returning an empty/failed result when any admission predicate
  fails;
- product entry points that accept the admitted type, never a raw fixture.

The legacy wrapper is allowed only after the admitted candidate has completed,
as a separately accounted reference path.

The six input vectors are referenced in place after selective decode; main may
not copy them into an owning collection. Each admitted product receives a
fixed role, derives exactly one complete input-vector root, compares it with
the immutable role root and binds role plus input root into its result. The
frozen roots, in required execution order, are:

1. projected RHS `0df32db6fb6c7b3a6fb5e3760c7ff810f92e31baa1544a0911a8e0b6a11274ef`;
2. original RHS `64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1`;
3. baseline state 0 `11170afba73f379a19c02f4a9a1514944cc8f9f18bf7c79872d77abc3f478292`;
4. baseline state 1 `f6118706ae0f5190861396eefb97fab1c9b9b3d4052c6d23029b85e9e212f42b`;
5. baseline state 2 `38f8d0b2c99126ff046d2bd6471c3c0a018b50fba66f38b418a8e3eb0600baea`;
6. common-projected state 2 `bc4b4f31ca17c84fb21eeb6239f29169d1187b0c65671b829fe6a0dda7e32c15`.

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

The valid receipt owns `10` predicate checks, `1` tangent-payload hash, `2`
frozen binary128 constant parses, `32130` copied tangent components, `1`
context-root derivation, `1` receipt-root derivation and `1`
admission-result-root derivation. Failure stops at the first failed predicate,
never constructs a context, never performs work after that stage, and still
returns a sealed failure receipt/result.

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
2. hashes the complete input once, compares its immutable role identity once
   and binds both values into the admitted product;
3. invokes exactly one `T(T^T x) * sigma` binary128 numerical kernel without
   calling the legacy payload-validating wrapper; its dot arithmetic must stay
   bit-identical to the frozen dot2 arithmetic but must not derive the unused
   per-dot witness roots;
4. owns the kernel's `315` inner dots, `102` outer dots, `32130` inner terms,
   `32130` outer terms and `102` scale products;
5. owns the kernel's four final vector-root derivations and one result-root
   derivation; all `417` admitted dot calls execute with sizes established by
   the admitted context/input guard and derive no per-dot witness roots;
6. derives one typed product-receipt root and one admitted-result root.

Across six admitted products the exact totals are `6` input guards, `6` input
hashes, `6` input-identity checks, `6` kernels, `1890` inner dots, `612` outer
dots, `192780` inner terms, `192780` outer terms, `612` scale products, `30`
kernel root paths, `6` receipt roots and `6` admitted-result roots. The
candidate dot implementation indexes the immutable tangent directly: it owns
zero temporary operand-vector allocations and zero copied operand components.
The four necessary kernel result buffers per product are counted separately.
No tangent-payload validation or full tangent hash may execute after admission.

Run the legacy wrapper on the same six inputs only as reference. Its distinct
reference receipt owns `6` targeted tangent/scale validations, `60` predicate
checks, `6` full tangent hashes, `12` frozen-scalar parses, the same six
numerical kernels, `36` legacy kernel structural guards, `5004` nested dot
guard checks, `7506` nested dot witness root paths and `30` final kernel root
paths. Compare exact/product guards, all `612` binary128 output components, all
numerical work fields and all six legacy product roots with the admitted
outputs. The candidate receipt and reference receipt must never be merged.

The reference receipt additionally owns `2502` temporary operand-vector
allocations and `385560` copied operand components across its six kernels
(`6 * (315 + 102)` vectors and `6 * (315 * 102 + 102 * 315)` components).
Both candidate and reference own `24` necessary kernel result-buffer
allocations. Return/storage moves are allowed only when they transfer already
constructed immutable results without copying component payloads.

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
- a real serialized cache with a first-field admission mutation reaches a
  sealed top-level `ADMITTED_TANGENT_ADMISSION_REJECTED` result before any
  context access or product;
- fully resealed candidate-receipt, reference-receipt and final-result-root
  mutations each reach `RouteInput::Seal` and select `INCONCLUSIVE`.

Every executable control has an exact separate control receipt which binds its
name, expected and observed route/stage, outcome, mutation digest and complete
work. The aggregate control root commits the ordered individual receipt roots,
not counters alone. Compile-time API inspection covers the
unconstructible/unmodifiable control.

The checker independently reconstructs selective-read, admission, every
candidate product, aggregate candidate/reference and final-result semantics
before accepting their roots. It checks its final work values only after all
result seals and root paths have been recorded. A final result object contains
all route inputs and child roots; a package-local validator recomputes its
route and root. The printed result is accepted only after a second independent
reconstruction. A root inequality control which is not passed through this
validator has no credit.

## Routes and allowed claim

If selective decode, admission, all six products, component semantics, exact
candidate/reference receipts, controls, result sealing, Dev/Release
determinism and legacy regressions pass, select
`ADMITTED_TANGENT_WORK_BOUNDARY_CANDIDATE`.

This permits only the claim that, for the frozen fixture and six frozen inputs,
the admitted type removes five repeated full tangent validations/hashes and
unused nested dot witness hashes from the candidate path while preserving
bit-exact legacy products and exposing all remaining declared work.

Selective-read or admission failure selects
`ADMITTED_TANGENT_ADMISSION_REJECTED` before any context/product access;
product or semantic mismatch selects `ADMITTED_TANGENT_PRODUCT_REJECTED`;
work mismatch selects `ADMITTED_TANGENT_WORK_REJECTED`; control or result-seal
failure selects `INCONCLUSIVE`. Unknown classifier input is fail-safe
`INCONCLUSIVE`.

## Verification and stop rules

- Build and run the Dev target once and a clean Release target twice; stdout
  must be byte-identical.
- Re-run R63ZI, R63ZG, R63ZH, R63ZJ and R63ZK Release binaries; their frozen
  stdout hashes must remain exact.
- Freeze an author evidence packet, then request one independent review. At
  most one batched repair and one re-review are allowed. Revision 3 is that
  repair; there is no further implementation attempt after its re-review.
- A review `GO` authorizes only a new recurrence-checker package to consume the
  admitted boundary. It does not convert R63ZL into physics correspondence or
  production evidence.
- Any remaining load-bearing work-ownership finding after the one repair
  closes R63ZL as `INCONCLUSIVE`.
- Portable representation, width three, dynamic building, corpus, timing,
  runtime/Rust/GPU integration and ProductChecks remain blocked.
