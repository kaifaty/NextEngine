# NSR3-B4E2D7R18R4R1 normalized Krylov forcing replay evidence

Date: `2026-08-22`

Status: `PASS / DIMENSIONLESS_FORCING_RETAINS_SECOND_ITERATION / D7R19_BLOCKED`

Implementation commit: `4455dc3c4dbd7c7b3d779dac0a476f3a18a8168b`.

## Result

The replay confirms R4's scale-defect mechanism, but the selected
dimensionless adaptive forcing still requires the second Krylov iteration:

```text
route = DIMENSIONLESS_FORCING_RETAINS_SECOND_ITERATION
```

At the exact active outer-11/trial-0 boundary:

| Quantity | Value |
|---|---:|
| `||r0||` | `2.170366555165981e-11` |
| `||r1||` | `4.931226196396423e-16` |
| `q=||r1||/||r0||` | `2.2720706714996822e-5` |
| inherited `eta` | `4.658719303806553e-6` |
| mapped dimensional `eta` | `3.953054413639542e-4` |
| dimensionless `eta` | `1.2388346565200968e-5` |
| fixed observable | `0.5` |

Thus the inherited and dimensionless rules both continue, while the mapped
dimensional control stops. The mechanism correspondence is exact. The
dimensionless separation is `1.0332360149795854e-5`, or
`3.0495744537708705e15` binary64 ULPs, far outside the frozen 4096-ULP
ambiguity band.

## Reproducibility

Raw evidence:
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r18r4r1.vhcliX`.

Two independent clean GCC 15.2 Release builds produce byte-identical
5,619,248-byte executables:

```text
SHA-256  66770028e2791e7b6aa4ec0ccdfa98ab41af3cb33180fe16236a7a3b3e6595f3
Build ID a1351634d264b8b9e60374450f775583ccfac33e
```

Build directories:

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r4r1-a.294EEk
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r4r1-b.Gi0imk
```

One process from each build exits zero with empty stderr and emits the same
3,485-byte stdout:

```text
stdout SHA-256 26d3bf53272e9eaa2a67d579cdf3a51e5f9ab0435277b1141f92cbcd95298f31
semantic result f295cbee3002b1fb872363a4f2288b0640bfcd5c255170d05e94e393303b077c
```

The frozen regressions remain exact:

```text
D7R18R4 32e4369a19564c769148c9d0bc534bce8f0d22dac4a0eeb085aa191a46dc38c6
D7R18R3 e0b36e34a047de1d8bb1435fa6f68cf5a53ab8a4260a95719775c03e4608d3e5
D7R18R2 3659eac888c22eae5bcf7ae8c5f8a426bcbc12bd24bd3320c23be0c176815d77
D7R13   514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de
```

R4 still exits one on its frozen `WORK_LIFECYCLE` control. R4R1 does not
rewrite that negative evidence.

## Controls

The replay recovers the exact active transaction root
`9a3a57e7fcb29700c72c710936ef02ea7459cf2470b7d59ca663605df00c5ee9`
and the frozen stationarity, step, prediction and divided-reduction anchors.
Reference and independently derived aligned normalized diagnostics have the
same root:
`3edaf2e581b6462a239e6fd587ca4c615c8a98c4d5230ea1a2bd68ebf560bf29`.

Candidate work is exactly two workspace builds/releases and two HVPs, with
maximum live workspace one and zero all-pair calls. Four invalid profile,
dual, binding and shape cases reject before workspace/HVP work. The command
performs zero candidate acceptances, zero candidate outer updates, no full
changed-policy transaction and exact rollback.

## Decision

The extra R4 iteration is caused by a scale-dependent inherited forcing rule,
but replacing it with the selected dimensionless adaptive rule does not reduce
work at the observed boundary. This is useful: 39 HVPs is now a principled
candidate baseline for this normalized policy, not a post-observation edit to
R4.

Research/freeze a separate R4R2 complete rollback-only transaction with the
dimensionless rule explicit in the solver. Derive its full work ledger under
that policy before implementation and retain all R4 semantic, precision,
cross-profile and rollback gates. Do not alter R4, run D7R19, a nominal
substep, macro, trajectory or timing lane.
