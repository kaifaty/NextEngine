# Nonlocal continuum — productionization roadmap

Status: `ACTIVE / NPR1_B_TERM_CONTROLS / REPORT_ONLY / NO_RUNTIME_AUTHORITY`

This roadmap starts from the completed
[fixed-work performance decision](../../development/nonlocal-continuum-performance-decision-2026-08-20.md).
It defines the ordered work required before the Nonlocal/SISSM candidate can
be proposed for one production water consumer. It does not promote
[SPEC-38](../../architecture/38-continuum-material-physics.md), amend
[ADR-076](../../architecture/adr/076-continuum-material-physics-track.md),
close W2/W3, or let the standalone CUDA tool publish gameplay state.

The retained laboratory implementation remains:

```text
nuv-gather-directed-r0
+ pointer-swap-o1
+ nuv-terms-specialized-o2
+ stable-sample-v0
+ fused-owner-terms-p1
+ compact-csr-u16-p2
```

DFSPH remains the correctness reference and authored dry basin remains the
pre-activation fallback.

## Product question

> Can a separately rooted Nonlocal water profile reproduce the bounded
> SPEC-38 basin physics, define a credible canonical authority, compose with
> one PhysX crate atomically, persist exactly and fit the complete
> `world-dynamics-step` budget?

Passing the old 50k standalone benchmark answers only whether the retained GPU
work is fast on its own research fixture. It does not answer this question.

## Newly exposed profile gap

The completed performance fixture and the selected product consumer share a
similar sample count but not the same physical identity:

| Property | Retained performance fixture | Product consumer |
|---|---:|---:|
| Samples | 48k/50k | nominal 48k, hard 50k |
| Lattice orientation | 48k `80×40×15`; 50k `100×25×20` | `80×15×40`, `+Y` height |
| Spacing | `0.005 m` | `0.05 m` |
| Horizon/support | `0.015 m = 3dx` | `0.1 m = 2dx` |
| Time step | `0.001 s` | `1/240 s` |
| Boundary | none | sealed analytical basin, later one moving crate |

The paper's resolution sweep keeps `h=3dx`, `dt=0.001 s` and fixed material
coefficients over a narrow spacing range. It does not establish invariance for
a tenfold spacing change, the product cadence, `h=2dx` or the sealed basin.
NPR0 therefore precedes any production correctness or integration claim.

## Stage graph

```text
completed fixed-work performance baseline
  -> NPR0 product-profile bridge/boundary                COMPLETE
      -> NPR1 independent physical/correctness reclosure ACTIVE
          -> NPR2 canonical-authority decision           NOT_STARTED
              -> NPR3 private owner/admission/fault path  NOT_STARTED
                  -> NPR4 one-pass PhysX coupling         NOT_STARTED
                      -> NPR5 exact persistence/epochs    NOT_STARTED
                          -> NPR6 basin product vertical  NOT_STARTED
                              -> NPR7 integrated budget   NOT_STARTED
                                  -> NPR8 promotion ADR   NOT_STARTED
```

| Stage | Required result | Exit gate |
|---|---|---|
| NPR0 | exact product-scale, cadence, support and boundary bridge identities | one selected profile or `PROFILE_RECLOSURE_STOP`; no coefficient is selected by visual tuning |
| NPR1 | independent CPU `f64` oracle, canonical quantization and broader water/coupled corpus | hydro/free-fall/dam-break/orifice plus viscosity/surface controls pass predeclared quality, conservation and repeatability gates |
| NPR2 | explicit authority choice | CPU `f64`, deterministic GPU, or research-only outcome selected by evidence; GPU mirror status is not silently promoted |
| NPR3 | private bounded continuum owner candidate | pre-admission capacities, typed failures and complete-or-none candidate publication pass without public schemas |
| NPR4 | one closed composition DAG and reaction identity | one water candidate plus one bounded reaction batch feed exactly one PhysX integration; stale/collision/failure cases publish neither owner |
| NPR5 | composite active checkpoint | save-at-N/resume-to-M equals the uninterrupted run across fixed checkpoint epochs; corrupt input fails before mutation |
| NPR6 | one real basin consumer | production commands drive one sealed basin/crate scene in `game` and `headless`; debug presentation remains read-only; dry fallback applies only before activation |
| NPR7 | successor compositional performance row | all substeps, PhysX, continuum, validation, merges and root publication pass one mutually exclusive `world-dynamics-step` budget |
| NPR8 | consumer-backed architecture promotion | an Accepted successor decision promotes the smallest exact schemas and synchronizes affected SPECs, routing, traceability and roadmap |

