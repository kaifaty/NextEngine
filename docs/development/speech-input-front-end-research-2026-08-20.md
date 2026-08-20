# Speech-input front-end for dialogue, whisper and affect — 2026-08-20

## Status and decision

This is a bounded research result for the optional local
`SpeechTimelineService` prototype. It does not promote Deferred Proposed
SPEC-16/ADR-017, select a shipped dependency or change gameplay authority.
ADR-005 remains the Accepted boundary: ASR/audio understanding stay optional,
isolated and replaceable, with text input as the fallback.

The current always-selected `gain pre -> DPDFNet -> gain post` ASR branch must
not become the default. The user's paired listening test finds raw capture
cleaner than processed capture. That result is consistent with the exact active
profile: both gain stages target -26 dBFS, allow up to +32 dB, and the output
limiter permits peaks up to -1 dBFS. The earlier increase in retained frames and
RMS proved only that more energy survived, not that speech became more natural
or more recognizable.

The new baseline is therefore:

1. raw PCM remains the lossless control, immediate bypass and diagnostic source;
2. processing is a staged, capability-gated front-end rather than one denoiser;
3. ASR, activity/endpointing and affect receive different derived branches;
4. no processed branch becomes default until it beats raw on ASR, whisper
   admission, perceptual speech quality and latency together.

Here `raw` means the PCM delivered to our service before our own DSP, not a
guarantee of untouched ADC samples: device firmware, Bluetooth transport or the
OS may already have transformed the signal outside our observable boundary.

## Exact observation in the current prototype

- Browser capture asks for mono input with `echoCancellation`,
  `noiseSuppression` and `autoGainControl` disabled. The W3C contract describes
  these as user-agent processing controls; actual settings are already exposed
  in the dashboard and must remain recorded per take.
- The service receives 16-kHz mono PCM on one integer sample clock.
- The energy VAD reads raw PCM in 20-ms frames. Quiet calibration only moves its
  RMS thresholds; it does not make that detector whisper-aware.
- Vocal affect reads raw, VAD-admitted windows. This correctly avoids strong
  denoising and gain, but known game/TTS playback can still contaminate it.
- ASR alone receives the optional resident DPDFNet branch. The active profile is
  `pre_and_post_denoise`, target -26 dBFS, max gain +32 dB and limiter -1 dBFS.
- Pinned `dpdfnet==0.6.0` exposes only model/path/verbosity at construction and
  no online attenuation-strength control. Current upstream added an
  `--attn-limit-db` blend to its file-oriented ONNX/TFLite examples, explicitly
  marked offline-only; it is not evidence that our pinned online adapter has
  such a control.
- The branch preserves exact sample count and is fast enough on the present CPU,
  but subjective raw preference falsifies the assumption that energy retention
  plus real-time execution is sufficient acceptance evidence.

## What smart speakers actually solve

There is no single universal smart-speaker pipeline, but production systems
repeatedly combine the following distinct problems.

### 1. Capture hardware and spatial selection

Far-field devices commonly own a calibrated microphone array. They can estimate
direction, beamform, dereverberate and jointly train multichannel processing with
the acoustic model. Google Home reported an adaptive dereverberation frontend
and joint multichannel acoustic modeling; the combination reduced WER by more
than 18% relative in its evaluated production setting. Amazon and Google papers
likewise describe multichannel frontends and models trained on device-specific
room/noise distributions.

This benefit cannot be recreated from our current mono browser stream: inter-mic
phase and geometry were already discarded. Beamforming must therefore be an
optional future capability only when the native host exposes synchronized raw
channels and array geometry. Mono devices use the baseline path.

### 2. Acoustic echo cancellation with a known render reference

A smart speaker knows the exact samples it is playing. Its AEC models the path
from loudspeaker to microphone, subtracts the estimated echo and then suppresses
residual echo/noise. Amazon describes this as a multilayer system rather than a
generic noise gate.

