<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import type { DeckState, WaveformPeak } from "../types/dj";

const props = defineProps<{
  deckA: DeckState;
  deckB: DeckState;
}>();

const emit = defineEmits<{
  (e: "seekA", ratio: number): void;
  (e: "seekB", ratio: number): void;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);

// Zoom level in seconds visible across canvas (Djay Pro default: 8s)
const visibleSecs = ref<number>(8.0);

// 60FPS smooth playhead
let lastAnimTime = performance.now();
let animFrameId: number | null = null;

let smoothTimeA = 0;
let smoothTimeB = 0;

// ==================== ENVELOPE CACHE (កុំ allocate រាល់ frame) ====================
interface EnvelopeCache {
  length: number;
  upper: Float32Array;
  lower: Float32Array;
}

const envCache: EnvelopeCache = {
  length: 0,
  upper: new Float32Array(0),
  lower: new Float32Array(0),
};

interface GlobalPeakCache {
  deckId: string;
  peakAmp: number;
}

const globalPeakCache: GlobalPeakCache = {
  deckId: "",
  peakAmp: 0,
};

// ==================== DRAG STATE ====================
const isDraggingA = ref(false);
const isDraggingB = ref(false);
let dragStartX = 0;
let dragStartTime = 0;

// ==================== WATCHERS ====================
watch(
  () => props.deckA.progress,
  (newProg) => {
    if (!isDraggingA.value) {
      smoothTimeA = newProg * (props.deckA.durationSecs || 240);
    }
  }
);

watch(
  () => props.deckB.progress,
  (newProg) => {
    if (!isDraggingB.value) {
      smoothTimeB = newProg * (props.deckB.durationSecs || 240);
    }
  }
);

// ==================== INTERACTION ====================
function handleCanvasMouseDown(e: MouseEvent) {
  e.preventDefault();
  const canvas = canvasRef.value;
  if (!canvas) return;

  const rect = canvas.getBoundingClientRect();
  const clickY = e.clientY - rect.top;
  const isDeckA = clickY < rect.height / 2;

  const deck = isDeckA ? props.deckA : props.deckB;
  const totalSecs = deck.durationSecs || 240;
  const curTime = isDeckA ? smoothTimeA : smoothTimeB;

  dragStartX = e.clientX;
  dragStartTime = curTime;

  if (isDeckA) isDraggingA.value = true;
  else isDraggingB.value = true;

  const pixelsPerSec = rect.width / visibleSecs.value;

  function onMouseMove(ev: MouseEvent) {
    const deltaX = ev.clientX - dragStartX;
    const deltaTime = -deltaX / pixelsPerSec;
    const targetTime = Math.max(0, Math.min(totalSecs, dragStartTime + deltaTime));
    const ratio = targetTime / totalSecs;

    if (isDeckA) {
      smoothTimeA = targetTime;
      emit("seekA", ratio);
    } else {
      smoothTimeB = targetTime;
      emit("seekB", ratio);
    }
  }

  function onMouseUp() {
    isDraggingA.value = false;
    isDraggingB.value = false;
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  }

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
}

function handleOverviewSeek(deckId: "A" | "B", e: MouseEvent) {
  e.preventDefault();
  const target = e.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();

  function updateSeek(ev: MouseEvent) {
    const x = ev.clientX - rect.left;
    const ratio = Math.max(0, Math.min(1, x / rect.width));
    if (deckId === "A") emit("seekA", ratio);
    else emit("seekB", ratio);
  }

  updateSeek(e);

  function onMouseMove(ev: MouseEvent) {
    updateSeek(ev);
  }

  function onMouseUp() {
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  }

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
}

