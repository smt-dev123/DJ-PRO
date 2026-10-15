<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import Turntable from "./Turntable.vue";
import VerticalFader from "./VerticalFader.vue";
import TabSelector from "./deck/TabSelector.vue";
import HotCueTab from "./deck/HotCueTab.vue";
import LoopTab from "./deck/LoopTab.vue";
import StemsTab from "./deck/StemsTab.vue";
import FxTab from "./deck/FxTab.vue";
import type { DeckState, LibraryTrack, WaveBand } from "../types/dj";

const props = defineProps<{
  deck: DeckState;
  color: "cyan" | "rose";
}>();

const emit = defineEmits<{
  (e: "open"): void;
  (e: "play"): void;
  (e: "cue"): void;
  (e: "pitch", val: number): void;
  (e: "sync"): void;
  (e: "keyLock"): void;
  (e: "nudge", delta: number): void;
  (e: "resetPitch"): void;
  (e: "loadTrack", track: LibraryTrack): void;
  (e: "loadFile", path: string): void;
  (e: "seek", ratio: number): void;
  (e: "hotCue", pad: number): void;
  (e: "toggleLoop", beats: string): void;
  (e: "toggleStem", stem: string, active: boolean): void;
  (e: "fxType", fxType: string): void;
  (e: "fxAmount", amount: number): void;
  (e: "fxActive", active: boolean): void;
  (e: "fxOut", fxType: "echo" | "reverb", duration: number): void;
  (e: "cancelFxOut"): void;
  (e: "updateFxParams", params: any): void;
}>();

type TabMode = "HOT CUE" | "LOOP" | "STEMS" | "FX";
const activeTab = ref<TabMode>("HOT CUE");
const isDragOver = ref(false);

// ==================== DRAG & DROP ====================
function handleDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
  isDragOver.value = true;
}

function handleDragLeave(e: DragEvent) {
  const currentTarget = e.currentTarget as HTMLElement;
  const relatedTarget = e.relatedTarget as HTMLElement;
  if (!currentTarget || !relatedTarget || !currentTarget.contains(relatedTarget)) {
    isDragOver.value = false;
  }
}

function handleDrop(e: DragEvent) {
  isDragOver.value = false;

  const jsonStr = e.dataTransfer?.getData("application/json");
  if (jsonStr) {
    try {
      const track: LibraryTrack = JSON.parse(jsonStr);
      emit("loadTrack", track);
      return;
    } catch {}
  }

  const text = e.dataTransfer?.getData("text/plain");
  if (text && /\.(mp3|wav|flac|aac|m4a|ogg)$/i.test(text)) {
    emit("loadFile", text);
    return;
  }

  if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
    const file = e.dataTransfer.files[0];
    const path = (file as any).path || file.name;
    emit("loadFile", path);
  }
}

// ==================== SCRUBBER ====================
function handleScrubberMouseDown(e: MouseEvent) {
  e.preventDefault();
  const target = e.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();

  function updateScrub(ev: MouseEvent) {
    const x = ev.clientX - rect.left;
    const ratio = Math.max(0, Math.min(1, x / rect.width));
    emit("seek", ratio);
  }

  updateScrub(e);

  function onMouseMove(ev: MouseEvent) {
    updateScrub(ev);
  }

  function onMouseUp() {
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  }

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
}

// ==================== WAVEFORM SAMPLING ====================
const BAR_COUNT = 600;

const waveformBars = computed<WaveBand[]>(() => {
  const count = BAR_COUNT;
  const source = props.deck.waveform;

  if (source && source.length > 0) {
    if (source.length <= count) {
      if (source.length === count) return source;
      const interpolated: WaveBand[] = [];
      for (let i = 0; i < count; i++) {
        const srcIdx = (i / count) * (source.length - 1);
        const i0 = Math.floor(srcIdx);
        const i1 = Math.min(source.length - 1, i0 + 1);
        const frac = srcIdx - i0;
        const p0 = source[i0];
        const p1 = source[i1];
        if (!p0 || !p1) {
          interpolated.push({ low: 0.1, mid: 0.1, high: 0.1 });
          continue;
        }
        interpolated.push({
          low: p0.low + (p1.low - p0.low) * frac,
          mid: p0.mid + (p1.mid - p0.mid) * frac,
          high: p0.high + (p1.high - p0.high) * frac,
        });
      }
      return interpolated;
    }
    const step = source.length / count;
    const sampled: WaveBand[] = [];
    for (let i = 0; i < count; i++) {
      const idx = Math.min(source.length - 1, Math.floor(i * step));
      sampled.push(source[idx]);
    }
    return sampled;
  }

  const fallback: WaveBand[] = [];
  for (let i = 0; i < count; i++) {
    fallback.push({ low: 0.05, mid: 0.03, high: 0.02 });
  }
  return fallback;
});

