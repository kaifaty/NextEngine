# NSR3-B1R1 -- fine-state ownership contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / BOUNDARY_EXECUTION_BLOCKED`

Parent failure: `COARSE_STATE_COMPOSITION_REJECTED`, B1R semantic SHA-256
`377941cd110c07c265b882d8025d40daf0d22383847649c96ddd771a528346db`.

Objective/solver identity remains `nuv-variational-fcr2` /
`nuv-newton-krylov-r0 + outer-state-hessian-tape-v1`.

## Single isolated change

Retain B1R's macro frames, spectra, initial counts, local gates, maximum
refinement depth, cases, degenerate controls, work caps and independent
`96/192/384` reference ladders exactly.

For the first passing `(coarse,fine)` pair, commit the **fine** state instead
of the coarse state. The fine trajectory was already executed to evaluate the
gate, so this changes state ownership but adds no solver work:

```text
n versus 2n passes      -> commit 2n; discard n
2n versus 4n passes     -> commit 4n; discard n + 2n
4n versus 8n passes     -> commit 8n; discard n + 2n + 4n.
```

`accepted_substeps` and accepted nonlinear HVP calls belong to the committed
fine trajectory. `error_probe_substeps` identify the coarse member of the
passing pair. `discarded_work = total_executed - accepted_work`, including
all failed/coarser probes. Do not call the committed fine trajectory a
discarded comparator.

The theoretical execution/accepted-substep multipliers are therefore `1.5x`,
`1.75x` and `1.875x` at refinement depths zero, one and two. These ratios are
reported, not used as accuracy evidence.

## Correspondence and gates

- Reproduce all nine B1R fixed-reference phase hashes, convergence ratios and
  work records exactly.
- Preserve the first-frame B1R spectral estimates, initial counts and local
  pair observables exactly before ownership changes the next macro state.
- Run `0.99dx`, `0.98dx` and `0.97dx` for the same 12 frames and compare the
  composed fine-owned state with fixed 384 exactly as in B1R.
- Position remains `<=0.05dx`, velocity `<=0.001c`, relative kinetic error
  `<=0.15`; conservation, density, pressure-exit, transaction, capacity and
  `32/8/128` per-step work gates are unchanged.
- Accepted trajectories remain capped at 192 substeps per frame, every
  executed trajectory at 384, and total speculative execution at 768.
- Free flight and rigid translation retain 12 inactive fast-path frames with
  zero spectrum, probe and nonlinear-HVP work.

Two reports must be byte-identical. B1R and all earlier raw reports remain
byte-identical.

## Exit

PASS selects `NSR_MULTISTEP_CANDIDATE` and authorizes design only of the
NSR3-B2 static-boundary formula. FAIL keeps B2 blocked and authorizes a new
discriminator for horizon-aware local error allocation or a higher-order
accepted state; it does not authorize loosening the global gates or caps.

No boundary execution, hydrostatic, CUDA, runtime, public-schema, save/replay
or production authority is granted.