// ==================== WAVEFORM SAMPLING ====================
function sampleWaveform(
  peaks: WaveformPeak[] | undefined,
  t: number,
  durationSecs: number
): WaveformPeak {
  if (!peaks || peaks.length === 0) {
    return { low: 0.1, mid: 0.1, high: 0.05 };
  }
  if (t < 0 || t > durationSecs || durationSecs <= 0) {
    return { low: 0, mid: 0, high: 0 };
  }

  const exactIdx = (t / durationSecs) * (peaks.length - 1);
  const i0 = Math.floor(exactIdx);
  const i1 = Math.min(peaks.length - 1, i0 + 1);
  const frac = exactIdx - i0;

  const p0 = peaks[i0];
  const p1 = peaks[i1];
  if (!p0 || !p1) return { low: 0.1, mid: 0.1, high: 0.05 };

  return {
    low: p0.low + (p1.low - p0.low) * frac,
    mid: p0.mid + (p1.mid - p0.mid) * frac,
    high: p0.high + (p1.high - p0.high) * frac,
  };
}

// ==================== DRUM DETECTION (KICK ISOLATED) ====================
// ✅ Kick = low ខ្លាំង + mid ទាប (vocal/melody មិនរាប់)
// ✅ Vocal/Melody = mid ខ្ពស់ → កាត់ចេញ
function getDrumAmplitude(band: WaveformPeak): number {
  const low = band.low || 0;
  const mid = band.mid || 0;
  const high = band.high || 0;

  // ✅ Kick strength — ត្រូវការ low ខ្លាំង
  const kickStrength = low;

  // ✅ Vocal/Melody rejection — បើ mid ខ្ពស់ = vocal ច្រៀង
  //    vocalRatio ខ្ពស់ → កាត់ amplitude
  const vocalRatio = mid / (low + 0.03);

  // ✅ Kick ត្រូវមាន low >> mid (kick មាន low ខ្លាំង ប៉ុន្តែ mid តូច)
  //    vocal/melody មាន mid ខ្ពស់ស្មើ low
  let finalAmp = kickStrength;

  // បើ vocalRatio > 0.7 = vocal កំពុងច្រៀង → កាត់ចេញ 60%
  if (vocalRatio > 0.7) {
    finalAmp *= 0.4;
  }
  // បើ vocalRatio > 1.0 = vocal ខ្លាំងជាង kick → កាត់ចេញ 80%
  else if (vocalRatio > 1.0) {
    finalAmp *= 0.2;
  }
  // បើ vocalRatio < 0.4 = kick បុកច្បាស់ → រក្សាទុក 100%
  else if (vocalRatio < 0.4) {
    finalAmp = kickStrength * 1.0;
  }
  // កណ្តាល = កាត់ចេញ 30%
  else {
    finalAmp *= 0.7;
  }

  return Math.max(0, finalAmp);
}


function normalizeKickPeaks(
  normalized: Float32Array,
  rawLow: Float32Array,
  numSamples: number
): void {
  const kickPeaks: number[] = [];
  const MIN_PEAK_DISTANCE = 8;

  for (let i = 2; i < numSamples - 2; i++) {
    const curr = normalized[i];
    const prev1 = normalized[i - 1];
    const prev2 = normalized[i - 2];
    const next1 = normalized[i + 1];
    const next2 = normalized[i + 2];

    if (
      curr > prev1 &&
      curr > prev2 &&
      curr > next1 &&
      curr > next2 &&
      curr > 0.25 &&
      rawLow[i] > 0.35
    ) {
      if (
        kickPeaks.length === 0 ||
        i - kickPeaks[kickPeaks.length - 1] >= MIN_PEAK_DISTANCE
      ) {
        kickPeaks.push(i);
      } else {
        const lastPeak = kickPeaks[kickPeaks.length - 1];
        if (normalized[i] > normalized[lastPeak]) {
          kickPeaks[kickPeaks.length - 1] = i;
        }
      }
    }
  }

  if (kickPeaks.length === 0) return;

  let peakSum = 0;
  for (const idx of kickPeaks) {
    peakSum += normalized[idx];
  }
  const avgPeak = peakSum / kickPeaks.length;

  const targetPeak = 0.95;

  for (const idx of kickPeaks) {
    const currentPeak = normalized[idx];
    if (currentPeak < 0.01) continue;

    const scale = targetPeak / currentPeak;
    const clampedScale = Math.min(scale, 3.0);

    const RANGE = 3;
    for (let j = -RANGE; j <= RANGE; j++) {
      const k = idx + j;
      if (k < 0 || k >= numSamples) continue;

      const distance = Math.abs(j) / RANGE;
      const weight = 1.0 - distance * distance;
      const factor = 1.0 + (clampedScale - 1.0) * weight;

      normalized[k] = Math.min(1.0, normalized[k] * factor);
    }
  }
}

