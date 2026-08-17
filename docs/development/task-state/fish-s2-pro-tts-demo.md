# Fish S2 Pro TTS demo — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-17` |
| Task key | `fish-s2-pro-tts-demo` |
| Scope | Install and compare local Fish S2 Pro GGUF/CUDA profiles and C++ runtimes on the available RTX 3080 without integrating model code or weights into the engine runtime. |
| Definition of done | Pinned external `s2.cpp` and `audio.cpp` builds produce Russian WAV through repository-owned demo entry points; exact revisions, hashes, license constraint and measured local runtime are recorded. |
| Authority | Working context only; `AGENTS.md`, Accepted architecture and exact upstream repositories/model artifacts outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** Close the pinned Fish S2 Pro experiment and continue with other TTS models. `s2.cpp` Q4_K_M had generally acceptable quality but only RTF 0.874; the faster `audio.cpp` Q8_0 result at RTF 0.659 had poor/insufficient human-perceived quality.
- **Why:** Human listening rejected `fish-s2-pro-audio-cpp-q8-demo.wav`; its 25% RTF advantage over Q4_K_M and 8,189 MiB peak total GPU use therefore do not make it the next integration candidate.
- **Next action:** Select another local TTS model and apply the fixed warm protocol and reporting fields in `docs/development/tts-model-evaluations/fish-s2-pro/README.md`.
- **Current blocker:** None.
- **Do not retry:** Do not install the official BF16/PyTorch S2 Pro stack on this 10 GiB card for this task; upstream declares a 24 GiB inference recommendation and the requested bounded experiment has a smaller GGUF path.
- **Do not retry:** Do not report the first synthesis after server initialization as steady warm latency; the cached step graph becomes fast only after one complete synthesis request.
- **Do not retry:** Do not select the exact pinned audio.cpp Q8_0 candidate from speed alone; the delivered Russian example failed the human quality check.
- **Reconsider when:** A newer pinned runtime/model package or controlled voice-conditioning experiment materially improves quality, or an accepted `ai-host` consumer defines a different production constraint.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `nvidia-smi` on 2026-08-17 | `PASS`: RTX 3080, 10,240 MiB, driver 610.43.02, compute capability 8.6 | CUDA GGUF evaluation is available; available VRAM still needs runtime measurement. |
| `nvcc --version` | `PASS`: CUDA 13.3 | Exceeds current `s2.cpp` stated CUDA 12.4 minimum. |
| Fish Speech official install/inference docs | `REPORT_ONLY`: 24 GiB inference recommendation | Official BF16 stack is not the first path on this host. |
| `rodrigomatta/s2.cpp` README | `REPORT_ONLY`: alpha C++/GGML CUDA engine, Q8/Q6/Q4 variants and partial offload | Candidate is suitable only for a local prototype until measured. |
| Fish Audio S2 Pro license | `REPORT_ONLY`: research/non-commercial use; commercial use requires a separate license | Never represent this artifact as a distributable or commercial-ready NextEngine dependency. |
| external `s2.cpp` build at `2c332619…` | `PASS`: CUDA 13 build targets `sm_86`; bundled ggml `57ea0bc1…` | No Python/PyTorch runtime is required after build. |
| Q4_K_M `83963e1b…` | `PASS`: Russian float32 44.1 kHz WAV; codec CUDA; 5,124 MiB cold peak total GPU; steady-warm mean RTF 0.874; generally acceptable human-perceived quality | Retain as a recorded lower-memory quality baseline, but it misses the experimental RTF 0.5 target. |
| Q5_K_M `e445b0c8…` | `PASS`: Russian float32 44.1 kHz WAV; codec CUDA; 5,499 MiB cold peak total GPU; steady-warm mean RTF 0.918 | Valid middle comparator, but slower than Q4 in the fixed-text probe. |
| Q6_K `84ac9041…` | `PASS`: Russian float32 44.1 kHz WAV; codec CUDA; synthesis RTF 3.035; cold 18.14 s; peak total GPU 6,205 MiB | Selected local demo default; result does not pass `MODEL-TTS-P1`. |
| Q8_0 `e2043182…` | `PASS`: Russian float32 44.1 kHz WAV; codec CPU; synthesis RTF 5.650; cold 37.88 s; peak total GPU 5,701 MiB | Retained only as quality comparator on this host. |
| Q6 HTTP `/generate` | `PASS`: HTTP 200 and valid 3.947 s float32 44.1 kHz WAV | Local finalized-WAV server example is operational. |
| Q6 chunked streaming | `PASS` transport, `REPORT_ONLY` latency: valid 2.833 s PCM16 WAV; upstream RTF 6.24 | Streaming API works but is not real-time; no TTFA claim. |
| Q4/Q5/Q6 finalized HTTP after one prewarm request | `PASS`: three measured requests each; mean warm RTF 0.874/0.918/1.040 | Model init alone is insufficient prewarm; run one disposable synthesis before interactive traffic. |
| external `audio.cpp` build at `980bd416…` | `PASS`: custom Fish deployment build, CUDA `sm_86`, CUDA Graphs enabled, binary links CUDA 13 | Compatible alternative runtime is operational without modifying `s2.cpp`. |
| audio.cpp Q8_0 `4ffc1694…` | `PASS`: exact 6,317,911,232-byte package; mono PCM16 44.1 kHz Russian WAV; fixed-seed outputs byte-identical | The audio.cpp-specific GGUF package is valid and reproducible. |
| audio.cpp Q8_0 warm benchmark | `PASS`: 3 warm requests, mean 2.816 s for 4.272 s audio, RTF 0.659, 1.52x realtime; 8,189 MiB peak total GPU | Fastest measured local path, but still above Proposed RTF 0.5 and rejected by the separate human quality check. |
| audio.cpp thread/graph probes | `REPORT_ONLY`: 1/4/8 threads RTF 0.657/0.656/0.664; experimental graph optimizer RTF 0.657 | Keep the documented 4-thread default and do not enable the experimental graph override. |
| Human listening of `fish-s2-pro-audio-cpp-q8-demo.wav` | `FAIL`: quality judged poor/insufficient | Reject this exact audio.cpp Q8_0 candidate and move to other TTS models. |
| `docs/development/tts-model-evaluations/fish-s2-pro/README.md` | `REPORT_ONLY`: consolidated identities, timings, tuning probes, WAV examples and final quality verdict | Stable completed experiment record for future model comparisons. |

