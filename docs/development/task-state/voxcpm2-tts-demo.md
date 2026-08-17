# VoxCPM2 TTS evaluation — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-17` |
| Task key | `voxcpm2-tts-demo` |
| Scope | Install and evaluate the official VoxCPM2 PyTorch path and the pinned Nano-vLLM-VoxCPM optimized CUDA path on the available RTX 3080, with reproducible Russian Voice Design and emotion/delivery examples, cold/warm speed, RTF, VRAM and human-listening artifacts, without integrating weights or third-party runtime into NextEngine. |
| Definition of done | Pinned external PyTorch and Nano-vLLM installations produce valid adult-voice Russian WAV through repository-owned wrappers; exact source/model/runtime identity, cold/warm/streaming timings, VRAM, determinism, baseline listening examples, an emotion/delivery suite and a same-host comparison are recorded. |
| Authority | Working context only; `AGENTS.md`, exact official source/model revisions, their licenses and raw external evidence outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** Compiled/streaming PyTorch remains the VoxCPM2 quality baseline and now has a seven-case natural-language emotion/delivery suite. Nano-vLLM 2.0.3 with CUDA Graphs is faster at RTF 0.249, but its intonation is noticeably worse and it remains a throughput-only alternative.
- **Why:** The new suite generated valid unclipped audio at mean RTF 0.787 and 60.0 ms mean first PCM while keeping one resident model, fixed seed/CFG/steps and explicit control strings. Perceptual adherence of the seven controls is not yet scored.
- **Next action:** Listen to the seven emotion-suite WAVs and record which controls preserve the adult speaker and clearly convey the intended delivery; promote only the successful descriptions to candidate engine presets.
- **Current blocker:** None.
- **Do not retry:** Do not use the unqualified `main` branch, mutable model aliases at runtime, community quantizations, unlicensed reference speech or model/generated artifacts inside Git for the correctness baseline.
- **Reconsider when:** A later pinned Nano-vLLM/model revision materially improves the waveform, the workload explicitly accepts weaker intonation for throughput, or concurrency becomes a requirement.

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
| Human intonation comparison | `FAIL` for Nano-vLLM quality parity: the user judged PyTorch to have noticeably better intonation in the retained A/B examples | Keep PyTorch as the quality baseline; do not select Nano-vLLM solely by RTF. |
| PyTorch emotion/delivery suite | `PASS` technically: seven streaming 48 kHz float32 WAVs in one resident compiled session; mean RTF 0.787, first PCM 60.0 ms, peak total GPU 7,491 MiB, no non-finite or clipped samples | Exact Voice Design strings and timing evidence are reproducible; human emotion and speaker-consistency scoring remains pending. |

## Decisions that still constrain the work

### D-001 — Official Voice Design correctness baseline

- **Observation:** VoxCPM2 supports both voice cloning and reference-free Voice Design; the previous model's bundled voice reference biased the listening result toward a childlike timbre.
- **Evidence:** Official VoxCPM2 README/API and recorded CosyVoice listening verdict.
- **Decision:** Use the official BF16/PyTorch path and reference-free English control descriptions for one mature male primary benchmark plus a mature female listening variant. Keep CFG 2.0, 10 LocDiT steps and seed 1234 fixed unless a separately reported quality/speed tradeoff is tested.
- **Rejected alternatives:** Do not clone an arbitrary online voice or start with GGUF/Nano-vLLM because either introduces consent/provenance risk or confounds runtime parity before a correctness baseline exists.
- **Consequences:** Voice quality can be judged without third-party reference audio; timing includes the Voice Design prompt path.
- **Uncertainty:** Russian pronunciation, adult-timbre adherence and 4/6-step quality remain unjudged; concurrency and broader-corpus behavior remain unmeasured. Intonation parity is resolved in favor of PyTorch.
- **Reconsider when:** A newer pinned model/runtime or controlled Voice Design change produces fresh A/B listening evidence.

