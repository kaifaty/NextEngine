# NSR3-B4E2D7R20R60 dimension-generic centered certificate evidence

Status: `PASS / DIMENSION_GENERIC_CENTER_CANDIDATE`.

Claim status: `SUPPORTED_BOUNDED`.

Evidence classes: `EXACT_CERTIFICATE`, `NUMERICAL`, `CORRESPONDENCE`.

## Strongest result

R59's sole blind failure is not a mathematical dimension-65 boundary. Its
captured active system has dimension 102, and the unchanged centered
fixed-point certificate resolves all 102 solution signs already at depth 4.

- semantic `c8f1180669c8e4c75a094467fc5816ba45a2ae384726d4b0554a897e61c1ae2a`;
- byte-identical stdout SHA-256
  `4b87849bfa4d8572d64ba315e5cb48e576a91eec76af3a3ea8a5ce03a60f83ef`;
- parent R59 case root
  `0337d014a68fac594161b90817e534313671008cdc3fc0deb9a16d619e3d6644`
  preserved at 8 accepted iterations and 1,092 transitions;
- tuple material root
  `2d7693464876f24a11b205b8218e2a8c3042ba3256275c121d9f6ca2aa87857c`.

Tuple roots:

```text
matrix   aa401d0191ad53b7caa7837c827913fa711e1fdc433bc200c020719d297fb09d
inverse  467e815a4813e7774523699147db38fbcda06dea9b93bfb1696b410e0df0cfd7
rhs      64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1
solution 2c3a68b39bc82c4de5c09d9f4d4a091cbfd41024b0cdc3a8e509da7effcfe6e3
```

## Two-sided inverse audit

All 10,404 entries on each side are contained by the frozen Dot2 bounds with
no underflow.

| defect | exact outward infinity norm | Dot2 upper bound | result |
|---|---:|---:|---|
| `I-A X` | `9.802054407271511822201453046574904566e-4` | `9.802054407271511822201453046802457703e-4` | contractive |
| `I-X A` | `2.488543384840849253133379776407460564e-2` | `2.488543384840849253133379776412811039e-2` | contractive |

The left defect supplies the fixed-point contraction. The independent right
defect closes the frozen two-sided admission discriminator.

## Centered sign certificates

| depth | signs | unresolved | error radius | minimum separation | dots / input pairs |
|---:|---:|---:|---:|---:|---:|
| 4 | `24+ / 78-` | 0 | `2.7127883223498731e-6` | `32.42340332375679` | `11,220 / 1,145,358` |
| 8 | `24+ / 78-` | 0 | `1.9964719950750946e-15` | `32.42340603654511` | `11,628 / 1,187,382` |
| 16 | `24+ / 78-` | 0 | `1.9964719939446993e-15` | `32.42340603654511` | `12,444 / 1,271,430` |

Every fixed depth is exact, underflow-free, contractive and passing. Depth 4
already has a separation over seven orders of magnitude larger than its error
radius; deeper work reaches the arithmetic floor but is unnecessary for this
tuple.

## Controls and regressions

The literal 1x1 successful control has zero two-sided defect and a passing
centered certificate. The deliberately bad inverse has defect norm one and is
rejected. Their aggregate root is
`59c0f708cfb8fdc8a0fff6cc9b7b044f197a76b0192f469fdd9aee6ed456ce8c`.

- R59 remains byte-identical at `461c4962...44ac`, semantic
  `a0881eaa...faa9`;
- R50 remains `GENERIC_VERIFIER_TORSION_CANDIDATE`, semantic
  `190ac441...d86e`.

## Claim ceiling and next decision

R60 certifies one immutable dimension-102 tuple. It does not prove all
dimensions or authorize a callback change by itself. The next smallest test is
R61: a separately frozen dimension-generic, structural-work-capped callback
replaying the six immutable v5 cases once. Production, runtime/GPU integration
and performance remain `NOT_TESTED` and unauthorized.