Windows execution remains deferred under ADR-082 during Linux development.
It remains a production-promotion gate under ADR-081/076 unless a later
Accepted decision explicitly changes the shipping contract.

## Ordered execution rules

1. Each stage receives a specification and bounded discriminator before
   implementation; failures retain their exact profile and first boundary.
2. A stage may proceed only from the selected predecessor identity. Results
   from the old performance fixture cannot be relabelled as product evidence.
3. CPU/GPU correspondence, same-target exactness, physical-quality agreement
   and production authority are separate claims.
4. Public continuum contracts do not land for the laboratory or a test-only
   adapter. NPR8 owns their consumer-backed promotion.
5. Long corpus and percentile campaigns run only after tiny, boundary,
   capacity, nonfinite and repeatability preflights pass.
6. Performance reopens only after an integrated profiler or a new physical
   profile identifies a measured bottleneck. P3/P4 negative evidence remains
   closed meanwhile.

## Immediate queue

NPR0 is specified by the
[profile reclosure contract](00-profile-reclosure-contract.md). The hash-bound
scale/cadence/support bridge, machine audit, retained GPU preflights and
four-iteration scale-law discriminator are complete and recorded in the
[dated evidence](../../development/nonlocal-continuum-npr0-profile-bridge-evidence-2026-08-20.md).
The [static boundary discriminator](01-static-boundary-discriminator.md) now
selects separate rooted density support and swept contact. Full static-support
GPU preflights and the tiny negative wall discriminator are implemented. The
active step is the predeclared binary64 tiny physical corpus that decides
between unchanged-control and dimensionally-derived coefficients.

That corpus selected neither v3 profile. The frozen hydro remediation rejected
h2 through 50 iterations and selected an h3/16 support-ratio candidate. Its
named v4 profile then passed the expanded audit, repeated full-basin exact
P1/P2 preflights and all four h3-consistent tiny cases. NPR0 therefore selects
one report-only `NONLOCAL_PRODUCT_PROFILE_CANDIDATE` and NPR1 becomes the next
stage. Runtime, public contracts and production authority remain blocked.

NPR1 is now specified by the
[physical/canonical reclosure contract](05-npr1-physical-correctness-contract.md).
Its isolated binary64 canonical publication boundary now passes; independent
term/derivative controls are next. Runtime integration and public contracts
remain blocked through NPR2.

## Stop states

- `NONLOCAL_PRODUCT_PROFILE_CANDIDATE`: NPR0 selects one bounded profile for
  NPR1; no production claim.
- `NONLOCAL_AUTHORITY_CANDIDATE`: NPR1/NPR2 select a physically valid and
  deterministic authority route; integration may start privately.
- `NONLOCAL_LINUX_VERTICAL_CANDIDATE`: NPR3–NPR7 pass on the active Linux host;
  cross-target promotion remains deferred.
- `NONLOCAL_PRODUCTION_CANDIDATE`: every NPR gate including the required
  cross-target evidence passes; NPR8 may propose promotion.
- `NONLOCAL_PRODUCTION_RESEARCH_STOP`: a mandatory physical, authority,
  coupling, persistence or integrated-budget gate fails after its bounded
  remediation cycles.

No stop state replaces DFSPH or changes existing saves/runtime behavior.