### D-002 — Isolated Nano-vLLM comparison

- **Observation:** Nano-vLLM 2.0.3 supports VoxCPM2 safetensors, fixed seeds, CFG and selectable diffusion steps, but requires CUDA, Triton and FlashAttention.
- **Evidence:** Pinned source commit `0ef61b0…`, local import checks and repository wrapper `lab/scripts/voxcpm2_nanovllm_demo.py`.
- **Decision:** Reuse the exact pinned model and Voice Design text, keep Nano-vLLM in a separate external environment, set single-request scheduler bounds, measure first returned PCM plus full wall/audio RTF, and retain one prewarm plus three measured WAVs.
- **Rejected alternatives:** Do not compare upstream RTX 4090 RTF to the local 3080 baseline; do not replace FlashAttention with a Python shim; do not commit runtime binaries, model weights, logs or WAVs.
- **Consequences:** The comparison remains attributable and does not disturb the completed PyTorch baseline. The locally retained FlashAttention build is inference-only for BF16/head dimension 128 and cannot be used as a general training wheel.
- **Uncertainty:** Long-text completeness, pronunciation and adult-timbre adherence remain unjudged; multi-request concurrency is unmeasured and unsafe to infer from less than 1 GiB peak headroom.
- **Reconsider when:** A later doctor/model load exposes a compatibility or OOM boundary, or concurrency becomes required; then change only the smallest scheduler/memory setting and retain the failure evidence.

### D-003 — Preserve PyTorch as the quality baseline

- **Observation:** The optimized and PyTorch paths use the same text, Voice Design control, seed, CFG and diffusion-step count, but their retained waveforms are perceptually different.
- **Evidence:** Direct user comparison of `adult-male-nanovllm-cuda-graph-steps10.wav` and the retained PyTorch examples on 2026-08-17: PyTorch has noticeably better intonation.
- **Decision:** Keep 10-step compiled/streaming PyTorch as the VoxCPM2 quality candidate. Treat Nano-vLLM CUDA Graphs as a performance-only alternative whose quality tradeoff must be accepted explicitly.
- **Rejected alternatives:** Do not promote Nano-vLLM as the default solely because it reaches RTF 0.249 or 4.01x realtime; speed does not compensate for the observed intonation regression.
- **Consequences:** Future TTS evaluations compare intonation against PyTorch, not Nano-vLLM. Preserve Nano-vLLM measurements as an upper-throughput reference.
- **Uncertainty:** The verdict does not separately score pronunciation, adult timbre, artifacts or completeness of the shorter Nano-vLLM female dialogue.
- **Reconsider when:** A pinned backend/model change produces a new A/B sample that the user judges comparable to PyTorch, or a specific workload knowingly prioritizes throughput over intonation.

### D-004 — Evaluate emotion through explicit Voice Design controls

