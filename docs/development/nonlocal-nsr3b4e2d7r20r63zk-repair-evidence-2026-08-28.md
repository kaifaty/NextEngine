# NSR3-B4E2D7R20R63ZK revision-2 repair evidence

Status: `AUTHOR_REPAIR_PASS / SINGLE_RE_REVIEW_NEXT`.

## Bounded result

Revision 2 repairs the complete initial-review finding set without changing
the R63ZJ producer verdict. The checker still captures the R63ZJ DTO once as
untrusted data, then independently schedules the frozen K2 recurrence from the
immutable fixture. It now compares explicit scalar `(high, low)` payloads,
executes all four strict-positive gates in transaction order, compares every
declared guard/metadata/root/semantic field, and publishes distinct expected
producer, actual producer, checker and control receipts.

The author route is:

```text
HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE
```

This is author evidence only until the single re-review returns `GO`. R63ZJ
remains `INCONCLUSIVE` and is neither repaired nor promoted.

## Frozen identities

| Artifact | SHA-256 / Git identity |
|---|---|
| Revision-2 contract / implementation parent | `3097bc08a31a2a4c017cb696a6990a3138ff056a` |
| Repaired implementation snapshot | `7635d664fe35bbfde725956307e62f6a598009ab` |
| Complete parent-to-repair diff | `2e652018528d5d03f8921f970070254587c181fbc3a9f85942ef504c3f3fac63` |
| Contract blob | `0a231bb60646b4c4d655bcba2747251b9cd8a68edd38f3cb85af5ec288cc40ea` |
| Checker main | `02bec9996c754fc7c12ac3b86617ff3350049ccbf57c452018023d6a1e018063` |
| Private API | `13b0c894805d4a57968afbfd0f146c686c475d72b7ff0d78d7838c18f1ce4f77` |
| Boundary primitives | `621300de50157b1e3da1adfc114618990460f16217678aa99787d1c2caa0a899` |
| CMake | `397deaf39a47d3e1a2cdc1a265f741caa0bb092fd682953730ef5138fa3c6dd3` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Dev and both Release stdout | `73df9f1b7f7beeb97e16e90b9dd498c2c0ee33fe64eeb38425e0c5a11cb92755` |
| Author Release binary | `4ccee9be8f1cb34d98dfd14010b9d2dc31982af22122008306d057af42d1d35c` |

The output result semantic is
`98d2b772fb78e5dc6976d1d5698ae030b49e19aadd4f38f867d26d86c37a1172`.
The untrusted trace root remains `0f40978a...b800`; the independent semantic
trace root is `bfa96166...fd0f`.

## Repaired semantic boundary

- All `2,448` state-vector components and `1,224` product input/output
  components compare bit-exactly.
- Twelve scalar guards, `24` explicit scalar components and `30` associated
  scalar/operator/solve roots compare independently.
- Four positivity checks cover `rho0`, `denominator0`, `rho1` and
  `denominator1` before their dependent operations.
- Twenty-eight certificate fields, including the aggregate certificate-set
  identity, compare after their represented semantics.
- Product exact/site/dimension/entries, component sizes and all eight product
  identity/root fields are checked at each of three sites.
- The classifier retains the frozen first-specific mismatch precedence and
  exhausts `12 * 8 = 96` mismatch/work/control/ladder cases, including an
  unknown enum fail-safe.

The independently derived state-2 certificate remains `24+/78-/0?` with sign
root `89b2908b...6094`; the independent ladder is reject/reject/pass.

## Work receipts

Expected and actual producer payloads match at
`f9bc5686...c4a8`, while their role-separated receipt roots are
`cae77d58...4892` and `52347303...ed3`. Both publish the complete recurrence
and hybrid ledgers rather than opaque roots alone. The hybrid receipt includes
one fixture validation, one identity derivation, six tangent kernels and all
three certificate evaluations.

Checker receipt `b456b0ab...97ab` owns the independent numerical schedule,
three targeted factor/profile payload validations, four positivity checks,
all semantic comparisons, `33` declared root-derivation paths and its result
seal. Its comparison receipt is `a25a9409...bff1`.

Control receipt `a690823d...51f9` owns `64` controls, `61` trace comparisons,
`12` reseals, `78` declared root-derivation paths, `96` classifier cases and
five result-seal checks. Nineteen author-valid controls each execute and own a
full old-validator replay: in total `114` tangent kernels, `35,910` inner
dots, `11,628` outer dots, `3,662,820` inner and outer terms, `11,628` scale
products/input components/output projections and `5,814` output additions.
All 64 outcomes are true and the control comparison receipt is
`bb34ee54...0803`.

The fully resealed author-valid controls cover state data, scalar payloads and
roots, every certificate semantic field/root, recurrence work and newly
visible hybrid certificate work. Direct boundary controls cover every product
field, recurrence guards, callback roots, incomplete/nonfinite traces,
positivity, classifier precedence and result sealing.

## Checks and regressions

- strict-FP Dev build: `PASS`;
- strict-FP Release build: `PASS`;
- final Dev plus two concurrent Release executions: byte-identical, all exit
  zero and all report producer/checker/control `exact=true`;
- truncated cache: exit `1`, structured
  `INDEPENDENT_CHECKER_APPARATUS_REJECTED / cache_read`, stdout
  `54bc17c1...5aa`;
- `git diff --check`: `PASS`;
- R63ZI stdout: `98736993d50ae29deebdb48361c0079e0aa3443ca3bfd1775ecbddbcd0080a1e`;
- R63ZG stdout: `ca2a0f80b692a466aa97b4725fcc0ac3f553db95bb7ddf72494a754a7e2029c9`;
- R63ZH stdout: `181ac246c7a0d6e1c24543fb70e684a9357a0c183858c371867dcd068fe96722`;
- R63ZJ stdout: `b9ded7d71e0c19aed85ee51f8ba9d923569aeee9c11b02a7060a267672b27f8a`.

Cargo, `host-check` and continuum ProductChecks remain `NOT_RUN`: the repair
is a localized offline C++ research checker under Proposed SPEC-38/ADR-076 and
changes no Rust runtime or public engine contract.

## Claim ceiling

Pending re-review, the result supports only an author hypothesis that the
frozen untrusted hybrid trace is bit-exact with the independently scheduled K2
recurrence and verifier. A re-review `GO` may admit that fixed-fixture trace
correspondence and freeze the next portable product-representation
discriminator. It grants no representation selection, width-three, dynamic
builder, corpus/generalization, adaptive stop, timing, runtime, Rust, GPU,
ProductCheck or production authority.
