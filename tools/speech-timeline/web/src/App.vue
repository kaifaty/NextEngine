<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";

import EmotionTimeline from "./components/EmotionTimeline.vue";
import TranscriptPanel from "./components/TranscriptPanel.vue";
import {
  BrowserMicrophoneCapture,
  calibrateMicrophoneNoise,
  listAudioInputs,
  type AudioCaptureInfo,
  type NoiseCalibration,
} from "./lib/audioCapture";
import { emotionColor, emotionLabel, formatDuration } from "./lib/display";
import {
  loadBootstrap,
  loadDiagnosticAudio,
  SpeechTimelineClient,
  type AsrAudioRoute,
} from "./lib/speechClient";
import type {
  ConnectionState,
  DiagnosticAudioRecord,
  DashboardBootstrap,
  FinalUtterance,
  JsonObject,
  SpeechEvent,
  TimelineUpdate,
} from "./types";

const state = ref<ConnectionState>("loading");
const bootstrap = ref<DashboardBootstrap | null>(null);
const client = ref<SpeechTimelineClient | null>(null);
const capture = ref<BrowserMicrophoneCapture | null>(null);
const devices = ref<MediaDeviceInfo[]>([]);
const selectedDevice = ref("");
const selectedAsrAudioRoute = ref<AsrAudioRoute>("raw");
const timeline = ref<TimelineUpdate | null>(null);
const finalUtterance = ref<FinalUtterance | null>(null);
const events = ref<SpeechEvent[]>([]);
const errorMessage = ref("");
const notice = ref("");
const sentBytes = ref(0);
const inputRms = ref(0);
const captureInfo = ref<AudioCaptureInfo | null>(null);
const firstTranscriptLatencyMs = ref<number | null>(null);
const firstAffectLatencyMs = ref<number | null>(null);
const calibration = ref<NoiseCalibration | null>(null);
const diagnosticAudio = ref<DiagnosticAudioRecord[]>([]);
let limitStopScheduled = false;

const isRecording = computed(() => state.value === "recording");
const canStart = computed(() => ["ready", "complete", "error"].includes(state.value));
const canCalibrate = computed(() => ["ready", "complete", "error"].includes(state.value));
const currentSamples = computed(() => sentBytes.value / 2);
const durationMs = computed(() => (sentBytes.value * 1_000) / 32_000);
const limitMs = computed(() => client.value?.bounds?.maxTurnDurationMs ?? 30_000);
const progress = computed(() => Math.min(100, (durationMs.value / limitMs.value) * 100));
const levelDb = computed(() => 20 * Math.log10(Math.max(inputRms.value, 0.00001)));
const currentExpression = computed(
  () =>
    finalUtterance.value?.observed_vocal_expression ??
    timeline.value?.fusion.observed_vocal_expression ??
    "unknown",
);
const diagnosticAudioEnabled = computed(
  () => objectValue(bootstrap.value?.service, "diagnostic_audio")?.enabled === true,
);
const availableAsrAudioRoutes = computed<AsrAudioRoute[]>(() => {
  const routing = objectValue(bootstrap.value?.service, "asr_audio_routing");
  const routes = routing?.available_routes;
  if (!Array.isArray(routes)) return ["raw"];
  return routes.filter(
    (route): route is AsrAudioRoute => route === "raw" || route === "enhanced",
  );
});
const enhancedRouteAvailable = computed(() =>
  availableAsrAudioRoutes.value.includes("enhanced"),
);

const stateText: Record<ConnectionState, string> = {
  loading: "Загрузка интерфейса",
  ready: "Сервис готов",
  connecting: "Подключение",
  calibrating: "Калибровка микрофона",
  recording: "Идёт запись",
  finalizing: "Финальный анализ",
  complete: "Результат готов",
  error: "Требуется внимание",
};

