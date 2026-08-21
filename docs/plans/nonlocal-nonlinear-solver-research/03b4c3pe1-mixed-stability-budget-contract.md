# NSR3-B4C3PE1 -- mixed macro stability budget

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / ADAPTIVE_REDESIGN_BLOCKED`

Parent B4C3PE passes with JSON-without-final-LF SHA-256
`aebe7fbe218b507ae0ca8ebe7fde5ecafc38b5ec66fc894d669043649c51ea51`
and semantic SHA-256
`584db48c73b11131db26c5a76758111bd47a7569d85599936fb007951d089395`.
B4C3P and B4C3TR remain exact FAIL controls.

## Policy identity

```text
sha256  c10f0f7c961372dc2b766a0fb35511dc0435f6e5a3175a732adfbc4cc8a913d5
text    nextengine.nonlocal.mixed-stability-policy|v1|temporal-share=0.5|absolute-share=0.01|adjacent=48-96,48-96,96-192
```

Reuse the B4C3P macro profile, ledger policy, six lanes, binary references and
parallel scheduling exactly. Change only same-level representation admission.

## Per-frame, per-field admission

For candidate level index `i`, use binary adjacent pair index `j={0,0,1}`.
At every aligned macro frame and separately for position/velocity compute:

```text
e       = RMS(C_i, B_i)
D       = RMS(B_j, B_(j+1))
D_floor = b4b_rms_floor(B_j, B_(j+1))
A_x     = 0.05*dx
A_v     = 0.001*c
```

Classify in fixed order:

1. `TEMPORAL_BUDGET` if `D>D_floor` and `e<=0.5*D`;
2. `ABSOLUTE_REPRESENTATION_BUDGET` if `e<=0.01*A`;
3. otherwise `REJECTED`.

Report both utilizations even when one branch is selected. Require every field
in every lane/frame to select a non-rejected branch. Do not rename either
branch convergence order.

## Unchanged gates

Require both canonical final fields to have observed positive convergence ratio
`[1.25,2.75]`; representation-floor-only convergence is insufficient for a
selected reference. Retain exact contact times, terminal contacts, private KKT
and capacities, canonical geometry, P1/P2 physical conditions, macro energy/
ledger roots, prepublication rollback and byte repeatability. The obsolete
`32*P*q` tube remains a reported diagnostic and is not an admission gate.

## Historical and decision boundary

Reproduce full B4C3PE exactly at parent hash. Two complete B4C3PE1 reports must
be byte-identical.

PASS selects `MACRO_BOUNDARY_CANONICAL_FIXED_REFERENCE_CANDIDATE` and
authorizes only adaptive macro-transaction design. FAIL preserves B4C3TAR2 as
the last positive trajectory boundary. B4C3TC, nominal, B4C4/B4D, CUDA,
runtime/schema and production remain blocked.
