# NSR3-B4E2D7R20R63ZK independent recurrence checker evidence

Status: `AUTHOR_PASS / INITIAL_REVIEW_NO_GO / SUPERSEDED_BY_REVISION_2_REPAIR`.

## Author result

The separate R63ZK translation unit captures one R63ZJ hybrid DTO as untrusted
data and independently schedules the declared K2 recurrence from the immutable
fixture through generic arithmetic primitives. It does not call the R63ZJ
validator or classifier while constructing expected states, products,
certificates or work.

The author route is:

```text
HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE
```

All `2,448` state components, `1,224` product input/output components, `18`
state scalar identities, `27` certificate fields and `24` candidate-work
fields compare exactly. Independently derived state 2 is `24+/78-/0?` with
sign root `89b2908b...6094`; the complete independent ladder is
reject/reject/pass.

This is author evidence only. The initial independent review returned `NO-GO`
for checker completeness; see the
[review evidence](nonlocal-nsr3b4e2d7r20r63zk-independent-review-evidence-2026-08-28.md).
R63ZJ remains `INCONCLUSIVE`; R63ZK does not edit or rehabilitate its validator.

## Frozen identities

| Artifact | Identity |
|---|---|
| Contract snapshot | `388f7462d53808a644b798f59dca52ae0d938eb6` |
| Implementation snapshot | `33e24152cd0634f01df25b27f7e3fd18ebb67c56` |
| Contract-to-implementation diff SHA-256 | `e6872a8e519f88535f4db536f4c5c976c7ff507643b15e3cec1f7a5d9f710688` |
| Contract blob | `fa9d229e689c6742ffef16da01f5738732b2aace7992e64bff58b5309242daa7` |
| Checker main source | `20746b127dd6f5c70f58803d366ac62b99d4a808cdec2e4c00d26421c89a7ebd` |
| Private API source | `53ba5d87e0a62ca2e13d90363ed26d955dfe446080d1e6ca276e8083a44815c3` |
| Boundary primitive source | `f43c68c27ed846e85a8d088283653444929f1b4780a9785d3180398c2a6acd10` |
| CMake source | `397deaf39a47d3e1a2cdc1a265f741caa0bb092fd682953730ef5138fa3c6dd3` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Dev and two Release stdout | `371290103aee0ec18dbb40251785b44866fe52c4e167405a4418886713f29041` |
| Release binary | `49486450b598516263b37003059e892f583027e99271c901fb2c4aa243588175` |

The independent trace root is
`6f5881966ebae13124251b9d21a06a00b4c0e640daa0b59eed5cf48c89bdd763`.
The untrusted R63ZJ trace root remains `0f40978a...b800`; callback identity and
aggregate reproduce `a8330d4b...44c1` and `c8cdbf61...1564`. The R63ZK result
semantic is
`e8cd812c2b637ebc78876874d1da1d5fd72a9ddc6b2612b4c385e457d22c95e2`.

## Independent schedule and work

The checker derives from the fixture:

- projected K2 RHS and inverse scale;
- three K2 factor solves with `30,906` terms and `612` divisions;
- `Kx0`, `Kp0`, `Kp1` from six separate binary128 tangent calls followed by
  K2 projection/addition;
- two rho dots, two denominator dots and `408` terms;
- three scalar divisions, `204` solution updates, `204` residual updates and
  `102` direction updates;
- three certificate evaluations with `306` outer dots, `125,460` dot
  products, `93,636` radius terms, `306` solution dots and `306` sign
  comparisons.

Candidate and checker work are distinct. The checker additionally owns the
component, scalar, certificate and ledger comparisons above, 18 mutation
controls and all 32 classifier cases. Final checker-work root:
`64ba2720d428a1c881db160d5b5033eb54fa39fbe413a63d5398150b36299273`.

## Controls

All 18 semantic controls pass. They cover the four state vector families, a
state scalar identity, product input/output, four certificate mutations,
recurrence work, hybrid work, callback identity/aggregate, incomplete and
nonfinite traces, and result sealing. Classifier precedence closes all `32`
boolean cases. Control root:
`2b73034cfaaaed3c64803d47992f15362a22bb15ee09d800260a17f4bd8dc75a`.

The three load-bearing R63ZJ counterexamples are reproduced rather than hidden:

```text
resealed_state_author_valid       true
resealed_certificate_author_valid true
resealed_work_author_valid        true
```

Each is rejected by R63ZK at its independent state, certificate or
recurrence-work comparison. Thus R63ZK success does not depend on the repaired
R63ZJ validator becoming sound.

## Checks and regressions

- strict-FP Dev build: pass;
- strict-FP Release build: pass;
- Dev plus two Release outputs: byte-identical;
- `git diff --check`: pass;
- R63ZI stdout:
  `98736993d50ae29deebdb48361c0079e0aa3443ca3bfd1775ecbddbcd0080a1e`;
- R63ZG stdout:
  `ca2a0f80b692a466aa97b4725fcc0ac3f553db95bb7ddf72494a754a7e2029c9`;
- R63ZH stdout:
  `181ac246c7a0d6e1c24543fb70e684a9357a0c183858c371867dcd068fe96722`;
- R63ZJ stdout:
  `b9ded7d71e0c19aed85ee51f8ba9d923569aeee9c11b02a7060a267672b27f8a`.

Cargo, host-check and continuum ProductChecks were not run: this is a
localized offline C++ research checker under `Proposed` SPEC-38/ADR-076, with
no runtime or public engine-contract change.

## Claim and ceiling

Pending review, R63ZK author evidence supports only the hypothesis that the
captured frozen hybrid trace matches an independently scheduled K2 recurrence
whose deliberate difference from R63ZI is the three component-linear wide
tangent callbacks. If independent review returns `GO`, that fixed-profile
correspondence can become `SUPPORTED_BOUNDED / REVIEWED` and inform the next
portable product-representation discriminator.

No portable representation is selected. Width three, dynamic building,
corpus/generalization, adaptive stopping, timing, runtime/Rust/GPU integration
and production promotion remain blocked. ProductChecks remain `NOT_RUN`.
