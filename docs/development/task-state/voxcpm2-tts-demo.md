# VoxCPM2 TTS evaluation — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-17` |
| Task key | `voxcpm2-tts-demo` |
| Scope | Install and evaluate the official VoxCPM2 PyTorch path and the pinned Nano-vLLM-VoxCPM optimized CUDA path on the available RTX 3080, with reproducible Russian Voice Design examples, cold/warm speed, RTF, VRAM and human-listening artifacts, without integrating weights or third-party runtime into NextEngine. |
| Definition of done | Pinned external PyTorch and Nano-vLLM installations produce valid adult-voice Russian WAV through repository-owned wrappers; exact source/model/runtime identity, cold/warm/streaming timings, VRAM, determinism, listening examples and a same-host comparison are recorded. |
| Authority | Working context only; `AGENTS.md`, exact official source/model revisions, their licenses and raw external evidence outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** Nano-vLLM 2.0.3 with CUDA Graphs is the fastest measured VoxCPM2 path on the RTX 3080: warm RTF 0.249 (0.917 s wall for 3.680 s audio, 4.01x realtime), first 160 ms PCM in 149.8 ms and 9,271 MiB peak total GPU. It is 2.84x the throughput of compiled PyTorch RTF 0.707, but PyTorch streaming has lower TTFA at 56.9 ms and uses about 985 MiB less peak GPU.
- **Why:** Three fixed-seed Nano-vLLM graph requests were PCM-identical; eager and graph A/B, a longer female dialogue and a lower KV-cache-utilization probe all passed. The pinned runtime/model/FlashAttention closure and external raw/WAV archive are complete.
- **Next action:** User compares the Nano-vLLM and PyTorch male/female WAVs for pronunciation, prosody, adult timbre, artifacts and complete delivery of the shorter Nano-vLLM long sample.
- **Current blocker:** None.
- **Do not retry:** Do not use the unqualified `main` branch, mutable model aliases at runtime, community quantizations, unlicensed reference speech or model/generated artifacts inside Git for the correctness baseline.
- **Reconsider when:** Human quality rejects the Nano-vLLM graph waveform, a later pinned runtime changes quality/speed, or concurrency becomes a requirement.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| Host probe on 2026-08-17 | `PASS`: RTX 3080 10,240 MiB, compute capability 8.6, driver 610.43.02, CUDA toolkit 13.3, about 239 GiB free storage | Official approximately 5 GiB model download and bounded CUDA evaluation are feasible. |
| Official source repository | `REPORT_ONLY`: source tag 2.0.3 resolves to `19b6bf7590025418821a86dcb817504e0ad7e5df`; release notes include streaming VAE and CUDA-graph stability fixes | Use release 2.0.3 rather than current `main` for the baseline. |
| Official Hugging Face model | `REPORT_ONLY`: snapshot `bffb3df5a29440629464e5e839f4d214c8714c3d`, Apache-2.0, Russian among 30 languages, 4.96 GB repository | Pin this immutable snapshot and validate all files locally. |
| Official performance table | `REPORT_ONLY`: standard PyTorch RTF about 0.30 and VRAM about 8 GB on RTX 4090; Nano-vLLM about 0.13 | Treat as an upstream comparator only; measure the RTX 3080 directly with the shared protocol. |
| Previous local TTS reports | `REPORT_ONLY`: CosyVoice RTF 0.582 at 7,981 MiB but childlike official-reference timbre; Fish Q4 RTF 0.874 at 5,124 MiB with generally acceptable quality | Benchmark the same Russian text and record quality separately from runtime speed. |
| Official compiled 10-step benchmark | `PASS`: warm wall 2.601 s for 3.680 s audio, RTF 0.707, 1.41x realtime, 8,286 MiB peak total GPU | Technically usable faster-than-realtime baseline; slower than CosyVoice and audio.cpp Q8 but faster than Fish Q4. |
| Diffusion-step sweep | `PASS`: 6 steps RTF 0.664; 4 steps RTF 0.647; fixed-seed PCM stable within each run | Reducing steps gives only 6-9% lower RTF because autoregressive work and generated duration dominate; judge quality before adopting. |
| Streaming benchmark | `PASS`: 23 chunks of 160 ms; warm TTFA 56.9 ms; full RTF 0.745; 8,303 MiB peak GPU | VoxCPM2 has a genuinely low first-PCM boundary on this host, with modest full-request overhead. |
| Compile A/B | `PASS`: eager warm RTF 0.836 at 7,651 MiB; compiled RTF 0.707 at 8,286 MiB; first compilation load 111.1 s, cached compiled loads 23.8-31.8 s | `torch.compile` improves RTF about 15.4% for approximately 635 MiB more peak GPU and much higher startup cost. |
| External evidence archive | `PASS`: six listening WAVs plus canonical raw summaries/logs under `/home/kaifaty/.local/share/nextengine/tts-model-evaluations/voxcpm2-2026-08-17` | Quality and exact measurements remain reviewable without adding generated/model artifacts to Git. |
| Nano-vLLM source/runtime preparation | `PASS`: source tag 2.0.3 commit `0ef61b0ba634dbf2fad9e916bc4fb696a3c0f51f`; Python 3.11.15; Nano-vLLM 2.0.3; Torch 2.10.0+cu130; Transformers 5.15.0 | The optimized runtime is isolated from the PyTorch baseline and reuses the pinned 4.96 GB model closure. |
| FlashAttention build | `PASS`: 2.8.3.post1, BF16/head-dim-128 inference-only Ampere build; extension SHA-256 `b85683e47a0583b48f294633bfbcc17ab28289cd8cdb8279ec70ab68296f9ff1`; graph and eager GPU runs pass | Retain this exact build for the bounded inference path; it is not a general-purpose training wheel. |
| Nano-vLLM CUDA Graph benchmark | `PASS`: warm wall 0.917 s for 3.680 s audio, full RTF 0.249, 4.01x realtime, first PCM 149.8 ms, 9,271 MiB peak total GPU | 64.8% lower RTF and 2.84x the throughput of compiled PyTorch, with higher TTFA and VRAM. |
| Nano-vLLM eager control | `PASS`: warm RTF 0.884, first PCM 157.5 ms, 9,310 MiB peak total GPU | The speedup depends on CUDA Graphs; eager Nano-vLLM is about 5.8% slower than eager PyTorch. |
| Nano-vLLM longer dialogue | `PASS`: 6.560 s audio, warm RTF 0.236, first PCM 146.2 ms, 9,284 MiB, fixed-seed PCM-identical | Human review must check completeness because PyTorch generated 8.000 s for the same text. |
| Nano-vLLM external evidence archive | `PASS`: three listening WAVs and four raw run directories added under the existing VoxCPM2 archive | Exact speed and side-by-side quality evidence remain outside Git and reviewable. |

