# Continuum water W1 successor energy discriminator — 2026-08-18

Status: `REPORT_ONLY / IMPACT_ENERGY_CONTRACT_RECLOSURE_REQUIRED / NO_W1_CREDIT`.

## Outcome

The Linux successor oracle reaches the first full-corpus discriminator. The
frozen W0F solver passes the 1200-step hydro scenario and the exact 97-frame
free-fall control, then stops at `CW-DAMBREAK-001` output step 72. Its absolute
mechanical-energy residual is `10,570,530 ppb` against the frozen inclusive
`10,000,000 ppb` bound. Density and divergence converge, centre penetration is
zero, the pressure/contact reaction closes momentum to `116 ppb`, and contact
feature reduction differs by `0 ppb`.

The failure is not a sign error or a borderline implementation discrepancy.
A diagnostic continuation of the unchanged W0F operations completes all
`720/720` dam-break steps but reaches a maximum and final mechanical-energy
deficit of `638,407,503 ppb` (`63.8407503%`). The frozen profile is explicitly
dissipative at impacts, while the inherited common metric requires near
conservation. Those two requirements cannot both remain blocking.

`CONTINUUM-WATER-REF-P1` remains `NOT_RUN`. All reports in this document were
produced from a dirty research tree at baseline commit
`50f53140a5d805a3dd982f9f090c04677214d6a1`; they are discriminators only and
cannot receive clean-tree or corpus credit.

## Frozen-path observation at step 72

The runner records the exact kinetic/mechanical delta introduced by each
ordered substep stage. Values below are normalized by the initial mechanical
energy; their signed sum is the final signed energy balance at step 72.

| Stage | Cumulative delta |
| --- | ---: |
| Divergence pressure | `-2,396,563 ppb` |
| Gravity velocity update | `+288,432,051 ppb` |
| Density pressure | `+77,187,470 ppb` |
| Static contact | `-96,333,751 ppb` |
| Position integration / potential energy | `-277,460,325 ppb` |
| Canonical publication | `+586 ppb` |
| Signed total | `-10,570,532 ppb` (fold-rounding differs by `2 ppb` from the direct metric) |

Contact alone removes `9.6333751%`, but treating that value as static-wall work
would be incorrect: the wall is stationary. Making contact elastic in isolation
would also not repair the balance because density pressure has already added
`7.7187470%`. The defect belongs to the composition and its energy contract,
not to one scalar coefficient.

The bounded failure report has SHA-256
`f33abe8d14b34258c547190f8268c5136816713809d4bfa295dcb9021e590d32`
and remains outside Git at
`/tmp/nextengine-w1-linux-dambreak-stages-dev-50f5314.json`.

## Counterfactuals

Three counterfactual operator compositions were evaluated under separately
domain-separated diagnostic execution roots. None may inherit the W0F root or
W1 credit.

| Candidate | First result | Interpretation |
| --- | --- | --- |
| Contact-aware projected Jacobi-200 | step 72: `10,099,714 ppb`; density maximum `71` iterations | Marginally closer only because density pressure rises to `+8.8107031%` while contact loss rises to `-10.7355122%`; stronger cancellation, not a repair |
| Contact then PCG-50 | step 1 clearance failure | The terminal pressure correction immediately violates the already projected wall constraint |
| Contact → PCG-50 → contact | step 72: `10,893,637 ppb` | Geometry is restored, but the additional split increases the absolute energy defect |

Report SHA-256 values, in table order, are:

- `c90348701b68a8976d4fc512623b39efa0318fe05abeefa9f535df3c37dd50ea`;
- `d6c4aedff70457e14bd6e18c0937b4f4a7ec1f5e82e96369cd20cbf8241c5c85`;
- `8761933291f677b99ff556cd7472f018f71efded231afb301c6591119005cf46`.

These negatives agree with the broader coupling result that sequential fluid
pressure and contact constraints neglect mutual dependencies; a global system
is required when strong fluid/rigid coupling is the product requirement. See
[Probst and Teschner, 2023](https://doi.org/10.1111/cgf.14727) and the
constraint-based boundary derivation in
[Bender, Westhofen and Jeske, 2023](https://doi.org/10.2312/vmv.20231244).
W1 contains only a static boundary, so this observation does not authorize the
future W3 dynamic coupling design.

## Full unchanged-path continuation

The `diagnostic-frozen-observe-energy` mode suppresses only the early stop on
the absolute energy metric. It retains solver, finite-value, capacity,
penetration, count/mass and momentum failures. The unchanged W0F operations
complete `720/720` with:

- density at most `43` iterations and `99,946 ppb`;
- divergence at most `2` iterations and `999,351 ppb`;
- zero centre penetration;
- momentum residual at most `116 ppb`;
- `6,000` samples and exact mass retained;
- final frame root
  `2e4c02a894daf867b0a32eaad5dcbe29c4f6c06340fd6591ca3f44ec913513be`;
- trajectory root
  `a36eb204aefc9f76fc00ff10a13e6952cdec1cb57c48e6ce375c55a9263e11f9`.

The final normalized stage fold is:

| Stage | Cumulative delta |
| --- | ---: |
| Divergence pressure | `-428,444,877 ppb` |
| Gravity velocity update | `+839,404,257 ppb` |
| Density pressure | `+150,750,022 ppb` |
| Static contact | `-494,588,076 ppb` |
| Position integration / potential energy | `-705,529,419 ppb` |
| Canonical publication | `+590 ppb` |
| Signed total | `-638,407,503 ppb` |

The report has SHA-256
`ae871bd7fd49ee07ddc8d88dc024626fb0cd1e5b4262a433901d054c1cf43f93`
and remains outside Git at
`/tmp/nextengine-w1-frozen-observe-energy-dambreak-dev.json`.

## Decision

Do not raise the common `1%` number and do not relabel contact dissipation as
work by a static wall. Reclose the meaning of the energy check:

1. Keep two-sided `abs(E_n-E_0)/denominator <= 1%` for reversible controls and
   equilibrium scenarios (`CW-HYDRO-001`, `CW-FREEFALL-001`,
   `CW-STILL-001`, and `CW-ORDER-001`).
2. For scenarios whose frozen initial state or expected trajectory contains
   impact against an immovable boundary (`CW-DAMBREAK-001`,
   `CW-ORIFICE-001`, and `CW-SEALED-001`), block only positive mechanical
   energy creation above `1%`. Publish the mechanical-energy deficit and the
   per-operator deltas without treating them as wall work.
3. Keep dam-break front/height and orifice transfer comparisons mandatory.
   This change cannot turn a missing or failed independent reference into an
   internal pass.
4. Issue a new corpus/metric root and composite execution root before W1
   resumes. W0F geometry, capacities, solver order, fixtures and historical
   roots remain immutable.

This is a metric-semantics reclosure, not a solver selection. A future
energy-conserving or dynamic rigid/fluid solver must declare and root its own
strong-coupling formulation rather than inherit this static, dissipative
research profile.
