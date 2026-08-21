# NSR3-B4D external-reference re-attestation evidence -- 2026-08-21

Status: `FAIL / MISSING_ARTIFACT / NO_TRAJECTORY`

## Outcome

The B4D reader is implemented and fails closed at the exact external evidence
boundary. Its compiled identity reproduces
`47c78bdb115c0e5d7ed7132a6de3e62e3ba9533396a7f346b510b354dc5222fe`,
the frozen W0I attestation root is exact, and no trajectory starts.

All three required external files are absent:

| Scenario | Expected bytes | Observed bytes | Result |
|---|---:|---:|---|
| `CW-HYDRO-001` | 7,344,252 | 0 | `MISSING_ARTIFACT` |
| `CW-DAMBREAK-001` | 26,064,772 | 0 | `MISSING_ARTIFACT` |
| `CW-ORIFICE-001` | 26,064,772 | 0 | `MISSING_ARTIFACT` |

The deterministic first failure is
`CW-HYDRO-001:MISSING_ARTIFACT`. Both
`external_reference_candidate_selected` and `b4e_design_authorized` are
false; nominal execution, runtime and production authority are also false.

This is not a failure of the Nonlocal formula, solver trajectory or the
historical DFSPH result. The payload-dependent format, complete-hash and
mutation gates are `NOT_RUN(MISSING_ARTIFACT)` because inventing bytes to make
those controls execute would invalidate the evidence.

## Exact execution

The focused CMake target builds with the frozen strict-f64 flags. Two complete
reader executions both exit `1` and produce the same stdout-with-LF SHA-256:

```text
fbeb404093b0ff808c230c5784836912d843066654f49e9c944b1763ae23ddd9
```

Before the B4D implementation, the complete packaged-runner probe re-attested
the B4C4C1 identity and semantic result:

```text
identity = 66e318cb69e0b0c0a3a40a2beafa2099ebf151287581b191a242e82dac6d6f3c
result   = b4d5260012f4208026b814411891a221dda2d5823b242027d890d4066e69550c
wall     = 60.25 s
```

The formula/profile/source hashes are listed in the
[frozen B4D contract](../plans/nonlocal-nonlinear-solver-research/03b4d-reference-reattestation-contract.md)
and all matched at design preflight.

## Decision

Preserve B4D as `FAIL / MISSING_ARTIFACT`; do not reinterpret it as a physics
failure and do not authorize B4E. The next bounded research stage must audit
whether the exact W0I generator lineage is reproducible from retained source,
patch and toolchain artifacts. If the original adaptation is irrecoverable, a
new independently frozen reference profile must receive new roots and rerun
its own comparator closure; it cannot inherit W0I hashes or W1 credit.