// ==================== CANVAS SCRUBBER ====================
const scrubberCanvasRef = ref<HTMLCanvasElement | null>(null);
let redrawTimer: number | null = null;

function scheduleRedraw() {
  if (redrawTimer) return;
  redrawTimer = window.setTimeout(() => {
    redrawTimer = null;
    requestAnimationFrame(drawScrubberCanvas);
  }, 16);
}

function getSliceColor(band: WaveBand, deckColor: "cyan" | "rose"): string {
  const low = band.low || 0;
  const mid = band.mid || 0;
  const high = band.high || 0;

  const kickStrength = low;
  const vocalStrength = mid;
  const midiStrength = high;
  const maxStrength = Math.max(kickStrength, vocalStrength, midiStrength);

  if (maxStrength < 0.2) return "#1e293b";
  if (kickStrength === maxStrength && kickStrength > 0.5) return "#ef4444";
  if (vocalStrength === maxStrength && vocalStrength > 0.5) return "#eab308";
  if (midiStrength === maxStrength && midiStrength > 0.5) return "#a855f7";

  const r = Math.round(kickStrength * 239);
  const g = Math.round(vocalStrength * 180);
  const b = Math.round(midiStrength * 246);
  return `rgb(${r}, ${g}, ${b})`;
}

function drawScrubberCanvas() {
  const canvas = scrubberCanvasRef.value;
  if (!canvas) return;

  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const rect = canvas.getBoundingClientRect();
  const cssWidth = Math.floor(rect.width);
  const cssHeight = Math.floor(rect.height);

  if (cssWidth <= 0 || cssHeight <= 0) return;

  if (canvas.width !== cssWidth * dpr || canvas.height !== cssHeight * dpr) {
    canvas.width = cssWidth * dpr;
    canvas.height = cssHeight * dpr;
  }

  const ctx = canvas.getContext("2d", { alpha: false });
  if (!ctx) return;

  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

  ctx.fillStyle = "#0a0d14";
  ctx.fillRect(0, 0, cssWidth, cssHeight);

  const bands = waveformBars.value;
  const numBars = bands.length;
  if (numBars === 0) {
    ctx.restore();
    return;
  }

  const centerY = cssHeight / 2;
  const maxAmp = cssHeight / 2 - 3;
  const barWidth = cssWidth / numBars;
  const gap = Math.max(0.15, barWidth * 0.10);
  const actualBarWidth = Math.max(0.3, barWidth - gap);

  let globalKick = 0.01;
  let globalSnare = 0.01;
  let globalClap = 0.01;

  for (let i = 0; i < numBars; i++) {
    const band = bands[i];
    if (band.low > globalKick) globalKick = band.low;
    if (band.mid > globalSnare) globalSnare = band.mid;
    if (band.high > globalClap) globalClap = band.high;
  }

  for (let i = 0; i < numBars; i++) {
    const band = bands[i];
    const x = i * barWidth;
    const isPlayed = i / numBars <= props.deck.progress;
    const alpha = isPlayed ? 1.0 : 0.65;

    const kickNorm = band.low / globalKick;
    const snareNorm = band.mid / globalSnare;
    const clapNorm = band.high / globalClap;

    const amp = Math.max(kickNorm * 1.0, snareNorm * 0.4, clapNorm * 0.2);
    const normalized = Math.pow(amp, 0.75);
    const h = Math.max(0.8, normalized * maxAmp);

    const barColor = getSliceColor(band, props.color);

    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.fillStyle = barColor;
    ctx.fillRect(x, centerY - h, actualBarWidth, h * 2);
    ctx.restore();
  }

  // Center spine
  ctx.save();
  ctx.strokeStyle = "rgba(255, 255, 255, 0.25)";
  ctx.lineWidth = 0.8;
  ctx.beginPath();
  ctx.moveTo(0, centerY);
  ctx.lineTo(cssWidth, centerY);
  ctx.stroke();
  ctx.restore();

  // Dim unplayed
  if (props.deck.progress < 1) {
    ctx.save();
    ctx.fillStyle = "rgba(0, 0, 0, 0.15)";
    ctx.fillRect(
      props.deck.progress * cssWidth,
      0,
      cssWidth - props.deck.progress * cssWidth,
      cssHeight
    );
    ctx.restore();
  }

  ctx.restore();
}