const transcriberName = computed(() => modelValue("transcriber", "adapter_id") || "Voxtral");
const affectName = computed(() => modelValue("vocal_affect", "adapter_id") || "Emotion2Vec");
const preprocessorSummary = computed(() => {
  if (selectedAsrAudioRoute.value === "raw") return "RAW PCM · контроль без gain/NS";
  const model = objectValue(objectValue(bootstrap.value?.service, "models"), "audio_preprocessor");
  if (!model) return "enhanced route недоступен";
  const adapter = stringValue(model, "adapter_id") || "audio preprocessor";
  const gain = objectValue(model, "gain");
  const gainPlacement = stringValue(model, "gain_placement");
  return gain?.enabled === true
    ? gainPlacement === "pre_and_post_denoise"
      ? `${adapter} · усиление → шумоподавление → усиление`
      : `${adapter} · шумоподавление + усиление`
    : `${adapter} · шумоподавление`;
});
const transcriberCadence = computed(() => {
  const delay = numericModelValue("transcriber", "configured_delay_ms");
  const partial = numericModelValue("transcriber", "partial_decode_interval_ms");
  return delay !== null && partial !== null
    ? `delay ${delay} мс · partial ${partial} мс`
    : "параметры появятся после запуска";
});
const captureSummary = computed(() => {
  if (!captureInfo.value) return "";
  const info = captureInfo.value;
  const dsp = [
    info.echoCancellation && "AEC",
    info.noiseSuppression && "NS",
    info.autoGainControl && "AGC",
  ].filter(Boolean);
  const sourceRate = info.trackSampleRate ? `${info.trackSampleRate / 1_000} kHz → ` : "";
  const asrLatency =
    firstTranscriptLatencyMs.value === null
      ? "ASR …"
      : `ASR ${Math.round(firstTranscriptLatencyMs.value)} мс`;
  const affectLatency =
    firstAffectLatencyMs.value === null
      ? "affect …"
      : `affect ${Math.round(firstAffectLatencyMs.value)} мс`;
  return `${sourceRate}${info.contextSampleRate / 1_000} kHz · DSP ${dsp.join("+") || "off"} · ${asrLatency} · ${affectLatency}`;
});
const calibrationSummary = computed(() => {
  const result = calibration.value;
  if (!result) return "VAD: для тихой речи сначала нажмите «Калибровать тишину» в полной тишине.";
  return `VAD: whisper-aware · шум ${result.noiseFloorDbfs.toFixed(1)} dBFS · пик ${result.peakDbfs.toFixed(1)} dBFS · ${result.durationMs / 1_000} с`;
});
const identitySummary = computed(() => {
  const identity = objectValue(bootstrap.value?.service, "model_identity");
  if (!identity) return "точные revisions доступны после запуска";
  const voxtral = stringValue(identity, "transcribe_revision");
  const emotion = stringValue(identity, "emotion_revision");
  return [voxtral && `ASR ${voxtral.slice(0, 8)}`, emotion && `affect ${emotion.slice(0, 8)}`]
    .filter(Boolean)
    .join(" · ");
});

onMounted(async () => {
  try {
    bootstrap.value = await loadBootstrap();
    await refreshDiagnosticAudio();
    await refreshDevices();
    state.value = "ready";
  } catch (error) {
    fail(error);
  }
  window.addEventListener("beforeunload", cancelActiveSession);
});

onBeforeUnmount(() => {
  window.removeEventListener("beforeunload", cancelActiveSession);
  cancelActiveSession();
});

watch(selectedDevice, () => {
  calibration.value = null;
});

async function refreshDevices(): Promise<void> {
  devices.value = await listAudioInputs();
  if (!selectedDevice.value && devices.value.length > 0) {
    selectedDevice.value = devices.value[0]?.deviceId ?? "";
  }
}

async function startRecording(): Promise<void> {
  if (!bootstrap.value || !canStart.value) return;
  errorMessage.value = "";
  notice.value = "";
  timeline.value = null;
  finalUtterance.value = null;
  events.value = [];
  sentBytes.value = 0;
  inputRms.value = 0;
  captureInfo.value = null;
  firstTranscriptLatencyMs.value = null;
  firstAffectLatencyMs.value = null;
  limitStopScheduled = false;
  state.value = "connecting";
  client.value = null;
  let nextClient: SpeechTimelineClient | null = null;
  const nextCapture = new BrowserMicrophoneCapture();
  capture.value = nextCapture;
  try {
    // The resident service rotates its ephemeral WebSocket token on restart.
    // Refresh immediately before every session so a long-lived dashboard tab
    // never attempts authentication with its mount-time token.
    const freshBootstrap = await loadBootstrap();
    bootstrap.value = freshBootstrap;
    nextClient = new SpeechTimelineClient(freshBootstrap, handleEvent);
    client.value = nextClient;
    await nextClient.connectAndStart(
      "ru",
      selectedAsrAudioRoute.value,
      calibration.value
        ? {
            noiseFloorDbfs: calibration.value.noiseFloorDbfs,
            durationMs: calibration.value.durationMs,
          }
        : undefined,
    );
    captureInfo.value = await nextCapture.start(selectedDevice.value, 80, {
      onChunk: handleAudioChunk,
      onLevel: (level) => {
        inputRms.value = level;
      },
    });
    await refreshDevices();
    state.value = "recording";
  } catch (error) {
    nextClient?.cancel();
    await nextCapture.stop().catch(() => undefined);
    fail(error);
  }
}

