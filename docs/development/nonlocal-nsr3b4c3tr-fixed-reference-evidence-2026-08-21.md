# NSR3-B4C3TR fixed canonical reference evidence

Status: `FAIL / PER_SUBSTEP_PUBLICATION_REFINEMENT_UNSTABLE`

Date: `2026-08-21`

## Reproducible result

Command:

```text
nonlocal-formula-reclosure --fixed-canonical-reference-probe
```

Two isolated reports are byte-identical:

```text
status                 FAIL
raw JSON + LF          b04a7c93e3b90c67950b1460c48c99236f5cdc2829be4aabc010afa664b351be
raw JSON without LF    fbdce8bc1420b38c813f82a29bdae48cdb3e130d5d45c08d1077c56313d360
semantic result        bf92622cb719574b5d13a5bdae06b18adefee95dd34da1781bada1091f27e02f
wall time              19.73 s / 19.74 s
user CPU                60.45 s / 60.40 s
machine utilization    312% / 311%
maximum RSS            40,112 KiB / 39,572 KiB
```

The frozen stop rule prevented the two full parent-gated reports after the
isolated FAIL. B4C3TAR2 remains the selected parent.

## What passed

All six fixed canonical lanes finish their complete horizons and pass their
transactional solver/ledger gates:

| Case | Level | Steps | Nonlinear HVP | Max strict | Max KKT |
|---|---:|---:|---:|---:|---:|
| P1 | 48 | 384 | 2,322 | `7.10e-10` | `5.72e-10` |
| P1 | 96 | 768 | 4,325 | `9.43e-10` | `8.74e-10` |
| P1 | 192 | 1,536 | 5,722 | `8.00e-10` | `7.19e-10` |
| P2 | 48 | 768 | 170 | `3.82e-11` | `2.42e-11` |
| P2 | 96 | 1,536 | 336 | `3.84e-12` | `2.42e-12` |
| P2 | 192 | 3,072 | 678 | `1.78e-10` | `1.26e-10` |

Global steps, decoded continuation, legacy/policy ledgers and all three roots
are exact. The forced post-commit failure leaves state, counts, totals and all
roots exact. The independent binary64 reference converges normally: P1
position/velocity ratios are `1.782/1.837`; P2 ratios are `2.001/1.873`.

Parallel lane execution reduces about 60.4 CPU-seconds to 19.7 wall-seconds,
or `3.06x`, while producing byte-identical reports. This validates parallel
independent-lane scheduling for the research harness only.

## Blocking failures

P1 fixed-192 exceeds its same-level velocity tube in frame zero:

```text
error       8.6961119636e-3 m/s
bound       6.1440000000e-3 m/s
utilization 1.41538
```

Its first-contact time remains exact, but the terminal contact set contains 36
canonical versus 64 binary features. P1 canonical position refinement is not
first-order: `48->96 = 4.3765e-5 m`, while `96->192 = 1.23496e-4 m`.

P2 makes the mechanism clearer. Canonical contact occurs increasingly early:

| Level | Canonical contact | Binary contact | Error | Fixed-step limit |
|---:|---:|---:|---:|---:|
| 48 | `0.0448785 s` | `0.0451389 s` | `2.60417e-4 s` | `8.68056e-5 s` |
| 96 | `0.0446181 s` | `0.0451389 s` | `5.20833e-4 s` | `4.34028e-5 s` |
| 192 | `0.0440972 s` | `0.0451606 s` | `1.06337e-3 s` | `2.17014e-5 s` |

At P2 frame ten the 96/192 canonical-to-binary velocity errors are both about
`0.14986 m/s`, using `4.44x` and `2.22x` of their frozen bounds. Canonical
final differences grow under refinement: position ratio `0.523`, velocity
ratio `0.204`. They fit only the deliberately labelled representation-floor
branch and therefore cannot claim observed temporal convergence.

## Root cause

The nonlinear KKT solve, neighborhood, ledger and binary temporal scheme all
pass. The failure is caused by applying a fixed `1e-6` position/velocity
quantization after every solver substep and then feeding the decoded value into
the next solve. Refining `h` increases publication count per physical second,
so the representation perturbation is injected more often. Contact-event phase
error grows rather than shrinks.

This means the current per-substep canonical continuation is not merely a
serialization choice: it changes the numerical model as the timestep changes.
A fixed-192 lane is therefore not a valid higher-resolution reference for the
adaptive lane under this publication cadence.

## Decision

Preserve B4C3TR as FAIL. Do not widen contact/tube bounds and do not authorize
B4C3TC. The only next research allowed is a separately frozen publication-
cadence discriminator comparing:

1. current publish/decode after every solver substep;
2. binary64 private substeps with one balanced canonical transaction at the
   macro-frame boundary;
3. optionally a timestep-scaled representation only as a diagnostic, not a
   selected schema.

Nominal, B4C4/B4D, CUDA, runtime/schema and production work remain blocked.
