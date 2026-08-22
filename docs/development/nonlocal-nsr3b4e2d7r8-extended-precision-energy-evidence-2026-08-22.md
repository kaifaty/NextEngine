# NSR3-B4E2D7R8 extended-precision energy evidence

Date: `2026-08-22`

Status: `PASS / BINARY64_ENERGY_EVALUATION_RESEARCH / PRIVATE_ONLY`

## Reproducibility

Two clean Release builds produce byte-identical 4,921,888-byte executables at
SHA `cce6173723fd064ec58f3f59d3cc8b56270ae857008843c71bfbccb42dbdd1b7`
and Build ID `bb886961f99e9f645972b59c17639f00886bd8d6`.

Both fresh D7R8 processes exit zero with empty stderr and byte-identical
39,878-byte stdout reports at SHA
`42ce10541b29dd03b589e1e8699436e65cb5a794a45b8644fc040990dd4648b1`.
The semantic result is
`dbdfcf009a44ddaf36cd643d750a0859f93678904a27668a020de9c21a245cac`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r8.miezBO`.

The parent reports remain byte-exact in both builds:

```text
D7R5  7ea5489fa90346f385ce8db08dba445f2e2389e65fdf556a13a17e837ba916d7
D7R6  6979ebf9f0b88fb57cbcbbaed6721951cb7fa05acb8f7d4b8c39962f257b9f6f
D7R7  3615964074fd477ab384f5710b274d8da4f4d1c77fd3f11ecb08a45eec0039ff
```

D7R6 retains its expected exit code one; all other listed commands exit zero.
The frozen Linux x86-64 profile closes at `FLT_RADIX=2`,
`sizeof(long double)=16` and `LDBL_MANT_DIG=64`. Formula inputs round-trip
exactly, the three failed-state roots and common tight state remain exact,
and public state is unchanged.

## Sign result

The independent evaluator promotes only the frozen binary64 inputs and then
recomputes radius, cubic kernel, density, PHR energy and inertia in long-double
arithmetic. Fixed-order and compensated sums must agree in sign and both
exceed 1024 extended total-energy ULPs before a sign is called resolved.

| Inner stationarity request | Trials | Resolved positive | Resolved negative | Unresolved | Pair-membership mismatches |
|---|---:|---:|---:|---:|---:|
| `1e-8` | 10 | 7 | 0 | 3 | 0 |
| `1e-9` | 8 | 4 | 0 | 4 | 0 |
| `1e-10` | 5 | 0 | 0 | 5 | 0 |

The first `1e-8` trial is representative:

```text
quadratic predicted reduction     +1.7274297352018516e-15
binary64 direct reduction         -2.2113026171673391e-15
extended compensated reduction    +1.7273098201059638e-15
extended total-energy ULP ratio   +4,078,495
```

The first `1e-9` trial similarly changes from binary64 direct ascent
`-3.6161310545478104e-15` to extended descent
`+1.6807251255893454e-17`, or `39,685` extended ULPs. Thus the false ascent is
not explained by a compact-support membership disagreement or by an analytic
model that is already known to point uphill.

The `1e-10` trials remain deliberately unresolved. For its first trial the
naive and compensated extended reductions agree as positive, but their
magnitudes are only `616` and `468` extended ULPs, below the frozen 1024-ULP
resolution rule. Later microtrials include naive/compensated sign
disagreements. D7R8 therefore does not authorize long double as a complete
oracle or production representation.

## Interpretation

The ordinary subtraction of two binary64 total energies is not the only
problem: D7R7's factored direct formula still consumes independently rounded
absolute densities and active coefficients. D7R8 shows that their errors are
large enough to reverse the sign of the much smaller current-to-trial energy
difference.

The evidence selects a narrower remedy than higher-precision state:
reformulate the binary64 current-to-trial computation so common intermediate
terms cancel symbolically before rounding. It must propagate pairwise
radius, kernel, density, PHR and inertia differences directly and use stable
summation. This is diagnostic until it agrees with every D7R8-resolved sign
and preserves all branch/topology facts.

## Decision

Select `BINARY64_ENERGY_EVALUATION_RESEARCH`. Freeze D7R9 as a replay-only
computational divided-difference discriminator. Do not change trial
acceptance, trust limits, stationarity/pressure gates, `beta`, kernel support,
state precision, solver family or trajectory authority.

