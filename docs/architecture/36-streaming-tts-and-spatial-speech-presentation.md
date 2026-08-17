# SPEC-36: Streaming TTS and spatial speech presentation

| Поле | Значение |
|---|---|
| ID | SPEC-36 |
| Статус | Proposed |
| Lifecycle | Bounded Proposed |
| Версия | 0.1 |
| Последняя проверка | 2026-08-17 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-16](16-text-canonical-multimodal-dialogue-and-model-packs.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-017](adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md), [ADR-028](adr/028-platform-session-and-presentation-authority.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-075](adr/075-bounded-streaming-tts-through-ai-host-and-audio-scene.md) |
| Заменяет | отсутствует |

## Статус и назначение

SPEC-36 проектирует один ограниченный TTS vertical:

```text
validated canonical sentence
  → optional resident ai-host
  → validated dry mono PCM segments
  → bounded speech stream
  → positioned AudioScene emitter
  → directivity + attenuation + propagation + room response
  → dialogue bus + mixer
  → ordinary device or displayless sink
```

Документ имеет статус `Proposed`: он не меняет текущий Accepted runtime,
не объявляет TTS обязательным для v1 и не выбирает shipping model. Он
специализирует TTS-часть Deferred Proposed SPEC-16/ADR-017, не принимая ASR,
LLM, remote providers либо широкий multimodal stack.

Promotion требует реального NPC-речевого consumer, fault-injection, spatial
capture, offline fallback и renderer-co-resident resource evidence. До этого
текущие `AudioSceneSnapshotV1`, `AudioEmitterRecordV1`, clip-only mixer и
desktop sink остаются implementation truth.

## Product outcome и scope guard

Игрок должен слышать сгенерированную реплику как звук персонажа в сцене:
источник следует за mouth/head attachment, меняет направление и громкость
относительно listener, учитывает occlusion/portal state и получает room
reflection/reverb. Исчезновение model process, GPU budget или audio device
может уменьшить качество presentation, но не меняет dialogue, quest, combat,
save либо replay outcome.

В scope входят:

- local separately installed TTS pack и resident isolated process;
- sentence-level requests, ordered PCM streaming, cancellation и barge-in;
- model-neutral voice profile и exact model-specific voice binding;
- один spatial speech source внутри общего `AudioScene`/mixer;
- engine-owned acoustic content/frame/result contracts с baseline fallback;
- non-blocking bounded buffers, cache/capture/replay rules;
- license, voice-consent, resource admission, diagnostics и ProductChecks.

В scope не входят:

- authoritative speech timing, phoneme или emotion state;
- LLM dialogue generation, ASR и gameplay prosody inference;
- mandatory cloud, accounts или network;
- training/fine-tuning at runtime;
- bundled model weights либо reference recordings in the base repository;
- lip-synchronization animation beyond a bounded optional timing projection;
- selection of Steam Audio, another vendor SDK or one model runtime as public
  architecture.

## Authority and sources of truth

| State | Единственный owner/source of truth | Не является authority |
|---|---|---|
| Dialogue text, speaker, commitments and transcript policy | RPG Framework after ordinary dialogue/command validation | TTS request, waveform, generated duration |
| Model route and immutable installed pack resolution | Application + Asset/Tool model-pack registry under project security policy | Provider suggestion, mutable cache path, model output |
| Model execution and temporary generation state | optional external `ai-host` | Core Runtime, audio device |
| Speech stream lifecycle, buffer position and playback clock | Presentation audio runtime | Dialogue aggregate, save/replay gameplay state |
| Emitter/listener transform and mouth attachment resolution | immutable presentation/animation projection | `ai-host`, mixer channel, backend voice |
| Acoustic zones, portals, materials and fallback room response | activated neutral content + immutable presentation frame | Steam Audio scene or device effect object |
| Gameplay hearing | deterministic acoustic facts from SPEC-08 | rendered PCM, HRTF, propagation adapter output |
| Audio device and final output queue | private platform adapter | simulation or dialogue state |

Generated PCM, stream position, propagation state, cache warmth, device
latency and model timing are `PresentationOnly`. They MUST NOT enter
authoritative state, command order, save root or gameplay replay root.

## Layering and required modules

