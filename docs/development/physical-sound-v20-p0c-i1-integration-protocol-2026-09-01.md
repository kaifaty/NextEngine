# Physical sound V20 P0c — frozen I1 integration protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN_BEFORE_I1_IMPLEMENTATION / I1_METADATA_ONLY / VALUES_SEALED` |
| Prerequisite | [M0b repeat-exact pass](physical-sound-v20-m0b-confound-resistant-metric-result-2026-09-01.md), SHA-256 `465e806a9052f4f86ceeb07407a42d09703272f4d36054fb5a164b3526d663c9` |
| Roadmap | [V20](../plans/physical-sound-synthesis-roadmap-v20.md) |
| Product effect | None; even an I1 pass is synthetic capability evidence and cannot admit clips, real acoustics, demo or runtime ML |

## Question

Do the frozen B0 global scaffold, C0 coverage policy and F0 signed contact field
compose on fresh objects into a complete phase-consistent modal endpoint that
passes the independently calibrated M0b instrument twice exactly?

P0c freezes identities, formulas, thresholds, aggregations, mutations,
fallback and access before I1 code may construct a mesh or generate truth,
prediction, metric or waveform values.

## Immutable dependencies

| Dependency | Identity |
| --- | --- |
| M0b implementation commit | `cfee8cb844b945d4ba85ef3fe768947ee563e8ed` |
| M0b common / metric / test | `8f5ad986847f971e32e1c7e814cfb7c162143ad2a869824b6dc5cc5fb624fcce` / `b74208de60fa1889cd035b177919244b5a50e7d7416eaa1eaf7b347d232c4bcb` / `16021bbbdfee2d0b099fd1015fb472a487a39be92b0d54ce269b9a0e45982a62` |
| M0b complete tree | `7eb5c75edcff8bd67fef592af30aa9fa90ebc73a25aa2629cba43935e23bab22` |
| V18 B0 complete tree | `bffd8bf5671fcb066f50b90774f26fedaf8087f82fa740586c6fb92884291111` |
| V19 F0 complete tree | `7918d8b4fd29f5aabb201b1dcc51d85fce9df44463ecb72ebd4eceb42d608718` |
| I1 metadata row root | `f1709914a56a82482f1c79133453c13da1de6d36aac25c1f5c9db21fb654b7b8` |

B0, C0, F0, their normalization, model members and M0b metric code are read-only
dependencies. I1 performs no fit, optimizer step, checkpoint selection,
threshold selection or case selection from generated values.

## Fresh I1 metadata

I1 contains twelve physical groups and a primary/remesh twin for each, yielding
24 views. For cell `c=0…11`:

```text
n          = 1701 + c
material   = (Steel, Wood, Glass)[floor(c/4)]
topology   = (Plate, Cylinder, Bowl, RolledSheet)[c mod 4]
support    = (Free, BaseClamped)[(c+1) mod 2]
length_m   = 0.21  + 0.27  * radical_inverse(n, 2)
aspect     = 0.72  + 0.76  * radical_inverse(n, 3)
slenderness= 0.004 + 0.004 * radical_inverse(n, 5)
wall_m     = length_m * slenderness
grid       = (23 + c mod 2, 18 + c mod 3)
group      = "v20-i1-integration-{material}-{topology}-{support}-{n}" lowercase
role       = "integration"
```

The primary object uses `grid`; its twin uses `(grid_u+1, grid_v-1)`. Object
suffixes are `-primary` and `-twin`. Canonical JSON of the 24 ordered
`FieldRow.record()` values must equal the frozen row root above before any mesh
is constructed. The identifiers are disjoint from all `1301…1601` opened bands
and optional `1801…2101` remains sealed.

## Evaluation surface

For every view, use the production C0 construction and F0 inference path:

1. build the deterministic mesh and composite coverage analysis;
2. choose the required FPS context without signal or candidate values;
3. evaluate every non-context query through C0;
4. predict F0 gains for the complete vertex field and evaluate all accepted
   queries, remesh probes, numerical corruptions and structural corruptions;
5. compute B0 frequencies/damping and unchanged physical metrics;
6. assign an authored-clip fallback record to every C0-rejected query;
7. select exactly eight accepted acoustic queries using ordered positions
   `floor(i*(count-1)/7), i=0…7`; duplicate positions or fewer than eight
   accepted queries reject before rendering;
