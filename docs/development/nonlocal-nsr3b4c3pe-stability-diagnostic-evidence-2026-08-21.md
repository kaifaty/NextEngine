# NSR3-B4C3PE macro stability diagnostic evidence

Status: `PASS / MIXED_STABILITY_BUDGET_DESIGN_AUTHORIZED`

Date: `2026-08-21`

## Reproducible result

Full command:

```text
nonlocal-formula-reclosure --publication-stability-self-test
```

Two parent-gated reports are byte-identical:

```text
status                 PASS / MEASUREMENT_ONLY
raw JSON + LF          a2b55ae6e200bded7832d839bc1c892a5b7863e09af3e04747fb22199d70b986
raw JSON without LF    aebe7fbe218b507ae0ca8ebe7fde5ecafc38b5ec66fc894d669043649c51ea51
semantic result        584db48c73b11131db26c5a76758111bd47a7569d85599936fb007951d089395
wall time              43.03 s / 43.94 s
maximum RSS            12,960 KiB / 13,204 KiB
parent B4C3P exact      true
```

The isolated measurement also passes twice byte-identically:

```text
raw JSON + LF          75ead4cfaefc625f546c284302a845e9399ee6bfb1644546927df16e1f3801f0
raw JSON without LF    a9f550ac97e98b156ce3a40883b74496bf9d4855b95d295f08e0dbd894cad55c
semantic result        87aad8364706699cb048b4730822bf36015d35e9514fffc91e3659cae6461d6d
wall time              21.66 s / 22.00 s
```

## Exact decomposition

Every P1/P2, `48/96/192`, macro-frame position and velocity entry satisfies
the forward triangle closure:

```text
published error <= propagated error + direct publication error + fp bound.
```

All frame alignments, private/canonical roots, contact time, terminal contacts
and prepublication rollback remain exact. B4C3P remains FAIL; this report does
not change its gate.

## Gain-bound result

A universal local propagation-gain bound is rejected:

- P1 maximum resolved velocity gains are `106.80`, `81.14`, `75.19` for
  `48/96/192`, while position gains stay `<=1.162`;
- P2 velocity starts are unresolved in 47 of 48 level/frame measurements;
- the lone resolved P2/192 contact-frame ratio is `7.74e9`, caused by a
  denominator just above the binary64 floor rather than physical instability.

Active contact and near-zero velocity-start errors make a scalar Lipschitz
ratio numerically ill-conditioned. It is useful as a diagnostic, not an
acceptance gate.

## Fine-reference contamination

For P1 fine-192, resolved representation/temporal ratios reach `1.739` for
position at frame zero and `0.366` for velocity at frame three. At the blocking
final velocity frame, the ratio is only `0.137`. For P2, position reaches
`1.108` at frame zero; 15 velocity frames are correctly marked unresolved
because uniform free-flight velocity has no resolved `96/192` difference. The
contact frame resolves at `0.170`.

Absolute utilization of the existing physical comparison scales remains very
small:

| Case | Position / `0.05dx` | Velocity / `0.001c` |
|---|---:|---:|
| P1 maximum | `0.000419` | `0.002858` |
| P2 maximum | `0.000685` | `0.001628` |

Thus a purely relative temporal budget is invalid in early or exact-free-flight
frames, while a purely physical tolerance would be unnecessarily broad.

## Decision

Authorize design of B4C3PE1 with a mixed, independently scaled rule for the
fine reference:

```text
TEMPORAL branch  error <= 0.5 * resolved binary temporal difference
ABSOLUTE branch  error <= 0.01 * existing physical comparison scale
```

The `0.5` share ensures representation error does not dominate a resolved
temporal estimate; the `1%` share reserves only a small, pre-existing fraction
of the accuracy envelope when the temporal denominator is unresolved or below
material scale. These constants are frozen before B4C3PE1 execution and are
not fitted to the observed `1.10574` old-bound utilization.

B4C3P remains FAIL until a complete selected-policy replay passes. Adaptive,
B4C3TC, nominal, CUDA, runtime/schema and production stay blocked.
