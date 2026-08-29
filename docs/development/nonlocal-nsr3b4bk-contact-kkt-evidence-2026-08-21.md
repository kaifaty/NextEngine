# NSR3-B4BK box-contact KKT evidence -- 2026-08-21

Status: `FAIL / SIDE_FACE_HYPOTHESIS / NUMERICAL_CANDIDATE_NOT_REJECTED`

## Reproduction

```text
nonlocal-formula-reclosure --box-contact-kkt-self-test
```

Three executions are byte-identical:

```text
raw JSON plus LF  21d63ff86b2d248623bd5bdd8ba363e7caca4142ff9a2cbdcb337be6553438e9
JSON without LF   b110585a9e8894126666c4f5c941b447361b15e3ea771deebdf0e760afbec716
semantic result   7cb256e5b1c62db865c7230a03a94a9553b07f1112a7e9552c78bd6b273a9f93
```

The first failure is `P1_KKT_48`. All three historical split replays are
exact, including the B4B fixed-96/192 floor failures.

## Constrained numerical result

The bound-constrained solver itself returns PASS for every P1 step:

| fixed steps/frame | projected KKT impulse, N s | mixed limit, N s | complete ledger | trials / HVP | floor accepts |
|---:|---:|---:|---:|---:|---:|
| 48 | `1.4667673350843271e-13` | `5.1093840760732262e-12` | `6.5508514441638343e-11` | `3 / 4` | 1 |
| 96 | `1.1211852092612074e-15` | `2.5546920380366131e-12` | `9.688450291723178e-13` | `3 / 4` | 2 |
| 192 | `3.8468723262332739e-13` | `1.2773460190183065e-12` | `2.7108011612359769e-10` | `2 / 2` | 1 |

Every state has exact feasibility/complementarity, nonnegative multipliers,
zero penetration, pressure-support translation closure below `4e-16`, exact
contact impulse closure and an objective below the feasible projected
predictor. Detached P2 is bit-identical to free flight with zero pressure,
multiplier and reaction.

## Why r0 still fails

r0 froze `lateral_or_upper_axes == 0`. Each solution instead contains:

```text
lower axes = 40, upper axes = 24
lower-y axes = 16
lateral or upper axes = 48
```

This is the exact face population of the filled cross-section:

```text
x- 12, x+ 12, y- 16, y+ 0, z- 12, z+ 12.
```

The aggregate x/z contact impulses are only roundoff (`<=1.1e-16 N s`), so
the lateral multipliers express symmetric wall pressure, not drift. The r0
gate is a derivation error in fixture geometry, not evidence against the KKT
candidate.

## Decision

- Preserve r0 FAIL and its raw artifact.
- Freeze r1 before implementation.
- Change only the face-population assertion: require exact counts, no upper-y
  multiplier and opposing lateral impulse symmetry under the inherited
  `1e-12 N s` contact-closure scale.
- Retain every solver action, KKT/ledger threshold and detached negative.