| Layer | Proposed responsibility | Forbidden dependency |
|---|---|---|
| `crates/contracts` | nominal TTS IDs, host protocol values, pack/voice/audio-source/acoustic contracts and stable diagnostics | Python, Torch, CUDA, model/vendor, SDL or propagation SDK types |
| `crates/application` | optional process launch, handshake, health, resource admission, request/cancel routing and fallback | mutable ECS access or audio callback work |
| `crates/presentation` | speech-stream state, PCM validation/buffering, source admission, spatialization, propagation normalization, mixing and displayless capture | dialogue mutation or backend-native public values |
| platform adapter | final bounded stereo PCM sink and device recovery | TTS/model IPC or gameplay decisions |
| external `ai-host` adapter | exact model load/prewarm/inference and output conversion to negotiated PCM | world mutation, device access, arbitrary network/filesystem authority |
| tooling | explicit pack validation/install/benchmark and bounded reports when a real consumer is implemented | implicit download, unpinned aliases or generated artifacts in Git |

Rust engine crates remain safe Rust. A C/C++ propagation backend, if later
chosen, requires the ordinary narrow engine-owned FFI boundary from ADR-002;
its handles cannot cross `crates/contracts`.

## Nominal identities

All IDs below are engine-owned nominal values, not aliases for UUID strings,
process handles or vendor request IDs.

| Type | Contract |
|---|---|
| `SpeechStreamId` | Opaque 128-bit identity scoped to one presentation epoch and dialogue sentence attempt. Derived from session/epoch, `DialogueTurnId`, sentence index and attempt ordinal; wall clock and provider IDs are excluded. |
| `SpeechRequestId` | Opaque 128-bit idempotency identity scoped to one `TtsHostSessionId`; exact retry has identical canonical bytes. |
| `TtsHostSessionId` | Random or host-generated process-session identity created at successful handshake; never persisted as gameplay state. |
| `TtsPackId` | Stable namespaced identity of a separately installed immutable TTS pack; exact revision is the manifest hash. |
| `SpeechVoiceProfileId` | Model-neutral authored character voice/presentation identity. |
| `TtsVoiceBindingId` | Exact immutable binding of one voice profile to one pack, conditioning/provenance closure and generation parameter hash. |

Collision, reuse with different bytes or stale presentation epoch cancels the
affected stream and selects fallback; it never creates an alternate identity.

## Local process protocol

### Transport and handshake

The default local profile uses one supervised child process with private
stdin/stdout pipes:

- stdout contains only length-prefixed protocol frames; logs use stderr;
- each frame is `u32` little-endian length followed by `CanonicalBinaryV1`;
- canonical metadata is at most 4,096 bytes, binary PCM payload at most
  262,144 bytes and the encoded envelope at most 266,240 bytes;
- the application reads the outer length and rejects an oversized frame before
  allocating its declared payload;
- no HTTP/TCP listener is required or allowed by the default local profile;
- filesystem and network are default-deny except the private immutable pack
  closure granted at process launch; raw paths never enter the public wire
  contract.

The closed V1 message set is:

```text
Hello → LoadProfile → Ready
                 ├→ Synthesize → (SpeechSegment → SegmentAck)* → Complete
                 ├→ Cancel ─────────────────────→ Complete|Fault
                 ├→ Ping → Pong
                 └→ Shutdown
```

`Hello` negotiates one exact protocol version, supported PCM profiles, maximum
frame/request sizes and adapter build hash. `LoadProfile` binds exact pack,
model, runtime, voice-binding and output-profile hashes plus resource envelope.
`Ready` may publish only after those hashes validate, model load succeeds,
warm-up completes and joint presentation resource admission still holds.

Unknown message kind/version, malformed length, stdout noise, sequence reuse,
hash mismatch or message before `Ready` terminates the host session. Restart
always creates a new handshake/session; the application never silently
reattaches in-flight streams.

### `TtsHostEnvelopeV1`

Every frame contains:

| Field | Contract |
|---|---|
| `schema_version`, `message_kind` | exact supported version and closed enum |
| `host_session_id` | exact current session after `Hello`; zero only in initial handshake |
| `message_sequence` | monotonic `u64` per sender and session; no wrap/reuse |
| `request_id_or_none`, `stream_id_or_none` | required only for request-scoped kinds |
| `metadata_length`, `payload_length` | checked before decode/allocation and within negotiated maxima |
| `metadata_hash`, `payload_sha256` | exact hashes; empty payload has the canonical empty hash |
| `metadata`, `payload` | canonical metadata plus optional binary PCM; no native struct layout |

