# NSR3-B4E2D7R19R46 filter-globalization evidence

Date: `2026-08-25`

Status: `PASS / FILTER_FEASIBILITY_STEP_CANDIDATE / ROLLBACK EXACT`.

## Outcome

The exact R43 contact-safe normal step is a robust filter-feasibility
candidate when the normalized inertial objective and density violation are
kept separate. It passes all 24 predeclared sloping-filter envelopes and even
the strongest `gamma=1/2` envelope by feasibility. It passes none by
objective reduction.

This resolves the R45 obstruction narrowly:

```text
monotone combined AL merit: rejects R43
separate (h,f) filter:       admits R43 as a feasibility candidate
```

No state, runtime filter or following outer was committed. R46 is an
admission lemma, not a complete filter-SQP algorithm.

## Separate coordinates

The filter uses the mathematically distinct normalized coordinates

```text
f(y) = 0.5 * ||y - y_hat||^2
h(y) = ||max(c(y),0)||_2.
```

| Coordinate | Source | R43 trial | Change |
|---|---:|---:|---:|
| inertial objective `f` | `7.6129413895435875e-13` | `8.0152688022051136e-13` | reduction `-4.0232741266152028e-14` |
| violation norm `h` | `8.1139951163476689e-8` | `3.7024372773750754e-8` | trial/source `0.45630262580705666` |
| hinge `psi=0.5*h^2` | `3.2918458374056909e-15` | `6.8540208964482797e-16` | reduction `2.6064437477608628e-15` |
| positive rows | `1420` | `464` | `-956` |

Thus the normal step increases the pure objective by about `5.28%`, while it
reduces the constraint-violation norm by about `54.37%`. Combining both into
one AL merit had hidden this trade-off.

The pre-cancelled objective difference repeats byte-exactly. Both objectives
are finite and nonnegative; the PHR density term is excluded from `f` rather
than counted again in both filter coordinates. Coordinate root:
`6accd10d3b82912d3ba55c36b854aa0a15610719a24bc2f6f3b336fb5d9cbc20`.

## Filter and model result

Every fixed `gamma=2^-k`, `k=1..24`, accepts by the `h` branch:

```text
accepted margins 24 / 24
h branches       24
f branches        0
strongest gamma   0.5
strongest h slack 3.5456028079875901e-9
resolution guard 1.8449088921324048e-20
slack / guard     1.9218308411368048e11
```

Margin root:
`3a9736f23ca28116b3a9c4f2f64585e178197a4bea5bddbbf8fee28cd4220142`.
The inherited normal-step model remains trustworthy: predicted hinge
reduction is `2.6064437667433723e-15`, actual reduction is
`2.6064437477608628e-15`, and `rho=0.99999999271708462`.

The initially empty filter root is
`83c5b0bf7a124ab0b9732f2a77b4dfa6b3c96f077306c57f23f3424071e44428`.
The source entry root is
`6dd5f24766c14efd390df5fe178b0ee32d6a7afa5cfc51b517762c48175ad550`,
and the hypothetical one-entry next-filter root is
`19c1311bf84aaa236a8ce4ec82359ffadb6db404a721ea804f66daabef472c19`.
The filter update remains report-only.

Exact source/trial superset coverage, independent canonical pair membership,
all 36,000 contact-face tests, two workspace builds/releases, maximum two
live workspaces and rollback close. New/worsened contact counts are both zero.

## Identity reclosure retained

The first executable attempt failed before scientific classification because
the documented identity SHA owned a trailing line feed while the C++ raw
string did not. The immutable 2,406-byte identity text and every gate were
unchanged; the annotation was explicitly reclosed to raw SHA
`afa98b317f26e6cc6a11d26253d2d27723d4e77f7f44b89ff3eabd9100214ee4`
in commit `50c1246a`. The invalid attempt receives no filter evidence credit.

## Reproducibility

Implementation commit: `b292aec5`.

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r46-a.qwMGij
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r46-b.44Akkq
binary SHA-256 301544602dddec29444de23958cc2a99bf6f5b0300b503f90bce6eee4c816824
size           7744160
ELF build-id   21c5b2c5ebc0f1d4f0c686b9fe62c6d4f0807117

/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r46-a.ld1TK2
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r46-b.TWMDiP
stdout bytes   2549
stdout SHA-256 ecbdc13b1cf1ca889fed1f0ce98f4f4613c06eb222d2e132271cd23cdddaf4e3
semantic       8b3390babbb896cef73a487bbf2512abc8f2031ff6ad7299d506928ac0d59cf0
route cases    10 / 10
route root     ea6cf7639cbabb3da1ea639e9d2390f6ec8086cb2ff6eef958a7c8949121cccd
```

Both clean Release binaries and stdout payloads are byte-exact. Each run
reproduces the exact R45 parent stdout SHA
`2ab62170eb816eee7f71ee330e5303cdb6852a241cd5e7c54160fe0914cbabc7`.
No performance timing was measured or interpreted on the shared host.

## Decision

Preserve R46 as positive filter-admission evidence, but do not directly
accept R43 or select `gamma=1/2` for production. Research/freeze R47 as one
complete rollback-only filter transaction: objective/feasibility switching,
finite filter insertion/removal, trust-radius response, restoration entry and
exit, exact ownership and rollback. Only after that lifecycle closes may a
following outer-state experiment be considered.
