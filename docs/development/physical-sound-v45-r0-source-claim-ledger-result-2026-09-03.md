# Physical sound V45 R0 — source-claim ledger result

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Status | `COMPLETE / REPEAT_EXACT / NO_PAYLOAD_AUTHORITY` |
| Decision | `R0_SOURCE_CLAIM_LEDGER_REPEATABLE_NO_PAYLOAD_AUTHORITY` |
| Claim | Metadata-only source-to-claim planning; no corpus, training, validator, protected, cooker, demo or runtime authority |
| Next | T0 Recipe V3 and masked-target contract |

## Frozen implementation

- profile: [`physical-sound-v45-r0-source-claim-ledger.v1.json`](../../lab/profiles/physical-sound-v45-r0-source-claim-ledger.v1.json),
  `18,780` bytes,
  SHA-256 `a98cd2b5881b673a0d09c0e59bdf4d4ef14699a61370f0c5040c1fe3006409ec`;
- owner: [`physical_sound_v45_r0_source_claim_ledger_v1.py`](../../lab/scripts/physical_sound_v45_r0_source_claim_ledger_v1.py),
  `18,859` bytes,
  SHA-256 `f1c682ef644e905765d2895aa4e4ba2944dff572b1a470e699486cd05ced396f`;
- focused tests:
  [`test_physical_sound_v45_r0_source_claim_ledger_v1.py`](../../lab/tests/test_physical_sound_v45_r0_source_claim_ledger_v1.py).

The profile binds the immutable V45 research report and owner, but deliberately
does not bind the living roadmap. This lets the roadmap record the completed R0
terminal without invalidating the already-run experiment.

## Exact source census

| Evidence class | Sources | Prospective lane | Current credit |
| --- | ---: | --- | ---: |
| Empirical prior | 1 Clatter source + 1 Giordano/McAdams metadata control | `empirical_prior = 2` | `0` |
| Synthetic modal teacher | NISR + VibraVerse | `modal_teacher = 2` | `0` |
| Structural transfer | Delft aluminium plate + cello bridges | `structural_transfer = 2` | `0` |
| Real acoustic candidate | CMU impacts + Zenodo three-object impacts | `real_acoustic = 2`, `validator_calibration = 2` | `0` |
| Literature-only control | Sounding Object | none | `0` |
| Excluded unavailable | confidential waste-acoustic dataset | none | `0` |

The ledger contains ten sources in total. Six are eligible only for a later
preflight, one first requires a metadata-only axis/cost audit, two remain
control-only and one is excluded. `protected_admission` has zero prospective
sources. Physical-parent and project-independence credit are both exactly zero.

## Binding corrections captured by R0

- Clatter is pinned to commit
  `79cac6cbe3f7c452ba28b56c7da4a0124ad04806`; its 84-file parameter tree has
  path-neutral SHA-256 root
  `8230a6192f189806b899a9113c08fefaf458e9e7f26322e2fc6fe5434a4c065e`.
- NISR is pinned to commit
  `20368791bcd7829e04ae3eb07c10aa0bb370e38a`; it remains in the
  ObjectFolder-derived alias component and its generation-source gate is open.
- VibraVerse is now a public dataset rather than a paper-only lead. R0 pins
  commit `8099f137e9a9171e758c0528fb4a38a7b5d6aab2` and README SHA-256
  `ed86532abb7113098346dda7a9271ea3a77a01c87b40dffe04b291ce9e522534`.
  It is still synthetic and adds no real-source independence.
- Delft record `10.5281/zenodo.7758683` has a canonical record/file-manifest
  SHA-256 root
  `b21596836ac538d6412475fa2c2d7e20d78fbe8a258d412aa0c69e144a9b4a47`.
- Cello record `10.5281/zenodo.20797149` has root
  `14508eb634a61a5a39325543f5c66ceae296ce3e0afccda755868f63c35884c4`;
  its large assembly-confounded payload is not authorized before X0b.
- CMU impact record `10.1184/R1/20205035.v1` has canonical article/file root
  `da83d383709090e653dd71c443c23605d57de409939b69cb007720e545f4ecc3`;
  recording notes are open but audio remains closed.
- Three-object record `10.5281/zenodo.2563718` has root
  `f16132cb5f93be99aca69fb576221b4c9052c19b4e7cf7f640dbf5485afded70`.

These roots are metadata receipts, not content admission. Every source row also
records its upstream alias component, observed axes, missing gate, payload
state, cost class, usage/retention policy and the full set of forbidden R0
authorities.

## Repeat-exact execution

Two independent external output directories produced no `diff -qr` difference:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access.json` | 671 | `01d99201b9c2fd43674f2516a94a67abbd62601b82bf6cdf8d78fc6d5dd7f092` |
| `ledger.json` | 17,629 | `a0178539b3830c1deb115d785e195ccb50a9f65141c8823af0a840302be6c1e8` |
| `report.json` | 1,808 | `af0234f975af03ef7c7bc5b855774f84a0113d7b3762fa661abee72300918857` |

All seven report gates pass:

- alias components are explicit;
- every R0 authority is forbidden per row;
- claim lanes are non-interchangeable;
- external payload access is zero;
- physical-parent and project credit are zero;
- the protected lane is empty;
- the observed census matches the frozen expected result.

All eight access counters are zero: audio decode, feature, model, network,
numeric payload decode, PCM decode, protected value and waveform byte access.

## Failure coverage

Eight focused tests pass. They cover canonical profile/owner binding, A/B
identity, exact lane counts, synthetic-to-real lane escalation, premature
parent credit, dependency mutation, repository-output/import guards, unknown
fields and unsorted source identities. Every mutation fails before publication.

## Consequence

R0 closes source-role ambiguity but does not improve the real corpus. The
planning baseline remains `71/105`, deficit `34`; PSEL, real training,
validator qualification, protected admission and runtime stay blocked. T0 may
now define Recipe V3 and missing-target semantics without reading any of the
external numeric/audio/model payloads. Real second-project discovery continues
in parallel.
