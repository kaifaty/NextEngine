# Nonlocal NPR0 v4 selection evidence — 2026-08-20

Status: `REPORT_ONLY / NONLOCAL_PRODUCT_PROFILE_CANDIDATE / NPR1_ELIGIBLE`

## Decision

`nuv-basin-48k-static-support-h3-physical.v4` passes every frozen
NPR0-R1B gate and becomes the sole `NONLOCAL_PRODUCT_PROFILE_CANDIDATE`.
NPR1 correctness research may start from this exact identity.

This is not production approval. The candidate differs from current SPEC-38
in support radius and static-boundary capacity, has no canonical authority,
has not passed the broader physical corpus, is not coupled to PhysX and owns
no runtime or public state.

## Exact identity and audit

| Field | Result |
| --- | --- |
| Profile SHA-256 | `624678f6ad4dbf2d3657b30880ad1cff2445d0348194549b37c71c1e55804327` |
| Profile input SHA-256 | `97bf611f78dc511d449e5836bca3c9472253dcf4bc2b80cecdd6cf97bcf4d7a0` |
| Fluid / boundary / total | `48,000 / 38,856 / 86,856` |
| Horizon / iterations | `0.15 m = 3dx / 16` |
| Coefficients | `kappa=9196.875, lambda=360, mu=0, gamma=0` |
| Maximum degree / directed-pair capacity | `123 / 10,683,288` |
| Audit raw JSON SHA-256 | `6a273ab6da996939a1e42609fc528cd86791c16dfae5a20fb225f9617faae6cd` |

The audit exposes `runtime_authority=false` and `npr1_authorized=false`
because V4-A alone cannot select the profile. This report applies the frozen
combined V4-A/B/C disposition.

## Full-basin exact GPU preflights

P1 and retained P2 each executed twice at 16 iterations. Both implementations
produce the same roots on both runs:

| Result | SHA-256 |
| --- | --- |
| Ordered output | `21e3cff7a4f2c2ef825d8e98ecb0666ec0bddd057dc60ca9024ee92d8d5a50c4` |
| Logical CSR | `51b4484001c0db58be576368151452322987d77eea730d592b6f67b44fd7263f` |

All retained self-tests, tiny binary64 oracles, stiff-surface controls,
target correspondence and memory correspondence pass. With 86,856 solver
participants, compact u16 is correctly ineligible; P2 selects the checked u32
fallback. Both retained and candidate paths report 59,215,650 device bytes.

Per-run wall time was about 8.7 seconds for P1 and 9.5 seconds for P2 with
about 473 MB maximum host RSS. Kernel totals in the captured single calls are
diagnostic only; no timing value participates in NPR0 and no old performance
claim is inherited.

## Complete NPR0-E rerun

The h3-consistent CPU corpus passes every frozen case:

| Case | Key result | Gate |
| --- | ---: | ---: |
| TPF-1 position / velocity error | `1.11e-15 m / 3.21e-14 m/s` | each `<=1e-12` |
| TPH-1 mean / maximum positive compression | `0 / 0` | mean `<=1e-4` |
| TPH-1 penetration / horizontal drift | `0 / 6.94e-18 m` | `<=0.0025 / 1e-10 m` |
| TPR-1 position / velocity ratios | `6.46e-15 / 2.54e-13` | each `<=1e-9` |
| TPW-1 features | exact face `[2]`, corner `[0,2,4]` | exact |
| TPW-1 penetration / impulse closure | `0 / 0` | each `<=1e-12` |

The corpus result root is
`6b11101d6c73ebf7efbfe56f2d588ab3efffe11a056e5cc5acae6f5952fee304`;
its raw JSON SHA-256 is
`0b02fc8c12d8870b1b9ae1d9c135fbabe684daa450d7ce73debc59ba4e581950`.

## Bounded conclusion

NPR0 is complete. The selected identity establishes that the retained
Nonlocal equations have one finite, exact and tiny-corpus-compatible basin
profile when support is expanded to h3 and capacity is declared honestly.

NPR1 must now try to falsify physical validity with independent density
metrics, canonical quantization and broader water/coupled cases. In
particular, the NPR0 hydro gate is one-sided and does not establish negative
density error or long-horizon rest. Failure in NPR1 leaves the candidate
research-only; it cannot trigger another NPR0 coefficient sweep.