`SegmentAck` returns the highest contiguous validated segment sequence and
frame offset; it never acknowledges bytes that have not entered the bounded
speech ring. Exact duplicate request bytes MAY return the previous terminal
receipt or resume the still-open stream without repeating already acknowledged segments.
The same `SpeechRequestId` with different canonical bytes is a collision and
terminates that request/session route.

## Synthesis and stream contracts

### `TtsSynthesisRequestV1`

One request represents one validated sentence and contains:

| Field | Contract |
|---|---|
| identity | request/stream/turn IDs, sentence index and presentation epoch |
| source | speaker `PersistentId`, locale, NFC canonical text ≤4,096 UTF-8 bytes and exact source-text SHA-256 |
| route | exact pack manifest, adapter/runtime, output profile and `TtsVoiceBindingV1` hashes |
| generation | closed parameter profile ID, seed when supported, and exact canonical parameter hash; no arbitrary kwargs |
| timing | presentation deadline token, prebuffer profile and cancellation key; no simulation target tick derived from wall clock |
| provenance | project/content/dialogue revision and voice/license/consent record hashes required by the chosen binding |

The request MUST NOT contain world/save bytes, mutable ECS values, raw model
paths, credentials, provider session, arbitrary Python, raw voice recording or
a gameplay command. Text splitting, truthfulness and commitment admission are
owned by SPEC-16 before the request exists.

### `SpeechPcmProfileV1`

The public PCM descriptor contains closed encoding `PcmF32Le` for the initial
profile, sample rate in `8,000..=192,000`, channel count `1..=2`, interleaved
frame layout, bytes per sample/frame, maximum segment bytes and canonical
profile hash. V1 TTS admission requires the exact 48,000 Hz mono descriptor;
stereo, compressed audio or implicit host-endian floats are incompatible
rather than guessed. A later format requires a new closed profile/version and
conversion before the real-time callback.

### `SpeechSegmentCandidateV1`

This specialization retains the SPEC-16 meaning and closes the first local
wire profile:

| Field | Contract |
|---|---|
| identity/order | host session, request/stream IDs, sentence index, monotonic segment sequence starting at zero, `is_final` |
| source binding | locale, source-text hash, pack/runtime/voice-binding hashes and cancellation key |
| audio | exact `SpeechPcmProfileV1`, first frame offset, frame count, byte length, payload SHA-256 |
| optional timing | zero or one bounded `SpeechTimingTrackV1` fragment whose frame range is inside this or prior accepted PCM |
| payload | at most 262,144 bytes and exactly aligned to whole PCM frames |

The initial engine profile is `PcmF32Le`, 48,000 Hz, one channel. A host with
another native format converts before the public boundary. Every sample MUST
be finite and in `[-1.0, 1.0]`; NaN, infinity, range violation, byte/frame
mismatch or inconsistent format cancels the whole stream. The audio callback
never performs decoding, resampling or validation.

Sequence handling is exact:

- next new segment must equal the expected sequence and frame offset;
- exact duplicate bytes may be acknowledged and discarded;
- duplicate identity with different bytes, gap, overlap, format change,
  post-final data or a late segment after cancellation is rejected;
- `Complete` binds terminal result, total frames, concatenated PCM SHA-256,
  first-PCM/generation measurements and exact provenance;
- host crash or missing `Complete` leaves no successful cache artifact.

### `TtsCompletionV1`

The terminal receipt contains request/stream IDs, closed result
`Completed | Cancelled | Failed`, last accepted segment sequence, total frame
count, whole-PCM SHA-256, exact pack/runtime/voice/output hashes, wall-clock
load/first-PCM/generation measurements for diagnostics, peak process RAM/VRAM
when available and one stable failure code. `Completed` requires an accepted
final segment and matching recomputed frame/hash totals. Timing/resource fields
are measurements only; they cannot select a gameplay result or turn a failed
stream into success.

### Optional `SpeechTimingTrackV1`