## Decisions that still constrain the work

### D-001 — Official Voice Design correctness baseline

- **Observation:** VoxCPM2 supports both voice cloning and reference-free Voice Design; the previous model's bundled voice reference biased the listening result toward a childlike timbre.
- **Evidence:** Official VoxCPM2 README/API and recorded CosyVoice listening verdict.
- **Decision:** Use the official BF16/PyTorch path and reference-free English control descriptions for one mature male primary benchmark plus a mature female listening variant. Keep CFG 2.0, 10 LocDiT steps and seed 1234 fixed unless a separately reported quality/speed tradeoff is tested.
- **Rejected alternatives:** Do not clone an arbitrary online voice or start with GGUF/Nano-vLLM because either introduces consent/provenance risk or confounds runtime parity before a correctness baseline exists.
- **Consequences:** Voice quality can be judged without third-party reference audio; timing includes the Voice Design prompt path.
- **Uncertainty:** Human Russian pronunciation, naturalness, adult-timbre adherence and 4/6-step quality remain unjudged; concurrency and broader-corpus behavior remain unmeasured.
- **Reconsider when:** The user records a listening verdict or a newer pinned model/runtime is evaluated on the same closure.

### D-002 — Isolated Nano-vLLM comparison

- **Observation:** Nano-vLLM 2.0.3 supports VoxCPM2 safetensors, fixed seeds, CFG and selectable diffusion steps, but requires CUDA, Triton and FlashAttention.
- **Evidence:** Pinned source commit `0ef61b0…`, local import checks and repository wrapper `lab/scripts/voxcpm2_nanovllm_demo.py`.
- **Decision:** Reuse the exact pinned model and Voice Design text, keep Nano-vLLM in a separate external environment, set single-request scheduler bounds, measure first returned PCM plus full wall/audio RTF, and retain one prewarm plus three measured WAVs.
- **Rejected alternatives:** Do not compare upstream RTX 4090 RTF to the local 3080 baseline; do not replace FlashAttention with a Python shim; do not commit runtime binaries, model weights, logs or WAVs.
- **Consequences:** The comparison remains attributable and does not disturb the completed PyTorch baseline. The locally retained FlashAttention build is inference-only for BF16/head dimension 128 and cannot be used as a general training wheel.
- **Uncertainty:** Human-perceived Russian quality and long-text completeness remain unjudged; multi-request concurrency is unmeasured and unsafe to infer from less than 1 GiB peak headroom.
- **Reconsider when:** A later doctor/model load exposes a compatibility or OOM boundary, or concurrency becomes required; then change only the smallest scheduler/memory setting and retain the failure evidence.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Official BF16 inference fits in 10 GiB VRAM | Confirmed for one resident request: observed maximum 8,455 MiB total GPU on the long sample | Remaining headroom is insufficient to assume concurrent requests | Keep single-request residency as the proven boundary. |
| H2: Adult Voice Design avoids a childlike output without reference speech | Adult male/female examples were produced without reference audio and have valid unclipped waveforms | Human timbre judgment is pending | User listens to archived examples. |
| H3: Warm RTX 3080 RTF is competitive with CosyVoice 0.582 | Confirmed for Nano-vLLM: RTF 0.249 is 57.2% lower than CosyVoice | Quality paths differ and Nano-vLLM uses 1,290 MiB more peak total GPU | Prefer Nano-vLLM only if listening quality passes. |
| H4: Nano-vLLM fits and materially accelerates one request on 10 GiB | Confirmed: RTF 0.249 at 9,271 MiB versus PyTorch RTF 0.707 at 8,286 MiB | Less than 1 GiB observed peak headroom; concurrency is not established | Keep one resident request as the proven boundary. |

