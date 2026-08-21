# NSR3-B4EP3I hot-path superset-cache evidence -- 2026-08-22

Status: `PASS / HOTPATH_CANONICAL_SUPERSET_CANDIDATE / B4EP4_DESIGN_AUTHORIZED`

## Result

The optional transaction-local cache replaces all 226 canonical topology
builds in the nominal work-only transaction with one fixed `0.04h` superset
build and 225 certified reuses. Every solver decision, physical output,
publication root and ledger root remains bit-exact to B4EP1. The parent
preflight and all existing commands remain on their unchanged canonical paths.

All three frozen Release timing pairs win. Median paired speedup is
`1.589914367x`; median one-macro wall time falls from `16.71 s` to `10.40 s`.
This is one serial CPU research macro, not a nominal corpus, GPU, runtime or
production result.

## Build and deterministic correspondence

| Field | Value |
|---|---|
| B4EP3I identity | `e617043b55349894356ce3eda95aca538e99aeec8b127c418d8273bc408a1474` |
| implementation commit | `65ce739e17def07266f0d2f72b00178451a1fb3e` |
| Release executable | 3,755,696 bytes; `caa03290e2cdfba75a975ff6f5aab38c525c5d05000ced2969b70bec37aa6dba` |
| GNU Build ID | `8d75e9797dc4bd877704985a5b19f8fa2b9e8c51` |
| cached stdout | 6,462 bytes; `b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055` |
| cached semantic result | `99a7e4b183f844390dca81beba0d83585bf938cb08e0dd6bd6794c82079fd3fe` |
| cached work chain | `492678c526bf47e22569c66d3c1f0506db0cb5e3f5d2d4ca7311a1d4fa4bbc84` |
| cached work receipt | `1fb59422151de92d38342f08003430a5118787d45d1666ebeb74ffe61ea822cd` |

Two independent Release builds produce the same executable SHA-256 and GNU
Build ID. Their fresh cached reports compare byte-for-byte. All six timed
processes exit zero with empty stderr. The existing reports retain their exact
stdout SHA-256 values:

- B4EP1 work-only oracle:
  `4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112`;
- B4EP3 all-state audit:
  `4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095`.

## Cache and work facts

| Fact | Result | Gate |
|---|---:|---:|
| transaction queries | 226 | 226 |
| superset rebuilds / certified reuses | `1 / 225` | `1 / 225` |
| certificate failures / fallback builds | `0 / 0` | `0 / 0` |
| maximum candidate degree | 122 | 122 |
| maximum anchor displacement squared | `8.3231208002159437e-9 m²` | certificate passes |
| superset-build cell tests | 8,673,618 | diagnostic |
| filtered candidate checks | 91,595,540 | diagnostic |
| active pair visits | 85,716,150 | diagnostic |
| candidate / active visits | `1.0685913914705689` | `<=1.25` |
| fluid records sorted | 12,000 | parent plus one cache build |
| flat workspaces / CSR transfers | `227 / 227` | unchanged |
| flat offsets / directed records | `1,362,227 / 151,461,068` | unchanged |

Filtering still materializes the current flat CSR and refreshes evaluation and
the pressure tape on every query. The cache removes repeated cell topology
construction only; it does not claim that the remaining 10.4-second path is
fully optimized.

The transaction reproduces initial/selected substeps `14/28`, attempted/
accepted/discarded `42/28/14`, 221 outer trials, zero rejected trials and
nonlinear/spectral HVP counts `411/48`. Frame, aggregate, trajectory and both
ledger roots match B4EP1 exactly, as do strain, zero energy creation and zero
penetration.

## Timing and memory

The fixed order was `UNCACHED, CACHED, CACHED, UNCACHED, UNCACHED, CACHED`.
Timing and RSS were recorded by external `/usr/bin/time -v` and do not enter
deterministic reports.

| Pair | Uncached | Cached | Uncached/cached |
|---:|---:|---:|---:|
| 1 | 16.41 s | 10.40 s | `1.577884615x` |
| 2 | 17.21 s | 10.40 s | `1.654807692x` |
| 3 | 16.71 s | 10.51 s | `1.589914367x` |

All processes use 98--99% of one CPU core. Median RSS changes only from
62,672 KiB to 62,416 KiB; the benefit is compute work, not material memory
reduction. The median paired speedup exceeds the frozen `1.10x` gate.

## Ownership and failure boundary

The cache is created empty for one macro transaction, reached only through an
optional internal trace pointer and destroyed afterwards. An RAII reset clears
the non-owning trace pointer on every function exit. Certificate, binding or
capacity failure rejects the cached command; no canonical fallback can hide
the failure. Existing defaults pass a null cache and preserve their report
bytes.

## Decision

Select `HOTPATH_CANONICAL_SUPERSET_CANDIDATE` for subsequent nominal research
commands and authorize only B4EP4 residual profiling/design. B4EP4 must
reattribute the optimized 10.4-second path before another implementation is
selected. B4E2, CUDA, runtime/public schema, reference changes and production
remain blocked.
