<script setup lang="ts">
import { computed } from "vue";

import type { AffectObservation, AffectSegment, SpeechActivitySegment } from "../types";
import { emotionColor, emotionLabel, formatTime } from "../lib/display";

const props = defineProps<{
  adapterName: string;
  segments: AffectSegment[];
  speechActivity: SpeechActivitySegment[];
  observations: AffectObservation[];
  currentSamples: number;
  revision: number;
}>();

const timelineSamples = computed(() =>
  Math.max(
    props.currentSamples,
    16_000,
    ...props.segments.map((segment) => segment.end_sample),
    ...props.speechActivity.map((segment) => segment.end_sample),
    ...props.observations.map((observation) => observation.end_sample),
  ),
);

const latestScores = computed(() => {
  const latest = props.observations.at(-1);
  if (!latest) return [];
  return Object.entries(latest.scores).sort((left, right) => right[1] - left[1]);
});

const visibleObservations = computed(() => props.observations.slice(-96));

function segmentStyle(segment: AffectSegment): Record<string, string> {
  const start = (segment.start_sample / timelineSamples.value) * 100;
  const width = ((segment.end_sample - segment.start_sample) / timelineSamples.value) * 100;
  return {
    left: `${Math.max(0, start)}%`,
    width: `${Math.max(0.7, width)}%`,
    background: emotionColor(segment.label),
  };
}

function observationStyle(observation: AffectObservation): Record<string, string> {
  const score = Math.max(0, Math.min(1, observation.scores[observation.top_label] ?? 0));
  return {
    background: emotionColor(observation.top_label),
    opacity: `${0.28 + score * 0.72}`,
  };
}

function activityStyle(segment: SpeechActivitySegment): Record<string, string> {
  const start = (segment.start_sample / timelineSamples.value) * 100;
  const width = ((segment.end_sample - segment.start_sample) / timelineSamples.value) * 100;
  return {
    left: `${Math.max(0, start)}%`,
    width: `${Math.max(0.7, width)}%`,
  };
}
</script>

<template>
  <section class="panel affect-panel">
    <header class="panel-header">
      <div>
        <p class="eyebrow">{{ adapterName }} · observed expression</p>
        <h2>Эмоциональный таймлайн</h2>
      </div>
      <span class="tag">rev {{ revision }}</span>
    </header>

    <div class="timeline-stage">
      <div class="timeline-track" aria-label="Сглаженные эмоциональные сегменты">
        <div
          v-for="(activity, index) in speechActivity"
          :key="`activity-${activity.start_sample}-${activity.end_sample}-${index}`"
          class="activity-segment"
          :class="activity.state"
          :style="activityStyle(activity)"
          :title="`${activity.state === 'speech' ? 'речь' : 'тишина'} · ${formatTime(activity.start_sample)}–${formatTime(activity.end_sample)}`"
        ></div>
        <div
          v-for="(segment, index) in segments"
          :key="`${segment.start_sample}-${segment.end_sample}-${index}`"
          class="timeline-segment"
          :style="segmentStyle(segment)"
          :title="`${emotionLabel(segment.label)} · ${formatTime(segment.start_sample)}–${formatTime(segment.end_sample)} · score ${segment.score.toFixed(3)}`"
        >
          <span v-if="segment.end_sample - segment.start_sample > timelineSamples * 0.09">
            {{ emotionLabel(segment.label) }}
          </span>
        </div>
        <div v-if="segments.length === 0 && speechActivity.length === 0" class="timeline-empty">Ожидание VAD · первое эмоциональное окно после речи</div>
      </div>
      <div class="timeline-axis">
        <span>0</span>
        <span>{{ formatTime(timelineSamples / 2) }}</span>
        <span>{{ formatTime(timelineSamples) }}</span>
      </div>
    </div>

    <div class="observation-block">
      <div class="subheading-row">
        <h3>Окна модели</h3>
        <span>{{ observations.length }} наблюдений · шаг 250 мс</span>
      </div>
      <div class="observation-rail">
        <span
          v-for="observation in visibleObservations"
          :key="observation.observation_id"
          class="observation-cell"
          :style="observationStyle(observation)"
          :title="`${formatTime(observation.start_sample)}–${formatTime(observation.end_sample)} · ${emotionLabel(observation.top_label)}`"
        ></span>
        <span v-if="observations.length === 0" class="rail-empty">raw observations появятся здесь</span>
      </div>
    </div>

    <div class="scores-block">
      <div class="subheading-row">
        <h3>Последнее окно</h3>
        <span>оценки не калиброваны как вероятности</span>
      </div>
      <div v-if="latestScores.length" class="score-list">
        <div v-for="([label, score]) in latestScores" :key="label" class="score-row">
          <span>{{ emotionLabel(label) }}</span>
          <div class="score-track"><i :style="{ width: `${Math.max(0, Math.min(1, score)) * 100}%`, background: emotionColor(label) }"></i></div>
          <b>{{ score.toFixed(3) }}</b>
        </div>
      </div>
      <p v-else class="muted-copy">Распределение появится после первой секунды речи.</p>
    </div>
  </section>
</template>
