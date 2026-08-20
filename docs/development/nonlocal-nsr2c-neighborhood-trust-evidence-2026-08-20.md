# Nonlocal NSR2-C neighborhood trust scaling evidence -- 2026-08-20

Status: `FAIL / SCALE_8X8X8_WORK_GATE / NSR3_BLOCKED`

## Outcome

The canonical neighborhood operator remains exact and bounded, and the
unpreconditioned trust solver converges on every scaling fixture. The frozen
operation-count gate nevertheless fails at 512 particles: the solver needs 13
rejected trials and 153 Hessian-vector products, above the declared limits of
8 and 128. The failure is preserved; the limits are not widened after seeing
the result.

| Fixture | Result | Outer / rejected | HVP | Max pairs | Max neighbors | Scaled residual |
|---|---|---:|---:|---:|---:|---:|
| 2x2x2 correspondence | exact PASS | `7 / 0` | 18 | 28 | 7 | `3.849611176203042e-11` |
| 3x3x3 correspondence | exact PASS | `7 / 0` | 16 | 351 | 26 | `1.9825890765778257e-10` |
| 4x4x4 correspondence | exact PASS | `8 / 0` | 24 | 1880 | 63 | `2.2772710983764147e-9` |
| 5x5x5 scale | PASS | `10 / 0` | 35 | 3409 | 116 | `5.2960585873218706e-10` |
| 8x8x8 scale | **FAIL** | `26 / 13` | 153 | 19492 | 122 | `4.1954374504256254e-11` |
| 10x10x10 scale | PASS | `13 / 0` | 45 | 42144 | 122 | `5.3986699290156735e-9` |

The failing case still reduces the objective from
`-1435.295643928913` to `-1439.7146476510072`, converges under the frozen
scale-aware criterion and preserves internal momentum to
`3.543861934791831e-16`. It is a work/rejection failure, not divergence or a
capacity failure. The non-monotone size result (512 fails while 1000 passes)
does not justify a generic conditioning claim without a per-trial trace.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| raw report, run 1 | `efc5a14d4b096afc15ed099b166786da1c30a276eecfec8018e2f4b6fcf720a4` |
| raw report, run 2 | `efc5a14d4b096afc15ed099b166786da1c30a276eecfec8018e2f4b6fcf720a4` |
| semantic result | `856e89cede4e7032700f697312c8af3c42e1892df4567848c7dfc1491d1f9d34` |

## Decision

Do not authorize NSR3 performance work. Freeze NSR2-C1 as a report-only causal
diagnostic over the unchanged 512-particle execution. It must distinguish
trust-model/topology mismatch, pressure active-set change and arithmetic-floor
termination before any global preconditioner or trust-policy remediation is
proposed. The rejected `block-gn-metric-v1` remains closed.

