# NSR3-B4E2D7R20R63ZL admitted tangent work-boundary evidence

Status: `AUTHOR_PASS / INDEPENDENT_REVIEW_REQUIRED`.

## Bounded result

R63ZL implements the revision-2 one-time tangent admission boundary without
changing R63ZJ or R63ZK. The admitted type has a private constructor, no
default construction, no move and no assignment. Its only payload owner is a
private `shared_ptr<const vector<binary128>>` created from one full fixture
copy; callers receive no payload pointer/reference or mutator.

The author route is:

```text
ADMITTED_TANGENT_WORK_BOUNDARY_CANDIDATE
```

This is author evidence only. It does not restore recurrence correspondence or
grant representation/production authority before independent review.

## Frozen identities

| Artifact | SHA-256 / Git identity |
|---|---|
| Contract / implementation parent | `de277c8ddff90ddbe6cc969d9d28600d93728a2d` |
| Implementation snapshot | `f6475040616d0f4f55f933ffd4b7c12ec0068c2f` |
| Complete parent-to-implementation diff | `127248d10141ee4dd8ac14fa9cfb589290271ef42dafc889710c4bf5792b61aa` |
| Revision-2 contract blob | `3eb6e131aab7c70d8d69c857383bab14b41dc14fc80bfffed0c49dd3fcdc248d` |
| Boundary main | `64501c47ea2e5451a20c1a7ca88e12597ccf98361665a7c646eb6e7c1585f995` |
| Private API | `7f9240bfc9927f580a500f1ba7213d8e17b6085ab963d0a01c0d6bfcf06c0016` |
| Boundary implementation | `da25cd6463b1296a149ddb5bc00d81839a3f3a232c6b59887666e8d6553858bd` |
| CMake | `a950a7b8a196d19821d876ce2ea656949adde5e2d5fec6f0d8ce68a3381bc3bd` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Dev and both Release stdout | `329ee21f1e3ad6f9066c773da7359caf4677f7b65a3cd3397f3dea10e5b1f28c` |
| Author Release binary | `6e3425b0681c9daab9780b45d00f60dfb88ff41427be83057de53122ab4b4d7e` |

The result semantic is
`08a0506457a65aa8de8538418610170cc43b81a3a9861c5d510331efcf1d25f3`.

## Admission and product evidence

The valid admission executes and seals:

- `10` ordered tangent/scale predicates;
- `1` full `32130`-component tangent payload hash;
- `2` frozen binary128 scalar parses;
- `32130` copied components;
- one context root, one admission-work root and one admission-result root.

Admission work/root/result are respectively
`21852f1d...d4a31`, `38409cc3...eb6af` and `9428fa32...c5354`.

Six admitted products use the frozen projected/original RHS, three baseline
solutions and common-projected solution 2. They perform `1890` inner dots,
`612` outer dots, `192780` inner and outer terms, `612` scale products and
`30` final kernel root paths. The admitted candidate performs zero repeated
payload validations/hashes, zero legacy kernel guards, zero nested dot guards
and zero nested dot witness-root derivations. Its receipt root is
`ab28ce7e...f4b0b`.

The separate legacy reference executes six payload validations, `60`
validation predicates, six full tangent hashes, `12` frozen-scalar parses,
`36` kernel structural guards, `5004` dot guards, `7506` dot witness roots and
the same `30` final kernel roots. Its receipt root is `02f7ee6c...8e2f1`.

All `612/612` output components, exact guards, numerical counts and all six
legacy product roots match the admitted path bit-exactly. The package-local
checker receipt is `ee9b2f52...8e214`.

## Controls

All seven controls pass with control receipt `bb8cae66...64530`:

- dimension rejects at predicate 1;
- declared tangent-root drift rejects before hashing;
- tangent-component drift executes one payload hash and rejects;
- zero sigma rejects at the strict-positive predicate after the two frozen
  parses;
- a 101-component input rejects before any numerical kernel;
- compile-time traits reject default construction, aggregate construction,
  move, copy/move assignment and raw-fixture construction;
- a mutated receipt digest is detected.

All seven classifier inputs, including unknown, select their frozen
first-specific/fail-safe routes.

## Checks and regressions

- strict-FP Dev build and execution: `PASS`;
- strict-FP Release build: `PASS`;
- final Dev plus two Release executions: byte-identical, exit zero;
- `git diff --check`: `PASS`;
- R63ZI: `98736993d50ae29deebdb48361c0079e0aa3443ca3bfd1775ecbddbcd0080a1e`;
- R63ZG: `ca2a0f80b692a466aa97b4725fcc0ac3f553db95bb7ddf72494a754a7e2029c9`;
- R63ZH: `181ac246c7a0d6e1c24543fb70e684a9357a0c183858c371867dcd068fe96722`;
- R63ZJ: `b9ded7d71e0c19aed85ee51f8ba9d923569aeee9c11b02a7060a267672b27f8a`;
- R63ZK: `73df9f1b7f7beeb97e16e90b9dd498c2c0ee33fe64eeb38425e0c5a11cb92755`.

The fresh regression build used the same local header-only Boost 1.90 include
at `/tmp/nextengine-r63zh-boost/usr/include` required by R63ZH; R63ZL itself
does not include Boost. Cargo, `host-check` and continuum ProductChecks remain
`NOT_RUN`: this is a localized offline C++ research boundary under Proposed
SPEC-38/ADR-076, with no Rust runtime or public engine-contract change.

## Claim ceiling

Pending independent review, R63ZL supports only an author hypothesis that the
frozen tangent can be admitted once and consumed through a closed type with
bit-exact legacy products and complete declared work ownership. A review `GO`
may authorize a new package to build a recurrence checker on this admitted
boundary. It cannot rehabilitate R63ZJ/R63ZK or authorize portable
representation, width three, dynamic building, corpus/generalization, timing,
runtime, Rust, GPU, ProductChecks or production promotion.
