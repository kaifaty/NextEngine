# V28 R2 result — official M0c feasibility

| Field | Result |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / A_CANONICAL / B_CANONICAL / REPEAT_EXACT / REPRESENTATION_REJECT / HOLDOUT_NOT_OPENED / DISCLOSED_REAL_NOT_OPENED / GENERATOR_BRANCH_CLOSED` |
| Protocol | [V28 R0](physical-sound-v28-r0-m0c-prefix-resource-protocol-2026-09-02.md) `28cab4283e9ea0178b59100960245d27e04c5e749842dd87fa011ec788ca9000` |
| R1 predecessor | [M0c owner/resource pass](physical-sound-v28-r1-m0c-owner-resource-result-2026-09-02.md) |
| Implementation | commit `d316982dc92b1a2c6e86d33928a913155a42015e`, root `524411d3c299420cfebebf63b2d66b1c444847f60f42b67218446203825f6d32` |
| Official manifest | `8d6bef9d4bf1d7eaeabbc1be815fc29f5b505e9794fa064fdf65649d56cf0cb0` |
| Terminal decision | `REPRESENTATION_REJECT` |
| Allowed claim | The compact causal student reproducibly improves several synthetic errors but fails frozen decay, remesh and physical-conditioning requirements |
| Forbidden claim | Naturalness, real material quality, Metal/Glass admission, runtime authority or evidence that a waveform/codec model would pass |

## Outcome

Official M0c A and B both complete inside the resource envelope and publish the
same six canonical artifacts byte-for-byte. Both return
`REPRESENTATION_REJECT` before candidate freeze. The reject is therefore a
reproducible model/representation result, not a timeout, preprocessing failure
or random execution event.

The model learns useful correlations: frequency, contact gain and short-window
spectrum all beat the frozen ridge control by the declared margin. It does not
learn the required causal structure: decay is worse than ridge, zeroing
geometry barely changes frequency error, remesh consistency misses by three
orders of magnitude, and material/scale counterfactual ratios stay near one
instead of following known physics.

M0c is closed. Its opened official values may explain the failure but cannot
select another seed, capacity, loss weight, horizon, threshold, contact subset
or nearby checkpoint.

## Frozen inputs and access

The fresh M0c manifest binds:

| Artifact | SHA-256 |
| --- | --- |
| Combined manifest | `c43ba8eac68e32a8ef8fbe37d3ffa3d21db1d59e0443766cee1d98f51cad70bb` |
| T0 evidence | `884da56ff9005e9dd63e11ec74bafa8796b187b15ed6099fbca99dda7617e7ed` |
| X0 lineage | `e4f6bb117a09f77b2bf8602ec895c95ae63262bc9d42db1233d972d73242534f` |
| `ffmpeg` | `bdf6aabffdba7411edff8d36c389d695257fcdf823d196020176e117612862f6` |
| `ffprobe` | `a6dac1e9e8631e04075d06854cc4069308aec279bb04683b4a0bca5e4f54b2fe` |

The run used a new external root and did not reuse M0b manifest, output,
MLflow, staging, weights or metrics. Admission shadow and sealed RealImpact row
`2407` stayed inaccessible.

## Execution and repeat evidence

Both runs used one thread, `MemoryMax=4G`, `MemorySwapMax=0`,
`IPAddressDeny=any` and a `900 s` outer timeout.

| Metric | Run A | Run B |
| --- | ---: | ---: |
| Exit | `0` | `0` |
| Terminal decision | `REPRESENTATION_REJECT` | same |
| Final step per trained variant | `2,000` | same |
| Parameter count | `23,142` | same |
| Internal candidate A/B exact | `true` | `true` |
| `/usr/bin/time` elapsed | `173.25 s` | `164.09 s` |
| `/usr/bin/time` max RSS | `1,146,208 KiB` | `1,132,708 KiB` |
| Swap / socket messages | `0 / 0` | `0 / 0` |

Every canonical file matches:

| Artifact | SHA-256 |
| --- | --- |
| `candidate-predictions.bin` | `cecd5c3a9706f29c70c5d013ea3c2889ffab03eef26adcc445cea8b3b359c02f` |
| `candidate-weights.bin` | `815ad27db638828f4ec52677203a1d4dd9c02883151af29a831c468b24d87a5a` |
| `control-report.json` | `4bb747cb3c9599143f4269b818d473281b1c983479032675a456d65c4eae1363` |
| `preprocess-manifest.json` | `9a58a9812b08744bb89e0ed08aa39833d0c5371acec2a8d657a4ab6d48096d13` |
| `report.json` | `32a20e83123db28e93f078d2f2ea34a696aedae3a9282b87bbf0d25536db2454` |
| `surface-query-report.json` | `da7996806e6272236a04236450e4aefcfb766c00382579dbacaa6c3d073c941a` |

Evidence root:
`/tmp/nextengine-v28-r2-official-PUouY9`.

## Frozen synthetic gates

The official gate is a conjunction. Passing individual metrics cannot rescue
another failed mandatory condition.

### Candidate versus ridge

Lower error is better; required ratio is `<=0.90`.

| Metric | Neural | Ridge | Ratio | Result |
| --- | ---: | ---: | ---: | --- |
| Frequency | `0.475594` | `1.794846` | `0.264977` | `PASS` |
| Decay | `0.449470` | `0.233570` | `1.924347` | `FAIL` |
| Contact gain | `0.638802` | `2.452679` | `0.260451` | `PASS` |
| Short-window spectrum | `1.175589` | `2.508726` | `0.468600` | `PASS` |

### Causal and ablation gates

| Gate | Observed | Required | Result |
| --- | ---: | ---: | --- |
| No-geometry frequency error | `0.479271` | `>=0.499373` (`full ×1.05`) | `FAIL` |
| No-contact gain error | `1.266761` | `>=0.670742` (`full ×1.05`) | `PASS` |
| Maximum remesh gain difference | `0.01156194` | `<=0.00001000` | `FAIL` |
| Maximum render peak | `0.007482` | `<0.95` | `PASS` |
| Contact gradients finite/nonconstant | `true` | `true` | `PASS` |
| Force scaling exact | `true` | `true` | `PASS` |

### Known-physics counterfactuals

Each relative error must be `<=0.10`.

| Axis | Expected ratio | Observed ratio | Relative error | Result |
| --- | ---: | ---: | ---: | --- |
| Young's modulus | `2.0` | `1.023539` | `0.488231` | `FAIL` |
| Density | `0.5` | `0.983387` | `0.966773` | `FAIL` |
| Thickness | `2.0` | `1.000026` | `0.499987` | `FAIL` |
| Scale | `0.25` | `1.000683` | `3.002733` | `FAIL` |

The dominant pattern is not generic inability to fit sound. The network fits
several output-space metrics while behaving almost invariantly to physical
coordinates whose intervention should move modal frequency. That falsifies the
current representation/training hypothesis as a causal physical generator.

## Protected-role consequences

Because synthetic gates reject:

- no `candidate-freeze.json` is published;
- method holdout stays unopened and no holdout report exists;
- disclosed-real query stays unopened and no real report exists;
- admission shadow stays unopened;
- no real material, runtime, cooker or demo authority is granted.

## Decision and next action

`R2 = REPEAT_EXACT_REPRESENTATION_REJECT`. Close M0c and keep its artifacts
external. Do not tune from the opened control values.

Roadmap work that does not depend on a passing generator may continue:

1. Q0 signal-blind validator source/role/protocol freeze is next.
2. A future generator successor requires a new bounded research result and
   preregistration. It should explain how physical interventions and remesh
   invariance become structural, rather than merely adding waveform capacity.
3. A waveform/codec latent family is not selected automatically: R2 proves the
   compact causal student failed, but its hardest failures are physical
   conditioning and remesh consistency, not only waveform reconstruction.
4. Metal candidate/admission, cooker/demo, Glass and Wood remain blocked;
   authored clips remain authoritative fallback.
