# NSR3-B4E2D7R20R63ZE denominator/residual evidence

Status: `SUPPORTED_BOUNDED / REVIEWED`.

## Result

The reviewed fixed Linux x86-64 strict-binary128 execution returns:

```text
status       PASS
route        DENOMINATOR_STEP_SUFFICIENT
lane order   TT, TC, CT, CC
pass pattern 1100
```

Changing only the terminal residual product changes the sealed residual and
state roots but not the state-2 solution or R63Y certificate. Selecting the
common-product quadratic denominator changes `alpha1`, the state-2 solution
and the certificate. The final-slot certificate loss is therefore localized
to the scalar denominator/step length for this exact recurrence.

This is profile-bound causal localization. It neither selects the physically
authoritative operator nor authorizes a repair.

## Frozen identity

| Artifact | Identity |
|---|---|
| Parent commit before candidate diff | `5f73bf6b4e25ecb44a05122d0a61441607237aa0` |
| Reviewed executable diff | `26f217ac70e714e396c23d8fa3e248e4024e8f386e4f67e43042f0704e77bf33` |
| Source | `16037d654e8be5dbb39f8ff7f1dc0a05794163c8bd657f1da8a75298a525c524` |
| CMake input | `75803b9c15abd4a6bdbf22d808888dcd5f7a362881a4b5af6dfc7ac9b152f3d2` |
| Release binary | `4eb39d9fd6c492b37f0a8a88dac4368c3453a5d2caa9fbfce95308bbc15c7c70` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Parent fixture root | `7780543a21d3b32e39a1fd18e5056f61c610d69929b4c6b69640075d1e7c4553` |
| Reviewed R63ZD result | `1fec1e31c506e65761909a6af99e44ee30b52f0fbb50432f2b74cbaa555d149f` |
| Reviewed R63ZD stdout | `90c73a8a6396036f6855268fc892edcab6ec7bc40542934d8cf051964dd89461` |
| Final stdout, 11,369 bytes | `4fcb94cd94c176e515aedcdb7d0d724e17bac77ec0a94c215ced2dbf4d8ef4a4` |
| Work root | `cbe54f2956567e518d5e473d03868f02381a71c7571fe69c44ba039077bcc058` |
| Controls root | `6d9caf82bf715b233034b81fd0e822a812b7aace823183212ee66030c89dd0a7` |
| Result semantic | `2a291006153c6f91032c692c7d635c281e42cd81c4c73f5240881eb46b4b731c` |

The two Release outputs and the strict-FP development output are byte-exact.
Raw outputs remain outside Git:

```text
/tmp/nextengine-r63ze-repair-release-run1.json
/tmp/nextengine-r63ze-repair-release-run2.json
/tmp/nextengine-r63ze-repair-dev-run.json
```

The Release command is:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-denominator-residual-release \
  /tmp/nextengine-r63zc-parent-release-v2.bin
```

## Decisive observables

Both final products and both denominators are computed in every lane before
selection. Their roots are identical across all lanes:

```text
tangent product     c24382a3...d29ff7
common product      0edf282e...22112b
tangent denominator 13ddc46e...e7347e
common denominator  b2b6d749...c7a730
```

The executed binary128 values are:

```text
d_t      0x1.0aa2a45ce0a757d6d8d6a216fb75p-2
d_c      0x1.0d0965b4944f15fb2a261d799f16p-2
alpha_t  0x1.12a78d1467ea7a19c272f3cdbfeep+0
alpha_c  0x1.1033f5858aac3dfb3b0ae944a492p+0
```

`TT/TC` share solution root `38f8d0b2...0baea`, certificate root
`d4ba011a...b8ecd` and the passing `24+/78-/0?` certificate while their
residual roots differ. `CT/CC` share solution root `f7e28b44...f59e8`,
certificate root `59170126...e5d4` and the rejecting `12+/24-/66?`
certificate while their residual roots also differ. All state-0/state-1 and
ordered final product/denominator aliases pass.

## Exact work

```text
certificates / products / solves        12 / 16 / 12
factor terms / divisions                123624 / 2448
scalar dots / divisions                 20 / 12
solution / residual / direction updates 816 / 816 / 408
tangent inner / outer dots              3780 / 1224
tangent inner / outer terms             385560 / 385560
tangent scale products                  1224
common dots / terms                     408 / 41616
adaptive stops                          0
```

Every lane independently matches the frozen per-lane ledger. No adaptive
branch or early certificate stop exists.

## Controls and review

Classifier enumeration, selector duplicate/missing/out-of-range/relabel
rejection, unselected identity binding, certificate isolation, invalid
selector, finite product overflow and nonpositive selected-denominator
controls pass with exact failure prefixes. The repaired top-level control also
removes one lane's states/roots/certificates and passes that incomplete lane
through the production auditor/classifier; it returns
`DENOMINATOR_RESIDUAL_APPARATUS_REJECTED` without partial publication.

Initial independent review returned `NO-GO`: identity was folded into the
apparatus flag and an incomplete primary lane could be indexed before its
structural surface was validated. One batched repair introduced the common
size/selector auditor, guarded mutation/output access and the failing-lane
control. The single permitted re-review returned `GO` with no remaining
load-bearing finding and independently recomputed the aggregate, lane,
controls and result roots. No third review was run.

The reviewer reused the already reviewed parent cache and frozen binary rather
than regenerating the cache or performing a clean core rebuild. A fresh
Release execution was byte-identical to both captured runs.

## Claim ceiling and next discriminator

R63ZE supports only denominator/step-length sufficiency at the final product
slot on one frozen Linux x86-64 strict-binary128 recurrence. It establishes no
operator authority, repair, tolerance, extra iteration, corpus validity,
performance, runtime, GPU or production claim.

R63ZF should audit the exact/twofold quadratic discrepancy
`p1^T(H_common-H_tangent)p1`, the induced `alpha1` difference and the R63Y
certificate boundary before any operator modification is proposed.
