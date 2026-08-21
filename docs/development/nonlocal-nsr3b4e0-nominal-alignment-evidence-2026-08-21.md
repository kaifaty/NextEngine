# NSR3-B4E0 nominal-alignment evidence -- 2026-08-21

Status: `PASS / NOMINAL_ALIGNMENT_CANDIDATE / B4E1_CONTRACT_DESIGN_AUTHORIZED`

## Result

Hydro and Dam reconstruct the exact R1D fluid, boundary and scenario roots,
the same canonical initial state, and capacity-valid nominal flat
neighborhoods. ID swap, one-micrometre boundary mutation and one-step schedule
mutation all change their owned roots. Two fresh processes and two independent
Release builds agree byte-for-byte.

No external reference curve was opened and no spectral estimate, KKT solve,
time step or nonzero publication ran. Orifice remains excluded until B4O.

## Implementation and build closure

| Field | Value |
|---|---|
| B4E0 identity | `c53a112830cb94c4139da75c2044d28f61673cb1aa05c116098967aee41f7bbe` |
| implementation commit | `941c291f653e729930ea97d1b8f0752a544d6997` |
| `boundary_reference.cpp` | `36bee4992f3b0481b238fe7b3087498e84d8e341afb18654325e13fb7e5583c8` |
| `boundary_reference.hpp` | `f295e81021508c1970723c6e023669a9b77275524d83b7ec5e026dd3c7d9f369` |
| `formula_reclosure_main.cpp` | `b03186d2bef0717ab6710e4a15cd041135f36e2b06ba4ea19a5d9b9b84c8144c` |
| executable | 3,633,560 bytes; `fdd16a0ce1c4116b4acc9407c18e33bb544b51ec78063692b855234dc0091978` |
| GNU Build ID | `16e9d0635a7e4808c28c45f37ea4d95a1226161b` |

Both fresh GCC 15.2 Release builds produce that same executable and Build ID.

## Exact scenario alignment

| Fact | Hydro | Dam |
|---|---:|---:|
| fluid / support samples | 6,000 / 5,824 | 6,000 / 16,384 |
| fluid root | `7d4e661d08de08b18d43a76342329b51f6ae98bca9baee3d850e0f403eae5606` | `9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76` |
| boundary root | `25de85b5eeec041c12bbb5de10e00b8374457b4cf09cb61d99dfc4d5511d8d62` | `1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d` |
| scenario root | `c430b679dfeec33a6ac12c51df75ddee7e7bc48484c6c05188219f0a727909a0` | `8d0a0a85adba50d4784245d460c83757bcce92841d804f4739b81faf27fd5f09` |
| canonical support-set root | `776007ce7caca18b2b0c33c0beff205c9d6305ccf0a96489e06715a367cb01da` | `fd9bc06f22d1fc2e7d07a11b9b761175b2537f54ef167ff5276b32893a6394aa` |
| static-index root | `daafa32e95eea258c51704d30d7654a702778d560d59fab749a96180b0a6b297` | `a2d97ab6f26383d826366eba2a3d4392f5ef9610dda87e89509e93bd9daf61e8` |
| flat pair root | `26ab8b79194d53686510d59a11e013900c79ccad57585549acc2007aa1e66414` | `c330a0aecb913d92e95478dc3325f9bc326d5e8d3057485723dd62eef1493889` |
| unique pairs / directed records | 354,630 / 615,072 | 335,814 / 596,256 |
| maximum degree | 118 | 117 |
| cell distance tests | 2,219,628 | 2,089,212 |

Both maximum degrees remain below 160 and both pair counts below 960,000.
Each case constructs exactly 6,001 flat offsets, zero nested rows and repeats
the static index, pair/evaluation and flat work exactly.

The two scenarios share initial aggregate root
`37d83c159ff913afef290b9dc1cc7affe9f6018d726dd7b13ab9308f3bf741d3`:
6,000 samples, 750 kg, COM `(0.5,0.375,0.5) m`, q99 x/y
`0.975/0.725 m` and zero velocity. Their distinct canonical frame roots bind
the distinct scenario identities.

Hydro reports nine pressure-active centres with maximum positive strain
`6.6613381477509392e-16` and zero minimum branch margin. This is a retained
machine-floor active-set diagnostic, not physical pressure and not a B4E0
failure. B4E1 must preserve it when explaining the one-macro spectral cost.
Dam reports zero active centres and the same value as its minimum margin.

## Repeatability and regression

Two fresh reports are byte-identical:

| Field | Value |
|---|---|
| report | 3,518 bytes; `458f0780abdd74689bca7946d241e63502b2de5d9c0961f4d8e1655428d45692` |
| semantic result | `79a8932136a64c1cfa953caafe323cb9e860cb134c1d4dc0907be7f40891587a` |
| process A | 0.40 s wall, 47,052 KiB maximum RSS |
| process B | 0.41 s wall, 47,176 KiB maximum RSS |

The complete B4C4C1 flat-adjacency regression also remains exact: raw report
SHA-256 `cd2d5279776c4a1dce2c98038e3ca8a13bc9d1defba4685a95abb1a71d33882f`
and semantic result
`b4d5260012f4208026b814411891a221dda2d5823b242027d890d4066e69550c`.

## Decision

B4E0 is `PASS` and selects `NOMINAL_ALIGNMENT_CANDIDATE`. Only B4E1
one-macro resource-probe research/contract design may begin. Reference-curve
decode, comparison execution, a full nominal run, runtime/CUDA integration
and production claims remain blocked.
