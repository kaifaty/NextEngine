# Original Nonlocal GPU dynamic visual capacity corrigendum — revision 2

| Field | Value |
| --- | --- |
| Research ID | `NGQ2` revision 2 |
| Status | `FROZEN / GAME_QUALITY_ONLY / TOOL_ONLY` |
| Supersedes | Only the CSR capacity in NGQ2 revision 1 |

## Decisive revision-1 result

The first valid 4k run completed 41 accepted steps. At the impact transition,
step 42 required maximum directed degree `124`, while revision 1 admitted only
`123`. The state before failure was finite, contained and connected; maximum
speed was `2.4181353867 m/s`, analytic-contact velocity error was
`9.9130423e-6 m/s`, and step-24 silhouette satellite fraction was zero.

Revision 1 is therefore `APPARATUS_INCONCLUSIVE / CSR_CAPACITY_123` and grants
no visual-quality result. The failure is preserved and must not be relabelled
as a physical rejection.

## Single apparatus change

For both dynamic visual lanes only:

```text
directed-pair capacity = dynamic_sample_count * 160
maximum admitted degree = 160
neighbor ID width = u16, unchanged
```

The approximately 30% capacity headroom is selected before the revision-2 run.
It changes allocation only: neighbor predicates, solver equations, five fixed
iterations, fixtures, 96-step schedule, observer and every game-quality gate
remain byte-for-byte revision 1.

The NGQ1 48k timing profile remains `N*123`; revision 2 cannot inherit its
4 ms result. If the visual corpus passes, the 48k performance measurement must
be repeated with `N*160` capacity before any combined speed-and-quality claim.

## Stop rule

If either revision-2 lane needs degree above 160 or exhausts `N*160` directed
pairs, stop with `APPARATUS_INCONCLUSIVE / CSR_CAPACITY_160`. Do not add a third
capacity lane under NGQ2.
