# PS-2 existing-Iron broad-band counterfactual preregistration — 2026-08-28

## Outcome

Freeze the first read-only real counterfactual for the synthetic-supported
broad-band Gabor/common-pole candidate. It uses only the already acquired
`17_IronSkillet` impact-zero rows `0..14` and may decide whether the
mathematical method transfers to this opened development recording.

There is no known real modal truth. Therefore this experiment does not compare
against invented “correct Iron frequencies” and cannot establish material
identity or perceptual quality. It instead requires scale invariance, spatial
replication across microphone partitions, bounded reconstruction and future-time
advantage over an otherwise identical undamped ablation.

## Frozen lineage and observation

- synthetic parent report:
  `dcd832525fbd1955c9122fc52579e4a8ddd10ab76bc5d79393f8f9df9d270094`;
- frozen candidate runner:
  `317fa3a2e4f05f6c212ccdbee0575554d3610753573db359a68f3e756a73ec8d`;
- Iron decode report/block:
  `47acdc35449390f639cda34990520c5f74cb8627d0932079345ca35735ffc099` /
  `e26d1df1d547d059bbcbc57453c522b8c23402461feec09ff6149bc37a9c7eeb`;
- selector/tail parent:
  `02551f2995764197575d5dd30bb36bd44da5452e2af79533b43922f44fec48d1`;
- rows `0..14`, onset sample `29`, 48 kHz, no normalization;
- exactly the first 60,000 post-onset samples are the bounded observation span.

The 60,000-sample span reuses the supported synthetic candidate’s observation
length; it is not chosen from an Iron tail. The input slice is read from the
existing decoded block only. No network request, acquisition retry, prefix
growth, new object, physics or Planter access is permitted.

## Frozen candidate and controls

The candidate is unchanged:

1. 1024-sample Blackman-Harris Gabor transform, 32-sample hop, first 256
   frames;
2. local input-energy regions over 500–12,000 Hz at `-30 dB`, then neighboring
   ±1 bins;
3. common-pole order selection with minimum score margin `50×`;
4. single-link duplicate clustering within `1 Hz`;
5. joint 15-output cosine/sine amplitude fit;
6. mode-energy pruning at `-25 dB` only after estimation;
7. reconstruction over the frozen 60,000-sample observation.

Operational capacity is frozen at no more than 16 discovered regions, 48 analysis
bins and 64 pre-prune clusters. Exceeding a cap is a method-transfer rejection,
not permission to rank or discard Iron regions.

Three controls provide falsifiable evidence without truth labels:

- discovery bins must be exact after scales `0.125/1/8`;
- every retained full-output mode is injectively matched within `40 cents`
  against estimates from even microphones and odd microphones separately;
- the same frequencies are fitted with every decay set to zero. Damped and
  undamped amplitudes are separately calibrated on samples `0..8191`, then
  compared on untouched windows `8192..16383` and `16384..32767`.

## Frozen gates and decision

`ExistingIronBroadbandCounterfactualSupported` requires all gates:

- 1..16 regions and at most 48 analysis bins;
- at least one adjacent-bin duplicate removed;
- 6..64 pre-prune clusters and 6..32 retained modes;
- exact scale-invariant discovery;
- at least `0.50` of retained modes matched in each spatial partition;
- full-observation candidate NRMSE at most `0.95`;
- full candidate/undamped NRMSE ratio at most `0.95`;
- damped/undamped squared-error ratio at most `0.95` in each future window.

Any failed gate decides `ExistingIronBroadbandCounterfactualRejected`.
Thresholds, partitions, time windows, capacity and candidate parameters MUST NOT
change after preflight. Runs A/B must be byte-identical.

Support grants only existing-Iron mathematical method-transfer evidence and
allows preregistration of an independent internet-sourced real holdout. It does
not admit a material/object domain, authorize Planter or mechanics, judge sound
quality, replace authored clips, or promote runtime code. Rejection preserves
the block and requires research on the failed gate without tuning Iron.

## Hash-closed preflight

Runner SHA-256:
`63d46bf476798e540cfc726a0e2bca58beaacc0f829ffff7ecc2b1083efb2e76`.

External manifest SHA-256:
`8a445a4f0474ec560e659f6bf8ada9a90f4fb04fd5c04ef3e308843fcb43ff18`.
Preflight A/B are byte-identical at
`04e54415a3cf140364d27d541ccca4947039cde5608a668a3077d1452aefd357`
and decide `ExistingIronBroadbandCounterfactualFrozen`. Each binds the synthetic
parent, Iron decode/block and selector/tail diagnostic, and records zero
additional payload, network, physics and Planter access plus no quality,
admission or runtime credit.

The first preflight attempt stopped before analysis because the diagnostic
block hash was checked at the wrong JSON nesting level. The lineage-only repair
changed that lookup to `parents.decoded_block_sha256`; it changed no candidate,
gate, threshold, observation slice or numeric path. No report or numeric result
was produced by the failed attempt.

The manifest and preflight reports remain external under
`~/.codex/experiments/nextengine/physical-sound/ps2-realimpact-iron-skillet-broadband-counterfactual-v1/`.
This document and the runner MUST be committed before either numeric analysis.
