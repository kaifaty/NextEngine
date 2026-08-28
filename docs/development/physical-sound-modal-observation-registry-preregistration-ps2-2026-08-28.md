# PS-2 modal-observation registry V0 preregistration — 2026-08-28

## Decision

`ModalObservationRegistryV0InputsFrozen`.

Two byte-identical preflights freeze a deterministic report-only builder over
the exact Iron Skillet development report and the independently frozen Iron
Mortar holdout report. The preflight hashes 1,620,459 JSON bytes without
parsing their payloads and reads no waveform, network, physics or Planter data.

This authorizes one repeated registry build. It does not authorize a public
schema, a runtime consumer, perceptual quality, material/domain admission or
replacement of the authored-clip fallback.

## Frozen lineage

| Artifact | SHA-256 / decision |
| --- | --- |
| Builder runner | `a958c7955f5194046398c02d8cc1bd308e9acae0fb4f6ee5455c7327c07c064b` |
| Manifest | `0f473f4d262fcc5c5d97cb0c0088a2f076e04a8ea56847ef62ad6b6636505c20` |
| Preflight A/B | `fbffbe1e5572c72c56a97d90a3157dbe85a0c3210b2985ee2f996ddcd16441c0` / `ModalObservationRegistryV0InputsFrozen` |
| Skillet source report | `f2fb359faaccb36544203780f07cc1d359c5d529be7f7f0d533ae39bd50b6a73` / 924,196 bytes |
| Mortar source report | `64bc63aa0cd02cfe9ad3959670102e883893f7d53a1427fddb5f660a8951d0cc` / 696,263 bytes |

External artifacts are under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-modal-observation-registry-v0`.

## Frozen record boundary

Each `ModalObservationV0` entry must preserve:

- source report, producer runner and producer manifest hashes;
- object, impact and ordered source-listener output identity;
- input/Gabor/decoded identities available in the source report;
- discovery regions and bins, raw/duplicate/pre-prune/retained/pruned counts;
- every fitted cluster, representative frequency and decay, source members,
  energy and 15 complex spatial amplitudes, split by retained/pruned state;
- even/odd spatial replication, scale invariance and the complete method gate;
- full and future-window absolute residuals plus the undamped ablation;
- explicit method, quality, domain, runtime and fallback states.

Physical listener coordinates are not present in both source reports. The V0
record therefore identifies listeners only as ordered source output indices
`0..14`; it must not invent positions or microphone semantics.

## Admission and fallback

The registry freezes these states for both seed entries:

| Axis | Frozen state |
| --- | --- |
| Method | `supported_by_frozen_source_gate` |
| Quality | `disabled_no_validated_absolute_residual_threshold` |
| Exact domain | `disabled_no_exact_domain_evidence` |
| Runtime | `disabled_proposed_report_only_experiment` |
| Fallback | `authored_clip_required` |

The builder must reject changed source identity, failed method gates, altered
counts, malformed modal/listener shapes, missing residuals or any source that
gained quality/runtime credit. It may not derive an admission threshold from
these two opened seed reports.

## Next action

Run the frozen builder twice and require byte-identical `registry.json` and
`report.json`. If it succeeds, extend the same report-only record to a frozen
object-grouped batch manifest and evaluate explicit transient/residual model
candidates without per-object human admission. If it fails, keep both source
reports immutable and repair only the deterministic adapter or reject V0.
