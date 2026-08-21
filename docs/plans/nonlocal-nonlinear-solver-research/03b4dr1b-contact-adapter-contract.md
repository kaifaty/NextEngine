# NSR3-B4DR1B -- standalone external contact-adapter contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / TRAJECTORY_FORBIDDEN`

Identity projection:

```text
nextengine.nonlocal.nsr3b4dr1b-contact-adapter|v2|parent=ade621f889a08fd26ca713592625c50316b893af8c4f1797d25f4e4d4c96b86a|upstream-lib=172e6777027564566d6282f8679f4d93cbd4cdc193fc236eb9fdaa210ea07d20|geometry=outer-unit:[0,1]^3;orifice:[0,2]x[0,1]^2;wall-x=1;opening-y=.2:.4;opening-z=.4:.6|float=dt:0x3f71111111111111;radius:0x3f9999999999999a;guard=32eps|order=outer0..5,face16,edges17..20,corners21..24|schedule=8|vectors=outer-face,outer-fast,pass,graze,edge+restart,corner|env=locale-C,round-nearest,ftz-off,daz-off,omp1,dynamic-false|mutations=order-swap17-18,radius+1ulp,round-down,ftz-on|trajectory=forbidden
```

Identity SHA-256:
`c65346ec7b215a7a173eabfdd6c91e20869d4a0679b9d014a1e897369cb71f84`.

The rejected v1 draft root
`52ee4f3b4c476bfb6ddef375b1b7fda48a25b61031a1314da07ae25b5f91b8fd`
incorrectly placed the internal wall on the maximum face of the same unit
outer box. It has no implementation or evidence authority.

## Ownership and build boundary

Implement a standalone C++17 research executable under
`crates/continuum-water/tools/nonlocal-reference-adapter`. It may reuse the
engine-owned research SHA-256 utility, but it must not include, call or copy
the NextEngine water solver/contact implementation. Its contact types and
algebra are independently implemented.

The executable links the R1A pinned static closure and references the upstream
DFSPH library only as an ABI/provenance anchor. It must not construct a fluid
model, initialize a neighborhood, invoke a DFSPH step or write a payload.
External source/build/binary products remain outside Git.

The adapter build inherits the exact R1A compiler and strict flags. Exit
evidence records the complete tracked-source root, configure arguments,
path-normalized compile/link root, executable SHA-256, `ldd` closure,
`sizeof(SPH::Real)`, IEEE-754 facts and upstream DFSPH anchor.

## Process preflight

Default execution fails before contact work unless all facts hold:

- locale after environment adoption is exactly `C`;
- `fegetround()` is `FE_TONEAREST`;
- x86 MXCSR has FTZ bit 15 and DAZ bit 6 clear;
- environment values are exactly `OMP_NUM_THREADS=1` and
  `OMP_DYNAMIC=FALSE`;
- OpenMP reports dynamic teams disabled, maximum threads one and an observed
  parallel team size of one;
- `SPH::Real` is an eight-byte IEC 559 binary64 scalar with 53 radix-two
  digits; the host is little-endian with eight-byte pointers.

Private negative modes set downward rounding or FTZ before the same preflight.
They must exit nonzero with `contact_started=false`. Mutated locale/OpenMP
environment values receive the same fail-closed result. Unknown arguments are
rejected.

## Frozen geometry and response

Use a unit closed box only for the two outer-clamp vectors. Internal vectors
use the orifice outer box `[0,2] x [0,1] x [0,1]`. Radius/clearance bits are
`0x3f9999999999999a`, time-step bits are `0x3f71111111111111` and inverse time
step bits are `0x406e000000000000`. The internal wall is `x=1`; its closed
opening is `y=[0.2,0.4]`, `z=[0.4,0.6]`.

Outer component projection uses feature IDs `0..5`. Internal enumeration is
face `16`, y-edge capsules `17,18`, z-edge capsules `19,20`, and endpoint
spheres `21..24`. Capsule axial interiors are open; endpoint spheres own exact
ends. The earliest hit wins by numeric time, then equal time bits by lower
feature ID.

For at most eight hits, advance to the hit, retain the tail and remove only a
meaningfully inward normal component. The inward threshold is
`32 * epsilon * |tail|`. A touching or microscopically embedded state with an
inward tail is a `t=0` hit. Exhausting eight hits, non-finite arithmetic, a
zero normal or a nonzero remainder after the schedule is a hard failure.

## Six fixed vectors

Inputs and expected canonical outputs are in metres and metres/second. Integer
outputs use round-to-nearest/ties-to-even micrometres.

| ID | Start | Velocity | Required result |
|---|---|---|---|
| `OUTER-FACE-INTERIOR` | `(0.95,0.5,0.5)` | `(12,0,0)` | velocity `(6,0,0)`, end `(975000,500000,500000) um`, feature `1` once |
| `OUTER-HIGH-SPEED` | `(0.5,0.5,0.5)` | `(-300,0,0)` | velocity `(-114,0,0)`, end `(25000,500000,500000) um`, feature `0` once |
| `APERTURE-PASS` | `(0.9,0.3,0.5)` | `(30,0,0)` | unchanged, end `(1025000,300000,500000) um`, no feature, legal chord |
| `APERTURE-EDGE-GRAZE` | `(0.9,0.225,0.5)` | `(30,0,0)` | unchanged, exact radius graze, no inward hit, legal chord |
| `APERTURE-EDGE-IMPACT` | `(0.9,0.22,0.5)` | `(30,0,0)` | canonical velocity `(26544000,4608000,0) um/s`, end `(1010600,239200,500000) um`, feature `17` once, legal chord |
| `APERTURE-CORNER-SPHERE` | `(0.9,0.2,0.4)` | `(30,0,0)` | velocity `(18,0,0)`, end `(975000,200000,400000) um`, feature `21` once and no edge feature |

Two additional algebra sentinels are mandatory because the six-vector parent
list does not exercise every critical branch:

1. internal solid face: `(0.9,0.1,0.5)` at `(30,0,0)` ends at
   `(975000,100000,500000) um`, velocity `(18,0,0)`, feature `16` once;
2. edge restart: derive the feature-17 contact point for `y=0.22`, move its
   x-coordinate one representable value toward the edge axis and apply
   `(30,0,0)`; `t=0` must activate feature `17`, yield canonical velocity
   `(19200000,14400000,0) um/s` and a legal chord.

## Independent validation and roots

For every accepted result independently validate finite state, outer-box
radius clearance, internal face/edge/corner clearance and absence of a
straight wall chord outside the radius-safe closed opening
`y=[0.225,0.375]`, `z=[0.425,0.575]`. Tangency is accepted; penetration is
not. The contact solver cannot call the validator as its distance oracle.

A canonical contact-profile projection binds all scalar bits, geometry,
feature order, schedule and vector inputs. Swapping IDs `17/18` in the order
projection and incrementing the clearance bit pattern by one must each produce
a distinct SHA-256 root. They do not alter the selected base profile.

Default execution runs twice in fresh processes with byte-identical stdout and
zero stderr. A one-byte mutation of the captured stdout must change its hash.
R1B PASS authorizes only R1C scenario-manifest design. It creates no historical
W1 credit, external-reference selection, trajectory, runtime authority, public
schema or production claim.
