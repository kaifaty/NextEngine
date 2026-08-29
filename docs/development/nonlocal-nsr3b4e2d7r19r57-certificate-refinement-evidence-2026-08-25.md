# NSR3-B4E2D7R19R57 certificate-aware refinement evidence

Date: `2026-08-25`

Status: `PASS / CERTIFICATE_REFINEMENT_ENCLOSURE_FIXED_POINT_REQUIRED / ROLLBACK ONLY`.

Implementation commit: `cd9782ea`.

Frozen identity SHA-256:
`2a41c16b73c553a966c2e73f2a435a70f99a6d1eb766f359e78e247da69b5d9e`.

## Result

R57 solves one new minimum-norm grouped-box problem from the exact R55
cycle-64 witness using its complete directed upper vector as the shifted
halfspace source. All four checkpoints are contact/ball feasible, keep every
positive row inside the 494-row master, and already have zero raw-positive
rows:

```text
cycles                  8              16              32              64
raw positive            0               0               0               0
directed positive     476             474             407             245
maximum upper    1.5765910e-21   1.9271785e-22   8.8434439e-24   6.3619021e-24
model maximum    1.5771680e-21   1.9250096e-22   2.8477815e-24   6.1443737e-28
outside master          0               0               0               0
```

The unchanged binary64 directed upper improves by `6511.43x` from the R55
source. The frozen first-certified rule finds no certified checkpoint and
therefore selects cycle 64, yielding route
`CERTIFICATE_REFINEMENT_ENCLOSURE_FIXED_POINT_REQUIRED`.

## High-precision terminal sign

The selected witness has no true raw violation:

```text
full binary128 resolved positive       0
full binary128 resolved negative    6000
full binary128 unresolved               0
pair-positive binary64                   0
maximum full binary128 raw   -5.198662514661014e-22  row 5196
maximum binary128 lower      -0x1.3a3d67edacdb78698adbceb43e35p-71
maximum binary128 upper      -0x1.3a3d67edacdb784241d9104bc1cbp-71
```

Thus the solver refinement has crossed into strict raw feasibility, and even
the binary128 forward enclosure is strictly negative on every row. The 245
remaining positives exist only under the unchanged, more conservative
binary64 gamma enclosure. This is no longer a physical residual or operator
alignment issue.

The added correction norm is only `1.163112923000567e-18`; the resulting
witness norm is `5.043743828321581e-7`, far inside the normal radius. The box
remains exact without a post-terminal projection.

## Work and roots

New work is exactly 64 density sweeps, 64 grouped box blocks, 64 fresh
pair-once residual refreshes, four directed checkpoint audits and one terminal
pair/binary128 decomposition: 69 pair passes and one quad row traversal. No
row basis, Gram column, projection, HVP, nonlinear trial or outer is built.

```text
source upper  1a4a52596e67056eed2b80f48b7ba553410be1bb858c288781ee5b5f30302994
selected      856dc573d7b1320659a554d0b33522a74b03eb34dc1747518e7692a684158231
witness       e4c5aa2d5cdddaaeb55c620d37f43856d5ed797f199fad2253175bef2ddd341d
correction    18bc74476b5887a4b832e226c8f09984a91e91d84d8636d4ceffd4a9c824d53b
box dual      e58553b7864502442b95d8b20e20e16a255afe0b59f64d43bf1f3e6fc7ca7d5f
comparison    b32472cf77980de92e3d1caeb65d56f100babcb3896c2c1dfcf62e5ecc7655fb
state         952d2541ed707bc36bec78e10f32ae6c60bd062c081f3811a611fe68ead27d63
dense         3a2d9b973b464e9264cf4428bd46ec153e796103bd07447ff0fa00f36a4ef98c
routes        332199c3de27d7d01d09431059c9ac5fb47d74798ad1972914a36b6efc2696c7
semantic      666e9f62ab3e0ac027bd855893f90c08dc2d61433f913170e736c6d9e50a39b6
```

R56 remains byte-exact at stdout SHA-256
`e0ececa2d57448248e86d623d4c1e15e69abe68576c503e1e802ef990b7c46aa`.
All source and runtime state rolls back exactly; no witness is applied.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r57-final-a.6xX1U5
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r57-final-b.fAydf3
binary SHA-256 e44488b6a487182006ae8be3ea83d96bd5ecd475d74d2173befc4262b3ef9d95
size           8255784
ELF build-id   4d7b5a92b7fbb055adc2b3b6e3ce232daed6b151
stdout bytes   4779
stdout SHA-256 a1930bc92665cb08f5f46ff53e9b53da48d09438d3bc52c25ec848a2db0a6b5c
```

Both independent Release binaries and concurrent outputs are byte-exact. Wall
time is not performance evidence. Runtime and production authority remain
false.

## Consequence

Retain the R57 raw-feasible witness as private diagnostic state only. Do not
fit a tolerance, weaken gamma or claim compatibility from the binary128 oracle.
Research R58 as a bounded fixed-point update of the unchanged binary64 directed
upper: use the exact R57 selected witness/upper as the next certificate-aware
source and test whether one additional refinement closes all 245 bound-only
rows. Predeclare stalling/contraction routes. SHQP or extreme-point-corrected
semismooth Newton remains the fallback if fixed-point refinement does not close.
