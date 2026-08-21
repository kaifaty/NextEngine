# NSR3-B4E1S Hydro spectrum evidence -- 2026-08-21

Status: `PASS / NOMINAL_HYDRO_SPECTRUM_CANDIDATE / B4E1M_DESIGN_AUTHORIZED`

## Result

The exact B4E0 Hydro state produces a finite deterministic pressure spectrum.
The selected 48-HVP estimate gives maximum eigenvalue
`31179.700485618621`, eigenfrequency `499.43728723929792 s^-1` and therefore
14 initial substeps under the frozen adjacent-level policy. Four estimates --
two per process in two independent builds -- agree bit-for-bit.

The 14-substep result is below the predeclared capacity boundary of 96, so
B4E1M one-macro contract research is authorized. No KKT solve, time advance,
nonzero publication or external-reference decode occurred.

## Implementation and build closure

| Field | Value |
|---|---|
| B4E1S identity | `76453ea9d74f996c52a710a126c444b57662485aaad74c71e041e9e952fe17ac` |
| implementation commit | `00b0a9a5a972506df3bae18401aefd4b6cb2df1d` |
| `boundary_reference.cpp` | `545d55fec61bdbd27fb035c432c31307e228248efabff0018d2ac9e880869e1c` |
| `boundary_reference.hpp` | `fc4d937b7b4362b5ab4e0901b02aa8f9a438c85ebf686d234c653ee565d94cc2` |
| `formula_reclosure_main.cpp` | `3dc7a46a1eb3148a320c697a80f8eda42f3b76282f7bcf39ef404e253a1df874` |
| executable | 3,642,608 bytes; `9846db6ab1493d1245509ddfbc13fff75360f565078826aa7acb3afc228ae695` |
| GNU Build ID | `d51b4547c063ea95d75af3f295e28104e81775aa` |

Both fresh GCC 15.2 Release builds produce the same executable and Build ID.

## Frozen-state and spectrum facts

| Fact | Value |
|---|---:|
| fluid / support samples | 6,000 / 5,824 |
| unique pairs / directed records | 354,630 / 615,072 |
| maximum degree | 118 |
| cell distance tests | 2,219,628 |
| flat-tape payload | 5,369,332 bytes |
| active centres | 9 |
| maximum positive strain | `6.6613381477509392e-16` |
| Lanczos HVPs per estimate | 48 |
| maximum eigenvalue | `31179.700485618621` |
| maximum eigenfrequency | `499.43728723929792 s^-1` |
| derived initial substeps | 14 |

The pair root remains
`26ab8b79194d53686510d59a11e013900c79ccad57585549acc2007aa1e66414`.
The workspace and query-chain roots are respectively
`37fb0e72e40331ad5f75785f1c87fd4c6a0f04a7dda998d4f199c462586fcc48`
and
`fa11eb79e14622afb0d1e5083b80006a6efc1c40f0855bc1737e8c9b02868d5d`.

Each estimate owns one evaluation, 48 joint HVPs, one flat workspace and at
most one live workspace. Candidate/audit all-pairs evaluations and HVPs are
all zero, and explicit release leaves zero live workspaces.

## Repeatability, cost and regression

The two fresh process reports are byte-identical:

| Field | Value |
|---|---|
| report | 2,335 bytes; `771aa94575919bce513bf361f8126555d50b252b67b8051a448d15259362f2ad` |
| semantic result | `f57c9ed22f8988a20f88b2f877b9bdf6b328f17b677cc68ceaf02251287e37c9` |
| process A | 0.51 s wall, 65,132 KiB maximum RSS, 99% CPU |
| process B | 0.51 s wall, 65,560 KiB maximum RSS, 99% CPU |

Each process includes two complete spectrum estimates, so this timing is a
preflight cost fact, not a one-macro or production-throughput claim.

The B4E0 report remains exactly 3,518 bytes with raw SHA-256
`458f0780abdd74689bca7946d241e63502b2de5d9c0961f4d8e1655428d45692`
and semantic result
`79a8932136a64c1cfa953caafe323cb9e860cb134c1d4dc0907be7f40891587a`.
The formula-reclosure base self-test also remains PASS.

## Decision

B4E1S is `PASS` and selects only `NOMINAL_HYDRO_SPECTRUM_CANDIDATE`.
B4E1M may now freeze one exact Hydro macro transaction, including solver,
refinement, rollback and external cost gates. External-reference comparison,
multi-step trajectory, runtime/CUDA integration and production authority
remain blocked.