// ==================== RENDER LOOP ====================
function render() {
  const canvas = canvasRef.value;
  if (!canvas) return;

  const now = performance.now();
  const dt = Math.min((now - lastAnimTime) / 1000, 0.1);
  lastAnimTime = now;

  if (props.deckA.isPlaying && !isDraggingA.value) {
    const rateA = 1.0 + (props.deckA.pitch || 0) / 100;
    smoothTimeA = Math.min(
      props.deckA.durationSecs || 240,
      smoothTimeA + dt * rateA
    );
  }
  if (props.deckB.isPlaying && !isDraggingB.value) {
    const rateB = 1.0 + (props.deckB.pitch || 0) / 100;
    smoothTimeB = Math.min(
      props.deckB.durationSecs || 240,
      smoothTimeB + dt * rateB
    );
  }

  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const rect = canvas.getBoundingClientRect();
  const cssWidth = Math.floor(rect.width);
  const cssHeight = Math.floor(rect.height);

  if (cssWidth <= 0 || cssHeight <= 0) {
    animFrameId = requestAnimationFrame(render);
    return;
  }

  if (canvas.width !== cssWidth * dpr || canvas.height !== cssHeight * dpr) {
    canvas.width = cssWidth * dpr;
    canvas.height = cssHeight * dpr;
  }

  const ctx = canvas.getContext("2d", { alpha: false });
  if (!ctx) return;

  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

  // Pure black background
  ctx.fillStyle = "#000000";
  ctx.fillRect(0, 0, cssWidth, cssHeight);

  const halfH = cssHeight / 2;
  const needleX = cssWidth / 2;
  const pixelsPerSec = cssWidth / visibleSecs.value;

  // ================= DECK A (TOP) =================
  drawDjayProWaveform(
    ctx,
    props.deckA,
    smoothTimeA,
    halfH / 2,
    halfH / 2 - 6,
    cssWidth,
    needleX,
    pixelsPerSec,
    1
  );

  // ================= DECK B (BOTTOM) =================
  drawDjayProWaveform(
    ctx,
    props.deckB,
    smoothTimeB,
    halfH + halfH / 2,
    halfH / 2 - 6,
    cssWidth,
    needleX,
    pixelsPerSec,
    2
  );

  // ================= CENTER DIVIDER =================
  ctx.strokeStyle = "rgba(60, 60, 65, 0.9)";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(0, halfH);
  ctx.lineTo(cssWidth, halfH);
  ctx.stroke();

  // ================= RED PLAYHEAD =================
  drawPlayhead(ctx, needleX, 0, cssHeight, halfH);

  ctx.restore();
  animFrameId = requestAnimationFrame(render);
}

