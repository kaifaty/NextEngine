# VoxCPM2 TTS evaluation — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-17` |
| Task key | `voxcpm2-tts-demo` |
| Scope | Install and evaluate the official VoxCPM2 CUDA inference path on the available RTX 3080, with reproducible Russian Voice Design examples, cold/warm speed, RTF, VRAM and human-listening artifacts, without integrating weights or third-party runtime into NextEngine. |
| Definition of done | A pinned external installation produces valid adult-voice Russian WAV through a repository-owned wrapper; exact source/model identity, license, runtime closure, cold/warm/streaming timings, VRAM, determinism and listening examples are recorded. |
| Authority | Working context only; `AGENTS.md`, exact official source/model revisions, their licenses and raw external evidence outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** The pinned official compiled PyTorch baseline passes on the RTX 3080. At 10 steps it measures warm RTF 0.707 (2.601 s wall for 3.680 s audio, 1.41x realtime) at 8,286 MiB peak total GPU. Streaming returns the first 160 ms PCM chunk in 56.9 ms warm mean. Human Russian-quality judgment is pending on the archived adult Voice Design examples.
- **Why:** Three fixed-seed warm requests were PCM-identical, all 9 model hashes passed, 10/6/4-step and eager/streaming runs completed, and the longer adult-female sample produced 8.0 s audio at RTF 0.717 without clipping.
- **Next action:** User listens to the archived 10/6/4-step male examples and female dialogue, then records the quality verdict before selecting a production candidate.
- **Current blocker:** None.
- **Do not retry:** Do not use the unqualified `main` branch, mutable model aliases at runtime, community quantizations, unlicensed reference speech or model/generated artifacts inside Git for the correctness baseline.
- **Reconsider when:** Human quality is acceptable and the 10-step RTF 0.707 or 8.3 GiB footprint motivates a separately pinned Nano-vLLM/llama.cpp-omni comparison.

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

## Decisions that still constrain the work

### D-001 — Official Voice Design correctness baseline

- **Observation:** VoxCPM2 supports both voice cloning and reference-free Voice Design; the previous model's bundled voice reference biased the listening result toward a childlike timbre.
- **Evidence:** Official VoxCPM2 README/API and recorded CosyVoice listening verdict.
- **Decision:** Use the official BF16/PyTorch path and reference-free English control descriptions for one mature male primary benchmark plus a mature female listening variant. Keep CFG 2.0, 10 LocDiT steps and seed 1234 fixed unless a separately reported quality/speed tradeoff is tested.
- **Rejected alternatives:** Do not clone an arbitrary online voice or start with GGUF/Nano-vLLM because either introduces consent/provenance risk or confounds runtime parity before a correctness baseline exists.
- **Consequences:** Voice quality can be judged without third-party reference audio; timing includes the Voice Design prompt path.
- **Uncertainty:** Human Russian pronunciation, naturalness, adult-timbre adherence and 4/6-step quality remain unjudged; concurrency and optimized serving backends remain unmeasured.
- **Reconsider when:** The user records a listening verdict or an optimized official backend is evaluated on the same closure.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Official BF16 inference fits in 10 GiB VRAM | Confirmed for one resident request: observed maximum 8,455 MiB total GPU on the long sample | Remaining headroom is insufficient to assume concurrent requests | Keep single-request residency as the proven boundary. |
| H2: Adult Voice Design avoids a childlike output without reference speech | Adult male/female examples were produced without reference audio and have valid unclipped waveforms | Human timbre judgment is pending | User listens to archived examples. |
| H3: Warm RTX 3080 RTF is competitive with CosyVoice 0.582 | Faster than Fish Q4 and faster than realtime | 10-step RTF 0.707 is 21.5% higher than CosyVoice; 4 steps still 11.1% higher | Prefer by quality/TTFA only if listening result justifies the speed/VRAM tradeoff. |

## Required context

1. `AGENTS.md`.
2. `docs/development/tts-model-evaluations/fish-s2-pro/README.md` for the shared timing protocol.
3. `docs/development/tts-model-evaluations/fun-cosyvoice3-0.5b-2512/README.md` for the current speed and voice-quality comparator.
4. Official `OpenBMB/VoxCPM` release 2.0.3 source and `openbmb/VoxCPM2` snapshot/license.

## Next action

1. Listen to `adult-male-steps10.wav`, `adult-male-steps6.wav`, `adult-male-steps4.wav` and `adult-female-dialogue-steps10.wav` in the external archive.
2. Record pronunciation, naturalness, adult-timbre and step-quality verdicts.
3. Continue to an optimized backend only if quality is acceptable; preserve the same text/control/seed/CFG/steps protocol.

## Do not retry

- Using an uncontrolled bundled/online reference voice for the primary quality sample — it prevents a clean adult-timbre judgment and may introduce consent/licensing ambiguity.
- Treating model load or generator creation as TTFA — measure the first returned PCM chunk for streaming.
- Passing `device="cuda:0"` to release 2.0.3 when measuring the compiled path — `optimize()` checks exact string equality with `"cuda"` and silently falls back to eager execution; select the physical GPU through `CUDA_VISIBLE_DEVICES` and pass `device="cuda"`.

## Handoff

- **Workspace state:** Repository-owned wrapper, tests, durable report and task-state are tracked; model, source, venv, compile cache, logs, raw summaries and WAVs remain in external machine-local roots.
- **Checks:** All 9 model hashes, source revision, isolated runtime/CUDA/BF16 doctor, six unit tests, canonical compiled/eager/step/streaming runs, WAV structure, no-clipping analysis and archive copy checks passed.
- **Remaining risk:** Human Russian quality, multi-request concurrency, broader corpus behavior and optimized serving backends remain unmeasured.
- **Promotion needed:** None for a bounded lab experiment.
