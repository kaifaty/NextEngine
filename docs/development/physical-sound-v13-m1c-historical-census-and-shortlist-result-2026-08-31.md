# Physical Sound V13-M1c — historical census and shortlist result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `M1C_HISTORICAL_CENSUS_FRESH_SHORTLIST_PASS` |
| Protocol | [M1c protocol](physical-sound-v13-m1c-historical-census-and-shortlist-protocol-2026-08-31.md) |
| Product effect | None; opens only M2 zero-signal source/role freeze |

## Exact result

Two corrected runs are byte-identical under:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v13-m1c.0tpuYg
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v13-m1c.sOOIJ8
```

The census parsed `1,298` first-party JSON files (`115,681,803` bytes,
`3,254,570` scalar values), retained `42` stable tool exclusions and found
`1,858` direct plus `1,102` path-token exposure records. M1b independently
rehashes every included artifact and parses 130 identity-bearing files.

All network, source-byte, signal, force and protected-signal build counters are
zero. The global historical ledger uses conservative unknown-count exposures;
it is not a clean-source record.

## Fresh ObjectFolder glass shortlist

| ID | Object | Result |
| --- | --- | --- |
| `59` | `Soap_Dish` | `FreshMetadataOnly` |
| `82` | `Can` | `FreshMetadataOnly` |
| `92` | `Glass_Red` | `FreshMetadataOnly` |
| `93` | `Vase` | `FreshMetadataOnly` |

Seven other official glass objects are excluded by prior evidence. Object `22`
is excluded despite zero decode because its contacts were already committed as
representation/protected roles. No winner is selected in M1c.

## Hashes

| Artifact | SHA-256 |
| --- | --- |
| Runner | `174fb2528129707abb72277f52d61babf65fef72cfb6c35e2306e3f32e3bde54` |
| Tests | `09d2ff750f46cfe9f09ee64a16084733a0d6f6d84cef96ce2fa3233776178b84` |
| Protocol | `fc01898e529a0774db142bae3b40540d5987abb2523daabb6d7fc6d45897dcb1` |
| Census | `e8fd435d70a3688e5f67e24002c8b328c0ffa220c2f0a7aa711dd0f733856be9` |
| Catalog | `3af1a4a91e26fdb7595330af37644a2176ce12bfdcc9396dffe381843b7cff7e` |
| Ledger | `5d69b8211d451dff5a6ef9fadfe2609dafbfde378007ff77475fc2db7cafbbea` |
| Shortlist | `0bd87fc3e62bfa6e574c4c206e795282223a4ecbd6221790e1ff4c60a113f6d7` |
| Report | `d4298c7fbc35983bea99e71589b18529090d9112ed699b95d64e3f417bf3665f` |

An initial A/B attempt correctly failed repeat because generated evidence was
enumerated in the diagnostic exclusion list. The implementation was corrected
to exclude the frozen generated subtree as a rule; C/D then matched exactly.

Verification: 31 focused M1 tests and Python compilation pass. Boundary scan
still reports only the pre-existing `SOURCE_LAYOUT_ESCAPE_HATCH` from
`a26f070f`; M1c adds no finding.
