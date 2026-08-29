# NSR3-B4EP10SIRD source-local discriminator evidence -- 2026-08-22

Status: `PASS / DIRECTED_STRUCTURAL_AUDIT_SELECTED`

## Result

Three fresh serialized processes reproduce B4EP10SIR semantic result
`62a2205d...10aa3` and B4EP10SII result `f7b1542f...25fb2`. In every process
all four groups are positive and sum exactly to reported `source_local_ns`.

| Group | Run 1 | Run 2 | Run 3 | Median | Range |
|---|---:|---:|---:|---:|---:|
| directed | 0.388626 | 0.389496 | 0.389073 | 0.389073 | 0.000870 |
| setup | 0.093845 | 0.095028 | 0.095676 | 0.095028 | 0.001831 |
| compression | 0.057199 | 0.056722 | 0.056670 | 0.056722 | 0.000529 |
| local scalar | 0.047631 | 0.047075 | 0.046904 | 0.047075 | 0.000727 |

All ranges pass the frozen `0.05` gate. Directed work owns 38.91% of the
transaction and leads setup, the second group, by `4.094282x`. It therefore
selects one directed structural audit.

## Interpretation

The timer combines arithmetic with allocation/value-initialization of full
directed arrays. It cannot show which part dominates. The selected next audit
must prove write-before-read and ownership for reusable scratch storage and
count projected initialization/allocation work. It may not implement scratch
reuse, skip initialization or fuse HVP passes.

The result grants no performance credit and does not alter the external
B4EP10SII A/B decision.

## Decision

Close B4EP10SIRD as PASS. Research and freeze one exact, timing-free directed
scratch-liveness audit. B4E2, broad corpus, runtime, CUDA/GPU, schema and
production remain blocked.
