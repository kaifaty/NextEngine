# NSR3-B4E2D7R20R63ZD operator-use schedule evidence

Status: `SUPPORTED_BOUNDED / REVIEWED / ALL_SINGLE_OPERATOR_USES_MINIMAL_REJECTS`.

## Outcome first

The frozen eight-lane Release experiment passes its apparatus and reports:

```text
pass mask by schedule  TTT CTT TCT CCT TTC CTC TCC CCC
                       1   0   0   0   0   0   0   0
minimal rejecting masks: 1, 2, 4
compact route: INITIAL_PRODUCT_MINIMAL_SUFFICIENT
```

The full minimal-set list is authoritative under the contract. Therefore the
provisional bounded interpretation is stronger than the compact precedence
route: replacing **any one** of `Hx0`, `Hp0` or `Hp1` by the frozen common
operator is independently sufficient to lose the state-2 certificate on this
exact original-input two-update recurrence. No pairwise or three-way
interaction is required for rejection.

This interpretation is independently reviewed within the claim ceiling below.

## Frozen execution identity

| Item | Value |
|---|---|
| Parent commit | `cba7229d7d5cba55c8b1be58902ebcfea6535b20` |
| Parent tree | `664d5f874c9d8d8436e73470c76759953ff22b53` |
| R63ZD source SHA-256 | `d888dd7eea0a912fc7a73139310c7df4a93b84fbc99d8483013823eed58b65e7` |
| R63ZD Release binary SHA-256 | `549c00b100e6967dac6d7f9b46879fb47b5b8b4ca4f432cb72940f34708fb327` |
| Release cache producer SHA-256 | `d5e3b6879fb58372798d6cb739cd6157bc90bcae4b4b39635904f05132b07edb` |
| Cache file SHA-256 | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Cache bytes | `1,033,625` |
| Cache fixture root | `7780543a21d3b32e39a1fd18e5056f61c610d69929b4c6b69640075d1e7c4553` |
| Release stdout SHA-256 | `90c73a8a6396036f6855268fc892edcab6ec7bc40542934d8cf051964dd89461` |
| Release stdout bytes | `9,272`, newline terminated |
| Result semantic SHA-256 | `1fec1e31c506e65761909a6af99e44ee30b52f0fbb50432f2b74cbaa555d149f` |

The decisive command was:

```text
nonlocal-formula-operator-schedule-release \
  /tmp/nextengine-r63zc-parent-release-v2.bin
```

Two Release executions are byte-identical. The author/dev output is also
byte-identical for this frozen snapshot, but carries no evidence authority.
Raw outputs remain outside Git at
`/tmp/nextengine-r63zd-release-repair-run1.json` and
`/tmp/nextengine-r63zd-release-repair-run2.json`.

## Cache sealing and producer correspondence

The independent R63ZC review found that consistently resealed RHS, common
payload and common endpoint mutations could pass the first external-cache
validator. Before R63ZD, the validator was bound to the frozen original and
projected RHS roots, common-component root, common solution-set root and common
certificate-set root. Product, factor and certificate kernels now validate
their relevant payload before consumption.

The initial independent review rejected promotion because the scale lineage
remained only self-consistent and because negative controls did not execute
transaction failure prefixes. One batched repair now binds the complete
fixture root and exact original/projected scale values. The mutation smoke
rejects consistently resealed original-RHS, common-component,
common-endpoint, tangent, factor, scale and verifier-profile identities at
their relevant parent/kernel boundaries. Its repaired stdout SHA-256 is
`e4ff7dc634c04338d980d61214de0ca95dd9480ee34f4a139e62feb26cd381d3`.

The Release producer regenerated the cache from the parent computation. The
new cache is byte-identical to the previous author cache, proving that the
sealing repair changed admission rules rather than the frozen numerical
payload.

## Endpoint and work closure

- `TTT` reproduces the R63X/R63ZC raw solution set and the required
  reject/reject/pass ladder (`24+/78-/0?` at state 2).
- `CCC` reproduces the three frozen R63ZC common/original solution roots and
  rejects every state at `12+/24-/66?`.
- All eight masks complete exactly once in fixed integer order.
- Classifier and certificate-isolation controls pass.
- An invalid-dimension transaction rejects at `identity` with zero work and no
  state publication.
- A synthetic `1 x 1` dense Dot2 product with finite binary128 maximum
  operands rejects at `initial_product` after exactly one product/dot/term and
  publishes no states.
- A zero-RHS recurrence rejects at `initial_rho` after exactly two solves, one
  product and one scalar dot, with no states.

Aggregate fixed work is exact:

```text
certificates / products / solves        24 / 24 / 24
factor terms / divisions                247248 / 4896
scalar dots / divisions                 32 / 24
solution / residual / direction updates 1632 / 1632 / 816
tangent inner / outer dots              3780 / 1224
tangent inner / outer terms             385560 / 385560
tangent scale products                  1224
common dots / terms                     1224 / 124848
adaptive stops                          0
```

## Independent review

The initial review returned `NO-GO` for two load-bearing apparatus defects:
the scale lineage was self-consistent but not frozen, and negative controls did
not execute transaction failure prefixes. The permitted single batched repair
closed both. The single re-review returned `GO` with no remaining load-bearing
finding.

The reviewer independently confirmed staged diff
`724b2784d66baf64b305f373f26067d6886d3694eb542748c241ff4753876796`,
all supplied source/binary/producer/cache/stdout identities, every lane root,
control root `cce52d367b1d87006836153b68a96745e10f6f9d45240971e23fc27f4133849e`
and result root. A fresh Release execution reproduced `9,272` bytes at stdout
SHA-256 `90c73a8a...9461`.

## Claim ceiling

R63ZD can localize only which executed product slots are sufficient for
certificate loss under the frozen Linux x86-64 strict-binary128 recurrence.
It does not select a corrected operator, measure solver speed, validate a
corpus, authorize another precision, add iterations, or support CPU, GPU,
runtime or production integration.

The reviewed result selects only the next minimal discriminator over the
common/tangent operator-value discrepancy. It does not authorize applying a
correction or changing factor, precision, iteration count or tolerance.