WebRTC APM formalizes the same boundary: it consumes capture frames and a
reverse/render stream in 10-ms linear-PCM chunks; the render stream is necessary
when echo processing is enabled, and capture/render delay must be supplied.
Its current implementation orders high-pass analysis, echo cancellation, noise
suppression and gain processing rather than applying unrelated gain stages
around a denoiser.

For Next Engine this is especially valuable during TTS, music and game-audio
playback because the engine owns the `AudioScene` render mix. AEC must be enabled
only when an aligned engine render reference exists. A denoiser without that
reference cannot reliably distinguish the user's speech from clean TTS speech.

### 3. Device-directed speech, not merely “some acoustic energy”

Wake-word systems use an explicit activation event and may use the wake-word
acoustics as a temporary speaker anchor. Amazon reported up to 15% relative WER
reduction in interfering speech by conditioning recognition on that anchor.
Follow-up modes combine acoustics, ASR confidence/lexical evidence and sometimes
vision to decide whether speech is directed at the device.

The present push-to-talk interaction already provides a stronger explicit
intent gate than a wake word. We do not need keyword spotting now. We still need
speech activity inside the pressed interval so that silence/noise is not sent to
emotion or used to finalize an empty utterance.

### 4. Speech activity and endpointing are separate decisions

Frame VAD answers “is speech present now?” Endpointing answers “has the user
finished the query?” Google reports that a fixed silence interval is suboptimal:
an endpointer can use acoustic history, speaking rate, fillers and ASR evidence
to distinguish a hesitation from the end of a turn. Amazon's two-pass endpointer
similarly verifies a first acoustic endpoint with segment-level acoustic and
recognition information.

The current explicit Finish button remains correct for the prototype. Automatic
finish should later combine neural speech probability, hysteresis/hangover and
ASR stability; it should not be introduced as another RMS timeout.

### 5. Whisper is a phonation mode, not just quiet normal speech

Amazon's far-field whisper work describes absent periodic vocal-fold excitation,
less low-frequency energy and different spectral/formant characteristics. Its
detector uses sequential log-filterbank evidence and utterance-level posterior
trajectories, not absolute loudness alone. Whisper-activity research also notes
that the pitchless, noise-like signal makes ordinary VAD harder.

The ASR problem remains after successful gain: a 2023 study attributes much of
the mismatch to missing glottal information and obtains an 18.2% relative WER
improvement only after adding pseudo-whispered training data. Therefore neither
microphone calibration nor a stronger denoiser can guarantee whispered ASR. We
need to measure three independent failures: capture audibility, speech admission
and acoustic-model recognition.

## Proposed model-neutral front-end

```text
host capture PCM + exact sample clock + device/settings metadata
             │
             ├────────────── source/raw PCM ───────────────┐
             │                                             │
             ▼                                             ▼
  format validation / channel map / DC-HPF          diagnostic A/B buffer
             │                                      (explicit opt-in only)
             ▼
  AEC when aligned AudioScene render reference exists
             │
             ├── optional array beamform/dereverb
             │   (only synchronized raw multichannel input)
             │
             ├── activity branch: AEC/light PCM -> neural speech probability
             │      -> hysteresis + pre-roll + hangover -> speech mask
             │
             ├── ASR branch: conservative bounded NS -> one speech-gated gain
             │      -> headroom limiter -> resident streaming ASR
             │
             └── affect branch: raw when no playback; AEC-only during playback
                    -> speech-mask windows -> emotion model

speech probability + ASR stability -> endpoint candidate/arbitrator
all branches retain source sample intervals and explicit algorithmic delay
```

### Stage contracts