## Decisions that still constrain the work

### D-001 — External bounded experiment, not runtime integration

- **Observation:** Accepted ADR-005 requires TTS in an optional separate `ai-host`; SPEC-16/ADR-017 model-pack integration remains `Deferred Proposed`.
- **Evidence:** `docs/architecture/adr/005-offline-first-ai-process-boundary.md`, `docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md`, and `docs/architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md`.
- **Decision:** Add only a lab/demo entry point and documentation; install third-party source/model/output in a machine-local external root.
- **Rejected alternatives:** Vendoring weights or linking `s2.cpp` into a production crate would violate repository/model hygiene and prematurely implement a Proposed boundary.
- **Consequences:** The demo must remain replaceable, optional and absent from gameplay/product checks.
- **Uncertainty:** `s2.cpp` is alpha; only one fixed Russian text has steady-warm timing, while corpus-level pronunciation, quality and latency percentiles remain unmeasured.
- **Reconsider when:** A production `ai-host` consumer and promoting ADR define an accepted adapter/model-pack contract.

### D-002 — Q4_K_M latency candidate with Q6_K operational default

- **Observation:** Q8_0 transformer offload fits, but CUDA codec allocation fails and falls back to CPU. Q4/Q5/Q6 keep transformer and codec on CUDA. After one disposable synthesis, Q4 steady-warm RTF is lower than Q5 and Q6 on the fixed text.
- **Evidence:** External logs under `/home/kaifaty/.local/share/nextengine/fish-s2-pro/outputs/` and the measured rows above.
- **Decision:** The wrapper maps `q4/q5/q6/profile` to `--codec-follow-backend` and `q8/profile` to `--codec-cpu`. Keep Q6 as the default pending human listening; use Q4 explicitly for the current latency experiment.
- **Rejected alternatives:** Do not retry Q8 CUDA codec on this exact 10 GiB profile; the measured allocation failed. Do not make Q5 the speed default from this evidence because it was slower than Q4 while its quality advantage remains unmeasured.
- **Consequences:** The demo avoids a known OOM probe, exposes all measured quantization profiles, and distinguishes model init, first-synthesis prewarm and steady-warm latency.
- **Uncertainty:** Subjective Russian quality and warm long-form behavior still require human listening and a bounded corpus.
- **Reconsider when:** A pinned upstream revision reduces codec residency enough to pass an explicit Q8 CUDA allocation probe.

### D-003 — audio.cpp Q8_0 speed candidate rejected after listening

