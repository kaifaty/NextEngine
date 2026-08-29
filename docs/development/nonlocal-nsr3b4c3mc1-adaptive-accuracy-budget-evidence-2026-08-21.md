# NSR3-B4C3MC1 adaptive accuracy budget evidence

Status: `PASS / TINY_CORPUS_ACCURACY_SELECTED / TEMPORAL_EQUIVALENCE_NOT_SELECTED`

Date: `2026-08-21`

## Reproducible result

Full command:

```text
nonlocal-formula-reclosure --adaptive-accuracy-budget-self-test
```

Two parent-gated reports executed concurrently and are byte-identical:

```text
status                 PASS / TINY_CORPUS_ACCURACY_ONLY
raw JSON + LF          71c5297d914e7bc19ad10b099daf2282d389e50f383ba31f8e121915e2289297
raw JSON without LF    d19d2f451a48bdc400de03edb157356e3eec8b7ef20e79d3fb2a808b69c44d7c
semantic result        91d6eae1e9789276797785d0672fe748dcaf99178635f322385cb9e6c95dae24
wall time              112.76 s / 112.90 s
CPU utilization        310% / 310%
maximum RSS            15,944 KiB / 16,576 KiB
parent B4C3MC0 exact    true
```

The isolated gate passes in `20.88 s` at raw-with-LF
`453511706bea5394c372f493fb0bb175665e2dba128ad76ba7edf5ba057fcdf4`,
raw-without-LF
`d32e45a8d6d0f235b1100b8b0fefe63bf5457b033d2f2b8fe56d0110601c9dbb`
and the same semantic result. A concurrent B4C3MC0 regression probe retains
its exact raw-with-LF hash
`7de053eb710a5c1e82b067c6ac88b1885df5fb8ab2cc713c26ec0f94af5fc8ae`.

## Budget utilization

All thresholds are inherited unchanged from B4B. Values below are fractions
of their respective limits.

| Case | Position | Velocity | Center | q99 height/front | Kinetic | Contact time |
|---|---:|---:|---:|---:|---:|---:|
| P1 supported | `0.00647` | `0.07028` | `0.00368` | `0.00320 / 0` | `0.13680` | `0.78125` |
| P2 released | `0.18631` | `0.79151` | `0.16853` | `0.11420 / 0.07860` | `0.19892` | `0.35417` |

P1 onset error/limit is `7.7505e-5 / 9.9206e-5 s`; P2 is
`3.6892e-4 / 1.0417e-3 s`. Every per-frame and final terminal contact set is
exact. All adaptive and fixed trajectory, legacy-ledger and policy-ledger roots
remain separately bound in the report.

## Temporal evidence

| Case / field | Resolved ratio | Floor coincident | Stable-reference separation |
|---|---:|---:|---:|
| P1 position | `8` | `0` | `0` |
| P1 velocity | `8` | `0` | `0` |
| P2 position | `16` | `0` | `0` |
| P2 velocity | `1` | `14` | `1` |

The P2 contact-transition velocity is intentionally reported as a stable
adaptive/fine separation, not as ratio zero. Ratio values remain diagnostic;
there is no ratio threshold and this PASS does not select temporal equivalence.

## Discriminators

All controls pass: exact state/aggregate/relative-kinetic/floor/onset boundaries
are accepted, the next representable value above each boundary is rejected,
wrong contact identity and non-finite input are rejected, and all three temporal
classes plus invalid temporal input are discriminated.

## Decision

Select `ADAPTIVE_MACRO_TINY_CORPUS_ACCURACY_CANDIDATE`. Authorize only design
of a nominal, diverse pressure/contact corpus. The selected claim is bounded to
P1/P2; temporal equivalence, B4C4/B4D, CUDA, performance, runtime/schema and
production remain blocked.
