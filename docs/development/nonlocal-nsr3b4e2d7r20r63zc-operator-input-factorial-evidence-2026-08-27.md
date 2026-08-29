# NSR3-B4E2D7R20R63ZC operator/input factorial evidence

Status: `SUPPORTED_BOUNDED / REVIEWED / COMMON_OPERATOR_PERTURBATION_SUFFICIENT`.

## Strongest result

For the frozen Linux x86-64 binary128 revision-3 factorial, both lanes using
the tangent product reproduce the required reject/reject/pass R63Y ladder,
while both lanes using the reconstructed common-block product reject all three
states. Therefore the operator representation change is sufficient on this
one frozen recurrence; projected RHS and inverse scale are not necessary for
the loss of the state-2 certificate.

This is numerical and implementation-correspondence evidence over four fixed
lanes. The candidate and endpoint comparator share solve/product/certificate
primitives, so the result is not an independent mathematical proof of those
primitives or a claim about another block, precision or solver.

## Frozen evidence

| Item | Identity |
|---|---|
| Contract | `NSR3-B4E2D7R20R63ZC`, revision 3 |
| Commit / tree | `cba7229d7d5cba55c8b1be58902ebcfea6535b20` / `664d5f874c9d8d8436e73470c76759953ff22b53` |
| Parent commit | `01804f594e6bc6b489feb43fc8ef2559958883d2` |
| Candidate diff SHA-256 | `04bbc0db24ec0c9477b6dbf3f4f11f35c5f047c3a55ed984f9e7acfcd90ee756` |
| Main source SHA-256 | `1bb01c0a1599613f8dbef5f4815a3a5d46fd5a5498c12727db2666feb168d060` |
| Release binary SHA-256 | `3af5d35e4a10300413c15f3952f876c6d8526ad091397f373c85b82f5e86b81b` |
| Command | `nonlocal-formula-reclosure --nonlocal-al-generalization-v5-operator-input-factorial` |
| Run 1 / run 2 stdout | byte-identical, 15,667 bytes, SHA-256 `e59c1948892b12f30e7b249c20662792e06372d8eacc280806e2d98a368b8fa3` |
| Result semantic | `cdf132308c8a52ad183d5d109701ff7f4f263228e49a7d2528885f9a6e68c9b4` |

Raw stdout remains outside Git at
`/tmp/nextengine-r63zc-release-run1.json` and
`/tmp/nextengine-r63zc-release-run2.json` on the producing host.

## Decisive matrix

| Operator | Inputs | R63Y ladder | Cell result |
|---|---|---|---|
| tangent | original | reject / reject / pass | required R63X endpoint |
| tangent | projected K2 | reject / reject / pass | input perturbation alone is insufficient |
| common K2 | original | reject / reject / reject | operator perturbation alone is sufficient |
| common K2 | projected K2 | reject / reject / reject | required R63ZB comparator endpoint |

The tangent/original lane reproduces all three R63X solution roots and frozen
solution-set root `d0b42562...c98e`. The common/projected lane reconstructs
the R63ZB comparator root `7db8a84e...c8e3`. The two hybrid bits therefore
distinguish all four classifications without a fitted threshold.

The fixed ledger closes at 12 products, solves and certificates; 16 scalar
dots; 12 scalar divisions; 123,624 factor terms; 2,448 factor divisions;
816 solution updates; 816 residual updates; 408 direction updates; and zero
adaptive stops. Work root is `021e51f1...f9d6`.

## Independent review

Fresh review verified the supplied hashes before interpreting output and did
not rebuild or rerun because every frozen identity matched. It independently
recomputed transaction, work and result roots; checked actual consumed
operator/factor/permutation/RHS/scale/profile identities, recurrence order,
endpoint reconstruction, work prefixes, overflow at `initial_product`,
positivity failures, mutation/isolation controls and classifier precedence.
Verdict was `GO` within the contract ceiling.

The review also found a major but non-load-bearing defect in the later external
parent-fixture cache/API: several payloads could be consistently resealed
without a frozen-parent comparison, and public kernels did not enforce their
relevant fixture identity. R63ZC does not read that API, so its result remains
valid. Cached successor work is blocked until the API is repaired and
mutation-tested.

## Decision and ceiling

Select only the cause class `COMMON_OPERATOR_PERTURBATION_SUFFICIENT` for one
subsequent frozen discriminator. Do not infer that either operator is a
production authority or that common arithmetic, PCG, K2, binary128 or the
Nonlocal model fails generally.

The cheapest successor is an exhaustive fixed three-product schedule over the
original-input recurrence: choose tangent/common independently for `Hx0`,
`Hp0` and `Hp1`. Its eight lanes can determine whether the initial product
alone is sufficient or whether later applications/interactions are required,
without changing precision, factor, iteration count, certificate or tolerance.
First repair the cached-fixture evidence boundary; do not consume its current
self-sealed claim.

ProductChecks: all continuum checks remain `NOT_RUN(ResearchOnly)`; Release
focused repeatability and independent review passed, while runtime, GPU,
performance and production checks were not authorized.
