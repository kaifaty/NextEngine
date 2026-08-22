# NSR3-B4E2D7R18R4R2 dimensionless-forcing transaction evidence

Date: `2026-08-23`

Status: `PASS / FULL_NORMALIZED_DIMENSIONLESS_FORCING_STATE_CONFIRMED / D7R19_RESEARCH_AUTHORIZED`

Implementation commit: `70a9aabf09d2b57efc04d64aa1d74c8f274b13c6`.

## Result

The complete normalized, pairwise-precancelled private transaction passes
with explicit dimensionless Krylov forcing:

```text
route = FULL_NORMALIZED_DIMENSIONLESS_FORCING_STATE_CONFIRMED
```

Each active run reaches provisional outer `11`, confirmation `12` and
admissible holdout `13` with `19 accepted / 0 rejected / 39 HVP`. Each inactive
run reaches `0/1/2` with no trial or HVP work. Across five runs the ledger is
exactly `48 outer / 57 trials / 57 accepted / 0 rejected / 117 HVP / 153
workspaces`.

All 57 trust solves select dimensionless forcing; none selects inherited
forcing. The active root is
`9a3a57e7fcb29700c72c710936ef02ea7459cf2470b7d59ca663605df00c5ee9`;
the inactive root is
`be761a3c07bc4a7f558486c5e9bc5d3b80e1cc0ba2982e1698e7f5a3d24c2585`.
Reference repeat and independently derived aligned profiles are byte-exact.

## Reproducibility

Raw evidence:
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r18r4r2.d4NeGk`.

Two independent clean GCC 15.2 Release builds produce byte-identical
5,649,984-byte executables:

```text
SHA-256  516d33468377d0c3f7e22d415454b0059c5407e272781e5bd509c211e3233a51
Build ID acfe4b5132baebb3b2cf26df3656f1624c5b0d12
```

Build directories:

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r4r2-a.7kaYk8
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r4r2-b.LEjYM8
```

One process from each build exits zero with empty stderr and emits the same
4,594-byte stdout:

```text
stdout SHA-256 5bb8f5abcb6abf0903c681529771ee6814b567f09d6e12f138514c3f0dab0dc3
semantic result ffcede5263bdedf21486d3068d5ded6f0d986ae2f05b437410e047535a2494d3
```

All frozen regressions remain exact:

```text
D7R18R4R1 26d3bf53272e9eaa2a67d579cdf3a51e5f9ab0435277b1141f92cbcd95298f31
D7R18R4   32e4369a19564c769148c9d0bc534bce8f0d22dac4a0eeb085aa191a46dc38c6
D7R18R3   e0b36e34a047de1d8bb1435fa6f68cf5a53ab8a4260a95719775c03e4608d3e5
D7R18R2   3659eac888c22eae5bcf7ae8c5f8a426bcbc12bd24bd3320c23be0c176815d77
D7R13     514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de
```

R4 remains an intentional hard `WORK_LIFECYCLE` FAIL against its separate
38-HVP contract.

## Precision and lifecycle

Every one of the 57 accepted trials receives the existing direct normalized
long-double audit; none resolves negative. All 12 candidate-effect acceptances
resolve positive in direct normalized binary128 and pass the magnitude bound.
Runtime binary128 remains prohibited.

The forcing dominance control uses the actual frozen spacing:
`dx*sqrt(8)=0.14142135623730953 < 1`. Four invalid profile, dual, binding and
budget cases reject before workspace, HVP, forcing-policy or precision work.
All-pair candidate calls remain zero, maximum live workspace is two and all
input state rolls back exactly.

## Decision

The tiny normalized private solver is now formulation-, precision-, scale-
and policy-consistent under its frozen fixture. R4R2 authorizes research and
contract design for D7R19, not its execution.

Research D7R19 as one aligned nominal-substep rerun using the confirmed
normalized transaction and explicit dimensionless forcing. Before freezing
execution, revalidate mapping, watchdog, physical admission, resource bounds
and the exact relationship to D7R17. Continue to forbid a second substep,
macro, trajectory and timing lane until that contract is frozen.