| Stage | Required behavior | Bypass/failure |
| --- | --- | --- |
| Capture | Preserve native facts, monotonic sample identity and actual browser/native settings. Do not retain raw audio by default. | Reject malformed PCM; never synthesize missing time. |
| Minimal conditioning | Optional DC/high-pass/channel mapping only; exact config and delay are observable. | Identity transform. |
| AEC | Consume the exact render reference, capture/render delay and double-talk state. | Disabled without a trustworthy reference; raw/light path continues. |
| Spatial | Use channel geometry only when the host supplies synchronized raw channels. | Mono baseline; never advertise beamforming on downmixed PCM. |
| Activity | Emit continuous speech probability plus versioned segments, not only a binary RMS result. Keep pre-roll so quiet onsets are recoverable. | Explicit PTT interval and raw ASR remain available; affect abstains without sufficient speech evidence. |
| Enhancement | Constrain maximum attenuation and/or wet/dry mix; do not erase plausible low-energy speech. | Per-utterance raw ASR route, with a stable diagnostic. |
| Gain | Exactly one adaptive digital stage, updated only from admitted speech and bounded for noise, slew and headroom. | Unity gain. No pre+post duplication. |
| Affect | Preserve prosody: no general AGC or strong NS. AEC-only is allowed when removing engine-owned playback. | Raw speech windows or abstention. |
| Endpoint | Fuse temporal activity and ASR evidence; endpoint decisions never change the acoustic clock. | Explicit Finish remains authoritative for the prototype. |

All public capabilities stay engine-owned: sample rate/channels, stage IDs,
configuration hash, algorithmic delay, branch routing and bypass reason. WebRTC,
Silero, DPDFNet, DeepFilterNet or RNNoise types remain adapter internals.

## Candidate components and their bounded role

| Candidate | Useful role | Important limit | Research decision |
| --- | --- | --- | --- |
| WebRTC APM | First classical baseline for AEC, moderate NS and a single AGC; standalone 10-ms processing and render-reference API. | C++/FFI integration and tuning; its VAD is not accepted as whisper proof. | Evaluate first when an engine render reference is available. Do not enable every module blindly. |
| Silero VAD v6 | Small streaming 8/16-kHz neural VAD with ONNX path; repository reports sub-ms processing for 30+ ms chunks on one CPU thread. | General speech detector, not a published whisper-specific guarantee; thresholds and temporal post-processing remain ours. | First neural replacement candidate for the current RMS gate, gated by whisper recall/false alarms. |
| DPDFNet | Existing fast 16-kHz causal enhancement candidate. | Current pinned online API lacks attenuation control; current double-gain output is subjectively worse than raw. | Keep only as a diagnostic candidate. Re-evaluate with bounded attenuation/wet-dry and one gain stage. |
| DeepFilterNet3 | Full-band 48-kHz real-time Rust implementation with an attenuation-limit control. | More resampling/integration for current 16-kHz ASR; a stronger model can still over-suppress. | Second enhancement A/B candidate after the harness exists. |
| RNNoise | Small recurrent 48-kHz noise-suppression baseline with permissive code license. | Not whisper-specific and lower-capacity; requires exact model/provenance evaluation. | Low-resource reference, not assumed quality winner. |
| Whisper-mode classifier | Route/annotation based on sequential spectral evidence. | Adds no missing speech by itself and requires representative Russian microphone data. | Defer until the neural VAD/ASR corpus shows a separable whisper failure. |

No component gets permission to mutate system audio configuration or create a
virtual microphone. A future native implementation receives PCM from the game
host and returns derived PCM/metadata in process or through the existing bounded
host-service seam.

## Competing hypotheses and discriminating experiments

| Hypothesis | Evidence for | Evidence against / uncertainty | Smallest discriminator |
| --- | --- | --- | --- |
| H1: current degradation is primarily overprocessing from double gain plus unconstrained DPDF suppression | Raw is preferred; active profile allows +32 dB twice and peaks near -1 dBFS; DPDF previously removed most whisper frames before pre-gain. | The processed signal retains much more whisper energy and exact timing. | Blind paired raw/current/one-gain playback plus WER and P.835 SIG/BAK/OVRL on identical takes. |
| H2: current whisper failure begins at RMS VAD | The detector is energy-only; whisper has reduced energy and lacks normal pitch cues. | Quiet calibration admitted some tested whisper after threshold lowering. | Annotate speech intervals and measure frame recall, onset loss and false speech for RMS versus Silero on raw/AEC-light PCM. |
| H3: Voxtral itself has a whisper-domain mismatch | Whisper-ASR literature shows gain cannot restore absent glottal cues. | No labelled Russian whisper WER exists for our exact model/device. | Fixed Russian prompt corpus: raw admitted normal versus whisper WER/CER after loudness-matched capture. |
| H4: AEC is more valuable than generic denoising during actual gameplay | Smart-speaker systems use the exact playback reference; game/TTS speech otherwise resembles the target. | Current dashboard tests mostly isolated microphone input. | Replay identical utterances with controlled game/TTS render at several levels, with exact-reference AEC on/off. |
| H5: one processed stream cannot optimize both ASR and emotion | NS/AGC can alter spectral/prosodic cues; current raw affect branch avoids that. | AEC may improve affect during playback by removing known interference. | Affect macro-F1/abstention on raw, AEC-only and ASR-enhanced branches with aligned labels. |

