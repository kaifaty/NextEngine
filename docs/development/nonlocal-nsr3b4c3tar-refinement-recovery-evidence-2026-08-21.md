# NSR3-B4C3TAR refinement-recovery evidence

Status: `FAIL PRESERVED / KKT RECOVERY VALID / LEDGER NORMALIZATION BLOCKED`

Date: `2026-08-21`

## Complete frozen report

Command:

```text
nonlocal-formula-reclosure --canonical-adaptive-recovery-self-test
```

Result:

```text
status                 FAIL
first failure          p1-supported-adaptive-balanced-recovery:ADAPTIVE_LANE
candidate failure      FRAME_CANDIDATE:CANONICAL_STAGE_RUN_GATE
raw JSON + LF          3b39938df86955045815d39b771c993dccbb064bdbe7effc4b47d6133d1f3dcc
raw JSON without LF    b5ea40b96a812fdf090e7982d036ac3a893f13e6d8720e5381d131d4a02d0d50
semantic result        5a87dde2b546df086124abe2d1597bda5e8d0a651088f233cea91dbb53acc4b1
wall time              87.47 s
maximum RSS            8,352 KiB
```

The report re-executed the preserved B4C3TA FAIL and both diagnostic parents
at their exact hashes. The recovery classifier, non-adjacent selection,
exhaustion and rollback negatives pass. The post-commit injected failure also
preserves all 42 committed frame/ledger entries exactly.

P1 recovers two real nonlinear failures:

| Frame | Initial | Failed level | Passing pair | Selected |
|---:|---:|---:|---:|---:|
| 4 | 16 | 16 | 32 / 64 | 64 |
| 5 | 21 | 21 | 42 / 84 | 84 |

It commits seven macro frames and 332 accepted substeps before frame seven.
All 563 attempted KKT substeps, including the failing attempts, are accounted
for exactly. P2 completes unchanged with 82 accepted / 124 attempted substeps;
its schedule, contacts, binary envelope, publication-energy budgets and the
restored local free-flight gate all pass.

At P1 frame seven the 16-substep level passes, while the 32-substep level
completes all KKT solves and canonical publications but fails only
`publication_ledger_exact`. It is correctly non-recoverable under the frozen
B4C3TAR contract, so no frame-seven state or ledger entry commits.

## Isolated lane repeat

The r1 P1/P2 lanes without parent-chain cost were executed twice and were
byte-identical:

```text
raw JSON + LF          a2fd2d95a2bd961837a10e40e54e5100dea419ae96c38a3e0b9d4cf5300a8dda
raw JSON without LF    81b8e4eda7405736bb72b9feb793c2dfcdd59ba67a0451ce658c82d2288664e9
semantic result        fd503e57f72ec139512116c3ffecb250da387618296fd58d1bb056d9c98b8194
```

## Ledger refinement probe

The exact committed frame-seven state was evaluated at all four planned
levels. Two runs were byte-identical. The final diagnostic report has:

```text
raw JSON + LF          d9977614b92b5edddd3919b60daa4f10cf885c3e466aa1dc6f63c68057ecc829
raw JSON without LF    721768b14325fecab3e64477007ec8e85673c92741ca55869acc49c33b0eea8f
semantic result        0b4a104e2c9b41fb1aeee38e904d65ce6a328135a8272e1b7a9dca434fa9271c
```

| Substeps | Stage | Publication residual | KKT residual | Closure |
|---:|---|---:|---:|---:|
| 16 | PASS | `1.270e-11` | `1.199e-11` | `4.14e-25` |
| 32 | FAIL | `1.1268e-9` | `9.9175e-10` | `0` |
| 64 | PASS | `1.8717e-11` | `1.6329e-11` | `0` |
| 128 | FAIL | `1.0709e-9` | `9.9829e-10` | `3.31e-24` |

The failed 32-substep residual has absolute norm `1.9995e-11 N s` at a
publication scale `1.7745e-2 N s`; the 128-substep failure is
`4.5490e-12 N s` at `4.2477e-3 N s`. Refinement is non-monotonic: 16 and 64
pass, 32 and 128 fail, so no adjacent passing pair exists.

Most importantly, the compensated publication vector reproduces the KKT
ledger to zero or near-zero closure, while the publication residual rejects a
KKT residual already below its unchanged `1e-9` gate. The two paths use
different normalizers:

```text
KKT scale          |delta momentum| + |gravity| + |support| + |contact|
publication scale  max(|delta momentum|,
                       |gravity| + |support| + |contact|)
```

This is a gate-semantics conflict. It is neither quantization impulse, state
corruption nor evidence that additional refinement will converge monotonically.

## Decision

Preserve B4C3TAR FAIL. Do not classify `CANONICAL_STAGE_RUN_GATE` or ledger
failure as recoverable and do not raise the `1e-9` threshold. Freeze a separate
ledger-normalization reclosure that keeps the strict residual as a diagnostic
but gates the compensated physical ledger using the same normalizer as the KKT
state it claims to reproduce. Revalidate one-frame and long-state transactions
before resuming the complete controller.
