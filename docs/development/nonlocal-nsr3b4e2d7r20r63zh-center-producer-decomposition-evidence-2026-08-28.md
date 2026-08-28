# NSR3-B4E2D7R20R63ZH center-producer decomposition evidence

Status: `SUPPORTED_BOUNDED / REVIEWED / GO`.

## Reviewed result

The fixed Linux x86-64 strict-binary128/twofold author execution returns:

```text
status PASS
route  COMMON_SOLUTION_OUTSIDE_VERIFIER_AFFINE_MODEL
```

The independent `cpp_int` dyadic oracle reconstructs all 204 tangent/common
component-center residuals from the immutable profile and solution expansions.
All `204/204` exact centers lie inside the existing R63Y closed intervals. The
exact and finite common maximum both select row `65`, whose exact center and
immutable interval are approximately:

```text
center  30660474591488.29993693035073134350619070
lower   30660474591488.29737725350577092929202361
upper   30660474591488.30418524649422907070797639
```

The interval is strictly positive. All `102/102` literal canonical-dyadic
identities close:

```text
r_c[i] - r_t[i] = -sum_j M[i,j] (x_c[j] - x_t[j])
```

This result supports H2 over H0 on the frozen fixture: the large common center
is enclosed by the unchanged finite producer and exactly transported by the
solution displacement. A fresh independent review returned `GO` with no
findings and reproduced both stdout and Release-binary identities. The full
review record is [preserved separately](nonlocal-nsr3b4e2d7r20r63zh-independent-review-evidence-2026-08-28.md).

## Frozen identities

| Artifact | SHA-256 or identity |
|---|---|
| Parent commit before executable diff | `7a09048d0050202c4b329e25cd63eba446d786b4` |
| Author snapshot commit | `12bd39864931278c5598c3342cbc9cdb1f2bd041` |
| Frozen executable diff, parent to author snapshot | `74fbb15c104369356c0f9c1bb17abbbd204d15a95b8862b17494ec020f62f986` |
| Center-producer main source | `d94a223af13125fd0256fc5e3cefb5ce311db3e8c29d0de84c280cdc43f3d950` |
| Guarded R63ZG main source | `9575e7b107ee1a63b915e085249883d3242384fdc7ae6467af91d66cd48513bc` |
| Private API | `d316d5c399b69cffb735b9896865031afcbecdc0ff21cae0407602cdad593b65` |
| Boundary producer/wrapper | `5960e1906e8ff910f4ce363fef0a7a0bac5b37c894111b167d9789a76c613e4c` |
| CMake input | `3a7fc34cb903c753f3190cdef9e46e5933ee1561aa08f4af1c425413ce5a2180` |
| Boost 1.90 development package used for `cpp_int` | `7b89698c907fd5d33ccd439674fa53803139923822181c87b910579827aca379` |
| Release binary, 12,375,320 bytes | `05f2ff12d44fa8668007808886e09dac8997c2aaafa5ffd584b4f25032fa52f0` |
| Parent cache, 1,033,625 bytes | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Dev/Release stdout, 3,065 bytes | `181ac246c7a0d6e1c24543fb70e684a9357a0c183858c371867dcd068fe96722` |
| Parent prefix root | `8137eb3885523247266772e9b2dd0f8b9fd8dbe611320f30be9deeab38eaf180` |
| Exact center root | `5ca4c7181ef66289eea09f9d68c4bd81b3da7450aa1b038b2ad330dea67a847a` |
| Containment root | `0b0163b79e1aaedb330b80b04a5388bc97131d8883a5c14232eb6546d1349d3e` |
| Maximum-row root | `b37b1e146d5c12d5dc42d49547a2eb12b6676e860c380649ec2154826408039b` |
| Transport root | `32e400301175cb2c9b2c67375f9689c49d3a70e5129a6845204bf97ce06f718d` |
| Work root | `40794c9c2c5ee8088bc99304affb354d984e5af0d803c89aa0c2c798c760ab69` |
| Controls root | `9d8fc1522c5f55b4c54742840dd75108ac91b437b82c9774c9bb54cbbc5a920a` |
| Audit root | `4c3255ed5a7097c4acc9d265808ecfed4896dfbbe29aa53316d5b3c42ddf99f5` |
| Result semantic | `f1577fd413feb492e7d1ebfd2965e8f895cc170b57187d90fb907fe598a6a5ea` |