## Evaluation protocol: “better than raw” means multiple outcomes

Use consented, externally stored test audio; do not commit recordings. Record at
least ten fixed Russian phrases per speaker/condition and keep source/derived
files paired by content hash:

- normal near-field in a quiet room;
- natural whisper near-field, not merely a quiet voiced sentence;
- normal and whisper with stationary fan noise;
- normal and whisper with keyboard/transient noise;
- speech during controlled game audio and TTS playback;
- headset/Bluetooth and desktop/analog microphone profiles where available;
- silence/no-user intervals for false activation.

Primary measurements:

1. **ASR:** WER/CER, first stable partial, final latency and revision churn.
2. **Activity:** speech-frame recall, false-positive duration per silent minute,
   start/end boundary error, whisper recall and truncated leading phonemes.
3. **Perception:** blinded raw-versus-processed preference and P.835-style SIG
   (speech), BAK (background) and OVRL. DNSMOS is a useful no-reference proxy,
   not a replacement for listening.
4. **Over-suppression:** voiced-frame energy loss, missing speech duration and
   target-speech over-suppression. Microsoft explicitly identifies TSOS as a
   deployment-critical failure mode.
5. **Signal safety:** clipping percentage, peak/headroom, applied gain trajectory,
   output/input speech-energy ratio and discontinuities at stage/bypass changes.
6. **Runtime:** algorithmic delay, per-10/20/80-ms p50/p95 processing time, RTF,
   CPU/RAM, queue depth, failures and raw-bypass success.
7. **Affect:** macro-F1/abstention and transition timing on speech-only windows;
   silence and rejected speech cannot vote in the final utterance emotion.

A route may become the local default only if its predeclared comparison passes
all of these gates:

- statistically meaningful ASR improvement in its target noisy/whisper cohort;
- no material clean-speech or normal-voice regression;
- no worse whisper activity recall or onset truncation;
- human speech-quality preference does not favor raw;
- bounded real-time processing and exact sample/delay accounting;
- any affect-branch change passes its own labelled gate.

Do not optimize the candidate on the final evaluation set. Split a small tuning
set for thresholds and keep a frozen holdout across raw and all candidate routes.

## Implementation sequence after approval

1. Make raw ASR the selected baseline again; keep the current DPDF route only as
   an explicit dashboard A/B option. This is the immediate rollback, not a claim
   that raw is universally optimal.
2. Add a repeatable front-end benchmark that consumes one recorded source and
   produces route-paired WAV, VAD probabilities/segments, ASR text/latency,
   signal-safety metrics and blind playback IDs.
3. Replace the binary energy gate behind `VoiceActivityDetector` with a neural
   probability adapter plus the same integer-clock pre-roll/hysteresis contract.
   Evaluate raw and AEC-light input; retain RMS as fallback.
4. Introduce an engine-owned `AudioFrontEnd` branch graph. At first its stages
   are identity/raw, current DPDF diagnostic and one-gain variants; no vendor
   type crosses the seam.
5. Add WebRTC APM/AEC only together with an exact `AudioScene` render tap and
   measured capture/render delay. Keep browser-UA AEC as a separate opaque UX
   comparison, not the native-engine proof.
