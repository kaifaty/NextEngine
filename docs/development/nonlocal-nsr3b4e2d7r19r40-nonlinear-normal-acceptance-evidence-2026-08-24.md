# NSR3-B4E2D7R19R40 nonlinear normal-step acceptance evidence

Date: `2026-08-24`

Status: `PASS / NONLINEAR_TOPOLOGY_REJECTED / ROLLBACK EXACT`.

## Outcome

The exact R39 Hager--Zhang endpoint maps correctly into a very small physical
particle displacement and predicts nonlinear constraint reduction almost
perfectly. It is not an admissible complete step: the rebuilt trial has a
different pair-membership graph, slightly increases existing box penetration
and increases the complete normalized merit.

Frozen route precedence stops at the first boundary,
`NONLINEAR_TOPOLOGY_REJECTED`. This is a successful discriminator result, not
a solver or production pass.

## Exact mapping and feasibility

| Quantity | Result |
|---|---:|
| R39 terminal linear `psi` | `9.13328870609491e-27` |
| Dimensionless endpoint global L2 | `1.2282850523133922e-6` |
| Physical displacement global L2 | `6.141425261566961e-8` |
| Source nonlinear `psi` | `3.291845837405691e-15` |
| Trial nonlinear `psi` | `9.169374035643007e-27` |
| Source violation norm | `8.113995116347669e-8` |
| Trial violation norm | `1.354206338461241e-13` |
| Source/trial active rows | `1420 / 53` |
| Predicted reduction | `3.2918458373965575e-15` |
| Actual reduction | `3.2918458373965216e-15` |
| Feasibility ratio `rho` | `0.9999999999999891` |

The nonlinear feasibility model therefore passes decisively. The R39 normal
step is not invalidated as a density-restoration primitive.

## Three independent admission boundaries

### Pair membership

The current workspace reproduces the frozen topology root and has `340340`
pairs. The nonlinear trial has `340262` pairs, a net reduction of `78`, and a
different ordered pair-membership root:

```text
current 58ce6532f180f9f2803a069f8cfeb25278fbc3930796d7cf29700db08ce4fac3
trial   2e74673c3ae59c099c91902d7eac31b97b1c5a9496497128174d9a899f2bae06
```

R40 does not yet distinguish zero-weight horizon-shell crossings from
interior support changes. That is the first required follow-up; the frozen
gate is not relaxed by this observation.

### Contact

Maximum dam-box penetration changes from
`2.998548574370541e-8` to `3.132235994532384e-8`, an increase of about
`1.3368742e-9`. R40 deliberately does not run contact correction, so this
exposes a missing contact-tangent ownership rule for a later composite step.

### Complete merit

| Component | Actual reduction |
|---|---:|
| PHR | `+5.589684174801529e-14` |
| Inertia | `-5.849943708988075e-14` |
| Total precancelled | `-2.602595341865466e-15` |
| Raw total | `-2.6025953323736834e-15` |
| Long-double total | `-2.6025952637604656e-15` |

The feasibility benefit almost balances the inertia cost, but the complete
merit increases. Long double resolves the negative sign and agrees with the
precancelled binary64 result, so binary128 is correctly not executed. This is
not a subtraction floor or a reason to add runtime extended precision.

If topology/contact had passed, the frozen scientific branch would have been
`COMPOSITE_NORMAL_TANGENTIAL_STEP_REQUIRED` rather than acceptance.

## Work, rollback and reproducibility

- one exact R39 parent replay;
- two nonlinear workspace builds and releases, maximum live count two;
- one private nonlinear trial and two exact divided evaluations;
- one long-double audit, zero binary128 audits;
- zero new JVP, VJP, HVP, model or outer work;
- exact source/endpoint rollback;
- R39 regression remains byte-exact at stdout SHA
  `06812cb75177cd4666b103101f6bc9238fa20760179bc71563f194fe476db252`.

Two clean Release builds are binary-identical:

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r40-a.gW5CYl
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r40-b.MVa33r
binary SHA-256 942f034d550aa29a851d29596c4b575a2a30cb1ea30dd56f8d7836474bcea6e8
size           7541344
ELF build-id   11f930e5dd743a0d36b4ef66b2d2f8572ef2881d
```

Fresh runs are byte-exact with empty stderr:

```text
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r40-a.Q9OUlz
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r40-b.ofmNrp
stdout bytes   3989
stdout SHA-256 6db946e6c6e327520b60f98d34e60f346f0b02bfe84e88767fc1e36d7db9489d
semantic       d8ea97108543603e511c68632a8993eb010f0a7f8342f034a55c8d53555e98ff
stderr bytes   0 / 0
```

No timing was measured or interpreted on the shared host.

## Decision

Preserve R40 exactly. Research/freeze a rollback-only R41 crossing audit that
classifies every lost/entered pair by source/trial radius, kernel value and
derivative distance from the compact-support horizon, and attributes the
particles responsible for contact regression. Do not weaken R40 membership or
contact gates and do not begin composite-step design until those two first
boundaries are understood.