// ==================== DJAY PRO WAVEFORM (DRUM DOMINANT + CLEAR GRID) ====================
function drawDjayProWaveform(
  ctx: CanvasRenderingContext2D,
  deck: DeckState,
  curTime: number,
  centerY: number,
  maxAmp: number,
  width: number,
  needleX: number,
  pixelsPerSec: number,
  deckNum: 1 | 2
) {
  const duration = deck.durationSecs || 240;
  const peaks = deck.waveform;
  const effBpm = deck.baseBpm * (1.0 + (deck.pitch || 0) / 100);

  
  // ==================== BUILD ENVELOPE ====================
  const step = 1.5;
  const numSamples = Math.ceil(width / step) + 1;

  if (envCache.length !== numSamples) {
    envCache.length = numSamples;
    envCache.upper = new Float32Array(numSamples);
    envCache.lower = new Float32Array(numSamples);
  }
  const upper = envCache.upper;
  const lower = envCache.lower;

  // ==================== PASS 1: SAMPLE ====================
  const rawAmp = new Float32Array(numSamples);
  const rawLow = new Float32Array(numSamples);

  for (let i = 0; i < numSamples; i++) {
    const x = i * step;
    const t = curTime + (x - needleX) / pixelsPerSec;

    if (t < 0 || t > duration) {
      rawAmp[i] = 0;
      rawLow[i] = 0;
      continue;
    }

    const band = sampleWaveform(peaks, t, duration);
    rawLow[i] = band.low || 0;
    rawAmp[i] = getDrumAmplitude(band);
  }

  // ==================== GLOBAL PEAK (ថេរ មិនរីក-រួម) ====================
  if (globalPeakCache.deckId !== deck.title || globalPeakCache.peakAmp <= 0.01) {
    let maxAmp = 0.01;
    if (peaks && peaks.length > 0) {
      const scanStep = Math.max(1, Math.floor(peaks.length / 800));
      for (let i = 0; i < peaks.length; i += scanStep) {
        const amp = getDrumAmplitude(peaks[i]);
        if (amp > maxAmp) maxAmp = amp;
      }
    }
    globalPeakCache.deckId = deck.title;
    globalPeakCache.peakAmp = maxAmp;
  }

  const peakAmp = globalPeakCache.peakAmp;
  const normFactor = 1.0 / peakAmp;

  // ==================== PASS 2: NORMALIZE (STABLE) ====================
  const gamma = 0.95;
  const ampScale = 0.85;
  const floor = 0.12;

  const normalized = new Float32Array(numSamples);
  for (let i = 0; i < numSamples; i++) {
    let n = rawAmp[i] * normFactor;
    if (n > 1) n = 1;
    if (n < 0) n = 0;
    n = Math.pow(n, gamma);
    if (n < floor) n = n * 0.5;
    normalized[i] = n;
  }

  normalizeKickPeaks(normalized, rawLow, numSamples);

  // ==================== PASS 3: TRANSIENT DETECTION ====================
  const transientBoost = new Float32Array(numSamples);
  const ONSET_THRESHOLD = 0.12;
  const DECAY_RATE = 0.75;

  let activeBoost = 0;
  for (let i = 0; i < numSamples; i++) {
    const current = normalized[i];
    const prev = i > 0 ? normalized[i - 1] : 0;

    const diff = current - prev;
    if (diff > ONSET_THRESHOLD && current > 0.35) {
      activeBoost = 1.0;
    }

    if (activeBoost > 0) {
      activeBoost *= DECAY_RATE;
      if (activeBoost < 0.01) activeBoost = 0;
    }

    transientBoost[i] = activeBoost;
  }

  // ==================== PASS 4: SOFT GATE + APPLY ====================
  for (let i = 0; i < numSamples; i++) {
    const boosted = normalized[i] + transientBoost[i] * 0.35;
    const clamped = Math.min(1.0, boosted);
    const spikeFactor = transientBoost[i] > 0.5 ? 1.15 : 1.0;
    let finalAmp = Math.min(1.0, clamped * spikeFactor);

    const lowValue = rawLow[i] * normFactor;
    if (lowValue < 0.20) {
      finalAmp *= 0.70;
    } else if (lowValue < 0.40) {
      finalAmp *= 0.85;
    }

    const h = Math.max(0.8, finalAmp * maxAmp * ampScale);
    upper[i] = centerY - h;
    lower[i] = centerY + h;
  }

  // ==================== DRAW WAVEFORM ====================
  ctx.save();

  const fillGrad = ctx.createLinearGradient(0, centerY - maxAmp, 0, centerY + maxAmp);
  fillGrad.addColorStop(0.0, "rgba(80, 180, 255, 0.95)");
  fillGrad.addColorStop(0.35, "rgba(80, 255, 180, 0.95)");
  fillGrad.addColorStop(0.55, "rgba(220, 255, 120, 0.95)");
  fillGrad.addColorStop(0.78, "rgba(255, 200, 80, 0.95)");
  fillGrad.addColorStop(1.0, "rgba(255, 80, 80, 1.0)");

  ctx.beginPath();
  ctx.moveTo(0, upper[0]);
  for (let i = 1; i < numSamples; i++) {
    ctx.lineTo(i * step, upper[i]);
  }
  ctx.lineTo(width, upper[numSamples - 1]);
  for (let i = numSamples - 1; i >= 0; i--) {
    ctx.lineTo(i * step, lower[i]);
  }
  ctx.closePath();

  ctx.fillStyle = fillGrad;
  ctx.fill();

  if (needleX < width) {
    ctx.fillStyle = "rgba(0, 0, 0, 0.35)";
    ctx.fillRect(needleX, 0, width - needleX, ctx.canvas.height);
  }

  ctx.restore();

  // ==================== KICK SPIKE HIGHLIGHTS ====================
  ctx.save();
  ctx.globalCompositeOperation = "lighter";

  for (let i = 1; i < numSamples - 1; i++) {
    const boost = transientBoost[i];
    if (boost < 0.4) continue;

    const lowValue = rawLow[i] * normFactor;
    if (lowValue < 0.40) continue;

    const x = i * step;
    const h = centerY - upper[i];
    if (h < 4) continue;

    ctx.strokeStyle = `rgba(255, 100, 100, ${boost * 0.7})`;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(x, centerY - h * 1.15);
    ctx.lineTo(x, centerY + h * 1.15);
    ctx.stroke();

    ctx.fillStyle = `rgba(255, 150, 150, ${boost * 0.5})`;
    ctx.beginPath();
    ctx.moveTo(x - 2.5, centerY - h);
    ctx.lineTo(x + 2.5, centerY - h);
    ctx.lineTo(x, centerY - h * 1.25);
    ctx.closePath();
    ctx.fill();
  }

  ctx.restore();

  // ==================== SHARP OUTLINE ====================
  ctx.save();
  ctx.lineJoin = "miter";
  ctx.lineCap = "butt";

  ctx.strokeStyle = "rgba(255, 255, 255, 0.65)";
  ctx.lineWidth = 1.2;
  ctx.beginPath();
  ctx.moveTo(0, upper[0]);
  for (let i = 1; i < numSamples; i++) {
    ctx.lineTo(i * step, upper[i]);
  }
  ctx.stroke();

  ctx.strokeStyle = "rgba(255, 255, 255, 0.25)";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(0, lower[0]);
  for (let i = 1; i < numSamples; i++) {
    ctx.lineTo(i * step, lower[i]);
  }
  ctx.stroke();

  ctx.restore();

  // ==================== CENTER SPINE ====================
  ctx.save();
  ctx.strokeStyle = "rgba(255, 255, 255, 0.5)";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(0, centerY);
  ctx.lineTo(width, centerY);
  ctx.stroke();
  ctx.restore();

  // ==================== VERTICAL GRID LINES ====================
  if (effBpm > 20 && duration > 0) {
    const secPerBeat = 60.0 / effBpm;
    const secPerBar = secPerBeat * 4;
    const timeStart = curTime - needleX / pixelsPerSec;
    const timeEnd = curTime + (width - needleX) / pixelsPerSec;

    const firstBar = Math.max(0, Math.floor(timeStart / secPerBar));
    const lastBar = Math.ceil(timeEnd / secPerBar);

    // Beat lines
    const firstBeat = Math.max(0, Math.floor(timeStart / secPerBeat));
    const lastBeat = Math.ceil(timeEnd / secPerBeat);

    ctx.save();
    ctx.strokeStyle = "rgba(255, 255, 255, 0.22)";
    ctx.lineWidth = 1;
    for (let beat = firstBeat; beat <= lastBeat; beat++) {
      const beatTime = beat * secPerBeat;
      if (beatTime > duration) break;
      const bx = needleX + (beatTime - curTime) * pixelsPerSec;
      if (bx < -10 || bx > width + 10) continue;
      const beatInBar = ((beat % 4) + 4) % 4;
      if (beatInBar === 0) continue;
      ctx.beginPath();
      ctx.moveTo(bx, centerY - maxAmp - 4);
      ctx.lineTo(bx, centerY + maxAmp + 4);
      ctx.stroke();
    }
    ctx.restore();

    // Bar lines
    ctx.save();
    for (let bar = firstBar; bar <= lastBar; bar++) {
      const barTime = bar * secPerBar;
      if (barTime > duration) break;
      const bx = needleX + (barTime - curTime) * pixelsPerSec;
      if (bx < -30 || bx > width + 30) continue;

      ctx.shadowColor = "rgba(0, 0, 0, 0.9)";
      ctx.shadowBlur = 3;

      ctx.strokeStyle = "rgba(255, 255, 255, 0.85)";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(bx, centerY - maxAmp - 8);
      ctx.lineTo(bx, centerY + maxAmp + 8);
      ctx.stroke();

      ctx.shadowBlur = 0;
      ctx.strokeStyle = "rgba(255, 255, 255, 1.0)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(bx, centerY - maxAmp - 8);
      ctx.lineTo(bx, centerY + maxAmp + 8);
      ctx.stroke();

      ctx.shadowColor = "rgba(0, 0, 0, 0.95)";
      ctx.shadowBlur = 4;
      ctx.fillStyle = "rgba(255, 255, 255, 1.0)";
      ctx.font = "bold 12px 'Helvetica Neue', Arial, sans-serif";
      ctx.textAlign = "left";
      ctx.fillText(`${bar + 1}`, bx + 5, centerY - maxAmp + 12);
    }
    ctx.restore();

    // Phrase markers
    ctx.save();
    for (let bar = firstBar; bar <= lastBar; bar++) {
      if (bar % 8 !== 0) continue;
      const barTime = bar * secPerBar;
      if (barTime > duration) break;
      const bx = needleX + (barTime - curTime) * pixelsPerSec;
      if (bx < -30 || bx > width + 30) continue;

      ctx.shadowColor = "rgba(0, 0, 0, 0.95)";
      ctx.shadowBlur = 4;

      ctx.strokeStyle = "rgba(250, 204, 21, 1.0)";
      ctx.lineWidth = 3;
      ctx.beginPath();
      ctx.moveTo(bx, centerY - maxAmp - 10);
      ctx.lineTo(bx, centerY + maxAmp + 10);
      ctx.stroke();

      ctx.fillStyle = "rgba(250, 204, 21, 1.0)";
      ctx.beginPath();
      ctx.moveTo(bx - 5, centerY - maxAmp - 6);
      ctx.lineTo(bx + 5, centerY - maxAmp - 6);
      ctx.lineTo(bx, centerY - maxAmp + 4);
      ctx.closePath();
      ctx.fill();

      ctx.beginPath();
      ctx.moveTo(bx - 5, centerY + maxAmp + 6);
      ctx.lineTo(bx + 5, centerY + maxAmp + 6);
      ctx.lineTo(bx, centerY + maxAmp - 4);
      ctx.closePath();
      ctx.fill();
    }
    ctx.restore();
  }

  // ==================== DECK BADGE ====================
  ctx.save();
  const badgeX = 12;
  const badgeY = centerY - maxAmp - 2;

  ctx.fillStyle = "rgba(255, 255, 255, 0.95)";
  ctx.font = "bold 16px 'Helvetica Neue', Arial, sans-serif";
  ctx.textAlign = "left";
  ctx.fillText(`${deckNum}`, badgeX, badgeY + 14);

  ctx.fillStyle = "rgba(255, 255, 255, 0.7)";
  ctx.beginPath();
  ctx.moveTo(badgeX + 12, badgeY + 8);
  ctx.lineTo(badgeX + 18, badgeY + 8);
  ctx.lineTo(badgeX + 15, badgeY + 13);
  ctx.closePath();
  ctx.fill();

  ctx.restore();
}

