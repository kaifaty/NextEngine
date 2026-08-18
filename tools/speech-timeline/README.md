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