// ==================== WATCHERS ====================
watch(() => props.deck.waveform, () => scheduleRedraw(), { deep: false });
watch(() => props.deck.progress, () => scheduleRedraw());

function handleResize() {
  scheduleRedraw();
}

onMounted(() => {
  requestAnimationFrame(drawScrubberCanvas);
  window.addEventListener("resize", handleResize);
});

onUnmounted(() => {
  if (redrawTimer) clearTimeout(redrawTimer);
  window.removeEventListener("resize", handleResize);
});

// ==================== COMPUTED ====================
const remainingTime = computed(() => {
  const total = props.deck.durationSecs || 240;
  const current = (props.deck.progress || 0) * total;
  const rem = Math.max(0, Math.round(total - current));
  const mins = Math.floor(rem / 60);
  const secs = rem % 60;
  return `-${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
});

// ==================== TAB HANDLERS ====================
function handleHotCue(pad: number) {
  emit("hotCue", pad);
}

function handleLoopToggle(beats: string) {
  emit("toggleLoop", beats);
}

function handleStemToggle(stem: string, active: boolean) {
  emit("toggleStem", stem, active);
}

function handleFxType(fxType: string) {
  emit("fxType", fxType);
}

function handleFxAmount(amount: number) {
  emit("fxAmount", amount);
}

function handleFxActive(active: boolean) {
  emit("fxActive", active);
}

function handleFxOut(fxType: "echo" | "reverb", duration: number) {
  emit("fxOut", fxType, duration);
}

function handleCancelFxOut() {
  emit("cancelFxOut");
}

function handleUpdateFxParams(params: any) {
  emit("updateFxParams", params);
}
</script>

<template>
  <div
    class="relative bg-[#0c0d12] border rounded-xl p-2.5 shadow-2xl flex flex-col justify-between h-full select-none font-mono transition-all overflow-hidden"
    :class="[
      color === 'cyan' ? 'border-cyan-500/30 shadow-cyan-950/20' : 'border-rose-500/30 shadow-rose-950/20',
      isDragOver ? (color === 'cyan' ? 'ring-2 ring-cyan-400 border-cyan-400' : 'ring-2 ring-rose-400 border-rose-400') : ''
    ]"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
    <!-- ================= DRAG OVERLAY ================= -->
    <div
      v-if="isDragOver"
      class="absolute inset-0 z-50 rounded-xl backdrop-blur-xs flex flex-col items-center justify-center p-4 border-2 border-dashed pointer-events-none transition-all animate-pulse"
      :class="color === 'cyan' ? 'bg-cyan-950/90 border-cyan-400 text-cyan-200' : 'bg-rose-950/90 border-rose-400 text-rose-200'"
    >
      <div
        class="w-12 h-12 rounded-full flex items-center justify-center text-xl mb-2 shadow-lg"
        :class="color === 'cyan' ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-400' : 'bg-rose-500/20 text-rose-300 border border-rose-400'"
      >
        📥
      </div>
      <span class="text-xs font-black tracking-widest uppercase">
        DROP TRACK ON DECK {{ deck.id }}
      </span>
      <span class="text-[9px] text-zinc-400 mt-1 font-sans">
        Release to load into Deck {{ deck.id }}
      </span>
    </div>

    <!-- ================= 1. HEADER ================= -->
    <div class="flex items-center justify-between border-b border-zinc-800/80 pb-2 mb-1.5 shrink-0">
      <div class="flex items-center gap-2.5 min-w-0">
        <span
          class="w-2.5 h-2.5 rounded-full shrink-0 shadow-md transition-all"
          :class="[
            deck.isPlaying ? 'animate-pulse' : 'opacity-60',
            color === 'cyan' ? 'bg-cyan-400 shadow-cyan-400' : 'bg-rose-400 shadow-rose-400'
          ]"
        ></span>

        <div
          class="px-2 py-0.5 rounded text-[10px] font-black tracking-widest shrink-0 border"
          :class="color === 'cyan'
            ? 'bg-cyan-950/40 border-cyan-500/40 text-cyan-400'
            : 'bg-rose-950/40 border-rose-500/40 text-rose-400'"
        >
          DECK {{ deck.id }}
        </div>

        <div class="flex flex-col min-w-0">
          <span class="text-xs font-black text-zinc-100 truncate tracking-wide">
            {{ deck.title || "No Track Loaded" }}
          </span>
          <span class="text-[9px] text-zinc-400 truncate -mt-0.5 font-sans">
            {{ deck.artist || "Unknown Artist" }}
          </span>
        </div>
      </div>

      <div class="flex items-center gap-1.5 shrink-0">
        <button
          @click="$emit('keyLock')"
          title="Toggle Key Lock"
          class="px-2 py-1 rounded text-[8px] font-black border transition-all cursor-pointer"
          :class="deck.keyLock
            ? (color === 'cyan' ? 'bg-cyan-950/60 border-cyan-400 text-cyan-300' : 'bg-rose-950/60 border-rose-400 text-rose-300')
            : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
        >
          KEY LOCK
        </button>

        <button
          @click="$emit('open')"
          class="px-2.5 py-1 bg-zinc-800/90 hover:bg-zinc-700 active:scale-95 text-[9px] font-black rounded-lg border border-zinc-700 text-zinc-200 transition shrink-0 cursor-pointer flex items-center gap-1"
        >
          <span>📂</span> LOAD
        </button>
      </div>
    </div>

    <!-- ================= 2. WAVEFORM SCRUBBER ================= -->
    <div class="flex flex-col gap-1 mb-1.5 shrink-0">
      <div class="flex items-center justify-between text-[7px] font-bold text-zinc-400 px-1">
        <div class="flex items-center gap-1.5 font-mono">
          <span
            class="px-1 py-0.5 rounded text-[7px] font-black"
            :class="color === 'cyan' ? 'bg-cyan-950/70 text-cyan-300 border border-cyan-500/30' : 'bg-rose-950/70 text-rose-300 border border-rose-500/30'"
          >
            {{ deck.currentTime }}
          </span>
          <span class="text-[6.5px] text-zinc-500">/ {{ deck.totalTime }}</span>
        </div>

        <div class="flex items-center gap-2.5 text-[7.5px] font-sans font-semibold">
          <div class="flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-red-500"></span>
            <span class="text-red-400">KICK</span>
          </div>
          <div class="flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-yellow-500"></span>
            <span class="text-yellow-400">VOCAL</span>
          </div>
          <div class="flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-purple-500"></span>
            <span class="text-purple-400">MIDI</span>
          </div>
        </div>

        <div class="font-mono text-[7px] font-bold" :class="color === 'cyan' ? 'text-cyan-400' : 'text-rose-400'">
          {{ remainingTime }}
        </div>
      </div>

      <div
        @mousedown="handleScrubberMouseDown"
        class="h-12 bg-[#050609] border rounded-lg relative cursor-ew-resize group overflow-hidden select-none"
        :class="color === 'cyan' ? 'border-cyan-500/30 hover:border-cyan-400/80' : 'border-rose-500/30 hover:border-rose-400/80'"
      >
        <canvas ref="scrubberCanvasRef" class="w-full h-full block"></canvas>

        <div
          v-if="deck.status === 'LOADING...'"
          class="absolute inset-0 bg-black/70 backdrop-blur-xs flex items-center justify-center z-40"
        >
          <div class="flex items-center gap-2">
            <div
              class="w-4 h-4 border-2 border-t-transparent rounded-full animate-spin"
              :class="color === 'cyan' ? 'border-cyan-400' : 'border-rose-400'"
            ></div>
            <span class="text-[10px] font-black text-zinc-300">LOADING...</span>
          </div>
        </div>

        <!-- Hot Cue Markers -->
        <div
          v-for="(cuePos, cueIdx) in [0.05, 0.28, 0.52, 0.78]"
          :key="'cue-flag-' + cueIdx"
          class="absolute top-0 bottom-0 pointer-events-none z-20 flex flex-col justify-between"
          :style="{ left: `${cuePos * 100}%` }"
        >
          <div
            class="px-1 py-[0.5px] text-[5.5px] font-black rounded-xs -translate-x-1/2 shadow-sm"
            :class="color === 'cyan' ? 'bg-cyan-400 text-black' : 'bg-rose-400 text-white'"
          >
            {{ cueIdx + 1 }}
          </div>
          <div
            class="w-[1px] h-full opacity-40 -translate-x-1/2"
            :class="color === 'cyan' ? 'bg-cyan-400' : 'bg-rose-400'"
          ></div>
        </div>

        <!-- Playhead -->
        <div
          class="absolute top-0 bottom-0 w-[2px] pointer-events-none z-30 shadow-lg"
          :class="color === 'cyan' ? 'bg-cyan-300 shadow-[0_0_8px_#22d3ee]' : 'bg-rose-300 shadow-[0_0_8px_#fb7185]'"
          :style="{ left: `${deck.progress * 100}%` }"
        >
          <div
            class="absolute -top-0.5 left-1/2 -translate-x-1/2 w-2 h-2 rotate-45"
            :class="color === 'cyan' ? 'bg-cyan-300' : 'bg-rose-300'"
          ></div>
          <div
            class="absolute -bottom-0.5 left-1/2 -translate-x-1/2 w-2 h-2 rotate-45"
            :class="color === 'cyan' ? 'bg-cyan-300' : 'bg-rose-300'"
          ></div>
        </div>

        <!-- Elapsed Wash -->
        <div
          class="absolute left-0 top-0 bottom-0 pointer-events-none z-10"
          :class="color === 'cyan' ? 'bg-gradient-to-r from-cyan-950/20 to-cyan-500/5' : 'bg-gradient-to-r from-rose-950/20 to-rose-500/5'"
          :style="{ width: `${deck.progress * 100}%` }"
        ></div>
      </div>
    </div>

    <!-- ================= 3. MAIN CENTER ================= -->
    <div class="grid grid-cols-12 gap-2.5 items-center flex-1 my-0.5">

      <!-- 3.1 PADS -->
      <div class="col-span-4 bg-[#08090d] border border-zinc-800/90 rounded-xl p-2 flex flex-col justify-between h-[155px]">
        <TabSelector v-model="activeTab" :color="color" />

        <HotCueTab
          v-if="activeTab === 'HOT CUE'"
          :color="color"
          @trigger="handleHotCue"
        />

        <LoopTab
          v-else-if="activeTab === 'LOOP'"
          @toggle-loop="handleLoopToggle"
        />

        <StemsTab
          v-else-if="activeTab === 'STEMS'"
          @toggle-stem="handleStemToggle"
        />

        <FxTab
          v-else-if="activeTab === 'FX'"
          :fx-type="deck.fxType || 'none'"
          :fx-amount="deck.fxAmount || 0"
          :fx-active="deck.fxActive || false"
          :params="deck.fxParams || {}"
          @fx-type="handleFxType"
          @fx-amount="handleFxAmount"
          @fx-active="handleFxActive"
          @fx-out="handleFxOut"
          @cancel-fx-out="handleCancelFxOut"
          @update-params="handleUpdateFxParams"
        />
      </div>

      <!-- 3.2 TURNTABLE -->
      <div class="col-span-5 flex justify-center items-center">
        <Turntable :color="color" :deck="deck" />
      </div>

      <!-- 3.3 PITCH -->
      <div class="col-span-3 flex flex-col items-center justify-between bg-[#08090d] border border-zinc-800/90 p-1.5 rounded-xl h-[155px]">
        <button
          @click="$emit('sync')"
          class="w-full py-1 rounded text-[8px] font-black tracking-wider border transition-all cursor-pointer flex items-center justify-center gap-1 active:scale-95"
          :class="deck.isSynced
            ? 'bg-blue-500 border-blue-400 text-white shadow-[0_0_8px_#3b82f6]'
            : 'bg-zinc-900 hover:bg-blue-950/40 border-zinc-700 text-zinc-300 hover:border-blue-500/50'"
        >
          <span class="w-1 h-1 rounded-full bg-blue-400"></span>
          SYNC
        </button>

        <div class="flex items-center justify-center my-0.5">
          <VerticalFader
            :model-value="deck.pitch"
            :min="-8"
            :max="8"
            :step="0.1"
            :height="80"
            :center-detent="true"
            :reset-value="0"
            :color="color === 'cyan' ? '#06b6d4' : '#f43f5e'"
            @update:model-value="$emit('pitch', $event)"
          />
        </div>

        <div class="w-full flex items-center justify-between pt-1 border-t border-zinc-800/80 px-0.5">
          <button
            @click="$emit('nudge', -0.2)"
            class="w-4 h-4 rounded bg-zinc-900 border border-zinc-800 text-[9px] font-black text-zinc-400 hover:text-white flex items-center justify-center cursor-pointer active:scale-90"
          >-</button>

          <span
            @dblclick="$emit('resetPitch')"
            class="text-[7.5px] font-mono cursor-pointer font-bold"
            :class="deck.pitch === 0 ? 'text-zinc-500' : 'text-amber-400'"
          >
            {{ (deck.pitch >= 0 ? "+" : "") + deck.pitch.toFixed(1) }}%
          </span>

          <button
            @click="$emit('nudge', 0.2)"
            class="w-4 h-4 rounded bg-zinc-900 border border-zinc-800 text-[9px] font-black text-zinc-400 hover:text-white flex items-center justify-center cursor-pointer active:scale-90"
          >+</button>
        </div>
      </div>

    </div>

    <!-- ================= 4. BOTTOM TRANSPORT ================= -->
    <div class="grid grid-cols-12 gap-2 pt-1.5 border-t border-zinc-800/80 mt-1 shrink-0">
      <button
        @click="$emit('cue')"
        class="col-span-5 py-2 rounded-xl border flex items-center justify-center gap-1.5 font-black text-xs tracking-wider transition-all cursor-pointer shadow-md active:scale-95"
        :class="deck.cueActive
          ? 'bg-amber-400 border-amber-300 text-black shadow-[0_0_12px_#f59e0b]'
          : 'bg-[#181a22] hover:bg-zinc-800 border-amber-500/40 text-amber-400'"
      >
        <span class="w-2 h-2 rounded-full border border-amber-400 bg-amber-950"></span>
        CUE
      </button>

      <button
        @click="$emit('play')"
        class="col-span-7 py-2 rounded-xl border flex items-center justify-center gap-2 font-black text-xs tracking-wider transition-all cursor-pointer shadow-md active:scale-95"
        :class="deck.isPlaying
          ? (color === 'cyan'
              ? 'bg-cyan-500 border-cyan-300 text-black shadow-[0_0_16px_rgba(6,182,212,0.6)]'
              : 'bg-rose-500 border-rose-300 text-white shadow-[0_0_16px_rgba(244,63,94,0.6)]')
          : 'bg-[#181a22] hover:bg-zinc-800 border-zinc-700 text-zinc-200'"
      >
        <span
          class="w-2.5 h-2.5 rounded-full"
          :class="deck.isPlaying ? 'bg-white shadow-[0_0_6px_#ffffff]' : (color === 'cyan' ? 'bg-cyan-400' : 'bg-rose-400')"
        ></span>
        {{ deck.isPlaying ? "PAUSE" : "PLAY" }}
      </button>
    </div>

  </div>
</template>