// ==================== PLAYHEAD ====================
function drawPlayhead(
  ctx: CanvasRenderingContext2D,
  needleX: number,
  yTop: number,
  yBottom: number,
  midY: number
) {
  ctx.save();
  ctx.shadowColor = "rgba(239, 68, 68, 0.9)";
  ctx.shadowBlur = 6;

  ctx.strokeStyle = "#ef4444";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(needleX, yTop);
  ctx.lineTo(needleX, yBottom);
  ctx.stroke();

  ctx.fillStyle = "#ef4444";
  ctx.beginPath();
  ctx.moveTo(needleX - 4, yTop);
  ctx.lineTo(needleX + 4, yTop);
  ctx.lineTo(needleX, yTop + 5);
  ctx.closePath();
  ctx.fill();

  ctx.beginPath();
  ctx.moveTo(needleX - 4, yBottom);
  ctx.lineTo(needleX + 4, yBottom);
  ctx.lineTo(needleX, yBottom - 5);
  ctx.closePath();
  ctx.fill();

  ctx.restore();
}

// ==================== LIFECYCLE ====================
onMounted(() => {
  lastAnimTime = performance.now();
  smoothTimeA = (props.deckA.progress || 0) * (props.deckA.durationSecs || 240);
  smoothTimeB = (props.deckB.progress || 0) * (props.deckB.durationSecs || 240);
  animFrameId = requestAnimationFrame(render);
});

