<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  text: string;
  stablePrefix: string;
  final: boolean;
  revision: number;
  timingPrecision: string;
  adapterName: string;
  supportsStreaming: boolean;
  streamingMode: string;
}>();

const stable = computed(() =>
  props.text.startsWith(props.stablePrefix) ? props.stablePrefix : "",
);
const tentative = computed(() => props.text.slice(stable.value.length));
</script>

<template>
  <section class="panel transcript-panel">
    <header class="panel-header">
      <div>
        <p class="eyebrow">{{ adapterName }} · ASR</p>
        <h2>Транскрипт</h2>
      </div>
      <div class="header-tags">
        <span class="tag">rev {{ revision }}</span>
        <span class="tag" :class="{ accent: final }">
          {{ final
            ? "final"
            : streamingMode === "buffered_emulation"
              ? "buffered stream"
              : supportsStreaming
                ? "stream"
                : "final-only" }}
        </span>
      </div>
    </header>
    <div class="transcript-text" :class="{ empty: !text }">
      <template v-if="text">
        <span class="stable-text">{{ stable }}</span><span class="tentative-text">{{ tentative }}</span>
      </template>
      <span v-else>
        {{ supportsStreaming
          ? streamingMode === "buffered_emulation"
            ? "Начните запись — гипотеза будет обновляться bounded-окнами и может переписываться до final."
            : "Начните запись — здесь появятся стабильная и предварительная части текста."
          : "Начните запись и завершите фразу — эта ASR-модель выдаёт только финальный текст." }}
      </span>
    </div>
    <footer class="panel-note">
      <span class="legend-dot stable-dot"></span> стабильный префикс
      <span class="legend-dot tentative-dot"></span> предварительный текст
      <span class="spacer"></span>
      точность времени: {{ timingPrecision || "utterance" }}
    </footer>
  </section>
</template>