## Required context

1. `AGENTS.md`.
2. `docs/development/tts-model-evaluations/fish-s2-pro/README.md` for the shared timing protocol.
3. `docs/development/tts-model-evaluations/fun-cosyvoice3-0.5b-2512/README.md` for the current speed and voice-quality comparator.
4. Official `OpenBMB/VoxCPM` release 2.0.3 source and `openbmb/VoxCPM2` snapshot/license.

## Next action

1. Listen to `adult-male-nanovllm-cuda-graph-steps10.wav` beside `adult-male-steps10.wav`.
2. Listen to both female dialogue files and verify that the 6.560 s Nano-vLLM result omits no words versus the 8.000 s PyTorch result.
3. Record the quality verdict before selecting Nano-vLLM as the serving candidate.

## Do not retry

- Using an uncontrolled bundled/online reference voice for the primary quality sample — it prevents a clean adult-timbre judgment and may introduce consent/licensing ambiguity.
- Treating model load or generator creation as TTFA — measure the first returned PCM chunk for streaming.
- Passing `device="cuda:0"` to release 2.0.3 when measuring the compiled path — `optimize()` checks exact string equality with `"cuda"` and silently falls back to eager execution; select the physical GPU through `CUDA_VISIBLE_DEVICES` and pass `device="cuda"`.
- Recompiling the full stock FlashAttention source on this host for the bounded VoxCPM2 inference test — it spends most of its build time on unused backward, FP16 and other head dimensions. Reconsider only for a general-purpose distributable wheel; the retained inference build is deliberately BF16/head-dim-128 only.

## Handoff

- **Workspace state:** PyTorch and Nano-vLLM wrappers, tests, report and task-state are tracked; all model/runtime/generated/raw artifacts remain in external machine-local roots.
- **Checks:** Nano-vLLM pinned doctor, all 12 focused VoxCPM2 unit tests, Python compile, canonical graph/eager/long-dialogue/0.80-utilization runs, fixed-seed PCM determinism, WAV structure, no-clipping analysis and external archive copies pass. Documentation path/link validation and `git diff --check` pass.
- **Remaining risk:** Human Russian quality, long-text completeness, broader corpus behavior and multi-request concurrency remain unmeasured.
- **Promotion needed:** None for a bounded lab experiment.
