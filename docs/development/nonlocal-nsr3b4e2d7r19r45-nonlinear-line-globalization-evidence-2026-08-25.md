# NSR3-B4E2D7R19R45 nonlinear line-globalization evidence

Date: `2026-08-25`

Status: `PASS / COMPOSITE_MERIT_RECOVERY_REQUIRED / ROLLBACK EXACT`.

## Outcome

The scale-aware R44 line is finite, topology-stable and contact-safe, and it
contains precision-confirmed nonlinear common-descent steps. None of its 25
predeclared dyadic candidates recovers positive complete merit relative to
the original source. The simple composition

```text
R43 projected normal step + one R44 projected-gradient line
```

is therefore rejected for this state. This is not a formula failure and not a
line-domain failure; the fixed normal step creates more complete-merit debt
than this tangential steepest-descent line can recover.

## Independently bounded domain

| Bound | Dimensionless `alpha` |
|---|---:|
| inherited global-L2 trust | `0.24999897094557438` |
| R42 skin certificate | `0.6671069219637521` |
| exact inactive-face contact slack | `4.443790006595189e-6` |
| executable `alpha0` | `4.4437900065951881e-6` |

Contact slack owns the finite interval, but the observed best region is over
an order of magnitude inside that cap. Trust and Verlet-superset capacity are
not the active obstruction.

The normalized direction is exact global-L2 unit length with root
`95f6b64fe8819c17936535fc543c32aaa270daaea438a384db3cca770e799694`.
Its inherited unit slopes are strictly negative:

```text
complete merit  -5.4585174082622525e-9
density hinge   -5.3079373058229058e-9
```

## Finite-line result

All 25 fixed candidates pass direct trust/topology/contact traversal. The
first candidate that also closes the local nonlinear model gates is
backtrack 5:

```text
alpha                         1.3886843770609963e-7
source complete-merit change -1.2419675480823773e-15
```

The largest observed source-merit reduction occurs one rung earlier at
backtrack 4:

```text
alpha                         2.7773687541219926e-7
source complete-merit change -1.1966013910937007e-15
```

It remains strictly negative and has not yet passed the local nonlinear
common-descent gates. The gap is not a sign-resolution artifact: 20 normalized
long-double audits resolve without disagreement, binary128 is never required,
and `precision.blocked=false`.

The complete ladder root is
`c5992cf59a6bd33a8f4f30cc5260b1253ab83c35a975dcd532cab8dbb52a5d83`.
Work closes at 25 masked trial workspaces, 50 divided evaluations, 900,000
box-face tests, maximum three live workspaces, zero HVP/model/outer work and
exact rollback.

## Reproducibility

Implementation commit: `282679c0`.

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r45-a.WYW8Oh
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r45-b.SAMv7l
binary SHA-256 a693c9542776a891e40b204e4dedffca4ccc76dbec369098d2f12c0e8d53e515
size           7714840
ELF build-id   f52288bbcd00ea83c48c09859f7dcef7041c770d

/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r45-a.yhLUFn
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r45-b.QaWivj
stdout bytes   1807
stdout SHA-256 2ab62170eb816eee7f71ee330e5303cdb6852a241cd5e7c54160fe0914cbabc7
semantic       55c04d4294290e212a48911ac717df10be34e39013751ed387b184df479a3773
route cases    13 / 13
route root     f543f711a02dfbddcfc0094e98f075579bd2c0566823d9eaa583d276772b66fa
```

Both clean Release binaries and stdout payloads are byte-exact. Each R45 run
also reproduces the exact R44 parent stdout SHA
`2fb4299081627e36752d13b4d3ca53046b3e19e8b72596bc318bb4ab85355292`.
No performance timing was measured or interpreted on the shared host.

## Decision

Preserve R45 as negative evidence. Do not retry a denser scalar ladder along
the same direction and do not tune contact/trust/penalty to manufacture source
merit. Research filter-SQP/restoration globalization next: a feasibility
restoration step may be admitted through an explicit nondominance/sufficient-
decrease filter rather than requiring every normal step to monotonically lower
one combined merit. That is a new acceptance policy and must be frozen before
any state application or following outer.
