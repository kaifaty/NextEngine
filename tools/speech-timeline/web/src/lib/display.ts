const LABELS: Record<string, string> = {
  angry: "Злость",
  disgusted: "Отвращение",
  enthusiasm: "Воодушевление",
  fearful: "Страх",
  happy: "Радость",
  neutral: "Нейтрально",
  other: "Другое",
  sad: "Грусть",
  surprised: "Удивление",
  unknown: "Неопределённо",
};

const COLORS: Record<string, string> = {
  angry: "#ff665e",
  disgusted: "#ab79ec",
  enthusiasm: "#e895d5",
  fearful: "#7189ff",
  happy: "#f5ca58",
  neutral: "#66c6b8",
  other: "#8f98aa",
  sad: "#69a7e8",
  surprised: "#f28cc3",
  unknown: "#4e5667",
};

export function emotionLabel(label: string): string {
  return LABELS[label] ?? label;
}

export function emotionColor(label: string): string {
  return COLORS[label] ?? COLORS.unknown;
}

export function formatTime(samples: number, sampleRate = 16_000): string {
  const seconds = Math.max(0, samples) / sampleRate;
  return `${seconds.toFixed(seconds >= 10 ? 1 : 2)} с`;
}

export function formatDuration(milliseconds: number): string {
  return `${(Math.max(0, milliseconds) / 1_000).toFixed(1)} с`;
}
