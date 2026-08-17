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

- **Current conclusion:** `audio.cpp` Q8_0 is the fastest local candidate if its Russian quality and 8.2 GiB peak total VRAM are acceptable; retain `s2.cpp` Q4_K_M as the lower-memory fallback and Q6_K as the original wrapper default pending listening.
- **Why:** The repository-owned warm benchmark measured audio.cpp Q8_0 at RTF 0.659 versus 0.874 for `s2.cpp` Q4_K_M on the same fixed text: about 25% lower RTF and 1.33x realtime throughput, at the cost of roughly 3.1 GiB more peak total GPU memory.
- **Next action:** Listen to the delivered audio.cpp Q8_0 and `s2.cpp` Q4_K_M examples and decide whether audio.cpp's quality/VRAM trade is acceptable.
- **Current blocker:** None.
- **Do not retry:** Do not install the official BF16/PyTorch S2 Pro stack on this 10 GiB card for this task; upstream declares a 24 GiB inference recommendation and the requested bounded experiment has a smaller GGUF path.
- **Do not retry:** Do not report the first synthesis after server initialization as steady warm latency; the cached step graph becomes fast only after one complete synthesis request.
- **Reconsider when:** A newer pinned runtime materially changes RTX 3080 codec residency/RTF or an accepted `ai-host` consumer defines a production adapter.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `nvidia-smi` on 2026-08-17 | `PASS`: RTX 3080, 10,240 MiB, driver 610.43.02, compute capability 8.6 | CUDA GGUF evaluation is available; available VRAM still needs runtime measurement. |
| `nvcc --version` | `PASS`: CUDA 13.3 | Exceeds current `s2.cpp` stated CUDA 12.4 minimum. |
| Fish Speech official install/inference docs | `REPORT_ONLY`: 24 GiB inference recommendation | Official BF16 stack is not the first path on this host. |
| `rodrigomatta/s2.cpp` README | `REPORT_ONLY`: alpha C++/GGML CUDA engine, Q8/Q6/Q4 variants and partial offload | Candidate is suitable only for a local prototype until measured. |
| Fish Audio S2 Pro license | `REPORT_ONLY`: research/non-commercial use; commercial use requires a separate license | Never represent this artifact as a distributable or commercial-ready NextEngine dependency. |
| external `s2.cpp` build at `2c332619…` | `PASS`: CUDA 13 build targets `sm_86`; bundled ggml `57ea0bc1…` | No Python/PyTorch runtime is required after build. |
| Q4_K_M `83963e1b…` | `PASS`: Russian float32 44.1 kHz WAV; codec CUDA; 5,124 MiB cold peak total GPU; steady-warm mean RTF 0.874 | Fastest tested profile; latency candidate pending listening quality. |
| Q5_K_M `e445b0c8…` | `PASS`: Russian float32 44.1 kHz WAV; codec CUDA; 5,499 MiB cold peak total GPU; steady-warm mean RTF 0.918 | Valid middle comparator, but slower than Q4 in the fixed-text probe. |
| Q6_K `84ac9041…` | `PASS`: Russian float32 44.1 kHz WAV; codec CUDA; synthesis RTF 3.035; cold 18.14 s; peak total GPU 6,205 MiB | Selected local demo default; result does not pass `MODEL-TTS-P1`. |
| Q8_0 `e2043182…` | `PASS`: Russian float32 44.1 kHz WAV; codec CPU; synthesis RTF 5.650; cold 37.88 s; peak total GPU 5,701 MiB | Retained only as quality comparator on this host. |
| Q6 HTTP `/generate` | `PASS`: HTTP 200 and valid 3.947 s float32 44.1 kHz WAV | Local finalized-WAV server example is operational. |
| Q6 chunked streaming | `PASS` transport, `REPORT_ONLY` latency: valid 2.833 s PCM16 WAV; upstream RTF 6.24 | Streaming API works but is not real-time; no TTFA claim. |
| Q4/Q5/Q6 finalized HTTP after one prewarm request | `PASS`: three measured requests each; mean warm RTF 0.874/0.918/1.040 | Model init alone is insufficient prewarm; run one disposable synthesis before interactive traffic. |
| external `audio.cpp` build at `980bd416…` | `PASS`: custom Fish deployment build, CUDA `sm_86`, CUDA Graphs enabled, binary links CUDA 13 | Compatible alternative runtime is operational without modifying `s2.cpp`. |
| audio.cpp Q8_0 `4ffc1694…` | `PASS`: exact 6,317,911,232-byte package; mono PCM16 44.1 kHz Russian WAV; fixed-seed outputs byte-identical | The audio.cpp-specific GGUF package is valid and reproducible. |
| audio.cpp Q8_0 warm benchmark | `PASS`: 3 warm requests, mean 2.816 s for 4.272 s audio, RTF 0.659, 1.52x realtime; 8,189 MiB peak total GPU | Fastest measured local path, but still above Proposed RTF 0.5 and pending human quality review. |
| audio.cpp thread/graph probes | `REPORT_ONLY`: 1/4/8 threads RTF 0.657/0.656/0.664; experimental graph optimizer RTF 0.657 | Keep the documented 4-thread default and do not enable the experimental graph override. |

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

### D-003 — audio.cpp Q8_0 becomes the speed candidate