A timing track contains at most 256 monotonically ordered markers per sentence.
Each marker has sample-frame offset, duration, closed kind
`WordBoundary | Phoneme | Viseme`, bounded engine semantic ID and confidence
Q16. It is presentation-only, may drive subtitle highlighting or facial
animation and cannot change dialogue timing or gameplay. Missing/invalid timing
uses an amplitude-envelope jaw fallback or no lip motion; PCM remains usable.

## Voice profiles, bindings and consent

### `SpeechVoiceProfileV1`

This model-neutral PresentationOnly content record contains:

- `SpeechVoiceProfileId`, allowed locale set and fallback text/default authored
  clip policy;
- stable delivery family and bounded style-control IDs, not a free-form prompt;
- dialogue-bus gain/loudness target, priority, directivity profile and mouth
  attachment slot ID;
- dry close-mic requirement and prohibited baked room/reverb flag;
- optional accessibility substitute voice profile;
- canonical hash and content/license provenance.

The record expresses the intended game character voice without naming Torch,
VoxCPM, speaker embeddings or vendor parameters.

### `TtsVoiceBindingV1`

The separately installed immutable binding contains profile ID/hash, exact
pack/runtime/adapter hashes, conditioning kind
`LicensedPreset | ControlledClone | VoiceDesign`, immutable conditioning asset
hashes, closed generation parameter hash, supported locales and exact license/
rights/`VoiceConsentRecord` references.

Free-form Voice Design text, if needed for a model, is immutable conditioning
data inside the hashed local binding; it is never concatenated to canonical
dialogue or exposed as a gameplay prompt. `ControlledClone` is disabled by
default and cannot activate without current scoped consent. Missing/revoked
consent invalidates new requests and selects a licensed preset, authored clip
or subtitle; already committed gameplay is unchanged.

Stable character identity and requested emotion are independent evaluation
dimensions. A waveform that is valid or expressive does not prove speaker
identity continuity.

## TTS model-pack specialization

`TtsPackProfileV1` is a role-specific record inside the immutable
`AiModelPackManifest` closure from SPEC-16. It contains:

- exact upstream source revision, model snapshot hashes, adapter build/runtime
  hashes and conversion lineage;
- supported `SpeechPcmProfileV1`, locales, binding kinds, seed/control and
  streaming capabilities;
- load/prewarm policy, maximum one-request and optional prefetch concurrency,
  request/text/segment/output limits;
- declared CPU/RAM/VRAM/disk/startup/steady-state envelope and target triples;
- code, weights, conditioning-data and voice license classifications/notices;
- compatible protocol/ProductCheck IDs and immutable result references;
- ordered fallback to another compatible TTS pack, authored voice or subtitle.

Weights, runtimes, generated PCM, caches, datasets and reference recordings
are external machine-local assets and MUST NOT enter the engine repository or
base game package. A mutable branch, model alias or unverified cache directory
cannot satisfy pack identity.

## `AudioScene` integration

### Source and emitter successor

Implementation requires a current-only successor to the clip-only public
records. The proposed shape is:

```text
AudioSourceRefV2 =
  CookedClip { clip_revision, looped }
  | SpeechStream {
      speech_stream_id,
      source_text_hash,
      voice_profile_hash,
      output_profile_hash
    }

AudioEmitterRecordV2 {
  emitter_key,
  source: AudioSourceRefV2,
  transform,
  loudness_class,
  priority_class,
  occlusion_zone_or_none,
  directivity_profile,
  bus_id,
  canonical_hash
}
```

`AudioSceneSnapshotV2` retains one listener, bounded emitters/cues and
deterministic acoustic facts, and adds exact acoustic-scene revision plus V2
source records. It stores the resolved quantized emitter transform, not a bone
handle. Extraction resolves the authored mouth/head `SchemaId` from the latest
immutable render/animation pose; missing attachment uses the subject/root
transform with a stable diagnostic.

A speech stream uses `High` priority by default but still obeys the closed
voice limit. Preemption pauses/cancels only presentation playback according to
the profile; it cannot affect the canonical sentence or committed outcome.
TTS PCM MUST NOT be registered as a new cooked asset for every segment and
MUST NOT be sent directly to the platform sink.

### Spatial direction and directivity

`AudioDirectivityProfileV1` is a closed engine value:

