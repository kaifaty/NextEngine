# NCGP8 visible-surface control corrigendum

| Field | Value |
| --- | --- |
| Research ID | `NCGP8` revision 2 |
| Status | `FROZEN / DIAGNOSTIC_ONLY / REPORT_ONLY` |
| Supersedes | Only mandatory control 6 in NCGP8 revision 1 |

## Reason

Revision 1 requires the accepted depth p99 to be at most one presentation
sphere radius, `25 mm`, while its top-sheet negative control also shifts the
sheet by exactly `25 mm`. Equality is admitted by the frozen gate, so that
control is not guaranteed to reject.

## Corrected control

Retain the sparse one-pixel tail control unchanged. Replace only the complete
top-sheet mutation by:

```text
complete top-sheet vertical translation = +0.05 m = 2 * sphere radius
expected first semantic rejection        = depth p95/p99 distribution
```

The fixture remains inside the basin. All NCGP8 revision-1 physics, observer,
thresholds, work, evidence, stop and successor rules remain unchanged.