async function calibrateNoise(): Promise<void> {
  if (!canCalibrate.value) return;
  errorMessage.value = "";
  notice.value = "Калибруем фон 2 секунды — пожалуйста, не говорите.";
  state.value = "calibrating";
  try {
    const result = await calibrateMicrophoneNoise(selectedDevice.value, 2_000);
    calibration.value = result;
    notice.value = `Калибровка готова: фон ${result.noiseFloorDbfs.toFixed(1)} dBFS. Эти пороги используются только для VAD.`;
    await refreshDevices();
    state.value = "ready";
  } catch (error) {
    fail(error);
  }
}

function handleAudioChunk(chunk: Uint8Array): void {
  try {
    const result = client.value?.sendPcm(chunk);
    if (!result) return;
    sentBytes.value = result.sentBytes;
    if (result.limitReached && !limitStopScheduled) {
      limitStopScheduled = true;
      notice.value = `Достигнут безопасный предел ${formatDuration(limitMs.value)} — запись остановлена и отправлена на финализацию.`;
      queueMicrotask(() => void stopRecording());
    }
  } catch (error) {
    void abortAfterError(error);
  }
}

async function stopRecording(): Promise<void> {
  if (state.value !== "recording" && state.value !== "finalizing") return;
  state.value = "finalizing";
  const activeCapture = capture.value;
  capture.value = null;
  if (activeCapture) {
    await activeCapture.stop();
  }
  client.value?.finish();
}

function handleEvent(event: SpeechEvent): void {
  const handlingStarted = performance.now();
  events.value = [...events.value.slice(-39), event];
  if (event.type === "speech_timeline.update") {
    const update = mergeTimelineUpdate(event as unknown as TimelineUpdate);
    timeline.value = update;
    if (
      captureInfo.value &&
      firstTranscriptLatencyMs.value === null &&
      update.transcript.revision > 0
    ) {
      firstTranscriptLatencyMs.value =
        performance.now() - captureInfo.value.startedAtMonotonicMs;
    }
    if (
      captureInfo.value &&
      firstAffectLatencyMs.value === null &&
      update.vocal_affect.raw_observations.length > 0
    ) {
      firstAffectLatencyMs.value =
        performance.now() - captureInfo.value.startedAtMonotonicMs;
    }
  } else if (event.type === "utterance.final") {
    finalUtterance.value = event as unknown as FinalUtterance;
    state.value = "complete";
    client.value?.close();
    client.value = null;
    void refreshDiagnosticAudio();
  } else if (event.type === "error" && event.terminal === true) {
    const code = typeof event.code === "string" ? event.code : "UNKNOWN_ERROR";
    const detail = typeof event.detail === "string" ? event.detail : "";
    void abortAfterError(new Error(`${code}${detail ? `: ${detail}` : ""}`));
  }
  const handlingMs = performance.now() - handlingStarted;
  if (handlingMs >= 16) {
    console.info("[speech-timeline] ui_event_slow", {
      type: event.type,
      handlingMs: Math.round(handlingMs),
      eventCount: events.value.length,
    });
  }
}

async function refreshDiagnosticAudio(): Promise<void> {
  try {
    diagnosticAudio.value = await loadDiagnosticAudio();
  } catch (error) {
    console.warn("[speech-timeline] diagnostic_audio_list_failed", error);
  }
}

function mergeTimelineUpdate(update: TimelineUpdate): TimelineUpdate {
  if (
    update.vocal_affect.raw_observations_mode === "snapshot" ||
    !timeline.value
  ) {
    return update;
  }
  const observations = new Map(
    timeline.value.vocal_affect.raw_observations.map((observation) => [
      observation.observation_id,
      observation,
    ]),
  );
  for (const observation of update.vocal_affect.raw_observations) {
    observations.set(observation.observation_id, observation);
  }
  return {
    ...update,
    vocal_affect: {
      ...update.vocal_affect,
      raw_observations: [...observations.values()].sort(
        (left, right) => left.observation_id - right.observation_id,
      ),
    },
  };
}