- `Omnidirectional`;
- `Cardioid { rear_gain_q16 }`;
- `Cone { inner_angle_q16, outer_angle_q16, outer_gain_q16 }`.

The emitter forward vector comes from the resolved mouth/head transform; the
listener basis comes from `AudioListenerRecord`. The baseline computes direct
gain, left/right or selected headphone spatializer input and directivity before
room response. Backend HRTF objects remain private and may fall back to current
stereo panning without changing source identity.

## Acoustic scene and propagation contracts

### Activated content

`AcousticSceneManifestV1` is immutable neutral presentation content bound to
the activated content manifest and contains:

- up to 256 `AcousticZoneV1` records, each binding a stable `SchemaId`, volume
  asset revision and `RoomResponseProfileV1`;
- up to 512 `AcousticPortalV1` records with stable ID, two zone IDs, aperture
  geometry revision and open/closed transmission profiles;
- up to 256 `AcousticMaterialV1` records with low/mid/high absorption,
  transmission and scattering Q16 values;
- optional content-addressed static acoustic triangle geometry with material
  slots for a propagation adapter;
- exact unit/coordinate/bounds profile and canonical hash.

`RoomResponseProfileV1` contains at most 16 bounded early-reflection taps
(delay samples plus low/mid/high gain), one late-reverb decay/damping profile
and wet-send ceiling. This makes the engine-native fallback capable of audible
room context/reflection without a proprietary SDK.

Dynamic portal openness, listener/emitter transforms and a bounded list of
presentation occluders live in `AudioPropagationFrameV1`, bound to exact
presentation epoch/sequence and acoustic-scene revision. Gameplay door state
remains owned by RPG/physics; the audio frame is only its immutable projection.

### Normalized result

Every engine-native or optional adapter produces `AudioPropagationResultV1`
per emitter:

- input epoch/sequence, acoustic scene revision and emitter key;
- direct-path gain Q16, delay in output samples and low-pass cutoff;
- zero to 16 early-reflection taps with bounded delay/gain;
- room-response profile ID and late-reverb send Q16;
- closed validity `ExactFrame | FallbackZone | Unavailable` and canonical
  diagnostic code.

Stale sequence/revision, non-finite values, excess taps/delay or unknown
material/zone rejects the result. The mixer then uses the engine-native zone/
portal fallback. Steam Audio remains one `Proposed` private adapter; no Steam
Audio type, handle or serialized blob enters these contracts. Propagation
affects presentation only. Gameplay hearing continues to use SPEC-08
deterministic acoustic facts even if the optional adapter is enabled.

## Streaming, buffering and real-time rules

### State machine

One speech stream follows:

```text
Created → Buffering → Playing → Draining → Completed
                    ↘ Rebuffering ↗
       any nonterminal → Cancelled | Failed
```

Only validated ordered segments advance write position. Playback begins after
the profile prebuffer target; the initial candidate uses two 160 ms chunks.
One bounded rebuffer is allowed. An underrun renders silence for the missing
frames and records a diagnostic; exhaustion of the rebuffer/deadline budget
cancels the stream and retains subtitle/authored fallback. Samples are never
time-stretched to manufacture synchronization.

### Buffers and threads

- IPC parsing/validation runs outside the audio render thread.
- Accepted mono F32 frames enter a preallocated single-producer/single-consumer
  speech ring. Capacity is profile-bound from 0.5 to 4.0 seconds; the initial
  target is 2.0 seconds.
- A full speech ring applies pipe/producer backpressure. If the declared
  deadline or capacity contract is exceeded, cancel the stream; do not drop or
  reorder old speech samples.
- Mixer/render drains into caller-provided preallocated buffers, applies
  spatial/propagation/bus effects and writes final 48 kHz stereo S16 frames to
  a separate bounded SPSC device ring.
- The platform callback performs no model/IPC work, heap allocation, mutex
  acquisition, decode, resample, propagation query or logging. It only drains
  the final ring and fills an underrun with silence/counters.
- A safe-Rust private SPSC implementation is required. New engine-owned
  `unsafe` is forbidden without a separate narrow ADR/allowlist.