- **Observation:** audio.cpp uses a distinct standalone Q8_0 package, keeps the full model and codec on the RTX 3080, and is materially faster than the existing `s2.cpp` profiles in the fixed-text warm probe.
- **Evidence:** Pinned external code/model identities, repository-owned benchmark summary and stage profile under `/home/kaifaty/.local/share/nextengine/audio-cpp-fish-s2-pro/outputs/`.
- **Decision:** Add a separate audio.cpp lab wrapper and retain both engines. Use audio.cpp Q8_0 as the speed candidate, with `mem_saver=false`, CUDA Graphs enabled and 4 helper threads; do not replace the `s2.cpp` wrapper until human listening accepts quality and the 8.2 GiB peak fits the intended process budget.
- **Rejected alternatives:** The experimental CUDA graph optimizer and 8 helper threads showed no speed benefit. The existing `s2.cpp` GGUF cannot be reused because audio.cpp requires its own package schema/tensor layout.
- **Consequences:** Warm RTF drops from 0.874 to a conservative 0.659, while peak total GPU memory rises from 5,124 MiB to 8,189 MiB. The two external installations remain independently reproducible.
- **Uncertainty:** Only one Russian text and one GPU were measured; upstream's broader Q8 report does not state the exact benchmark hardware, and subjective quality is not machine-validated.
- **Reconsider when:** A pinned audio.cpp release adds a supported lower-bit Fish package, streaming Fish path, or an accepted production `ai-host` boundary supplies a different latency/VRAM budget.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Q8_0 transformer plus codec fits fully on RTX 3080 10 GiB | Transformer alone offloads successfully | Local codec CUDA allocation failed with OOM | Resolved false for code `2c332619…`, GGUF `a7320690…`, driver 610.43.02. |
| H2: Current `s2.cpp` produces usable Russian speech without a reference voice | Multiple valid non-silent Russian WAV files were generated for Q4/Q5/Q6/Q8 | Subjective pronunciation/voice quality is not machine-validated | Human listening comparison of the delivered quantization profiles. |
| H3: Q4 quality is sufficient to trade for its 16% warm RTF improvement over Q6 | Q4 generated a valid Russian WAV and is the fastest local profile | One fixed text and waveform validity do not establish pronunciation or voice quality | Blind or at least level-matched listening on a small fixed Russian gameplay corpus. |
| H4: audio.cpp Q8 quality justifies its speed and VRAM cost | Valid deterministic Russian WAV; RTF 0.659 beats every measured `s2.cpp` profile | No human quality comparison yet; peak total GPU is 8,189 MiB | Listen to audio.cpp Q8 against `s2.cpp` Q4/Q6 on the fixed prompt, then repeat on a small gameplay corpus. |

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, `docs/architecture/README.md`, `docs/architecture/00-product-contract.md`, `docs/architecture/01-system-architecture.md`, `docs/architecture/glossary.md`.
2. `docs/architecture/adr/005-offline-first-ai-process-boundary.md`, `docs/architecture/08-audio-navigation-and-world-services.md`, `docs/architecture/11-security-licensing-and-governance.md`.
3. `docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md` and `docs/architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md` as Proposed context only.
4. Current upstream `rodrigomatta/s2.cpp`, `rodrigomt/s2-pro-gguf`, `0xShug0/audio.cpp`, `audio-cpp/audio.cpp-gguf`, official `fishaudio/s2-pro` model card/license and Fish Speech install/inference docs.

## Next action

1. Listen to the generated audio.cpp Q8_0 and `s2.cpp` Q4/Q6 examples and choose the acceptable quality/VRAM profile.
2. If continued, measure first audio correctly at the PCM-byte boundary and compare pronunciation/quality against other TTS candidates on one fixed corpus.
3. Non-regression condition: keep weights, reference voices, generated audio and logs external, and preserve `TextOnlyFallback` for any future integration.

## Do not retry

- Official BF16/PyTorch S2 Pro on this 10 GiB GPU — upstream recommends 24 GiB; reconsider only if a supported low-memory official path is published and requested.
- Vendoring model weights/source/output into NextEngine — prohibited by repository hygiene; reconsider only through a separate accepted distribution/model-pack decision.
- Q8_0 codec on CUDA for the exact pinned 10 GiB profile — local allocation failed; reconsider only after a pinned runtime/model change with a fresh allocation probe.
- Treating model initialization alone as warm-up — the first synthesis still measured RTF 3.07–3.28; reconsider only after a pinned runtime changes first-request graph construction behavior.

## Handoff

- **Workspace state:** Both external pinned builds, all tested GGUF files and generated outputs exist outside Git; the coherent repository change contains separate wrappers/tests, the user guide and this task-state update.
- **Checks:** Existing `s2.cpp` checks plus audio.cpp source/model identity, CUDA build configuration, CLI generation, repeated warm metrics, WAV inspection, deterministic fixed-seed output and scoped repository checks passed.
- **Remaining risk:** Experimental inference paths, non-commercial Fish model license, subjective Russian quality, audio.cpp RTF above the Proposed 0.5 target, 8.2 GiB peak total GPU memory and no Fish streaming path in audio.cpp.
- **Promotion needed:** None for a bounded lab demo; production integration would require the normal SPEC-16/ADR workflow.