6. Trial conservative enhancement: one gain stage, lower gain/limiter grid and
   an attenuation-limited or wet/dry denoiser. Select settings on the tuning
   corpus, not by making one whisper loud.
7. Add whisper-mode routing only if labelled failures remain after steps 1-6 and
   show that a dedicated classifier/ASR profile can close them.

LLM, TTS orchestration and FunctionGemma remain out of scope for this cycle.
The only TTS-related work here is providing its render samples to future AEC.

## Rejected approaches

- RMS or output loudness as the acceptance metric for speech enhancement.
- Always-on maximum suppression followed by enough gain to make its residue loud.
- Two adaptive digital gain stages around the denoiser.
- One heavily processed PCM branch for ASR, VAD and emotion.
- AEC without an exact render reference and delay accounting.
- Beamforming claims from an already downmixed mono stream.
- Treating whisper as ordinary speech multiplied by a constant.
- Replacing explicit Finish with a fixed silence timeout before endpoint evidence.
- Installing PipeWire filters or virtual microphones for the embedded path.

## Primary sources

- [WebRTC Audio Processing Module](https://webrtc.googlesource.com/src/+/7c793a7dbe548735fe9e1d107e00d17937202f47/modules/audio_processing/g3doc/audio_processing_module.md)
  and its [10-ms capture/render API](https://webrtc.googlesource.com/src/+/8c51f2e9cde4ca72ae2f84bbbe3638540b84e565/modules/audio_processing/include/audio_processing.h).
- Amazon: [multilayer echo cancellation and voice enhancement](https://www.amazon.science/blog/amazon-scientist-outlines-multilayer-system-for-smart-speaker-echo-cancellation-and-voice-enhancement),
  [far-field LSTM whisper detection](https://assets.amazon.science/84/87/1d305099496dae49712a21df7022/lstm-based-whisper-detection.pdf),
  [anchored speech recognition](https://www.amazon.science/publications/end-to-end-anchored-speech-recognition),
  and [two-pass endpoint detection](https://assets.amazon.science/4b/7c/0903caee4ab2a19c5f185b0a5c1c/two-pass-endpoint-detection-for-speech-recognition.pdf).
- Google Research: [Acoustic Modeling for Google Home](https://research.google/pubs/acoustic-modeling-for-google-home/),
  [room simulation for far-field Google Home ASR](https://research.google/pubs/generation-of-large-scale-simulated-utterances-in-virtual-rooms-to-train-deep-neural-networks-for-far-field-speech-recognition-in-google-home/),
  and [improved end-of-query detection](https://research.google/pubs/improved-end-of-query-detection-for-streaming-speech-recognition/).
- Lin, Patel, Scharenborg: [pseudo-whispered augmentation for whispered ASR](https://arxiv.org/abs/2311.05179).
- [Silero VAD](https://github.com/snakers4/silero-vad),
  [DPDFNet](https://github.com/ceva-ip/DPDFNet),
  [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet), and
  [RNNoise](https://github.com/xiph/rnnoise) official repositories.
- Microsoft: [DNS Challenge P.835 plus word accuracy methodology](https://github.com/microsoft/DNS-Challenge),
  [DNSMOS](https://www.microsoft.com/en-us/research/publication/dnsmos-a-non-intrusive-perceptual-objective-speech-quality-metric-to-evaluate-noise-suppressors-2/),
  and [target-speaker over-suppression evaluation](https://www.microsoft.com/en-us/research/publication/personalized-speech-enhancement-new-models-and-comprehensive-evaluation/).
- [W3C Media Capture and Streams](https://www.w3.org/TR/mediacapture-streams/)
  for browser capture-processing constraints and actual settings.

## Smallest next action

Restore raw ASR as the selected control and add route selection plus a paired
benchmark before another denoiser or gain change. The first corpus run should
compare raw, the current DPDF double-gain route and a one-gain conservative
route on the same normal/whispered Russian takes. Only then should Silero VAD
and render-reference AEC be implemented as independent, falsifiable increments.
