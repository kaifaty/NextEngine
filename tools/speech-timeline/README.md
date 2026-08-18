# Speech timeline service and emotion2vec+ microphone probe

This is an isolated developer PoC for the Proposed `audio-understanding` role.
It is not a gameplay dependency, does not mutate engine state, and does not
make model output authoritative.

The project is the model-neutral home for the resident speech timeline service.
`next-speech-timeline` is the service entry point; the existing
`next-emotion-probe` command and JSON schema remain available for direct model
diagnostics. Model weights, runtime builds, ready files, tokens, and captured
audio stay outside the repository.

The default model is `emotion2vec/emotion2vec_plus_base`, pinned to Hugging Face
revision `b318240bfe67db81a8c572ecb37ce9c3759b81c9`. Model weights and the Python
environment belong outside the repository under
`~/.cache/nextengine/emotion2vec-plus-base/`.

## Install

```bash
uv venv --python 3.12 ~/.cache/nextengine/emotion2vec-plus-base/venv
source ~/.cache/nextengine/emotion2vec-plus-base/venv/bin/activate
uv sync \
  --project tools/speech-timeline \
  --active
deactivate
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-emotion-probe download
```

## Record the default PipeWire microphone

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-emotion-probe \
  record --seconds 5
```

Preserve the captured WAV only when it is explicitly needed:

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-emotion-probe \
  record --seconds 5 --keep-audio /tmp/emotion-sample.wav
```

Analyze an existing file:

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-emotion-probe \
  analyze /path/to/audio.wav
```

The JSON `scores` are upstream model scores, not calibrated probabilities or
facts about a speaker's internal state. The wrapper reports them as observed
vocal expression only.

## Resident service

The service accepts one explicit-finish utterance at a time over an
authenticated `ws://127.0.0.1` listener. It loads and warms both pinned models
before publishing its ready file. The explicit JSON profile, GGUF, emotion
cache, `transcribe.cpp` checkout/library, ready file, and any diagnostic output
must all live outside the repository.

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  serve --profile /path/outside/repository/speech-timeline-profile.v1.json
```

Once model warm-up completes, the ready JSON printed to stdout contains both
`uri` and `dashboard_uri`. Open the latter (for example,
`http://127.0.0.1:43721/`) in a browser, grant microphone access, select the
exact input, and press **Начать запись**. The Vue dashboard is served by the
same loopback process as the WebSocket, obtains its short-lived token from a
same-origin `no-store` bootstrap response, and never writes raw audio.

The dashboard deliberately keeps these representations separate:

- stable and tentative Voxtral transcript revisions;
- raw emotion2vec score windows on the 16 kHz sample clock;
- smoothed observed-expression segments;
- the final utterance-level fusion result and exact model lineage.

Timeline updates carry only newly observed affect windows after the first
revision (`raw_observations_mode: "append"`); the dashboard merges them by
`observation_id`. Smoothed segments remain a replaceable projection. This
keeps long-turn event payloads bounded instead of serializing the full affect
history on every ASR revision. Use `NEXTENGINE_SPEECH_LOG_LEVEL=DEBUG` (or
`serve --log-level DEBUG`) to inspect periodic ingress, slow model jobs, event
serialization/send timings, and browser-side slow-event diagnostics.

Emotion scores remain uncalibrated observations. Phase 1 doesn't fabricate
word timestamps, so emotional tags aren't attached to individual words in this
view. The final tagged-text form is a later derived consumer view.

The profile is strict schema version 1 and contains three objects:

- `voxtral`: exact GGUF path, byte size, `sha256:` digest, `transcribe.cpp`
  root/library/revision, backend, model delay, and partial-decode interval
  (240 ms by default; legacy v1 profiles without the field keep this default);
- `emotion`: pinned model ID/revision, existing cache directory, device, and
  local-only classification;
- `service`: loopback port (`0` selects an ephemeral port), external ready-file
  path, and aligned frame/turn byte ceilings.

The ready file is created with mode `0600`, contains the random session token,
and is removed on clean shutdown. Raw PCM, transcripts, and model outputs are
not written by the service.

In a second terminal, list inputs and connect the microphone client:

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  microphone --list-inputs

~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  microphone \
  --ready-file /path/outside/repository/speech-timeline-ready.json \
  --device 'pw:<exact-node-name>' \
  --locale ru
```

The client performs the same microphone preflight as the direct Voxtral probe,
sends 80 ms PCM chunks by default, and renders replaceable transcript and
emotion timeline updates until `utterance.final`. Use Ctrl-C to cancel by
disconnecting. `--save-wav` remains an explicit diagnostic exception and
refuses repository paths or overwrite.

One Phase 1 turn is bounded to 960,000 bytes: 30 seconds of 16 kHz mono signed
16-bit PCM. Both the CLI and browser now stop capture at the advertised server
limit and finalize the turn automatically. `TURN_TOO_LARGE` means a client sent
beyond that boundary; the service treats it as one terminal input error. It is
not a model failure. Longer conversation must be split into utterances (and
later may use bounded VAD) instead of increasing an unbounded in-memory turn.
The browser asks for a 16 kHz audio context, uses a stateful fallback resampler,
disables browser AEC/noise suppression/AGC for ASR fidelity, batches worklet
messages to 20 ms, and flushes the final partial block before `session.finish`.
Transient browser/audio bursts are flow-controlled at the service's bounded
ASR ingress instead of being expanded into an unbounded model queue.

## Rebuild the Vue dashboard

The production bundle is committed inside the Python package so the installed
service has no Node.js runtime dependency. After changing `web/src`, rebuild it
with the pinned lockfile:

```bash
cd tools/speech-timeline/web
pnpm install --frozen-lockfile
pnpm run build
```

Measure sequential resident sessions with an external 16 kHz mono PCM WAV:

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  benchmark \
  --ready-file /path/outside/repository/speech-timeline-ready.json \
  --audio /path/outside/repository/sample.wav \
  --mode unpaced \
  --runs 2 \
  --out /path/outside/repository/speech-timeline-benchmark.json
```

The report is atomically replaced with mode `0600` and contains model identity,
load counts, queue/inference latency, end-to-end RTF, and p50/p95 summaries. It
omits audio and transcript content. The terminal `metrics` payload keeps the
last 64 job records on the wire, while `jobs_total` and `job_summary` retain
exact counts/totals/percentiles for long turns; any future oversized event is
returned as one terminal protocol error rather than leaving the client waiting.
