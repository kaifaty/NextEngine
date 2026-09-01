# Physical sound V21 — F0 protocol conformance audit

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / PREDECESSOR_PROTOCOL_REJECT / ONE_LITERAL_MISMATCH / NO_NEW_VALUES` |
| Frozen protocol | [V19 P0b](physical-sound-v19-p0b-field-integration-protocol-2026-09-01.md), SHA-256 `1c2681a3ac4d505118d08e485cff9e38b23e72a897e8d149b21c909cd11ac24d` |
| Executed implementation | Git `b815cddb1f1ac89f623f63b012c73a6f24df7866`; oracle SHA-256 `7a9107f292bfe33a3bf14de900adec667a95ac8c9d16fb9784441aeb81cf57f4` |
| Official evidence | F0 tree `7918d8b4fd29f5aabb201b1dcc51d85fce9df44463ecb72ebd4eceb42d608718`; report SHA-256 `1fa32ba99e0c212fc6efe2de23e10b18b6ee35b967ae27fb75939d2ab1486bad` |
| Product effect | None; the artifact remains an exact control, but it is not a passing protocol certificate or admission prerequisite |

## Trigger

While translating the frozen F0 gates into V21 P1a, direct comparison found a
literal mismatch between the protocol and the committed runner. No new mesh,
truth, prediction or model value was generated; this audit reads only the
already-opened source and official report.

## Exact mismatch

P0b gate 3 requires every topology's mean edge-gradient p99 to be `<=0.50`.
The committed `_gates` implementation instead uses `<=0.55`:

```text
protocol: all(topology_gradient <= 0.50)
runner:   all(topology_gradient <= 0.55)
```

The official F0 test report contains:

| Topology | Mean edge-gradient p99 | Protocol `<=0.50` |
| --- | ---: | --- |
| Plate | `0.2729064284` | pass |
| Cylinder | `0.5304367382` | **reject** |
| Bowl | `0.5018203188` | **reject** |
| RolledSheet | `0.1931556930` | pass |

The runner therefore emitted `topology_gradient=true` and
`single_run_pass=true` only because it evaluated the wrong literal. Replaying
the frozen protocol against the already-opened report yields an exact reject.
All other inspected F0 gate literals match P0b; this audit makes no broader
claim about uninspected implementation semantics.

## Consequences

1. Historical F0 bytes and repeat identity remain valid observations, but the
   label `F0_CAPABILITY_PASS` is superseded by
   `REPEAT_EXACT_IMPLEMENTATION_CONFORMANCE_REJECT`.
2. V19/V20 integrations cannot use F0 as a passing protocol prerequisite.
   Their rejects remain rejects; I1's acoustic numbers remain diagnostics, not
   an integrated capability certificate.
3. The F0 artifact may remain a named V21 control because its exact behavior is
   useful and reproducible. It receives no baseline-pass credit.
4. V21 F1 inherits the original, stricter per-topology gradient mean
   `<=0.50`; `0.55` is closed and cannot be selected from old results.
5. No old test is rerun and no threshold is changed after values. F1 uses fresh
   train/development/test identities under a newly frozen protocol.

## Reconsideration and stop rule

This decision can change only if the exact committed P0b text or official
report bytes are shown not to be the executed authority. A later model pass
cannot retroactively repair F0. Do not patch the historical runner, edit P0b,
or regenerate F0 under a corrected literal; the next evidence-bearing attempt
is V21 F1 on fresh identities.
