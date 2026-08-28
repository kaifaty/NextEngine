# NSR3-B4E2D7R20R63ZL admitted tangent work-boundary research

Status: `SELECTED / CONTRACT_FROZEN / NOT_RUN`.

## Problem

R63ZJ and R63ZK both failed independent review on work ownership rather than
on a newly observed numerical mismatch. R63ZK revision 2 calls the legacy
`formula_probe_tangent_product` wrapper six times. Each call repeats a complete
tangent-payload validation and tangent hash, but the checker receipt records
only the six numerical kernels. A second R63ZK repair is forbidden, so any
closure must be a new package.

The bounded question is not yet whether the R63ZJ trace is correct. It is which
API boundary makes all validation and hashing work reviewable before another
full recurrence checker is attempted.

## Competing hypotheses

- **H1 -- immutable admission:** validate and hash the tangent payload once,
  construct a closed immutable context, and let later products rely on that
  type invariant.
- **H2 -- compositional receipts only:** keep accepting the mutable fixture at
  every product and require every nested helper to return a work receipt.
- **H3 -- hybrid:** use immutable one-time admission to remove repeated
  full-payload work, and a small typed receipt to expose the structural guards,
  numerical work and hashes that legitimately remain in each product.

H1 is falsified if a caller can construct or mutate the admitted payload, or if
the admitted product still calls the legacy validating wrapper. H2 is falsified
if any nested validation/hash remains invisible or if acceptance trusts a
self-reported mutable receipt. H3 is falsified by either condition.

## Evidence

Local production patterns already separate untrusted decoding from a private
validated representation:

- `ValidatedCreatorPackageV1` is constructible only after complete package
  validation and is then consumed downstream;
- `ValidatedCheckReportV1` distinguishes parsed-and-validated reports from raw
  bytes;
- the command ledger permits bounded incremental validation only for private
  bindings assembled through checked mutation APIs, while retaining complete
  validation at decode/restore/migration boundaries.

Primary sources support the same boundary without granting any formal-proof
claim to this C++ experiment:

- the [C++ Core Guidelines, C.40--C.42](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines#c40-define-a-constructor-if-a-class-has-an-invariant)
  require construction to establish an invariant and recommend a factory when
  construction can fail; post-construction `is_valid()` protocols are described
  as error-prone;
- Appel et al., [A Trustworthy Proof Checker](https://www.cs.princeton.edu/~appel/papers/flit.pdf)
  motivate a small named trusted checker boundary and describe the LCF-style
  abstract-data-type technique for preventing a large producer from fabricating
  accepted evidence;
- Barrett et al., [TVOC: A Translation Validator for Optimizing Compilers](http://theory.stanford.edu/~barrett/pubs/BFG+05.pdf),
  independently validate each produced translation against declared semantics,
  which supports keeping the later recurrence replay separate from its author
  producer.

The local and external evidence rejects H2 as the next step. A receipt-only
design leaves the mutable payload and every nested helper inside the trusted
accounting surface; it repeats the failure mode already seen twice. H1 alone is
also insufficient because the numerical kernel still performs structural
guards and derives four vector roots plus its result root. H3 is therefore the
smallest falsifiable boundary.

## Selected experiment

R63ZL will add a private-constructor, no-mutator admitted tangent value. Its
factory directly checks the ten frozen tangent/scale predicates, derives the
full tangent payload root exactly once, copies the `102 x 315` payload, and
seals an admission receipt. The admitted product cannot accept a raw fixture
and cannot call `formula_probe_tangent_product`; it accepts only the admitted
type and a vector.

The product receipt explicitly owns its input guard, one numerical kernel,
kernel dot/term/scale counts, the four kernel vector hashes, the kernel result
hash, its receipt hash and its result hash. Six frozen inputs are evaluated by
both the admitted and legacy paths and compared component-by-component. The
legacy comparison work is separate and explicitly owns its six repeated
payload validations and hashes.

The full scope and stop rules are frozen in the
[R63ZL contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r20r63zl-admitted-tangent-work-boundary-contract.md).

## Authority ceiling

R63ZL can establish only a fixed-fixture API/work-boundary candidate. It cannot
rehabilitate R63ZJ or R63ZK, prove recurrence correspondence, select a portable
representation or width, authorize timing/corpus work, or grant runtime, GPU
or production authority. SPEC-38/ADR-076 remain `Proposed`; later continuum
ProductChecks remain `NOT_RUN`.