8. render 16 kHz, 8,192-sample, eight-mode truth and combined candidate with
   common truth-RMS normalization and the frozen M0b metric code.

This yields exactly 192 candidate waveform cases. No listener/radiation,
arbitrary-force response, residual waveform decoder or room response enters I1.

## Frozen integration thresholds

Each family midpoint is the exact arithmetic mean of M0b's corpus-p95 maximum
acceptable and minimum harmful value:

| Family / metric | Midpoint |
| --- | ---: |
| frequency uniform / MRSC | `0.6872122563380927` |
| frequency alternating / MRSC | `0.6860209112961239` |
| mode removal / MRSC | `0.7600201884531498` |
| frequency uniform / MCLM | `0.4829917447253579` |
| frequency alternating / MCLM | `0.49325696833283994` |
| mode removal / MCLM | `0.46162132517706367` |
| positive damping / DSR | `0.15316169107246214` |
| negative damping / DSR | `0.14714357326302258` |
| onset / TE | `0.09215386292722896` |
| impulse / TE | `0.15534609756925377` |

The actual combined I1 candidate must satisfy the strictest applicable
corpus-p95 threshold:

```text
MRSC <= 0.6860209112961239
MCLM <= 0.46162132517706367
DSR  <= 0.14714357326302258
TE   <= 0.09215386292722896
```

Raw MRLM, absolute DE, early/full waveform and Hilbert envelope remain
diagnostic with no threshold. Corpus median and p95 are reported; only p95 is
blocking. No per-material threshold is selected from I1.

## Unchanged hard and physical gates

- exact dependency, protocol, row, mesh, context, ordering, finite/range and
  serialization identity;
- B0 frequency median/p95 `<=20/60 cents`, maximum `<=60 cents`; damping
  median/p95 `<=0.08/0.20`;
- every F0 absolute, gradient, control-ratio, paired-win, topology, remesh,
  mutation, structural and coverage gate unchanged;
- negative damping, unordered frequency, missing fallback and dependency hash
  reject `12/12` primary groups;
- fallback set equality covers every rejected query exactly once;
- every endpoint is eight ordered modes with finite signed gains and a declared
  coverage/fallback decision.

## Fresh counterfactual non-regression

In addition to the actual candidate, I1 renders the named harmful controls on
the same 192 cases without selecting thresholds:

- uniform `+90 cents` and alternating `-90/+90 cents` reject physical frequency
  and at least one frozen spectral threshold;
- damping `*1.35` and `*0.65` reject physical damping and DSR;
- remove two largest-gain modes and alternate modal signs reject their physical
  owner and at least one spectral threshold;
- delay 64 samples and add a sample-zero `8*truth_peak` impulse reject TE;
- global polarity is acoustic-invariant but rejects signed gain.

Every identity acoustic metric is `<=1e-12`. These mutations are fixed
non-regression checks, not a new calibration surface.

## Run gates and stop rules

One I1 run passes only when:

1. exact dependencies and 24-row metadata root validate before mesh creation;
2. 24 unique views, 12 primary/twin groups and 192 selected cases are complete;
3. every unchanged B0/C0/F0/hard/physical/fallback gate passes;
4. actual candidate corpus p95 passes all four strict thresholds;
5. every named counterfactual rejects through its assigned owner and identity/
   polarity invariants pass;
6. all scalars are finite and endpoint/geometry/metric serialization is exact;
7. zero real, force, dataset, source, protected, method-holdout, shadow,
   optional-successor or network access;
8. runtime is below ten minutes, RSS below 4 GiB and output below 100 MiB;
9. two independent executions emit the same nine files and bytes.

An I1 reject freezes all values and requires causal attribution before any G1
or F1 successor protocol. It cannot tune a threshold, rerun a seed, choose a
favorable contact or open real data. An I1 pass opens only the generator-side
real-evidence prerequisite when S1/M1 independently becomes ready; it does not
waive the source-role or independent-validator gates.

## Output boundary

Implementation and focused tests must be committed before the first I1 mesh or
value is generated. Each run emits externally only:

- `manifest.json`, `corpus.json`, `report.json`, `access-ledger.json`;
- `metrics.jsonl`, `decisions.jsonl`, `fallback.json`;
- deterministic `geometry.npz` and `endpoints.npz`.

No dataset, WAV, checkpoint, protected value or generated artifact enters Git.
Existing authored clips remain the sole product fallback.

