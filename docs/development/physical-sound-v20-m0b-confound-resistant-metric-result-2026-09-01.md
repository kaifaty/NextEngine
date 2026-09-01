# Physical sound V20 M0b — confound-resistant metric result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT_PASS / P0C_OPEN / I1_VALUES_STILL_SEALED` |
| Protocol | [M0b](physical-sound-v20-m0b-confound-resistant-metric-protocol-2026-09-01.md), SHA-256 `9480a5fba5a98e775b9ba6f14c08a20f0279949fc74f5939082ad3e0a36d9cbb` |
| Implementation | commit `cfee8cb844b945d4ba85ef3fe768947ee563e8ed` |
| External runs | `v20-m0b-confound-resistant-run-a` / `v20-m0b-confound-resistant-run-b` |
| Complete tree | both `7eb5c75edcff8bd67fef592af30aa9fa90ebc73a25aa2629cba43935e23bab22` |
| Product effect | Opens only P0c protocol authoring; no I1 value, real data, clip, demo or runtime authority |

## Decision

M0b passes twice byte-exactly. Mean-centered log spectral shape and normalized
backward-EDC log slope remove the two measured amplitude confounds without
changing B0/F0, weakening the `0.90x` margin or hiding physical gain checks.

This earns a development-only `PhaseConsistentMetricCertificateV0`. It does
not validate the model on fresh integration identities. P0c must freeze I1
metadata and thresholds before I1 implementation may generate a mesh, truth,
prediction or waveform value.

## Exact artifacts

| Artifact | SHA-256 |
| --- | --- |
| `access-ledger.json` | `b68fcf2ae50edcb7aedc27d2b44cd410ddfdf3de46af9871badbd54464eab306` |
| `corpus.json` | `c4c23b655e904173af6c39c8d0044bd42536e3b71194998552a454071b35dda0` |
| `manifest.json` | `063cd43bc3abbbf8027a843a725954032f3f8dd870a04d55e6db5c535b784d87` |
| `metrics.jsonl` | `259851b487f9baaad21df8d4caf15e6a21fa6b69da5d0d10234b2a8fdefc003f` |
| `report.json` | `4ddedcd285ac2a9d12cd266f2901067c1db97cf3fff27c2bb112b5117b8350ec` |

Both executions contain five files, 11,136 complete metric rows and an `11 MiB`
tree. Every run stayed below ten minutes, 4 GiB RSS and 100 MiB output.
Predecessor M0a, B0 and F0 trees were verified before rendering.

The access ledger records zero I1/integration/successor rows, zero real,
protected, force, source, dataset or checkpoint values and zero network
requests. M0a values did not select a case.

## Gate result

All twelve single-run gates and the independent repeat gate pass:

- complete/finite `24` views, `192` cases, `58` controls and `11,136` rows;
- exact hard, B0, C0, F0, fallback and `12/12` mutation evidence;
- identity and polarity invariants, including signed-gain polarity rejection;
- every harmful physical owner;
- all ten metric-family separations;
- all eleven severity correlations (`>=0.9999999999999999`);
- exact serialization, V19 legacy attribution and M0a failure reproduction;
- zero sealed access and resource ceilings;
- identical five-file trees across A/B.

M0a raw MRLM/DE summaries reproduce with maximum absolute delta `0.0`; the
same four predecessor failures remain visible as diagnostics.

## Separation certificate

Corpus-p95 thresholds may be placed strictly between each interval:

| Family / owner | Max acceptable | Min harmful | Ratio |
| --- | ---: | ---: | ---: |
| uniform frequency / MRSC | `0.647227358` | `0.727197155` | `0.890030102x` |
| uniform frequency / MCLM | `0.444507720` | `0.521475770` | `0.852403401x` |
| alternating frequency / MRSC | `0.647227358` | `0.724814465` | `0.892955907x` |
| alternating frequency / MCLM | `0.440593435` | `0.545920502` | `0.807065192x` |
| positive damping / DSR | `0.133601165` | `0.172722217` | `0.773503069x` |
| negative damping / DSR | `0.133601165` | `0.160685981` | `0.831442568x` |
| mode removal / MRSC | `0.647227358` | `0.872813019` | `0.741541823x` |
| mode removal / MCLM | `0.432306369` | `0.490936281` | `0.880575312x` |
| onset delay / TE | `0.026240490` | `0.158067236` | `0.166008403x` |
| impulse / TE | `0.026240490` | `0.284451706` | `0.092249366x` |

The narrowest passing interval is mode-removal MCLM at `0.880575312x`; it is
inside, not equal to, the frozen `0.90x` margin.

## Consequence

Freeze P0c using arithmetic midpoints and the strictest applicable metric
threshold before creating any I1 value. I1 remains a synthetic capability
check; even a repeat-exact pass cannot claim real material acoustics, admit a
clip or promote SPEC-45.

