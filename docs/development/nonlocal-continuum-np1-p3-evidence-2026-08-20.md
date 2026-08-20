# Nonlocal continuum NP1-P3 evidence — 2026-08-20

Status: `P3_NOT_RETAINED / P2_ROLLBACK / P4_NEXT / REPORT_ONLY`

## Decision

Do not add `dynamic-cell-local-p3` to the universal retained stack. The
implementation is exact and strongly validates the locality hypothesis: the
deliberately permuted 50k profile improves `1.779x` including sort/remap.
However, coupled advected controls regress beyond the frozen `2%` limit:
viscous by `2.11%` and surface by `3.15%`. P3 therefore exits through its
specified rollback and P4 uses retained P2.

The selectable P3 laboratory path remains valuable evidence for a future
explicit disorder classifier or caller-declared unordered import. This stage
does not authorize such a policy: paying for a full sort and then falling back
would not meet the current control gate, while choosing by benchmark profile
identity would not be an architectural solution.

## Artifact identity

- branch: `codex/nonlocal-continuum-n0`
- executable SHA-256:
  `eb9e70e9d6e549d2fab79ff8904ef4c39645e7809f3a77ea4763635f5fbae590`
- denominator: retained P1+P2 stable material/sample storage
- candidate: identical solver with per-solve packed-cell radix remap
- CUDA: `sm_86`, 256 threads, strict retained f32 compiler semantics
- tournament: 256 conditioning, 32 warm-up, 96 alternating measured rounds

## Implemented identity and correctness

Every candidate solve performs one timed key/radix-sort/cell-range pass,
constructs inverse storage maps, gathers immutable SoA state, builds compact
physical CSR, executes exact P1 arithmetic and scatters advected position and
velocity back to stable material-ID order. The sort is not repeated during
neighbor construction.

All correctness gates passed:

- retained P2, compact tiny CPU `f64` and P3 tiny CPU `f64` preflights;
- stiff `gamma=1000`, i2 exact output and logical CSR;
- coherent/permuted seed exact output and logical CSR;
- every output, logical CSR and canonical handoff over all three 32-step
  advected traces;
- finite/symmetric/capacity/momentum checks;
- bounded inverse map and deterministic map hashes at every captured step;
- exact added allocation of `33N + 4` bytes (`1,650,004` at 50k and `528,004`
  at 16k), with no shadow CSR or per-edge remap.

The stiff identity remains:

- output: `52a3d852c05b9cc7931a3133816e0ddb1445695b88ea25193d0981022080b65e`
- logical CSR: `a0304020eeb98889b47c0e179c85aaf4f93fb671bf58f15a99d3e034f1ad4698`

## Adjacent tournament

Times are same-process p95 milliseconds. Speedup is retained P2/P3, and P3
p99 includes the complete remap/scatter cost.

| Profile | P2 total | P3 total | Speedup | P3 remap | Pair speedup | P3 p99 | Gate |
|---|---:|---:|---:|---:|---:|---:|---|
| water 50k coherent control | 3.2259 | 3.2858 | 0.9817x | 0.2164 | 0.9005x | 3.2958 | diagnostic; stable selected |
| water 50k permuted | 6.2559 | 3.5161 | 1.7792x | 0.2291 | 1.1823x | 3.6891 | PASS |
| water 50k advected | 3.2038 | 3.1075 | 1.0310x | 0.2220 | 0.9122x | 3.4241 | PASS non-regression; stable selected |
| viscous 16k advected | 5.6383 | 5.7571 | 0.9794x | 0.3718 | 0.9683x | 5.9488 | FAIL (`2.11%`) |
| surface 16k advected | 7.2329 | 7.4610 | 0.9694x | 0.1887 | 0.9560x | 7.7038 | FAIL (`3.15%`) |

The permuted result confirms HP-1 and demonstrates that compact physical IDs
plus cell-local SoA can overcome the complete remap cost. The coherent and
coupled results also reproduce the earlier O4 boundary: sorting data that is
already acceptably local adds indirection and remap work, and may worsen pair
tails even when total happens to remain close.

## Raw report hashes

| Report | SHA-256 |
|---|---|
| coherent tournament | `2701f947cbc471daf85e803995eb4fcc7b7920612b28ca081932d268b2b1dc6a` |
| permuted tournament | `6d86e47cbab71d636057bae7ea39e27e99f3a736d7057690221f7011d7088a2f` |
| advected water tournament | `25af5d305e633867ae1181cf16d7caabf454a95cbe739c1829e7fcfe60ee9bd0` |
| advected viscous tournament | `d583cd073564a01711a7f1eb0851dbe1233de13b4be94d81034f41b9f15d920a` |
| advected surface tournament | `2e51481b065e45fe69dfc2976b4e141718c23541543375a4988c0e269af3c467` |
| stiff check | `31e06a08bb1db3b70de8c3e4adc64988456280b0e0ccf89076ddde29ccfd14cc` |

Raw JSON and binaries remain outside Git under `/tmp`.

## Consequence

P4 starts from retained P2, not P3. No generic locality candidate remains in
the fixed-work stack. A future P3 revival requires a bounded pre-sort disorder
classifier whose own cost is timed and whose false-positive policy passes the
coupled controls; these reports are its positive and negative corpus.
