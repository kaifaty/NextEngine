# NCGP2 surface translation-floor evidence — 2026-08-30

## Result and claim ceiling

Author result: `H1 SUPPORTED / INDEPENDENT_REVIEW_PENDING`.

The unchanged global-position `f32` solver still fails the retained pair, and
promoting only surface arithmetic to `f64` does not change that route. A
boundary-free representation that factors each position into a shared
binary32 high anchor plus a per-sample binary32 local part passes the original
pair and the complete translation sweep without changing the physical model,
solver tolerance or HVP ceiling.

This is evidence for loss of small global-position updates, not a scalable GPU
water solver. The selected representation is a tiny shared-anchor
specialization. Dynamic per-cell anchors, graph transitions, contacts,
transactions, 4k/50k trajectories and performance remain `NOT_RUN`.

## Frozen identities

| Item | Identity |
| --- | --- |
| Contract | `b4479af3ec41dd37aa02bd9f299c05387d416043d4f9701f0e08e8749752e4cb` |
| Candidate commit | `2a0c38a2598da20d70a8e59c2f8aa4b0846b50fe` |
| Candidate tree | `bac89146fc3aa670705e46cbf2fc09187c12180b` |
| Source root | `4fe0d45fe614b3ceddccfb6a76eb4313a1855b133851ef64059307e745e80568` |
| Release binary | `98de5b4aca877c92de9a0c4d0c7ff077ec154f2e138bd771c1d72b9c2ddb4b28` |
| GPU | NVIDIA GeForce RTX 3080, SM 8.6 |
| CUDA runtime/driver | `13030 / 13030` |

The two independent Release build directories produced byte-identical NCGP2
binaries. Their complete outputs were also byte-identical:

| Mode | Exit | Output SHA-256 | Result |
| --- | ---: | --- | --- |
| `--phase-a` | `0` | `b592265175947f7d3cd5c65194b7b7ba3b369eca055d29c160408d9c5625c9de` | primary and controls reproduced |
| `--surface-f64` | `4` | `e710702655e81de0b6c08d2466cec8abd802c68a94fb302966211a0d9b6b8c9b` | counterfactual failed |
| `--compensated-state-f32` | `0` | `b43266b5fbe3c34583170a29f146cafe747ee77d350aa8a92b4a3a4a60988f6b` | counterfactual passed |

Compiler controls remained `-ffp-contract=off -fno-fast-math` for C++ and
`--fmad=false --prec-div=true --prec-sqrt=true --ftz=false` for CUDA, targeting
SM 8.6.

## Phase A: unchanged solver

The shared initial binary32 bytes, CPU oracle, permutation route and work/result
roots all closed. CPU long-double solves passed at all centers. The unchanged
CUDA route produced this first-specific pattern:

| Center (m) | CUDA | `R_x` | HVP | Trials (accepted/rejected) |
| ---: | --- | ---: | ---: | ---: |
| 0.125 | pass | `3.709e-7` | 6 | 3 (3/0) |
| 0.25 | pass | `6.676e-7` | 6 | 3 (3/0) |
| 0.375 | pass | `3.158e-6` | 6 | 3 (3/0) |
| 0.5 | pass | `4.991e-6` | 6 | 3 (3/0) |
| 0.625 | fail | `1.5036265e-5` | 48 | 24 (3/21) |
| 0.75 | fail | `1.5036265e-5` | 48 | 24 (3/21) |
| 1.0 | pass | `2.215e-6` | 6 | 3 (3/0) |
| 1.5 | fail | `1.5311771e-5` | 46 | 23 (2/21) |
| 2.0 | fail | `1.9560920e-5` | 46 | 23 (2/21) |

The non-monotonic pattern rules out a simple distance-from-origin law. It is
consistent with discrete interaction between accepted updates and the local
binary32 lattice. The retained `x=0.25` control passed; `x=0.75, gamma=0`
passed at `R_x=1.3775e-7`.

## Phase B discriminators

### Surface-only binary64

Surface pair subtraction, distance, spline, gradient/HVP products and energy
were evaluated in binary64 and narrowed once to the existing vector outputs.
The failing centers, route counts and residuals were unchanged. In particular,
`x=0.75` remained `failure=9`, 48 HVP, 24 trials and
`R_x=1.50362650553e-5`. H2 is therefore falsified for this fixture.

### Shared-anchor compensated state

For the boundary-free tiny discriminator, each input was decomposed exactly as
`shared binary32 high anchor + binary32 local coordinate`. The solver ran in
the local coordinate and published one final binary32 global position. All 9
centers passed, all corrected/permuted work and result identities matched, and
all inputs reconstructed to the original binary32 bits.

The primary `x=0.75` result used 6 HVP, 3 accepted trials, no rejected trials
and reached `R_x=1.62124633789e-6`. The maximum final-position difference from
the independent CPU result across the sweep was
`3.84540017606e-8 m`, below the frozen `5e-6 m` state gate.

The sealed representation work receipt records 36 decompositions, 36 exact
reconstructions and 72 published components under root
`bdd916610535d133ed446aa01ecf79a2671fcd678f821d08a457cb2c04a4959c`.

An independent analytic FCR0 surface control passed:

- gradient relative L2: `7.57043481411e-9`;
- HVP relative L2: `8.83071361321e-8`;
- analytic directional relative error: `7.57043479309e-9`;
- equal/opposite closure: exactly `0`.

This selects H1 for the retained boundary-free pair. It does not yet select a
production representation: a shared anchor is only an algebraically exact
special case of the planned per-particle/per-cell `hi+lo` storage.

## Retained controls and sanitizers

The clean post-commit NCGP1 target retained:

- graph self-test `PASS`, output `e89ddaed5eeef864e92ca25bd2d39bb242e1347fb234c352f225fb33b9a27166`;
- operator self-test `PASS`, output `781f052b8ac72eb86bc3d4d60bb408726919804ad60c4fc0ea35c6778db280d6`;
- solver self-test `PASS`, output `bc8b71dcf6b7564dcd540b4e599102fee3112476448e1e6b3b8ca2aed58840ff`;
- original tiny route `PHYSICS_REFUTED`, exit 37, output
  `0b0db1e338854319005172453281faa31e159694a0e10071359d7468564e359d`.

`compute-sanitizer` memcheck, initcheck and synccheck each returned exit 0 and
`ERROR SUMMARY: 0 errors`. Their captured outputs were byte-identical at
`7560833f66a5c4846f68aa59a5f62cfe25c823da6fa1e485929d4fbcf1359837`.

## External mechanism boundary

NVIDIA documents IEEE binary32/binary64 behavior, round-to-nearest, operation
ordering and cancellation in the CUDA programming guide and its floating-point
whitepaper:

- <https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/mathematical-functions.html>
- <https://docs.nvidia.com/cuda/pdf/Floating_Point_on_NVIDIA_GPU.pdf>

Those sources explain why a local representation can preserve smaller updates;
they do not prove the solver-specific cause. The local counterfactuals above
are the causal evidence.

## Decision and next action

Pending independent review, the smallest justified next package is a scalable
state-representation contract. It must freeze per-cell or per-particle anchors,
canonical renormalization, graph-cell transitions, ghost/contact boundaries,
rollback, result/work receipts and memory cost before reopening 4k/50k
trajectories. Performance remains `NOT_RUN`, and the roadmap is unchanged.
