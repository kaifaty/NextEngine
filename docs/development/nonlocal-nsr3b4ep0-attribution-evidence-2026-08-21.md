# NSR3-B4EP0 nominal cost-attribution evidence -- 2026-08-21

Status: `PASS / HASHING_DOMINANT_LEAF / B4EP1_DESIGN_AUTHORIZED`

## Result

The instrumented GCC/gprof build reproduces the exact B4E1M stdout and records
3,138 ten-millisecond CPU samples. Unconditional SHA-256 is the largest leaf
at 41.36% self time. HVP, neighborhood construction and evaluation account for
most of the remaining sampled cost.

This selects query-evidence separation as the first controlled Release
ablation. It does not establish that hashing is the only important problem:
even eliminating its entire observed share has an Amdahl ceiling of about
`1.70x`, while the full-corpus routing gap is about `6.5x`.

## Build and correspondence

| Field | Value |
|---|---|
| B4EP0 identity | `bf65f79d98c6c9802da5a853d57f77b5cdf8d8fac175da9c9703efde7e667b60` |
| source implementation commit | `45111bd9662eeb931c80dfd601e9ef08190c18ab` |
| compiler | GCC 15.2.0 |
| compile flags | `-O3 -DNDEBUG -g -pg` |
| linker flags | `-pg` |
| gprof | GNU 2.46 |
| compile commands | `c0d2a26e7ff7887834b61ab1eea75628e2559dbdbe2e5f2f8d7f55634bd88274` |
| instrumented executable | 36,158,928 bytes; `d1ef30c6dd16e9e4072844e4a0532a8439098305adfa31a968c2fe0836abb04c` |
| GNU Build ID | `962d6892c8733a76c582c128a0530129d7924aa6` |

The instrumented stdout is exactly 6,151 bytes with SHA-256
`b9601aaad292c43201a5ab054192de4131478eb27e568e1eb78e6b613b07eecc`
and semantic result
`54d42af619dbd48ae400ce4656ba155a8726dbd07b00754ecfc137ee864c111d`.
It therefore matches the uninstrumented B4E1M oracle byte-for-byte.

Instrumented wall/RSS are 57.25 s, 93,560 KiB and 99% CPU. These values show
the profile completed normally; they are not compared with Release timing.

## Profile artifacts

| Artifact | Size | SHA-256 |
|---|---:|---|
| `gmon.out` | 1,364,829 bytes | `7cb9b6756264375642fb346d6813016777da7eedbe5c9113078faa7d06c09822` |
| flat profile | 42,248 bytes | `49be27ac58cdf66bb8506130ee6550b92a928421c65a3eb2d5a870ec574d4e06` |
| call graph | 284,449 bytes | `efde505216dd28f9c01d8e94d720d04362781d641a0057f397cc80c03af9b064` |

B4EP0 result attestation root:
`0b40131ee6820e75d3656aae744bc3e4f0a483758a7f0e127d12b1412df5ba5d`.

## Attribution

| Category | Sampled CPU | Share |
|---|---:|---:|
| `sha256_hex` self | 12.98 s | 41.36% |
| pressure HVP total | about 7.34 s | about 23.4% |
| neighborhood/flat construction total | about 7.11 s | about 22.7% |
| joint evaluation total | about 2.59 s | about 8.3% |

The flat profile records 973 SHA calls. The call graph attributes 454 direct
calls to the 227 workspace builds, 228 calls through pair hashing and 227
through pressure-tape hashing; small query/retention/publication roots account
for the remainder. Thus the largest hash volume is evidence bookkeeping over
inner solver states, not the final canonical publication.

The local-static neighborhood builder is reported under the misleading folded
symbol name `run_joint_pressure_tape_controls()` in this optimized gprof
binary. Its exact count is 227 and its children are canonicalization/cell/
adjacency operations, so the category is identified from the call graph rather
than the displayed folded name alone.

## Decision

Select `B4EP1_QUERY_EVIDENCE_SEPARATION`. Keep full-state hashing as the
default/oracle policy, add a work-only transaction policy that skips inner
workspace/pair/tape hashes, and require bit-exact physical roots/counters in a
Release A/B test. HVP and topology work remain explicit later roadmap stages;
B4E2 stays held.
