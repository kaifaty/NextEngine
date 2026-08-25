# NSR3-B4E2D7R19R58 directed-enclosure fixed-point evidence

Date: `2026-08-25`

Status: `PASS / FIXED_POINT_CONTRACTION_CANDIDATE / ROLLBACK ONLY`.

Implementation commit: `9c3bcc44`.

Frozen identity SHA-256:
`a0327b38e7d2f7552bfc223efdb99a796e968ec8cfa8b70560ac32493206542f`.

## Result

R58 reproduces the exact R57 public result, captures its selected witness and
directed upper vector privately, and evaluates the frozen R57 refinement map
exactly once more. All four checkpoints remain contact/ball feasible, contain
every positive row inside the 494-row master and have zero raw-positive rows:

```text
cycles                  8              16              32              64
raw positive            0               0               0               0
directed positive     237             235             234             234
maximum upper    1.1829781e-24   1.3678207e-24   1.3678207e-24   1.3678207e-24
model maximum    2.5232447e-26   1.1898440e-27   2.3922857e-30   9.1148107e-36
outside master          0               0               0               0
```

Against the exact R57 source, the terminal maximum directed upper contracts
by `4.65112286567982x` and the positive count falls from `245` to `234`.
Therefore frozen precedence selects `FIXED_POINT_CONTRACTION_CANDIDATE`, not
stalling. It does not certify compatibility.

The exact equality of maximum upper at cycles 16, 32 and 64, while the inner
model maximum falls by more than eight orders of magnitude, identifies a
binary64 enclosure plateau. More Dykstra depth does not target the observed
limiter.

## High-precision terminal sign

The selected witness remains strictly raw feasible:

```text
full binary128 resolved positive       0
full binary128 resolved negative    6000
full binary128 unresolved               0
maximum full binary128 raw   -5.198749910449811e-22
```

Thus all 234 remaining binary64 positives are bound-only. This is not evidence
that the current runtime certificate may be weakened: the binary128 traversal
is an offline attribution oracle, while the unchanged binary64 directed upper
remains authoritative.

## Work and roots

New work is exactly one passive R57 replay, 64 density sweeps, 64 grouped box
blocks, 64 fresh pair-once residual refreshes, four directed checkpoint audits
and one selected pair/binary128 decomposition: 69 new pair passes and one quad
row traversal. No third fixed-point outer, basis, Gram column, projection, HVP,
nonlinear trial or state mutation is admitted.

```text
source upper  9669608782c217887c3569a1be7490a9ed484806d1dbeb8c626055382c15d605
selected      86ad77c25f07d0337b0c2208ac208271c4d163d657c15711d5e78cfcd6c38e95
witness       040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd
correction    7f5a7c242443b35c54616e4604531bca77c0dd3e2e7e21e850b105559a30fb1b
box dual      da085eec472077b9ce00e8ce4881cfb3e9de52910cd5c8a60373fbf4fe3bde1b
comparison    a16fc80e265f227eb101a175b7427dce7f9bc3f2db62ae6ae6dffd6ca13247c2
state         a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2
routes        8016aa1de76edb6fa6ede730f247b89de90b81fb933c905d9d32153990361275
semantic      f7070521395450cc3b54bfe85c9c4c2648abfa87ff37cfaa419da317628d74c3
```

R57 remains byte-exact at stdout SHA-256
`a1930bc92665cb08f5f46ff53e9b53da48d09438d3bc52c25ec848a2db0a6b5c`.
All source and runtime state rolls back exactly; no witness is applied.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r58-final-a.pDCcLU
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r58-final-b.U8bD11
binary SHA-256 c219c824dbdeb00e931ee998acd5630fa035d8a2d9a7fd8fde9a570297da692f
size           8285152
ELF build-id   b2f881bdefc94a71fac2982c2c7bd54fcef60321
stdout bytes   4044
stdout SHA-256 8078c06230d6436253df966eb617bee202237143a1cf5def0e1dac49028ec257
```

Both independent Release binaries and outputs are byte-exact. Wall time is not
performance evidence. Runtime and production authority remain false.

## Consequence

Preserve R58 as the final bounded fixed-point composition. Do not execute a
third outer or add inner depth. Research R59 as a read-only decomposition of
the directed binary64 enclosure: attribute the 234 bound-only rows to row
degree, local product rounding and accumulation rounding under provable bounds.
Any tighter certificate must be derived and frozen before it is evaluated;
binary128 remains an offline oracle and may not become runtime authority by
convenience.
