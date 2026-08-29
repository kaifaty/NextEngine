# NSR3-B4EP8 fused residual-attribution research -- 2026-08-22

Status: `COMPLETE / PASS / SCOPED_PHASE_TIMING_SELECTED`

## Question

After B4EP7I fuses evaluation and tape construction, is either HVP or complete
fused workspace still a sufficiently dominant serial category to justify one
more mechanical optimization?

## Hypotheses

1. **HVP leads again.** Fusion reduces workspace enough that the unchanged
   pressure-tape apply routine becomes dominant.
2. **Fused workspace still leads.** Topology filtering plus the combined
   pair/center passes remain larger than all 459 HVPs.
3. **The residual is balanced.** HVP and workspace are close; sampled
   attribution cannot select another serial rewrite responsibly.
4. **Control/publication becomes material.** Once data passes shrink, the 221
   trials and canonical evidence pipeline may no longer be negligible.

B4EP6 numbers cannot be subtracted: fusion changes function boundaries,
memory access and gprof attribution.

## Measurement

Use one clean external GCC 15.2 `-O3 -DNDEBUG -g -pg
-ffp-contract=off -fno-fast-math` build of implementation
`a3aa054217cfd9d21193f8943effe10b7ebce7bf`. Run exactly once:

```text
nonlocal-formula-reclosure --nominal-hydro-fused-evaluation-tape-ablation
```

Admit only byte-exact B4EP7I stdout. Attribute top-level inclusive HVP,
complete fused workspace and residual control. Split workspace into cached
topology/filter/CSR, fused pair work and fused center work when caller/child
symbols permit it.

## Routing

Use the frozen `1.20x` leader/runner-up rule at top level and, if workspace
wins, inside workspace. No leader routes to scoped internal phase timing, not
an implementation. Parallelism, GPU, solver-policy, B4E2, runtime and
production remain blocked by this profile.

## Decision

Freeze one exact-output B4EP8 profile. It may authorize one B4EP9 design only.

The profile finds no `1.20x` leader and selects phase timing; see the
[dated evidence](nonlocal-nsr3b4ep8-fused-residual-attribution-evidence-2026-08-22.md).
