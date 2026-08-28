# PS-2 Iron Mortar broad-band holdout protocol preregistration — 2026-08-28

## Outcome

Freeze one synchronized 15-microphone observation group from official
REALIMPACT `43_IronMortar`, impact ordinal zero, before waveform access. The
candidate is the complete Iron V2 rank-7 partial-SVD common-pole method with
unchanged fit, pruning, spatial, damping and predictive controls.

This is an independent method-transfer holdout. It does not test perceptual
quality, recover physical material parameters or admit a metal domain.

## Frozen access

Discovery report `84308e8e…d080` and repeated audit `67c92621…8a42` prove an
exact `3000×208375` f32 observation without payload access. Future execution may
make exactly four requests once:

1. bytes `157..547` for `distance.npy`;
2. bytes `3044594..3045171` for adjacent `micID.npy` and `vertexID.npy`;
3. bytes `2305328734..2305329119` for `angle.npy`;
4. bytes `3046013..36600444`, exactly 32 MiB of the observation deflate stream.

The metadata total is 1,355 bytes. The audio prefix may decode only the first 15
full waveform rows, exactly 12,502,500 f32 bytes after the NPY header. Global
archive compression indicates that 32 MiB should be sufficient, but sufficiency
is not a gate that may be repaired: a short decode rejects the holdout without
prefix growth or retry.

## Frozen identity and analysis

Metadata must prove that rows 0…599 share one vertex, contain the expected 40
angle/distance groups and order microphones 0…14 inside each group. Analysis
uses rows 0…14 only. Onset is the first spatial-norm sample at or above 5% of its
maximum and must leave 60,000 post-onset samples.

The V2 algorithm remains unchanged except that Iron-specific exact counts
31 regions/91 bins become bounded holdout gates:

- 1…32 discovered regions and at most 96 analyzed bins;
- at least one duplicate removed;
- 6…64 pre-prune clusters and 6…64 retained modes;
- exact discovery under scales `0.125/1/8`;
- even/odd microphone match fractions at least `0.50` within 40 cents;
- full NRMSE at most `0.95` and damped/undamped ratio at most `0.95`;
- predictive damped/undamped SSE ratio at most `0.95` in both unchanged future
  windows.

All discovered regions are analyzed. Top-K selection is forbidden. Thresholds,
windowing, partial-SVD rank, duplicate clustering, −25 dB energy pruning and
amplitude/prediction fits cannot change.

## Stop rule

The protocol runner, manifest and repeated zero-access preflight MUST be
committed before execution-runner implementation. The execution runner and its
own repeated zero-access preflight must then be committed before acquisition.

Acquisition, decode, row identity, capacity, scale, spatial or predictive
failure publishes rejection and retains authored-clip fallback. No retry,
prefix growth, object substitution, new physics or Planter access is allowed.

## Frozen evidence identity

Runner, manifest and preflight hashes are filled after the protocol is committed
and its zero-access preflight repeats. External artifacts remain under
`~/.codex/experiments/nextengine/physical-sound/ps2-realimpact-iron-mortar-broadband-holdout-v1/`.
