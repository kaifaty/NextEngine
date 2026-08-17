# Fish S2 Pro TTS demo — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-17` |
| Task key | `fish-s2-pro-tts-demo` |
| Scope | Install and validate one local Fish S2 Pro GGUF/CUDA demo on the available RTX 3080 without integrating model code or weights into the engine runtime. |
| Definition of done | A pinned external `s2.cpp` build and GGUF file produce a Russian WAV through a repository-owned demo entry point; exact revisions, hashes, license constraint and measured local runtime are recorded. |
| Authority | Working context only; `AGENTS.md`, Accepted architecture and exact upstream repositories/model artifacts outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** Use Q6_K with all 36 transformer layers and codec on CUDA as this RTX 3080 demo default; keep Q8_0 as a quality comparator with codec on CPU.
- **Why:** Both pinned GGUF files synthesize valid Russian WAV, but Q6_K direct GPU codec used 6,205 MiB peak total VRAM and synthesis RTF 3.035, while Q8_0 cannot allocate its codec on GPU and CPU-codec synthesis measured RTF 5.650.
- **Next action:** Optional human listening comparison of the delivered Q6/Q8 WAV files; no further implementation is required for the bounded setup task.
- **Current blocker:** None.
- **Do not retry:** Do not install the official BF16/PyTorch S2 Pro stack on this 10 GiB card for this task; upstream declares a 24 GiB inference recommendation and the requested bounded experiment has a smaller GGUF path.
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
| Q6_K `84ac9041…` | `PASS`: Russian float32 44.1 kHz WAV; codec CUDA; synthesis RTF 3.035; cold 18.14 s; peak total GPU 6,205 MiB | Selected local demo default; result does not pass `MODEL-TTS-P1`. |
| Q8_0 `e2043182…` | `PASS`: Russian float32 44.1 kHz WAV; codec CPU; synthesis RTF 5.650; cold 37.88 s; peak total GPU 5,701 MiB | Retained only as quality comparator on this host. |
| Q6 HTTP `/generate` | `PASS`: HTTP 200 and valid 3.947 s float32 44.1 kHz WAV | Local finalized-WAV server example is operational. |
| Q6 chunked streaming | `PASS` transport, `REPORT_ONLY` latency: valid 2.833 s PCM16 WAV; upstream RTF 6.24 | Streaming API works but is not real-time; no TTFA claim. |

## Decisions that still constrain the work

### D-001 — External bounded experiment, not runtime integration

- **Observation:** Accepted ADR-005 requires TTS in an optional separate `ai-host`; SPEC-16/ADR-017 model-pack integration remains `Deferred Proposed`.
- **Evidence:** `docs/architecture/adr/005-offline-first-ai-process-boundary.md`, `docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md`, and `docs/architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md`.
- **Decision:** Add only a lab/demo entry point and documentation; install third-party source/model/output in a machine-local external root.
- **Rejected alternatives:** Vendoring weights or linking `s2.cpp` into a production crate would violate repository/model hygiene and prematurely implement a Proposed boundary.
- **Consequences:** The demo must remain replaceable, optional and absent from gameplay/product checks.
- **Uncertainty:** `s2.cpp` is alpha and its exact Russian quality/latency on RTX 3080 is not yet measured.
- **Reconsider when:** A production `ai-host` consumer and promoting ADR define an accepted adapter/model-pack contract.

### D-002 — Q6_K operational default with Q8_0 comparator

- **Observation:** Q8_0 transformer offload fits, but CUDA codec allocation fails and falls back to CPU; Q6_K keeps transformer and codec on CUDA with better local latency and headroom when codec auto-benchmarking is disabled.
- **Evidence:** External logs under `/home/kaifaty/.local/share/nextengine/fish-s2-pro/outputs/` and the measured rows above.
- **Decision:** The wrapper maps `q6/profile` to `--codec-follow-backend` and `q8/profile` to `--codec-cpu`.
- **Rejected alternatives:** Do not retry Q8 CUDA codec on this exact 10 GiB profile; the measured allocation failed. Q4_K_M is unnecessary until Q6 quality/headroom is rejected.
- **Consequences:** The demo avoids a known OOM probe, loads faster, and reports Q8 only as a listening comparator.
- **Uncertainty:** Subjective Russian quality and warm long-form behavior still require human listening and a bounded corpus.
- **Reconsider when:** A pinned upstream revision reduces codec residency enough to pass an explicit Q8 CUDA allocation probe.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Q8_0 transformer plus codec fits fully on RTX 3080 10 GiB | Transformer alone offloads successfully | Local codec CUDA allocation failed with OOM | Resolved false for code `2c332619…`, GGUF `a7320690…`, driver 610.43.02. |
| H2: Current `s2.cpp` produces usable Russian speech without a reference voice | Multiple valid non-silent Russian WAV files were generated | Subjective pronunciation/voice quality is not machine-validated | Human listening comparison of the delivered Q6/Q8 WAV files. |

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, `docs/architecture/README.md`, `docs/architecture/00-product-contract.md`, `docs/architecture/01-system-architecture.md`, `docs/architecture/glossary.md`.
2. `docs/architecture/adr/005-offline-first-ai-process-boundary.md`, `docs/architecture/08-audio-navigation-and-world-services.md`, `docs/architecture/11-security-licensing-and-governance.md`.
3. `docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md` and `docs/architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md` as Proposed context only.
4. Current upstream `rodrigomatta/s2.cpp`, `rodrigomt/s2-pro-gguf`, official `fishaudio/s2-pro` model card/license and Fish Speech install/inference docs.

## Next action

1. Listen to the generated Q6/Q8 examples and decide whether Fish S2 Pro merits a later corpus benchmark.
2. If continued, measure first audio correctly at the PCM-byte boundary and compare pronunciation/quality against other TTS candidates on one fixed corpus.
3. Non-regression condition: keep weights, reference voices, generated audio and logs external, and preserve `TextOnlyFallback` for any future integration.

## Do not retry

- Official BF16/PyTorch S2 Pro on this 10 GiB GPU — upstream recommends 24 GiB; reconsider only if a supported low-memory official path is published and requested.
- Vendoring model weights/source/output into NextEngine — prohibited by repository hygiene; reconsider only through a separate accepted distribution/model-pack decision.
- Q8_0 codec on CUDA for the exact pinned 10 GiB profile — local allocation failed; reconsider only after a pinned runtime/model change with a fresh allocation probe.

## Handoff

- **Workspace state:** External pinned build, Q6/Q8 files and generated outputs exist outside Git; the coherent repository change contains the wrapper, four focused tests, user guide and this task-state update.
- **Checks:** CUDA build, both CLI profiles, finalized HTTP and chunked HTTP transport passed; Ruff format/check, Python 3.12 unit tests, `git diff --check`, external path validation and the repository GGUF/WAV/MP3/S2VOICE scan passed.
- **Remaining risk:** Alpha inference engine, non-commercial model/source license, subjective Russian quality, RTF above real-time and structurally slow streaming.
- **Promotion needed:** None for a bounded lab demo; production integration would require the normal SPEC-16/ADR workflow.
