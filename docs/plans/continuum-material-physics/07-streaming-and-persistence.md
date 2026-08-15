# 07 — Streaming and persistence

## Outcome

Define and prove a bounded lifecycle for modified continuum regions without
persisting reconstructible grids, neighbor structures or render meshes.

## Region states

Candidate lifecycle: `InactiveImmutable`, `Active`, `SleepCandidate`,
`SleepingModified`, `WakePending`, `Failed`. Transitions occur only at fixed
commit boundaries and are keyed by stable region/profile/generation identity.
Wall time, camera distance and memory pressure may propose work but cannot
select an authoritative transition.

`Active` persists stable material samples plus constitutive history and
conservation totals. `SleepingModified` uses a separately versioned bounded
representation only after an explicit conversion receipt. Private staging,
MPM grids, neighbor lists, solver matrices and GPU buffers are discarded.

## Conversion receipt

Each active↔sleep transition records source/destination hashes and error in
mass, water mass, linear/angular momentum, volume/surface displacement and
material-specific history. Threshold failure retains the source generation.
Repeated sleep/wake cycles are tested for accumulated drift; one passing round
trip is insufficient.

## Save, load and streaming behavior

- freeze all physical owners at one commit point;
- validate the complete new owner segment before publication;
- save/load/restart reconstruct caches and reproduce the same region root;
- dirty state is never evicted without a valid durable representation;
- corrupt/incompatible data leaves the prior world/save generation intact;
- mandatory wake failure blocks activation or uses an explicitly authored
  immutable fallback before world mutation.

Until conversion passes, the only safe options are to pin the modified region
active, serialize exact bounded active state, or disable persistent
deformation. Reconstructing changed terrain from base content is forbidden.

## Exit

`CONTINUUM-PERSISTENCE-P1` runs modify→save→unload→restart→wake plus corrupt,
stale, capacity and repeated-cycle cases in `game` and deterministic
`headless`. It requires owner-root parity and all declared conversion bounds.