onUnmounted(() => {
  if (animFrameId) cancelAnimationFrame(animFrameId);
});
</script>

<template>
  <div class="w-full bg-black border border-zinc-800 rounded-lg p-1.5 shadow-2xl relative select-none font-mono">

    <!-- ================= DUAL CANVAS ================= -->
    <div
      class="relative w-full h-[140px] bg-black rounded overflow-hidden cursor-ew-resize group"
      @mousedown="handleCanvasMouseDown"
      title="Drag left/right to scratch & scrub"
    >
      <canvas ref="canvasRef" class="w-full h-full block"></canvas>
    </div>

    <!-- ================= CONTROLS BAR ================= -->
    <div class="h-6 w-full bg-[#0a0a0a] border-t border-zinc-800/80 flex items-center justify-between px-2 text-[7px] text-zinc-500 font-black mt-1.5">

      <!-- Deck A Phase -->
      <div class="flex items-center gap-1">
        <span class="text-cyan-400 font-bold">◄ A</span>
        <div class="flex items-center gap-0.5">
          <span
            v-for="b in 8" :key="'b1-' + b"
            class="w-1.5 h-1 rounded-[1px]"
            :class="b === 4 ? 'bg-cyan-400' : 'bg-zinc-800'"
          ></span>
        </div>
      </div>

      <!-- Drum Legend -->
      <div class="flex items-center gap-3 text-[8px] font-sans font-semibold">
        <div class="flex items-center gap-1" title="Kick Drum (ភ្លេងបុក - ខ្ពស់ជាងគេ)">
          <span class="w-2 h-2 rounded-full bg-red-500 shadow-[0_0_5px_#ef4444]"></span>
          <span class="text-red-400 font-bold">KICK</span>
        </div>
        <div class="flex items-center gap-1" title="Snare (ស្គរ - កណ្តាល)">
          <span class="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_5px_#22c55e]"></span>
          <span class="text-green-400 font-bold">SNARE</span>
        </div>
        <div class="flex items-center gap-1" title="Clap (ខ្ទាស់ដៃ - ខ្ពស់)">
          <span class="w-2 h-2 rounded-full bg-blue-500 shadow-[0_0_5px_#3b82f6]"></span>
          <span class="text-blue-400 font-bold">CLAP</span>
        </div>
      </div>

      <!-- Zoom -->
      <div class="flex items-center gap-1">
        <span class="text-zinc-500 text-[6.5px]">ZOOM:</span>
        <button
          v-for="z in [4, 8, 16]" :key="'zoom-' + z"
          @click="visibleSecs = z"
          class="px-1.5 py-[1px] rounded text-[7px] font-bold border transition-all cursor-pointer"
          :class="visibleSecs === z
            ? 'bg-amber-400 border-amber-300 text-black'
            : 'bg-zinc-900 border-zinc-800 text-zinc-400 hover:text-white'"
        >
          {{ z }}s
        </button>
      </div>

      <!-- Deck B Phase -->
      <div class="flex items-center gap-1">
        <div class="flex items-center gap-0.5">
          <span
            v-for="b in 8" :key="'b2-' + b"
            class="w-1.5 h-1 rounded-[1px]"
            :class="b === 4 ? 'bg-rose-400' : 'bg-zinc-800'"
          ></span>
        </div>
        <span class="text-rose-400 font-bold">B ►</span>
      </div>
    </div>

  </div>
</template>