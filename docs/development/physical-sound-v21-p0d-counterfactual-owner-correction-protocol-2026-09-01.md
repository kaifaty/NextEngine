# Physical sound V21 P0d — counterfactual owner correction protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN / PROTOCOL_ONLY / NO_VALUES_GENERATED / I1_REMAINS_REJECTED` |
| Trigger | [V20 I1 repeat-exact reject](physical-sound-v20-i1-frozen-integration-result-2026-09-01.md), SHA-256 `f072dd206fc497240121800c05913d156f987860f6fe954ad8f0594ead1da404` |
| Metric authority | [M0b protocol](physical-sound-v20-m0b-confound-resistant-metric-protocol-2026-09-01.md), SHA-256 `9480a5fba5a98e775b9ba6f14c08a20f0279949fc74f5939082ad3e0a36d9cbb` and [result](physical-sound-v20-m0b-confound-resistant-metric-result-2026-09-01.md), SHA-256 `465e806a9052f4f86ceeb07407a42d09703272f4d36054fb5a164b3526d663c9` |
| Predecessor conformance | [F0 audit](physical-sound-v21-f0-protocol-conformance-audit-2026-09-01.md), SHA-256 `78fa857cdc4ca643e451bc756c37cbf72f584a86bde615b321b25d722d35abb4`; P0d corrects only counterfactual ownership and grants no F0 pass credit |
| Superseded integration rule | The alternating-sign bullet and its derived conjunction in [V20 P0c](physical-sound-v20-p0c-i1-integration-protocol-2026-09-01.md), SHA-256 `b591164a09c4a9aeb078cef7ab5f4dbebd1c2117b09a8fd21277be79427218c5` |
| Product effect | None; this protocol cannot rerun I1, open F1 values, admit real roles, bake clips or authorize runtime ML |

## Decision boundary

M0b calibrated one blocking owner or owner family for each declared defect.
V20 P0c accidentally strengthened the alternating modal-sign control by
requiring both its signed-gain physical owner and an unrelated spectral
threshold. I1 proved the signed-gain owner rejects strongly while MRSC/MCLM do
not cross thresholds that were never calibrated for this defect.

P0d corrects only that ownership relation for a future V21 integration. It
does not change a metric formula, threshold, aggregation, control magnitude,
identity, fallback rule or historical decision. P0c and I1 remain immutable
rejects.

## Frozen owner matrix

| Counterfactual | Blocking owner in the next integration | Diagnostic only |
| --- | --- | --- |
| identity | `MRSC/MCLM/DSR/TE <=1e-12` and every physical delta zero | raw waveform/envelope/DE |
| uniform `+90 cents` | physical frequency reject **and** MRSC or MCLM reject | DSR, TE, raw waveform/envelope/DE |
| alternating `-90/+90 cents` | physical frequency reject **and** MRSC or MCLM reject | DSR, TE, raw waveform/envelope/DE |
| damping `*1.35` | physical damping reject **and** DSR reject | MRSC, MCLM, TE, raw waveform/envelope/DE |
| damping `*0.65` | physical damping reject **and** DSR reject | MRSC, MCLM, TE, raw waveform/envelope/DE |
| remove two largest-gain modes | missing-mode/signed-gain physical reject **and** MRSC or MCLM reject | DSR, TE, raw waveform/envelope/DE |
| alternate modal signs | signed-gain physical reject | every acoustic metric, including MRSC/MCLM/DSR/TE |
| global polarity | signed-gain physical reject **and** `MRSC/MCLM/DSR/TE <=1e-12` | raw waveform/envelope |
| delay 64 samples | TE reject | physical, spectral, decay and waveform diagnostics |
| sample-zero `8*truth_peak` impulse | TE reject | physical, spectral, decay and waveform diagnostics |

“Reject” means the unchanged physical or M0b threshold applicable to that
owner. Logical `and`/`or` above is exact. In particular, alternating modal signs
has no acoustic blocking conjunct and cannot borrow DSR merely because the
opened I1 DSR happened to cross its damping threshold.

## Unchanged metric values

The next integration inherits the strict P0c actual-candidate thresholds
without rounding or reinterpretation:

```text
MRSC <= 0.6860209112961239
MCLM <= 0.46162132517706367
DSR  <= 0.14714357326302258
TE   <= 0.09215386292722896
```

It also inherits every B0 frequency/damping gate, field gain/gradient/remesh
gate, hard corruption, identity/polarity invariant, serialization rule,
resource ceiling, zero-access rule and complete authored fallback. P0d gives
no permission to tune any of them from I1 or F1 values.

## Future use

P0d becomes one immutable input to V21 P1c. P1c must pin this document hash and
the selected F1 artifact before it freezes fresh `2101…2112` reintegration
metadata. The I2 runner must encode this owner matrix directly and test each
logical row with a focused failure case before any I2 value is generated.

## Stop rules

1. Do not edit P0c or report I1 as passing under corrected semantics.
2. Do not rerun `1701…1712`; their only permitted use is recorded attribution.
3. Do not lower M0b thresholds or turn a diagnostic into a substitute owner.
4. Do not let a physical reject waive the actual candidate's four acoustic
   gates; this correction applies only to named counterfactual attribution.
5. No F1, test, integration, real, protected, source-body or network value is
   opened by freezing this document.
