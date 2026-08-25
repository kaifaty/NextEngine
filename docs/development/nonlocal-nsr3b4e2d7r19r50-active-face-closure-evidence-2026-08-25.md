# NSR3-B4E2D7R19R50 active-face closure evidence

Date: `2026-08-25`

Status: `PASS / ACTIVE_FACE_HILDRETH_CLOSURE_CANDIDATE / ROLLBACK ONLY`.

Implementation commit: `50c47ef8`.

Frozen identity SHA-256:
`ab745dfcfede239ed739d5bec80f1bc6c9e74be36b3bd5daa6887f830f263c75`.

## Result

R50 reproduces exact R49 and executes the frozen minimum-norm active-face
Hildreth reference:

```text
accepted outers                 4
Hildreth sweeps                32
unique cached rows            492 / 512
source directed-positive      366
terminal raw-active           487
terminal directed-positive    488
terminal psi                   4.4684544709411563e-24
terminal h                     2.9894663306152676e-12
terminal maximum row upper     1.0297972392379541e-12
route                           ACTIVE_FACE_HILDRETH_CLOSURE_CANDIDATE
```

Every outer builds an exact active Gram from cached `A^T e_i` row gradients and
`A A^T e_i` columns, completes all eight fixed Hildreth sweeps, projects the
minimum-norm correction through the unchanged ball-box operator and admits only
a fresh directed dyadic candidate. All four outers make strict simultaneous
progress in `psi`, `h` and maximum upper.

The all-row certificate remains open. One raw-inactive row is positive only
under the directed enclosure, and 488 certified-positive rows remain. No
witness, R43 state, filter entry or following outer is committed; restoration
exit remains false.

## Comparison with R49

R50 improves the R49 endpoint by:

```text
h                              47.314033303827046x
psi                          2238.6177474756551x
directed maximum upper         19.199358012740909x
```

The witness norm changes only from `5.0385810268018367e-7` to
`5.0436317754490575e-7`, still negligible relative to the `0.03125` normal
radius. Contact/trust geometry is not the observed limiter.

The positive-row count grows from 366 to 488 while amplitude falls. This is
active-face migration: each R50 outer solves only rows positive at that outer
and resets its dual, so a previously released halfspace may re-enter later. The
cache reaches 492 of its frozen 512 slots. This does not justify a fifth outer
or a larger cache; it motivates persistent constraint generation.

## Work, controls and rollback

Exact new work is 492 row VJPs, 492 Gram-column JVPs and seven directed audit
JVPs: 991 pair passes. Four projections require four scans because the target
corrections remain interior. Basis/Gram caching is a correctness oracle, not a
scalable production structure.

Source, workspace, geometry, capacity, Gram symmetry/diagonal, Hildreth,
projection, globalization, directed audit, work, route and rollback controls
pass. Cache and outer roots are
`31a6713866e951e716b8fd77e40ae532c3d9d281d3b40b0d16cd8039c2c5112e`
and `04a91d0379f71f9c6dfb711bf385c8771837bffde1f176c995ebe7d814c5b20b`.
Fourteen precedence routes close at
`01535d31aca3be4e4502872c6d1c259877ba14fc99d7d6f281c1bb0b86c259e2`.

Semantic result SHA-256:
`a97b34d19b19b6d39725de8fdaf9ce38db924b7d39bfc823e07ea9bee86c7d8d`.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r50-a.dlBW9r
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r50-b.Ws0U6N
binary SHA-256 ff55c0c2974c07738c658beadbab9a2487bc1749980277d9bc3874990420896c
size           7934064
ELF build-id   231b575764aa7f1fced4259fd06538537afa9e94

/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r50-a.nVuC7R
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r50-b.T5reiW
stdout bytes   1706
stdout SHA-256 1ee396be61be90534573b8c476256d80b46ee803584617a6ab34900b8727b56d
```

Both independently built binaries and both concurrently executed stdout
payloads are byte-exact. Each run reproduces exact R49 parent stdout SHA
`7e1dde137b6164f67a448ab704eb713148e08c3a3a6e693614cb2645b01fa656`.
Concurrent wall time is not admitted as performance evidence.

## Consequence

The halfspace formulation is effective, but the current-positive-only master
does not preserve previously discovered constraints. R51 should reuse the exact
R50 492-row cache, merge any missing terminal-positive rows without exceeding
the unchanged 512 capacity, and solve the persistent union. Nonnegative dual
coordinates may release inactive rows naturally; rows themselves should not be
dropped from the master. The independent directed certificate remains
unchanged, and no runtime/restoration transaction is authorized first.
