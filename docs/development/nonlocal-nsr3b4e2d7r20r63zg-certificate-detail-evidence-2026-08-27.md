# NSR3-B4E2D7R20R63ZG certificate-detail evidence

Status: `SUPPORTED_BOUNDED / REVIEWED`.

## Result

The reviewed fixed Linux x86-64 strict-binary128/twofold execution returns:

```text
status PASS
route  AFFINE_IMAGE_CENTER_RESIDUAL_SUFFICIENT
```

The tangent-origin budget passes for both immutable solution representations.
The common-origin native and center-only budgets reject both representations
as `12+/24-/66?`, while its radius-only budget passes both as `24+/78-/0?`.
Using the unamplified image budget instead of the native global budget changes
no classification.

The common affine-image center is therefore sufficient to reproduce the
certificate loss on this fixture. The representation radius is not sufficient,
the roughly `1.00914x` contraction amplification crosses no observed sign
boundary, and the result is independent of choosing the tangent or common raw
solution under a fixed budget origin.

Approximate decomposed magnitudes are:

| Endpoint | center infinity | radius infinity | native global error |
|---|---:|---:|---:|
| tangent | `1.0012916073e-3` | `1.4837458393e-11` | `1.0104478172e-3` |
| common | `3.0660474591e13` | `3.4039964978e-3` | `3.0940845742e13` |

This localizes the next question to production of the common affine-image
center residual. It does not yet identify the responsible row, operand,
operator contribution or cancellation mechanism and does not prove a repair.

## Frozen identity

| Artifact | Identity |
|---|---|
| Parent commit before executable diff | `3079633c24d8aa348fb2e0a8ec9a60f3295f5e5d` |
| Reviewed staged executable diff | `e42b50c720161f4229633c500f6ac98f16cb40b72776bbb09ee45be36b83fbf9` |
| Main source | `16d6e5d550ad3dced3e1d166fd625270e08e53d7f5884958dffa350c52a3a5e6` |
| Private API | `5385dda64fa0e3b3cdc52bae3f4175b3f93e0d8dd5e7c0606ea4704250ec194b` |
| Boundary producer/wrapper | `794eb1e87e94a76f8722c43de14c5b06851af020a455165d6f9e0d1eb9d22c21` |
| Guarded R63ZF main source | `9537668bd8d7e4248355f1fcce7fb895087779e48e2a1b07ada2e768a894c5a6` |
| CMake input | `a36fe2e290819ea6536acb02f30626b1f420bed4c8a5632810b9cba88e832f6d` |
| Release binary | `9f63e48c1e4dffa6342c8eeba3115a41b6f9d4e12199c01cf0428730e0ddd98f` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Parent fixture root | `7780543a21d3b32e39a1fd18e5056f61c610d69929b4c6b69640075d1e7c4553` |
| Final stdout, 7,670 bytes | `ca2a0f80b692a466aa97b4725fcc0ac3f553db95bb7ddf72494a754a7e2029c9` |
| Parent work root | `7406ef19d34c78e366376eac7cfb70612caeaab16a1bc2eba1e7df6125da0ca2` |
| Detail work root | `5ce8dd9c0a6b2bbdf7b5cc00ec22775d91b5f2131525aecfbd6b31b4e5edd0ed` |
| Observations root | `0d506330c92bd05d4a48c3a6c76b3950420ee236a11798d021c333b863ff0804` |
| Controls root | `c3d2d81f6aa4a516990c57fc64551d9f1a49a2cfbd45a270c31a1ff0cbe831e8` |
| Transaction root | `4e322f5131d7d3bdc7ea19f96136582f53cb078036950321fab1476de570c2e4` |
| Result semantic | `989886a4707e03b76eea48ad92bfcbc1721eed9ba679edfed342079f01c6a5e5` |

The strict-FP development output and two repaired Release outputs are
byte-identical. Raw outputs remain outside Git:

```text
/tmp/nextengine-r63zg-repair-dev.json
/tmp/nextengine-r63zg-repair-release-run1.json
/tmp/nextengine-r63zg-repair-release-run2.json
```

The Release command is:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-certificate-detail-release \
  /tmp/nextengine-r63zf-parent-cache.bin
```

## Decisive cells

```text
solution  budget origin  mode  result
tangent   tangent        U     24+/78-/0? PASS
tangent   tangent        GC    24+/78-/0? PASS
tangent   tangent        GR    24+/78-/0? PASS
tangent   tangent        G     24+/78-/0? PASS
tangent   common         U     12+/24-/66? REJECT
tangent   common         GC    12+/24-/66? REJECT
tangent   common         GR    24+/78-/0? PASS
tangent   common         G     12+/24-/66? REJECT
common    tangent        U     24+/78-/0? PASS
common    tangent        GC    24+/78-/0? PASS
common    tangent        GR    24+/78-/0? PASS
common    tangent        G     24+/78-/0? PASS
common    common         U     12+/24-/66? REJECT
common    common         GC    12+/24-/66? REJECT
common    common         GR    24+/78-/0? PASS
common    common         G     12+/24-/66? REJECT
```

The two additional local-only cells reproduce the shared raw sign root and
pass `24+/78-/0?`. Native `(tangent,G_t)` and `(common,G_c)` cells reproduce
their immutable certificate roots exactly.

## Exact work

```text
detail certificates                         2
image dots / products              204 / 83640
image radius terms                       62424
native solution dots/signs           204 / 204
detail solution dots/products        204 / 408
maximum component visits                   612
derived budget divisions                     8
synthetic cells/sign comparisons       18 / 1836
candidate/operator/solver updates       0 / 0 / 0
```

## Controls and review

Classifier precedence is exhaustively exercised. Invalid solution shape,
finite overflow, noncontractive denominator and incomplete detail reject
before cell publication with exact work prefixes. Resealed center, radius,
selected-budget and native-error mutations exercise detail, cell and result
identity; an unsealed component swap rejects. A deliberately corrupted
non-profile parent-fixture field now rejects at the public detail DTO boundary
while preserving the verifier-profile payload.

Initial independent review returned `NO-GO` for two harness defects: the public
detail DTO admitted a fixture after checking only its verifier profile, and the
detail ledger omitted the contract-declared zero candidate/operator/solver
updates. One batched repair changed the DTO admission to full parent-fixture
validation, added a discriminating corruption control and made all three zero
counters explicit in work identities, failure prefixes and output. The single
permitted re-review returned `GO` with no remaining load-bearing finding. No
third review was run.

The reviewer verified every supplied hash, independently recomputed the work,
transaction and result roots, and reproduced the final Release stdout. R63ZF,
R63ZE and kernel-smoke regression outputs remain byte-identical at
`be74e412...de0e`, `4fcb94cd...f4a4` and `e4ff7dc6...381d3` respectively.

## Claim ceiling and next discriminator

R63ZG supports only that the common affine-image center residual is sufficient
to reproduce the certificate loss on one fixed Linux x86-64 R63Y profile. It
does not prove that the center is mathematically incorrect, identify its
producer defect, validate a correction, select an operator/profile/tolerance,
measure performance, validate a corpus or authorize runtime/GPU/production
integration.

The cheapest next discriminator is a frozen componentwise decomposition of the
common center producer `v - Mx`: identify the maximum row, bind the separately
rounded `v_i` and `M_i x` terms plus their exact/twofold residual, and cross the
immutable tangent/common operator-input lanes without changing the certificate.
Only after that audit may a correction hypothesis be frozen.