The current `Arc<Mutex<VecDeque<i16>>>` callback, callback-local `Vec`,
clip-only source and allocation-returning `mix_tick` are explicit
implementation gaps. The proposed successor provides `render_into`, bounded
preallocated voice/effect state and counters read outside the callback.

Cancellation closes admission, removes the emitter at the next presentation
boundary, flushes unplayed PCM and rejects every late segment. Barge-in cannot
roll back dialogue commands/events.

## Mix, room context and device path

Speech uses the ordinary dialogue bus:

1. per-stream gain and optional loudness normalization;
2. mouth directivity and distance attenuation;
3. pan/HRTF input and normalized propagation direct path;
4. early reflections plus room late-reverb send;
5. dialogue ducking/voice limit and final stereo mix;
6. final sink queue and device recovery.

An unavailable propagation adapter falls back to authored room response;
unavailable room content falls back to distance/panning/directivity; unavailable
device yields silence. None of these fallbacks reissue synthesis or change
gameplay.

Displayless audio uses the same source, stream, propagation normalization and
mixer code with canonical PCM/WAV output. Hardware callback timing is not a
correctness oracle.

## Resource admission and scheduling

`TtsResourceEnvelopeV1` declares maximum host RAM, model VRAM, temporary VRAM,
CPU workers, disk, load/prewarm time, segment/output buffers and concurrency.
`PresentationResourceAdmissionV1` compares the exact TTS envelope with the
active renderer/audio/capture reservation and a fixed safety margin before
launch and before `Ready` publication.

Separate process isolation does not isolate GPU memory. If the combined
envelope does not fit, the application MUST NOT load that route and selects a
smaller/local authored/subtitle fallback. Runtime OOM/crash invalidates the
host session and does not cause blind retry during the same sentence.

The first candidate profile allows:

- one resident model process;
- one active request;
- at most one fully bounded prefetched next sentence after truthfulness
  validation;
- prewarm during an explicit loading/presentation phase, never a fixed tick;
- deterministic route choice for the duration of one dialogue turn.

Throughput, cold load and warm generation are reported separately. A first
PCM mean, standalone VRAM peak or single sentence cannot be presented as p95
corpus/co-residency evidence.

## Cache, capture, persistence and replay

The optional local PCM cache key is the hash of:

```text
protocol + pack + model + runtime + adapter + voice binding + locale
+ canonical text + generation parameters + output PCM profile
```

Publication is staging → full stream/whole-PCM hash validation → atomic cache
entry. Partial/cancelled/failed output is not cached. Cache path, eviction,
warmth and hit rate are private PresentationOnly state.

`SpeechPcmArtifactRefV1` may bind stream/source/voice/protocol hashes, exact
PCM profile, frame count and whole-PCM hash in an optional capture package.
It contains no local absolute path. Raw PCM/WAV is generated evidence and is
not committed to the engine repository.

- Save stores canonical dialogue state/text according to transcript policy,
  not TTS progress or cache.
- Gameplay replay MUST NOT launch `ai-host` or synthesize again.
- Ordinary replay uses recorded canonical text plus subtitles/authored voice.
- An explicit audiovisual capture replay MAY consume an exact recorded PCM
  artifact; missing bytes degrade to text and mark capture audio unavailable.
- PCM, propagation, lip timing and device counters are excluded from gameplay
  roots; pinned displayless audio hashes remain separate presentation evidence.

## Security, privacy and failure semantics

All host messages, model outputs, pack files, voice bindings, conditioning
assets and propagation results are untrusted under SPEC-11. Validate schema,
version, lengths, counts, hashes, locale, finite numeric ranges, consent and
resource envelope before allocation/publication/use.

