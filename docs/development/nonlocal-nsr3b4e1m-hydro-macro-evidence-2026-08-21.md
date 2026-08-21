# NSR3-B4E1M Hydro one-macro evidence -- 2026-08-21

Status: `PASS / PHYSICS_VALID / PERFORMANCE_REMEDIATION_REQUIRED`

## Result

The exact nominal Hydro step-1 transaction passes in two independent Release
builds and produces byte-identical reports. Both temporal levels 14 and 28
solve successfully; their adjacent gate passes and only the 28-substep fine
state is published. No reference file was opened and no second macro ran.

This is a positive physics/correctness result but a negative cost decision.
One process takes about 48.8 seconds on one CPU core. Even the simplest linear
projection of one Hydro plus one Dam trajectory is about 26 machine-hours,
well above the predeclared four-hour boundary. B4E2 execution is therefore
held and work routes to B4EP performance attribution/remediation first.

## Implementation and build closure

| Field | Value |
|---|---|
| B4E1M identity | `0cdc26e1d0b406fecc39c64cebfee080be32804b993e6d433fa0a74658efb4cc` |
| implementation commit | `45111bd9662eeb931c80dfd601e9ef08190c18ab` |
| `boundary_reference.cpp` | `353d35eca5867ae6dcd5b2d87f062eed2bc3417054124562614f209b9a653b90` |
| `boundary_reference.hpp` | `ed599037dfe01a262bfbaadddbe31024d7164f83eaed2595563e755f76a735e6` |
| `formula_reclosure_main.cpp` | `29cf1319afe2320aed47226ea7c5abb911c9c9b1ccac33d04bc201ace3133593` |
| executable | 3,676,080 bytes; `a01a213ee0338f63aed57449656161820ea35318c372efc5ae9d5cf309355f62` |
| GNU Build ID | `d18ddf01fafae842bf78889112525f78c7863902` |

Both fresh GCC 15.2 Release builds produce the same executable and Build ID.

## Exact solver and physical facts

| Fact | Value |
|---|---:|
| initial / selected fine substeps | 14 / 28 |
| attempted / accepted / discarded substeps | 42 / 28 / 14 |
| outer / rejected trials | 221 / 0 |
| nonlinear / spectral HVP calls | 411 / 48 |
| adjacent position error | `3.8569787180118523e-5 dx` |
| adjacent velocity error | `5.2592706346801757e-6 c` |
| adjacent kinetic error | `0.024055559563491338` |
| initial / maximum mechanical energy | `2759.0624999999281 J` / same |
| energy creation / allowance | `0 J` / `27.590624999999282 J` |
| maximum positive density strain | `0.00045547995081940407` |
| private / decoded penetration | `0 m` / `0 m` |
| strict momentum-ledger residual | `1.5500929290499512e-11` |

The step-1 frame root is
`eaa6fe3aea4567594d82ce9fb76be16a4a99256e45b6ee10c4bc4c6417311eb5`
and its canonical aggregate root is
`8a634d69ec2bdb692d08e2c1cdbbf4fd8ce5a4cc39289a7c05a9acd8f0ca90dc`.
The committed trajectory, legacy ledger and policy ledger roots are
respectively
`4689e74310815f413212d006d02347d598d38ce70b0ca44eee2253cddb1f6fc4`,
`df58c67ec644ad717cf6d2192dd31743163bf65438289feedba1e93457f0c057`
and
`b8502f70738b39cad693e515ba1546a3f5ae2e1dae432d597cc4ae7b0e0c0444`.

## Work and ownership facts

| Fact | Value |
|---|---:|
| joint evaluations / total HVPs | 226 / 459 |
| static-index builds | 1 |
| flat workspace builds | 227 |
| flat offset / pair-index records materialized | 1,362,227 / 151,461,068 |
| nested rows | 0 |
| retained transfer / read / release | 42 / 42 / 42 |
| maximum / final live workspaces | 2 / 0 |
| candidate/audit all-pairs work | 0 |

The query-chain root is
`e00b171a8efee89ab000969ac5ddc08203490c4f0870d32fb59391dd548b0d8b`;
the retained-workspace receipt is
`80f02ba9425a365ac7fa01dbb983fd44a71a87b8d8a3766ec87cd33541b0be20`.
The counts show that retained post-step diagnostics are working, but every
outer/trial state still rebuilds and rematerializes a full nominal
neighborhood/tape. This is the first concrete target for B4EP attribution.

## Repeatability and cost

The two fresh reports are byte-identical:

| Field | Value |
|---|---|
| report | 6,151 bytes; `b9601aaad292c43201a5ab054192de4131478eb27e568e1eb78e6b613b07eecc` |
| semantic result | `54d42af619dbd48ae400ce4656ba155a8726dbd07b00754ecfc137ee864c111d` |
| process A | 48.83 s wall, 93,484 KiB maximum RSS, 99% CPU |
| process B | 48.80 s wall, 93,896 KiB maximum RSS, 99% CPU |

Using the 48.815-second midpoint only as a scheduling projection:

- one Hydro+Dam pair, `1,200+720=1,920` macros: about 26.03 machine-hours;
- both required repeats, 3,840 macros: about 52.07 machine-hours;
- B4E2's 28 macros repeated twice: about 45.6 minutes.

Later-state cost may differ, so these are not throughput guarantees. The
single-pair projection nevertheless exceeds the four-hour routing boundary by
about `6.5x`. Running independent processes in parallel can reduce wall time
but not machine-hours; it also cannot remove the serial one-core bottleneck
inside each trajectory.

B4E1S, B4E0 and the formula-reclosure base self-test all remain PASS with
their previously frozen semantic results.

## Decision

B4E1M selects `NOMINAL_HYDRO_ONE_MACRO_CANDIDATE` for correctness only. It
does not authorize a production solver. B4E2 contract design remains
logically available but its execution is held. Freeze and run B4EP profiling
before choosing an optimization; preserve all physics/root gates as the
non-regression oracle.