- **Observation:** audio.cpp uses a distinct standalone Q8_0 package, keeps the full model and codec on the RTX 3080, and is materially faster than the existing `s2.cpp` profiles in the fixed-text warm probe.
- **Evidence:** Pinned external code/model identities, repository-owned benchmark summary and stage profile under `/home/kaifaty/.local/share/nextengine/audio-cpp-fish-s2-pro/outputs/`.
- **Decision:** Retain the audio.cpp wrapper and measurements as reproducible evidence, but reject this exact Q8_0 output as the next candidate and continue with other models. Keep `mem_saver=false`, CUDA Graphs enabled and 4 helper threads only when reproducing the archived result.
- **Rejected alternatives:** The experimental CUDA graph optimizer and 8 helper threads showed no speed benefit. The existing `s2.cpp` GGUF cannot be reused because audio.cpp requires its own package schema/tensor layout.
- **Consequences:** The 0.659 RTF result is a performance baseline, not an adoption result. The consolidated report records all examples and numbers so the failed quality/speed trade-off is not repeated.
- **Uncertainty:** Only one Russian text and one GPU were measured; reference voice or sampling changes may affect quality but were outside this bounded comparison.
- **Reconsider when:** A pinned audio.cpp release adds a materially different Fish package/runtime, or a controlled consent-aware voice/sampling comparison produces an acceptable listening result without losing the required latency.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Q8_0 transformer plus codec fits fully on RTX 3080 10 GiB | Transformer alone offloads successfully | Local codec CUDA allocation failed with OOM | Resolved false for code `2c332619…`, GGUF `a7320690…`, driver 610.43.02. |
| H2: Current `s2.cpp` produces usable Russian speech without a reference voice | Multiple valid non-silent Russian WAV files were generated; Q4 was judged generally acceptable | Only one short prompt received a recorded human verdict | Resolved for this example only; a future candidate comparison needs a small fixed corpus. |
| H3: Q4 quality is sufficient to trade for its 16% warm RTF improvement over Q6 | Q4 was judged generally acceptable and is the fastest local s2.cpp profile | Its RTF 0.874 still misses the experimental 0.5 target | Resolved as a baseline, not an integration candidate; compare other models. |
| H4: audio.cpp Q8 quality justifies its speed and VRAM cost | Valid deterministic Russian WAV; RTF 0.659 beats every measured `s2.cpp` profile | Resolved false for the delivered example: human listening judged quality poor/insufficient; peak total GPU is 8,189 MiB | Reopen only after a materially changed pinned runtime/model or controlled voice-conditioning setup. |

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, `docs/architecture/README.md`, `docs/architecture/00-product-contract.md`, `docs/architecture/01-system-architecture.md`, `docs/architecture/glossary.md`.
2. `docs/architecture/adr/005-offline-first-ai-process-boundary.md`, `docs/architecture/08-audio-navigation-and-world-services.md`, `docs/architecture/11-security-licensing-and-governance.md`.
3. `docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md` and `docs/architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md` as Proposed context only.
4. `docs/development/tts-model-evaluations/fish-s2-pro/README.md` as the bounded completed experiment report.
5. Current upstream `rodrigomatta/s2.cpp`, `rodrigomt/s2-pro-gguf`, `0xShug0/audio.cpp`, `audio-cpp/audio.cpp-gguf`, official `fishaudio/s2-pro` model card/license and Fish Speech install/inference docs.

## Next action

1. Select another TTS candidate and reuse the archived fixed-text warm protocol before expanding to a small gameplay corpus.
2. Measure first audio correctly at the PCM-byte boundary and record the human listening verdict independently from RTF.
3. Non-regression condition: keep weights, reference voices, generated audio and logs external, and preserve `TextOnlyFallback` for any future integration.

## Do not retry

- Official BF16/PyTorch S2 Pro on this 10 GiB GPU — upstream recommends 24 GiB; reconsider only if a supported low-memory official path is published and requested.
- Vendoring model weights/source/output into NextEngine — prohibited by repository hygiene; reconsider only through a separate accepted distribution/model-pack decision.
- Q8_0 codec on CUDA for the exact pinned 10 GiB profile — local allocation failed; reconsider only after a pinned runtime/model change with a fresh allocation probe.
- Treating model initialization alone as warm-up — the first synthesis still measured RTF 3.07–3.28; reconsider only after a pinned runtime changes first-request graph construction behavior.
- Selecting audio.cpp Q8_0 from this exact pinned package on speed alone — the delivered example failed human quality review; reconsider only after a material model/runtime/conditioning change.

## Handoff

- **Workspace state:** Both external pinned builds and weights remain outside Git. Seven listening examples and the relevant raw metrics are consolidated at `/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17/`; the report lives at `docs/development/tts-model-evaluations/fish-s2-pro/README.md`.
- **Checks:** Existing runtime checks passed. Documentation-only validation passed: `git diff --check`, all referenced local paths, seven copied WAV checksums, archive structure and eight upstream links were verified; runtime tests were not rerun for this report-only change.
- **Remaining risk:** One-prompt subjective evaluation, non-commercial Fish model license, every measured profile above the Proposed 0.5 target, and no accepted Fish candidate for integration.
- **Promotion needed:** None for a bounded lab demo; production integration would require the normal SPEC-16/ADR workflow.
