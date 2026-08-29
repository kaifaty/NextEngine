# NSR3-B4C3L ledger-normalization evidence

Status: `PASS / CANONICAL_KKT_SCALE_LEDGER_CANDIDATE`

Date: `2026-08-21`

## Reproducible result

Command:

```text
nonlocal-formula-reclosure --ledger-normalization-self-test
```

Two complete reports are byte-identical:

```text
status                 PASS
disposition            CANONICAL_KKT_SCALE_LEDGER_CANDIDATE
raw JSON + LF          05e84d545a87fd2a275368cb9a15c8b808a3accfd1ef7c9c8522fbdb175d00b8
raw JSON without LF    ba1684f09575662d10fe8646780195afc6b650c2f2bcbcfc62a7a44918a29540
semantic result        c56c80ed7d1790d575147d4afc456102904b6264443e55febff71d8dcf8719ac
wall time              95.87 s / 95.93 s
maximum RSS            8,816 KiB / 8,396 KiB
```

The full report first reproduces the preserved B4C3TAR FAIL at exact raw hash
`b5ea40b9...0d50`; B4C3TAR in turn verifies the complete older parent chain.
No historical report or canonical frame identity changed.

The isolated no-parent probe also passes:

```text
raw JSON + LF          5a82c97e0200dde05e27287eed2c8957dab17c0aa9dce64d13b627a24e455cb8
raw JSON without LF    619e3328c4b7b71105c4a5c563cae9d17ec27a13fd3cc306c5174da0fe32e0ef
semantic result        bdfd1001ac367661e96e66adf98516ab6a0b0e9d54129e21e98bc580978e61b0
wall time              8.41 s
maximum RSS            7,956 KiB
```

## Synthetic controls

All algebraic and negative controls pass:

- equal momentum/external scales produce normalization ratio exactly `2`;
- the frozen separating case gives `r_kkt=0.75e-9` and `r_max=1.5e-9`, so KKT
  admission passes while strict-max diagnostic does not;
- a `1024:1` dominant scale gives ratio `1.0009765625`;
- exact zero remains finite through the common `1e-30` floor;
- corrupt closure, KKT residual overflow, nonfinite input and invalid scale are
  all rejected.

## One-frame replay

Legacy and candidate admission both pass every B4C3A1 coarse/fine entry. The
canonical trajectory roots remain:

| Stage | Strict max residual | KKT-scale residual | Root |
|---|---:|---:|---|
| P1 / 21 | `5.414e-13` | `3.671e-13` | `94c1ea54...fdd3` |
| P1 / 42 | `4.6652e-10` | `4.5711e-10` | `ece58396...939d` |
| P2 / 1 | `0` | `0` | `596c4fe4...655d` |
| P2 / 2 | `1.2072e-15` | `6.0359e-16` | `8138d520...3ddd` |

All publication impulse, closure, energy decomposition, sample/step identity,
decode-chain and non-ledger stage predicates remain exact. The maximum ratio
of residual correspondence error to its derived forward bound is `0.6854`.

## Frame-seven discriminator

The legacy pattern remains exactly `PASS/FAIL/PASS/FAIL` for
`16/32/64/128`. With KKT-scale admission every level passes:

| Substeps | Legacy | Strict max residual | KKT residual | Candidate |
|---:|---|---:|---:|---|
| 16 | PASS | `1.270e-11` | `1.199e-11` | PASS |
| 32 | FAIL | `1.1268e-9` | `9.9175e-10` | PASS |
| 64 | PASS | `1.8717e-11` | `1.6329e-11` | PASS |
| 128 | FAIL | `1.0709e-9` | `9.9829e-10` | PASS |

The maximum residual-correspondence bound utilization is `0.7538`. Once the
normalization conflict is removed, the unchanged 16/32 embedded gate passes at
`7.9844e-5 dx`, `5.5150e-6 c` and `2.2290e-4` relative kinetic error. No level
is committed by this discriminator.

## Decision

Select `CANONICAL_KKT_SCALE_LEDGER_CANDIDATE`. Keep raw-published and strict
max-scaled residuals as mandatory diagnostics; use only the compensated
KKT-scale residual for the physical ledger gate at the unchanged `1e-9`.

This authorizes only B4C3A2 one-frame selected-policy transaction/ledger
revalidation under a new evidence identity. Complete adaptive, fixed-reference,
nominal, CUDA, runtime and production work remain blocked.
