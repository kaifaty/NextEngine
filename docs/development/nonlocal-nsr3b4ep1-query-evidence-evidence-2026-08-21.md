# NSR3-B4EP1 query-evidence separation evidence -- 2026-08-21

Status: `PASS / WORK_ONLY_NOMINAL_RESEARCH_CANDIDATE / B4EP2_DESIGN_AUTHORIZED`

## Result

The controlled Release ablation removes only transient full-state evidence
hashing from the nominal Hydro transaction. It reproduces every frozen
physical decision, output root, ledger root and work counter while reducing
median one-macro wall time from 48.74 s to 16.15 s. All three alternating
pairs win; median paired speedup is `3.0168x`.

This is a research-path optimization, not a production or whole-corpus claim.
The parent preflight and all final publication/ledger hashes remain full-state;
the existing B4E1M command remains byte-exact. B4E2 execution remains held.

## Build and deterministic correspondence

| Field | Value |
|---|---|
| B4EP1 identity | `470a0f4ec9b51ac57f1ecb69e937bb9d2756db41299a0dc01fa427ae63a78e99` |
| implementation commit | `5e40aa26e6ba0fcf2d0f3c4ca8d6f77857a660f1` |
| Release executable | 3,694,560 bytes; `b82cc32bf41bbf2b66536a820167225cf5ef35227a9efe8b4221015b7235ca0f` |
| GNU Build ID | `7cf849cad6f2b403a3ee28a67c6da2a5df4caa12` |
| full-state stdout | 6,151 bytes; `b9601aaad292c43201a5ab054192de4131478eb27e568e1eb78e6b613b07eecc` |
| work-only stdout | 5,780 bytes; `4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112` |
| work-only semantic result | `25b1f00c0c7477a03532dca2acb97314853e9b4184962d9695792273280f5e04` |
| work-only chain | `43392066826af712dab417ba49d06b63293fabecc2cbb7423186a8187cf9004a` |
| work receipt | `36f15e202beda83bbd46e4ada5ff071c101d886e130f380c21426269637962ea` |

Two independent Release builds produce the same executable SHA-256. The three
full-state reports are byte-identical to the frozen B4E1M oracle, and the three
work-only reports are byte-identical to each other across both builds.

The work-only transaction computes zero transient workspace state hashes and
skips exactly 226. The separate parent preflight still computes one full-state
hash and skips zero. The candidate retains 227 total flat workspace builds,
151,461,068 flat directed records and balanced 42 retained transfers, reads and
releases.

## Frozen physical correspondence

The candidate reproduces exactly:

- initial/selected substeps `14/28`, attempted/accepted/discarded `42/28/14`;
- 221 outer trials, zero rejected trials and nonlinear/spectral HVP counts
  `411/48`;
- frame root
  `eaa6fe3aea4567594d82ce9fb76be16a4a99256e45b6ee10c4bc4c6417311eb5`
  and aggregate root
  `8a634d69ec2bdb692d08e2c1cdbbf4fd8ce5a4cc39289a7c05a9acd8f0ca90dc`;
- trajectory root
  `4689e74310815f413212d006d02347d598d38ce70b0ca44eee2253cddb1f6fc4`,
  legacy ledger
  `df58c67ec644ad717cf6d2192dd31743163bf65438289feedba1e93457f0c057`
  and policy ledger
  `b8502f70738b39cad693e515ba1546a3f5ae2e1dae432d597cc4ae7b0e0c0444`;
- strain `0.00045547995081940407`, zero energy creation, zero private and
  decoded penetration, and the same strict ledger residual.

Base formula self-test, B4E0 nominal alignment and B4E1S spectrum regression
all pass after the implementation. The B4E1S stdout remains exactly
`771aa94575919bce513bf361f8126555d50b252b67b8051a448d15259362f2ad`.

## Timing and memory

The fixed sequence was `FULL, WORK_ONLY, WORK_ONLY, FULL, FULL, WORK_ONLY`.
Timing and RSS were captured externally and are absent from deterministic JSON.

| Pair | Full | Work-only | Full/work-only |
|---:|---:|---:|---:|
| 1 | 48.74 s | 16.21 s | `3.0068x` |
| 2 | 48.48 s | 16.07 s | `3.0168x` |
| 3 | 48.86 s | 16.15 s | `3.0254x` |

All processes use 99% CPU. Median RSS falls from 93,060 KiB to 62,016 KiB,
a 33.36% reduction. The median paired speedup is `3.0168x`, well above the
frozen `1.10x` threshold.

The observed gain exceeds the B4EP0 sample-based `~1.70x` estimate because
gprof sampling assigned serialization/allocation work beneath callers and its
instrumentation perturbed relative costs. The controlled Release A/B is the
authoritative performance evidence for this ablation; B4EP0 remains useful as
the hypothesis generator, not as a calibrated wall-time model.

## Decision

Select `WORK_ONLY_NOMINAL_RESEARCH_CANDIDATE` for subsequent nominal research
commands while retaining `FULL_STATE` as the default/oracle. Authorize only
B4EP2 residual attribution and design. The remaining 16.15 s macro is still
far from an acceptable corpus cost, and B4EP0 already identifies HVP and
neighborhood/topology as live categories. No runtime/public schema, CUDA,
reference, B4E2 execution or production authority is created.
