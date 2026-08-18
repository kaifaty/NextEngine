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

The profile is strict schema version 1 and contains three objects:

- `voxtral`: exact GGUF path, byte size, `sha256:` digest, `transcribe.cpp`
  root/library/revision, backend, and delay;
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
sends 250 ms PCM chunks by default, and renders replaceable transcript and
emotion timeline updates until `utterance.final`. Use Ctrl-C to cancel by
disconnecting. `--save-wav` remains an explicit diagnostic exception and
refuses repository paths or overwrite.
