# Physical sound R3A V8 — ObjectFolder Real zero-decode inventory result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `READY_FOR_V8_REAL_FIT_EXTRACTION` |
| Protocol commit | `92f4f44f` |
| Implementation commit | `d45f4e5b` |
| Manifest SHA-256 | `e1651b64b3867b2823accc1923b69c8437887b0acd90f6ced953526ebd8d147b` |
| Report SHA-256 | `d98883020e140a52c48fce352a9115a17d5a127ea820b21adc487acece94dace` |
| Product effect | None; fit extraction only is the next authorized step |

## Result

Two independent runs over the exact official ObjectFolder Real 512 MiB prefix
produce byte-identical canonical manifest and report JSON.

| Check | Observed | Result |
| --- | ---: | --- |
| Prefix bytes | `536870912` | `PASS` |
| Prefix SHA-256 | `5ef9789a…7313` | `PASS` |
| Selected contacts | `5` | `PASS` |
| Selected members | `20` | `PASS` |
| Selected member bytes hashed | `5760769` | `PASS` |
| WAV headers validated | `10` | `PASS` |
| PCM boundary | mono PCM16, `48 kHz`, `288000` frames | `PASS` |
| Selected payloads emitted | `0` | `PASS` |
| Waveform sample values decoded | `0` | `PASS` |
| Development sample values decoded | `0` | `PASS` |
| Sealed sample values decoded | `0` | `PASS` |

Development and sealed member streams were hashed only to create commitments:
`1152154` bytes for each role. No content, YAML value, sample, feature or member
payload appears in the output.

## Frozen roles

- fit: contacts `18`, `12`, `4`;
- development: contact `20`;
- sealed: contact `27`;
- contact `9` and all other archive content: unused.

The exact per-member commitments remain in the
[preregistered source gate](physical-sound-r3a-v8-objectfolder-real-source-and-gate-freeze-2026-08-31.md)
and the external manifest. The inventory found no coordinate artifact in the
bounded source, so object `91` still cannot support R3B contact interpolation.

## Interpretation

The source boundary is now reproducible and native `48 kHz`; the V2 resampling
confound is absent. This is availability and lineage evidence only. No acoustic
metric, force value or perceptual result has been computed.

The decision authorizes exactly one next transition:

1. freeze the complete fit-only representation implementation, capacities,
   loss/initializer, cost and endpoint gates;
2. extract and numerically decode only fit contacts `18`, `12`, `4`;
3. stop before development contact `20` unless every fit gate passes;
4. keep contact `27`, method holdout and admission shadow unread.

## Checks

- focused inventory unit suite: `6/6 PASS`;
- Python bytecode compilation: `PASS`;
- two independent external inventories: `PASS / byte-identical`;
- real/development/sealed decoded sample counters: exact zero.