| Stable code | Required result |
|---|---|
| `TTS_PROTOCOL_MISMATCH` | terminate host session; do not decode as another version; fallback |
| `TTS_FRAME_TOO_LARGE` | reject before payload allocation; terminate offending session |
| `TTS_PACK_HASH_MISMATCH` | reject load/route and retain prior registry entry |
| `TTS_RESOURCE_UNAVAILABLE` | do not launch/load or retry blindly; select next route |
| `TTS_REQUEST_ID_COLLISION` | cancel request/session route; no alternate ID |
| `TTS_STREAM_SEQUENCE_INVALID` | cancel whole stream; preserve canonical text/gameplay |
| `TTS_PCM_INVALID` | reject non-finite/range/format/hash-invalid PCM and cancel stream |
| `TTS_BUFFER_OVERFLOW` | apply bounded backpressure, then cancel on budget exhaustion; no sample drop |
| `TTS_DEADLINE_EXCEEDED` | close admission, cancel best-effort and use authored/subtitle fallback |
| `TTS_CANCELLED` | flush/detach stream and reject late chunks idempotently |
| `TTS_VOICE_CONSENT_INVALID` | disable clone/reference route; use licensed voice/subtitle |
| `TTS_HOST_CRASHED` | invalidate host session and in-flight streams; gameplay continues |
| `AUDIO_PROPAGATION_UNAVAILABLE` | use engine-native zone/portal/room fallback |
| `AUDIO_DEVICE_UNAVAILABLE` | render/capture when requested or remain silent; never affect gameplay |

Diagnostics contain stable IDs, versions, hashes, bounds and aggregate timing/
resource counters. Canonical dialogue, Voice Design text, reference voice,
credentials, absolute paths and raw/generated PCM are default-redacted.

## Candidate profile: VoxCPM2 PyTorch quality path

The first bounded implementation candidate is not a normative model choice.
Current same-host evidence on RTX 3080 records:

| Observation | Current evidence | Architectural consequence |
|---|---|---|
| immutable candidate identity | official source release `2.0.3` commit `19b6bf7590025418821a86dcb817504e0ad7e5df`; model snapshot `bffb3df5a29440629464e5e839f4d214c8714c3d`; Apache-2.0 | sufficient for a lab binding; a production `TtsPackProfileV1` must additionally close the exact runtime/adapter/environment and notices |
| compiled 10-step warm generation | RTF `0.707`, `8,286 MiB` peak total GPU | faster than real time, but does not meet SPEC-16 `MODEL-TTS-P1` RTF `≤0.5` |
| streaming | mean first PCM `56.9 ms`, full RTF `0.745`, `8,303 MiB` peak | useful prebuffer candidate; mean is not p95 evidence |
| observed long-sample peak | `8,455 MiB` total GPU | renderer coexistence on a 10 GiB board is unproven and must fail closed at joint admission |
| Nano-vLLM CUDA Graph alternative | RTF `0.249`, first PCM `149.8 ms`, `9,271 MiB` | throughput reference only; user listening found worse intonation |
| emotion/identity suite | joy perceived but voice changed; subtle anger/sarcasm failed | reference-free Voice Design is discovery-only, not a stable character preset |

Therefore `voxcpm2-pytorch-quality-v1` MAY be an experimental
`QualityFirstLocal` binding for the first vertical, with 48 kHz mono F32,
single active request and subtitle fallback. It MUST NOT be called shipping
default until the complete checks below pass. Nano-vLLM is not an automatic
fallback because its quality and higher VRAM are different product tradeoffs.

## Product checks and promotion evidence

| Check ID | Scenario | Expected behavior | Fallback |
|---|---|---|---|
| `TTS-PROTOCOL-P1` | handshake, exact retry/collision, bad version/length/hash, stdout noise, crash/restart and cancel permutations | bounded decode before allocation, one terminal request result, fresh session after restart and stable diagnostics | disable host route; authored voice/subtitle |
| `TTS-STREAM-P1` | ordered stream plus duplicate/gap/overlap/post-final/invalid-float/overflow/underflow/barge-in faults | no reordering/drop, bounded prebuffer/rebuffer, late bytes rejected and callback remains non-blocking | cancel affected stream; subtitle |
| `TTS-SPATIAL-P1` | one moving/talking NPC across listener headings, distances, mouth orientations, two zones and open/closed portal in displayless capture | source follows mouth transform; direction/directivity, occlusion and room response change PCM within declared exact/tolerance assertions; no direct sink bypass | baseline distance/pan/directivity, then silence/subtitle |
| `TTS-OFFLINE-P1` | complete representative dialogue with no pack/process/network/device | dialogue/quest hashes and mandatory loop match; no tick waits for TTS | authored text/subtitle and optional authored clip |
| `TTS-REPLAY-P1` | record, Save/Load and gameplay replay with different/missing TTS installation | replay never launches TTS; gameplay roots remain exact; optional capture uses only exact recorded PCM | subtitle and audio-unavailable capture diagnostic |
| `TTS-RESOURCE-P1` | active renderer + resident model + audio/capture on declared Windows/Linux profiles, including OOM/device/host fault | joint admission prevents oversubscription, queue remains bounded, no callback stall and declared latency/VRAM tails are reported | reject route before load or downgrade explicitly |
| `TTS-VOICE-P1` | licensed preset and authorized fixed-reference neutral/anger/joy corpus with pronunciation/stress cases | speaker identity, emotion, intelligibility and artifact scores meet the declared profile; revoked/missing consent never clones | licensed default voice/subtitle |
| `MODEL-TTS-P1` | inherited SPEC-16 p95 corpus benchmark | first PCM p95 `≤500 ms`, RTF `≤0.5`, bounded continuation and valid schema | another declared route, authored voice/subtitle |

