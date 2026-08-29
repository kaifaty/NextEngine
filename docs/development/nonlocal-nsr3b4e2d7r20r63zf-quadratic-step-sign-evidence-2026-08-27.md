# NSR3-B4E2D7R20R63ZF quadratic-step/sign evidence

Status: `SUPPORTED_BOUNDED / REVIEWED`.

## Result

The reviewed fixed Linux x86-64 strict-binary128/twofold execution returns:

```text
status PASS
route  CERTIFICATE_ENCLOSURE_AMPLIFICATION_CANDIDATE
raw solution sign changes 0 / 102
raw solution zeros        0 / 204 endpoint components
```

The common final quadratic denominator is larger and the corresponding step
is smaller. Both outward forms of the quadratic discrepancy are strictly
positive and overlap; `alpha_c-alpha_t` is strictly negative; all 102 induced
solution displacements are contained. Nevertheless, the tangent and common
raw binary128 solution sign roots are identical.

The certificate transition is therefore not caused by a raw solution sign
crossing on this recurrence. It is accompanied by growth of the R63Y global
`error_upper` from about `1.01045e-3` to `3.09408e13`, a ratio of about
`3.06209e16` (`16.49` decimal orders). This selects decomposition of the
certificate image center, representation radii and contraction amplification;
it does not yet prove which term is sufficient.

## Frozen identity

| Artifact | Identity |
|---|---|
| Parent commit before executable diff | `5b5065d3780f2db8ede51e1f5f4f76ee0cf7032f` |
| Reviewed staged executable diff | `3b61bf066522d365e87526c53c05bbd29c23b1159f59466251f3860ece7df744` |
| Main source | `3d3cfc62c32968c752ae91e8b57019d378f853ca9d7a9579a254cb2fb25ea350` |
| Private scalar API | `24a94732c3173bbe9d4f77c8347761ee690193869d4d2253b4e69a4526908b78` |
| Boundary wrapper | `91458957bdb5060ce1081f57c86e0f4699f19be10cac4dd653c33b07f091eead` |
| CMake input | `9b851ad523d8788c07265bf39f3789286ac2f2c2a2e30ad39c4bae416ce4aba8` |
| Release binary | `4e58240979486e5175b6564c7bb5acaa12e7a3be7b87309fd24003640a4158b2` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Parent fixture root | `7780543a21d3b32e39a1fd18e5056f61c610d69929b4c6b69640075d1e7c4553` |
| Reviewed R63ZE stdout | `4fcb94cd94c176e515aedcdb7d0d724e17bac77ec0a94c215ced2dbf4d8ef4a4` |
| Reviewed R63ZE result | `2a291006153c6f91032c692c7d635c281e42cd81c4c73f5240881eb46b4b731c` |
| Final stdout, 4,281 bytes | `be74e412ec6d708128506eeef5cd7284a56a2f150de8e1decb5be12ca394de0e` |
| Core work root | `7406ef19d34c78e366376eac7cfb70612caeaab16a1bc2eba1e7df6125da0ca2` |
| Audit work root | `989c26dbb06ca825e3b46acb87d8fccf6b758e7be7edb2d1633cf7fc3d2a719f` |
| Primary root | `b7151b3f6f94aa25dbc93a5e9be33d9ae8b49a34ca78bf15314ba373b5b3a01e` |
| Controls root | `e211b307fd35b6156e2a2f1655710a8bd25e022847e306dd772646dace4cd22c` |
| Result semantic | `0770e6cf6339163088e4980482d74675360cd1267d4db7780a5ee02638ba38dd` |

The two repaired Release outputs and strict-FP development output are
byte-identical. Raw outputs remain outside Git:

```text
/tmp/nextengine-r63zf-repair-release-run1.json
/tmp/nextengine-r63zf-repair-release-run2.json
/tmp/nextengine-r63zf-repair-dev.json
```

The Release command is:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-quadratic-step-sign-release \
  /tmp/nextengine-r63zf-parent-cache.bin
```

## Decisive observables

```text
d_t       0x1.0aa2a45ce0a757d6d8d6a216fb75p-2
d_c       0x1.0d0965b4944f15fb2a261d799f16p-2
d_c-d_t   0x1.3360abd9d3df1228a7bdb151d080p-9
p1^(c-t)  0x1.3360abd9d3df1228a7bdb151d0e2p-9
alpha_t   0x1.12a78d1467ea7a19c272f3cdbfeep+0
alpha_c   0x1.1033f5858aac3dfb3b0ae944a492p+0
alpha_c-alpha_t -0x1.39cbc76e9f1e0f43b405448dae00p-7
```

The tangent/common raw solution roots remain the reviewed R63ZE roots
`38f8d0b2...0baea` and `f7e28b44...f59e8`, but their sign vectors share root
`89b2908b...6094`; the changed-index root is the empty SHA-256
`e3b0c442...b855`. Tangent certification remains `24+/78-/0?`, while common
certification remains `12+/24-/66?`.

## Exact work

```text
certificates / products / solves        2 / 4 / 3
factor terms / divisions                30906 / 612
recurrence scalar dots / divisions      5 / 4
quadratic discrepancy dots              1
solution / residual / direction updates 306 / 306 / 102
product-difference updates               102
solution-difference updates              102
predicted-displacement products          102
scalar-difference updates                  2
raw sign comparisons                     204
tangent inner / outer dots               945 / 306
tangent inner / outer terms              96390 / 96390
tangent scale products                   306
common dots / terms                      102 / 10404
adaptive stops                           0
```

## Controls and review

All 64 apparatus/identity/work/endpoint/algebra/displacement classifier
combinations and all 16 raw-zero/sign-change/ladder/error-order combinations
are exercised. Bound-only and resealed sign mutations change both Primary and
synthetic result identities. Reversed discrepancy, raw zero, invalid
dimension, nonpositive denominator, incomplete endpoint and guarded finite
overflow controls pass. Overflow consumes exactly one scalar dot and publishes
no solve, product, endpoint or certificate.

Initial independent review returned `NO-GO`: interval construction discarded
producer validity, the quadratic dot was double-classified in the work ledger,
classifier enumeration was incomplete, and overflow plus mutation sealing did
not exercise the promised Primary/result boundary. One batched repair
propagated producer exactness, restored the frozen `5+1` work split, exhausted
classifier combinations, routed overflow through guarded Primary evaluation
and resealed mutations transitively. The single permitted re-review returned
`GO` with no remaining load-bearing finding. No third review was run.

The reviewer reused the frozen parent cache and supplied Release binary rather
than independently regenerating the cache. A fresh Release execution matched
the repaired captures. The private DTO extension also preserves the R63ZE
stdout at `4fcb94cd...f4a4` and passes the kernel smoke at
`e4ff7dc6...381d3`.

## Claim ceiling and next discriminator

R63ZF supports only that raw sign changes do not explain the R63Y certificate
loss on one frozen Linux x86-64 strict-binary128/twofold recurrence. It does
not prove global-error sufficiency, select an operator or certificate repair,
measure performance, validate a corpus or authorize runtime/GPU/production
integration.

The next bounded audit must decompose each immutable R63Y certificate into
affine-image center magnitude, representation radius and the shared
`1/(1-rho_upper)` amplification, then test final sign classification under
identity-sealed component swaps. No certificate or solver policy changes are
admitted.