- **Observation:** VoxCPM2 accepts free-form Voice Design text rather than a documented fixed emotion-tag vocabulary, and loading the model for every variation would confound latency and waste startup time.
- **Evidence:** The repository `emotion-suite` run produced seven distinct fixed-seed streaming WAVs in one resident compiled process; exact controls and request metrics are retained in `summary.json`.
- **Decision:** Compare emotions with explicit English control descriptions, a repeated mature-adult voice description, common Russian text where semantically suitable, and fixed seed/CFG/steps. Treat requested laughter as a prompt attempt rather than guaranteed tag behavior.
- **Rejected alternatives:** Do not compare variants in separate cold processes; do not claim a prompt succeeded from waveform validity or timing alone; do not describe free-form controls as canonical tags.
- **Consequences:** Runtime differences are directly comparable and every candidate preset is auditable. Human listening is required before an emotion control can be promoted.
- **Uncertainty:** Emotion adherence, speaker identity consistency, Russian pronunciation and whether the requested chuckle is actually produced remain unscored.
- **Reconsider when:** Human listening identifies weak controls, at which point change one description at a time and retain the neutral control as the anchor.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Official BF16 inference fits in 10 GiB VRAM | Confirmed for one resident request: observed maximum 8,455 MiB total GPU on the long sample | Remaining headroom is insufficient to assume concurrent requests | Keep single-request residency as the proven boundary. |
| H2: Adult Voice Design avoids a childlike output without reference speech | Adult male/female examples were produced without reference audio and have valid unclipped waveforms | Human timbre judgment is pending | User listens to archived examples. |
| H3: Warm RTX 3080 RTF is competitive with CosyVoice 0.582 | Confirmed for Nano-vLLM: RTF 0.249 is 57.2% lower than CosyVoice | Nano-vLLM intonation is noticeably worse than PyTorch and it uses 1,290 MiB more peak total GPU than CosyVoice | Retain only as a throughput comparator, not the quality candidate. |
| H4: Nano-vLLM fits and materially accelerates one request on 10 GiB | Confirmed: RTF 0.249 at 9,271 MiB versus PyTorch RTF 0.707 at 8,286 MiB | Less than 1 GiB observed peak headroom; concurrency is not established; quality parity failed | Keep one resident request as the proven performance boundary. |
| H5: Free-form Voice Design can produce useful RPG emotion presets while preserving one adult speaker | Seven technically valid variants were generated from explicit mature-male controls | Perceptual emotion adherence and speaker consistency have not been scored | User listens to the neutral anchor and six expressive variants. |

## Required context

1. `AGENTS.md`.
2. `docs/development/tts-model-evaluations/fish-s2-pro/README.md` for the shared timing protocol.
3. `docs/development/tts-model-evaluations/fun-cosyvoice3-0.5b-2512/README.md` for the current speed and voice-quality comparator.
4. Official `OpenBMB/VoxCPM` release 2.0.3 source and `openbmb/VoxCPM2` snapshot/license.

## Next action

1. Listen to the seven files under `examples/emotion-suite-stream-steps10` and score intended emotion, adult-speaker consistency, pronunciation and artifacts.
2. If a control is weak, change only its Voice Design description and compare it against `neutral.wav` with the same seed, CFG and step count.
3. Revisit Nano-vLLM only after a pinned quality-relevant change or for an explicitly throughput-first workload.

## Do not retry

- Using an uncontrolled bundled/online reference voice for the primary quality sample — it prevents a clean adult-timbre judgment and may introduce consent/licensing ambiguity.
- Treating model load or generator creation as TTFA — measure the first returned PCM chunk for streaming.
- Passing `device="cuda:0"` to release 2.0.3 when measuring the compiled path — `optimize()` checks exact string equality with `"cuda"` and silently falls back to eager execution; select the physical GPU through `CUDA_VISIBLE_DEVICES` and pass `device="cuda"`.
- Recompiling the full stock FlashAttention source on this host for the bounded VoxCPM2 inference test — it spends most of its build time on unused backward, FP16 and other head dimensions. Reconsider only for a general-purpose distributable wheel; the retained inference build is deliberately BF16/head-dim-128 only.

## Handoff

- **Workspace state:** PyTorch and Nano-vLLM wrappers, tests, report and task-state are tracked; all model/runtime/generated/raw artifacts remain in external machine-local roots.
- **Checks:** Nano-vLLM pinned doctor, all 15 focused VoxCPM2 unit tests, Python compile, canonical graph/eager/long-dialogue/0.80-utilization runs, seven-case emotion suite, fixed-seed PCM determinism, WAV structure, no-clipping analysis and external archive copies pass. Documentation path/link validation and `git diff --check` pass.
- **Remaining risk:** Emotion adherence, speaker consistency, Russian pronunciation, requested chuckle, long-text completeness, broader-corpus behavior and multi-request concurrency remain perceptually or operationally unmeasured; the direct backend intonation verdict favors PyTorch.
- **Promotion needed:** None for a bounded lab experiment.