async function abortAfterError(error: unknown): Promise<void> {
  const activeCapture = capture.value;
  capture.value = null;
  if (activeCapture) {
    await activeCapture.stop().catch(() => undefined);
  }
  client.value?.close();
  client.value = null;
  fail(error);
}

function cancelActiveSession(): void {
  client.value?.cancel();
  client.value = null;
  void capture.value?.stop();
  capture.value = null;
}

function fail(error: unknown): void {
  errorMessage.value = error instanceof Error ? error.message : String(error);
  state.value = "error";
}

function modelValue(role: string, field: string): string {
  const models = objectValue(bootstrap.value?.service, "models");
  const model = objectValue(models, role);
  return model ? stringValue(model, field) : "";
}

function numericModelValue(role: string, field: string): number | null {
  const models = objectValue(bootstrap.value?.service, "models");
  const model = objectValue(models, role);
  const value = model?.[field];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function objectValue(value: unknown, key: string): JsonObject | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return null;
  const candidate = (value as JsonObject)[key];
  return typeof candidate === "object" && candidate !== null && !Array.isArray(candidate)
    ? (candidate as JsonObject)
    : null;
}

function stringValue(value: JsonObject, key: string): string {
  return typeof value[key] === "string" ? value[key] : "";
}
</script>

<template>
  <main class="dashboard-shell">
    <header class="topbar">
      <div class="brand-lockup">
        <span class="brand-mark">N</span>
        <div>
          <p>Next Engine Lab</p>
          <h1>Speech Timeline</h1>
        </div>
      </div>
      <div class="service-state" :class="state">
        <i></i>
        <span>{{ stateText[state] }}</span>
      </div>
    </header>

    <section class="hero-grid">
      <div class="hero-copy">
        <p class="eyebrow">Realtime speech observability</p>
        <h2>Слышим не только слова.</h2>
        <p>
          Здесь текст, эмоциональные окна и сглаженные переходы остаются отдельными
          ревизиями на одной аудиошкале. Это диагностические данные, а не вывод о
          внутреннем состоянии человека.
        </p>
      </div>

      <div class="capture-card">
        <div class="capture-row">
          <div>
            <p class="eyebrow">Источник</p>
            <select v-model="selectedDevice" :disabled="isRecording || state === 'finalizing'">
              <option value="">Системный микрофон</option>
              <option v-for="(device, index) in devices" :key="device.deviceId" :value="device.deviceId">
                {{ device.label || `Микрофон ${index + 1}` }}
              </option>
            </select>
          </div>
          <div>
            <p class="eyebrow">ASR-тракт</p>
            <select
              v-model="selectedAsrAudioRoute"
              :disabled="isRecording || state === 'finalizing'"
            >
              <option value="raw">RAW · контроль</option>
              <option v-if="enhancedRouteAvailable" value="enhanced">
                DPDFNet + gain · A/B
              </option>
            </select>
          </div>
          <button class="icon-button" :disabled="isRecording" title="Обновить список" @click="refreshDevices">↻</button>
        </div>

        <div class="record-controls">
          <button v-if="!isRecording" class="primary-button" :disabled="!canStart" @click="startRecording">
            <span class="record-dot"></span>
            Начать запись
          </button>
          <button
            v-if="!isRecording"
            class="secondary-button"
            :disabled="!canCalibrate"
            @click="calibrateNoise"
          >
            Калибровать тишину
          </button>
          <button v-else class="stop-button" @click="stopRecording">
            <span class="stop-square"></span>
            Завершить фразу
          </button>
          <div class="level-meter" title="Текущий RMS микрофона">
            <span :style="{ width: `${Math.max(0, Math.min(100, (levelDb + 72) * 1.4))}%` }"></span>
          </div>
          <b class="duration">{{ formatDuration(durationMs) }} / {{ formatDuration(limitMs) }}</b>
        </div>
        <div class="duration-track"><i :style="{ width: `${progress}%` }"></i></div>
        <p v-if="captureSummary" class="capture-diagnostics">{{ captureSummary }}</p>
        <p class="capture-diagnostics">{{ calibrationSummary }}</p>
        <p v-if="notice" class="notice">{{ notice }}</p>
        <p v-if="errorMessage" class="error-box">{{ errorMessage }}</p>
      </div>
    </section>

    <section class="model-strip">
      <div>
        <span>ASR</span>
        <b :title="transcriberCadence">{{ transcriberName }} · {{ transcriberCadence }}</b>
      </div>
      <div>
        <span>Vocal affect</span>
        <b>{{ affectName }}</b>
      </div>
      <div>
        <span>ASR signal</span>
        <b>{{ preprocessorSummary }}</b>
      </div>
      <div>
        <span>Lineage</span>
        <b>{{ identitySummary }}</b>
      </div>
      <div class="expression-card" :style="{ '--emotion': emotionColor(currentExpression) }">
        <span>Текущий окрас</span>
        <b>{{ emotionLabel(currentExpression) }}</b>
      </div>
    </section>

    <section class="content-grid">
      <TranscriptPanel
        :text="timeline?.transcript.text ?? finalUtterance?.text ?? ''"
        :stable-prefix="timeline?.transcript.stable_prefix ?? ''"
        :final="Boolean(finalUtterance || timeline?.transcript.final)"
        :revision="timeline?.transcript.revision ?? 0"
        :timing-precision="timeline?.transcript.timing_precision ?? 'utterance'"
      />
      <EmotionTimeline
        :adapter-name="affectName"
        :segments="timeline?.vocal_affect.segments ?? []"
        :speech-activity="timeline?.vocal_affect.speech_activity ?? []"
        :observations="timeline?.vocal_affect.raw_observations ?? []"
        :current-samples="currentSamples"
        :revision="timeline?.vocal_affect.revision ?? 0"
      />
    </section>

    <section v-if="finalUtterance" class="final-card">
      <div>
        <p class="eyebrow">Final utterance</p>
        <h2>{{ finalUtterance.text || "Текст не распознан" }}</h2>
      </div>
      <div class="final-expression" :style="{ '--emotion': emotionColor(finalUtterance.observed_vocal_expression) }">
        <span>Наблюдаемый окрас</span>
        <b>{{ emotionLabel(finalUtterance.observed_vocal_expression) }}</b>
        <small>alignment: {{ finalUtterance.alignment_grade }}</small>
        <small v-if="finalUtterance.observed_vocal_expression_source">
          {{ finalUtterance.observed_vocal_expression_source }}
        </small>
      </div>
    </section>

    <section v-if="diagnosticAudioEnabled" class="panel diagnostic-audio-panel">
      <div class="panel-header">
        <div>
          <p class="eyebrow">Explicit local diagnostic retention</p>
          <h2>Последние записи</h2>
        </div>
        <button class="icon-button" title="Обновить записи" @click="refreshDiagnosticAudio">↻</button>
      </div>
      <p class="muted-copy">Хранятся только последние пять raw-WAV и, когда включён preprocessing, их ASR-вариант.</p>
      <div v-if="diagnosticAudio.length" class="diagnostic-audio-list">
        <div v-for="(record, index) in diagnosticAudio" :key="record.id" class="diagnostic-audio-row">
          <span>Запись {{ diagnosticAudio.length - index }} · {{ formatDuration(record.duration_ms) }}</span>
          <div class="diagnostic-audio-variants">
            <label>
              <span>Raw микрофон</span>
              <audio controls preload="metadata" :src="`/api/diagnostic-audio/${record.id}.wav`"></audio>
            </label>
            <label v-if="record.enhanced_available">
              <span>ASR: обработанный сигнал</span>
              <audio controls preload="metadata" :src="`/api/diagnostic-audio/${record.id}.asr.wav`"></audio>
            </label>
          </div>
        </div>
      </div>
      <p v-else class="muted-copy">Завершите запись — она появится здесь.</p>
    </section>

    <details class="event-console">
      <summary>Протокол и последние события <span>{{ events.length }}</span></summary>
      <pre>{{ JSON.stringify(events, null, 2) }}</pre>
    </details>

    <footer class="footer-note">
      <span>16 kHz · mono · PCM S16LE</span>
      <span>{{ diagnosticAudioEnabled ? "последние 5 WAV локально сохранены" : "raw audio не сохраняется" }}</span>
      <span>localhost-only diagnostic</span>
    </footer>
  </main>
</template>