`TTS-PROTOCOL/STREAM` use focused `fast`; the production NPC and spatial/
offline path use `play`; replay semantics use `persistence-replay`; pack,
license and voice records use `content-package`; joint GPU/latency evidence is
conditional `performance` and platform evidence is conditional on affected
host/device code. Human listening is bounded evidence for voice quality, not a
replacement for protocol/spatial correctness.

Promotion from `Proposed` requires all of the following in one coherent
consumer:

1. one real project NPC sentence enters `AudioScene` through production
   application/ai-host paths and never bypasses the mixer;
2. authored subtitle/default-voice fallback works with absent/crashed host,
   missing GPU and lost device;
3. reordered/malformed/cancelled stream and host restart checks pass;
4. displayless spatial capture demonstrates mouth attachment, direction,
   directivity, portal occlusion and room response;
5. renderer-co-resident Windows and Linux resource profiles are measured, with
   honest `NOT_RUN` where unavailable;
6. exact pack/runtime/voice license and consent closure passes;
7. model-quality gate is reported independently; failure cannot be hidden by
   passing integration checks.

## Implementation slices

| Slice | Smallest production change | Completion signal |
|---|---|---|
| T0 contracts | nominal IDs, host envelopes, request/segment/completion, pack/voice values and stable diagnostics in `crates/contracts` | focused canonical/negative vectors; no vendor type boundary leak |
| T1 host | supervised local process, load/prewarm/readiness, one request, cancel/crash and authored fallback in `crates/application` | `TTS-PROTOCOL-P1` |
| T2 stream/mixer | `AudioSourceRefV2`, speech ring, `render_into`, V2 emitter/source admission and displayless sink | `TTS-STREAM-P1` with exact simple PCM control |
| T3 scene sound | mouth attachment/directivity plus acoustic scene/frame/result, room response and optional adapter seam | `TTS-SPATIAL-P1` |
| T4 device | preallocated non-locking final SPSC callback path and bounded recovery counters | callback fault/underrun test; conditional `platform` |
| T5 content/tooling | separately installed pack/voice binding validation, consent/license and local cache/capture | `content-package`, `TTS-VOICE-P1` |
| T6 product consumer | one reference-game NPC sentence, offline/replay/resource profiles and quality report | remaining TTS checks; only then consider promotion ADR/status change |

There is no requirement to implement T0–T6 in one commit. Each slice keeps the
current clip-only/audio/text fallback working and introduces no generic job,
network or multimodal framework before its consumer.

## Complexity and boundedness

- IPC decode and validation are `O(frame bytes)` under fixed frame bounds.
- Stream memory is `O(active streams × configured ring capacity)`; initial
  profile admits one generated stream and one bounded prefetched sentence.
- Mixing is `O(output frames × admitted voices + reflection taps)` with at
  most current voice limit and 16 taps per emitter.
- Acoustic content and dynamic-frame collections have explicit maxima;
  adapter acceleration structures are reconstructible private caches.
- No queue, cache, diagnostic transcript, retry or host restart loop is
  unbounded.

## Current-only evolution

All proposed V1/V2 values are pre-v1 current-only under ADR-046. The first
implementation may replace unconsumed proposed shapes directly. When the
production consumer promotes this design, it must update SPEC-08/SPEC-16,
ADR-075 status, index/routing/traceability and exact contracts together. A
recognizable retired version fails typed unsupported; it is not auto-migrated
or guessed.