The tangent/common source expansion roots are the frozen
`38f8d0b2...0baea` and `f7e28b44...f59e8`. Their DTO roots are
`03586a1a...c6ac` and `ed001e01...2cf5`.

Raw outputs and the external Boost package remain outside Git:

```text
/tmp/nextengine-r63zh-dev.json
/tmp/nextengine-r63zh-release-run1.json
/tmp/nextengine-r63zh-release-run2.json
/tmp/nextengine-r63zh-parent-cache.bin
/tmp/nextengine-r63zh-deps/libboost1.90-dev_1.90.0-6ubuntu1_amd64.deb
```

The Release command was:

```text
CPLUS_INCLUDE_PATH=/tmp/nextengine-r63zh-boost/usr/include \
OMP_NUM_THREADS=1 \
/tmp/nextengine-r63zh-build/nonlocal-formula-center-producer-release \
  /tmp/nextengine-r63zh-parent-cache.bin
```

The strict-FP development output and both Release outputs are byte-identical.

## Exact work

```text
solution expansions                              2
copied solution components/radii          408 / 204
profile vector/matrix decodes            204 / 20808
tangent/common solution decodes          204 / 204
exact center rows/products               204 / 83232
exact center containments                        204
solution-component differences                  204
exact transport rows/products            102 / 41616
exact transport comparisons                      102
maximum-row scans/contributions           204 / 102
exact additions/multiplications    157184 / 124848
normalizations/alignment bits      303862 / 7212130
candidate/operator/solver/certificate updates 0 / 0 / 0 / 0
```

Integer normalization and alignment are oracle-internal work and do not
receive candidate-solver credit.

## Controls and regressions

Classifier precedence exhaustively covers all 256 boolean combinations.
Resealed vector, matrix, tangent-solution and common-solution component
mutations change both exact audit and result identities; an unsealed component
swap rejects. Reversed transport sign and transposed matrix access both reach
`CENTER_PRODUCER_TRANSPORT_REJECTED`, while the unchanged audit closes all
rows. An unsealed one-step radius shrink rejects before exact-row publication;
a separately constructed exact interval excluding its center reaches
`FINITE_CENTER_ARITHMETIC_DEFECT` without being attributed to the frozen
fixture.

Invalid dimension, nonfinite input, invalid binary64 exponent, checked dyadic
exponent overflow and incomplete expansion reject before row publication.
Corrupting a non-profile parent-fixture field rejects at the public solution
DTO boundary. Maximum-row sealing binds the vector term, tangent/common
products and centers, displacement transport, exact interval, and one
102-record column root containing tangent/common/transport contributions.

The parent cache and reviewed regressions reproduce exactly:

| Regression | SHA-256 |
|---|---|
| R63ZG stdout | `ca2a0f80b692a466aa97b4725fcc0ac3f553db95bb7ddf72494a754a7e2029c9` |
| R63ZF stdout | `be74e412ec6d708128506eeef5cd7284a56a2f150de8e1decb5be12ca394de0e` |
| R63ZE stdout | `4fcb94cd94c176e515aedcdb7d0d724e17bac77ec0a94c215ced2dbf4d8ef4a4` |
| Kernel-smoke stdout | `e4ff7dc634c04338d980d61214de0ca95dd9480ee34f4a139e62feb26cd381d3` |

## Independent review and next action

The fresh reviewer verified all frozen identities, exact decode and
canonicalization, signs, row-major indices, interval containment, maximum-row
sealing, transport independence, work formulas, classifier/failure precedence
and result sealing. One detached clean build and focused run reproduced stdout
`181ac246...96722` and binary `05f2ff12...52f0`; no candidate edit or repair
occurred. Correspondence review is `GO`.

R63ZH is therefore `SUPPORTED_BOUNDED / REVIEWED` at
`COMMON_SOLUTION_OUTSIDE_VERIFIER_AFFINE_MODEL`. Stop certificate repair and
do not open a deeper center factorial, widen an interval or fit a correction.
Any successor common-operator work must preserve the original affine equation
and use the common operator only for acceleration or preconditioning. Timing,
runtime, GPU and production authority remain outside this result.
