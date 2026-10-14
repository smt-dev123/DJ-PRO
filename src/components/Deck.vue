<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import Turntable from "./Turntable.vue";
import VerticalFader from "./VerticalFader.vue";
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
}>();

type TabMode = "HOT CUE" | "LOOP" | "STEMS" | "FX";
const activeTab = ref<TabMode>("HOT CUE");
const isDragOver = ref(false);

const stems = ref({
  vocal: true,
  drum: true,
  bass: true,
  melody: true,
});

const activeLoop = ref<string | null>(null);

function toggleLoop(beats: string) {
  if (activeLoop.value === beats) {
    activeLoop.value = null;
  } else {
    activeLoop.value = beats;
  }
}

function handleDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = "copy";
  }
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

// ================= DJAY PRO DECK WAVE SCRUBBER =================
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

// ================= WAVEFORM SAMPLING (FIXED HIGH COUNT) =================
// ✅ Bar count ថេរ — ល្អិតខ្លាំង មិនគណនាតាម width
const BAR_COUNT = 1500;

const waveformBars = computed<WaveBand[]>(() => {
  const count = BAR_COUNT;
  const source = props.deck.waveform;

  // ✅ បើមាន waveform data ពិត
  if (source && source.length > 0) {
    // បើ data តិចជាង count → interpolate ដើម្បីបំពេញ
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

    // បើ data ច្រើនជាង count → sample តាម step
    const step = source.length / count;
    const sampled: WaveBand[] = [];
    for (let i = 0; i < count; i++) {
      const idx = Math.min(source.length - 1, Math.floor(i * step));
      sampled.push(source[idx]);
    }
    return sampled;
  }

  // ✅ Fallback ពេលគ្មាន data
  const fallback: WaveBand[] = [];
  for (let i = 0; i < count; i++) {
    fallback.push({
      low: 0.2 + Math.abs(Math.sin(i * 0.15)) * 0.5,
      mid: 0.15 + Math.abs(Math.cos(i * 0.2)) * 0.45,
      high: 0.1 + Math.abs(Math.sin(i * 0.35)) * 0.4,
    });
  }
  return fallback;
});

// ================= CANVAS WAVEFORM SCRUBBER =================
const scrubberCanvasRef = ref<HTMLCanvasElement | null>(null);

// ================= CANVAS WAVEFORM SCRUBBER (KICK DOMINANT + DYNAMIC COLOR) =================
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

  // Background ស្រាលបន្តិច
  ctx.fillStyle = "#0a0d14";
  ctx.fillRect(0, 0, cssWidth, cssHeight);

  const bands = waveformBars.value;
  const numBars = bands.length;
  if (numBars === 0) {
    ctx.restore();
    return;
  }

  const centerY = cssHeight / 2;
  const maxAmp = cssHeight / 2 - 4;

  // Bar width ល្អិត + Gap តូច
  const barWidth = cssWidth / numBars;
  const gap = Math.max(0.15, barWidth * 0.10);
  const actualBarWidth = Math.max(0.3, barWidth - gap);

  // ✅ គណនា global peaks
  let globalKick = 0.01;
  let globalSnare = 0.01;
  let globalClap = 0.01;

  for (let i = 0; i < numBars; i++) {
    const band = bands[i];
    if (band.low > globalKick) globalKick = band.low;
    if (band.mid > globalSnare) globalSnare = band.mid;
    if (band.high > globalClap) globalClap = band.high;
  }

  // ================= DRAW WAVEFORM (KICK DOMINANT + DYNAMIC COLOR) =================
  for (let i = 0; i < numBars; i++) {
    const band = bands[i];
    const x = i * barWidth;
    const isPlayed = i / numBars <= props.deck.progress;
    const alpha = isPlayed ? 1.0 : 0.85;

    // ✅ Kick DOMINANT — low frequency ជាធំ
    const kickNorm = band.low / globalKick;
    const snareNorm = band.mid / globalSnare;
    const clapNorm = band.high / globalClap;

    // ✅ គណនា amplitude ផ្អែកលើ Kick ជាចម្បង
    //    Kick = 100%, Snare = 40%, Clap = 20%
    const amp = Math.max(
      kickNorm * 1.0,
      snareNorm * 0.40,
      clapNorm * 0.20
    );

    // ✅ Normalize តាម contrast
    let normalized = Math.pow(amp, 0.75);
    const h = Math.max(0.8, normalized * maxAmp);

    // ✅ កំណត់ពណ៌តាម frequency content ពិត
    //    បើ Kick ខ្លាំង → ក្រហម
    //    បើ Vocal ខ្លាំង (mid ខ្ពស់) → លឿង
    //    បើ Midi ខ្លាំង (high ខ្ពស់) → ស្វាយ/ខៀវ

    const kickStrength = kickNorm * 1.0;
    const vocalStrength = snareNorm * 1.0; // mid = vocal
    const midiStrength = clapNorm * 1.0;   // high = hi-hat/midi

    const maxStrength = Math.max(kickStrength, vocalStrength, midiStrength);

    let barColor: string;

    if (maxStrength < 0.2) {
      // Quiet — ពណ៌ប្រផេះងងឹត
      barColor = "#1e293b";
    } else if (kickStrength === maxStrength && kickStrength > 0.5) {
      // ✅ Kick ខ្លាំង → ក្រហម
      barColor = "#ef4444";
    } else if (vocalStrength === maxStrength && vocalStrength > 0.5) {
      // ✅ Vocal ខ្លាំង → លឿង (សំឡេងច្រៀង)
      barColor = "#eab308";
    } else if (midiStrength === maxStrength && midiStrength > 0.5) {
      // ✅ Midi ខ្លាំង → ស្វាយ (មេឌី)
      barColor = "#a855f7";
    } else {
      // Blend ពណ៌តាម frequency ពិត
      const r = Math.round(kickNorm * 239);
      const g = Math.round(vocalStrength * 180);
      const b = Math.round(midiStrength * 246);

      // ✅ បង្កើនពន្លឺបើ amplitude ខ្ពស់
      const brightness = 0.6 + normalized * 0.4;
      barColor = `rgb(${Math.round(r * brightness)}, ${Math.round(g * brightness)}, ${Math.round(b * brightness)})`;
    }

    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.fillStyle = barColor;
    ctx.fillRect(x, centerY - h, actualBarWidth, h * 2);
    ctx.restore();
  }

  // ================= HIGHLIGHT: Kick peaks ភ្លឺ =================
  for (let i = 0; i < numBars; i++) {
    const band = bands[i];
    const x = i * barWidth;
    const isPlayed = i / numBars <= props.deck.progress;
    const alpha = isPlayed ? 1.0 : 0.35;

    const kickNorm = band.low / globalKick;
    if (kickNorm > 0.75) {
      const amp = Math.max(
        kickNorm * 1.0,
        (band.mid / globalSnare) * 0.40,
        (band.high / globalClap) * 0.20
      );
      const normalized = Math.pow(amp, 0.75);
      const h = Math.max(0.8, normalized * maxAmp);

      ctx.save();
      ctx.globalAlpha = alpha * 0.5;
      ctx.fillStyle = "#fca5a5"; // Highlight ភ្លឺ
      ctx.fillRect(x, centerY - h, actualBarWidth, h * 0.35);
      ctx.restore();
    }
  }

  // ================= CENTER SPINE =================
  ctx.save();
  ctx.strokeStyle = "rgba(255, 255, 255, 0.25)";
  ctx.lineWidth = 0.8;
  ctx.beginPath();
  ctx.moveTo(0, centerY);
  ctx.lineTo(cssWidth, centerY);
  ctx.stroke();
  ctx.restore();

  // ================= DIM UNPLAYED =================
  if (props.deck.progress < 1) {
    ctx.save();
    ctx.fillStyle = "rgba(0, 0, 0, 0.35)";
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

// ================= WATCHERS =================
watch(
  () => props.deck.waveform,
  () => {
    requestAnimationFrame(drawScrubberCanvas);
  },
  { deep: false }
);

watch(
  () => props.deck.progress,
  () => {
    requestAnimationFrame(drawScrubberCanvas);
  }
);

// ================= LIFECYCLE =================
function handleResize() {
  requestAnimationFrame(drawScrubberCanvas);
}

onMounted(() => {
  requestAnimationFrame(drawScrubberCanvas);
  window.addEventListener("resize", handleResize);
});

onUnmounted(() => {
  window.removeEventListener("resize", handleResize);
});

// ================= COMPUTED =================
const remainingTime = computed(() => {
  const total = props.deck.durationSecs || 240;
  const current = (props.deck.progress || 0) * total;
  const rem = Math.max(0, Math.round(total - current));
  const mins = Math.floor(rem / 60);
  const secs = rem % 60;
  return `-${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
});
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
    <!-- Drag & Drop Overlay -->
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
        Release to load into Deck {{ deck.id }} immediately
      </span>
    </div>

    <!-- ================= 1. DECK TOP HEADER ================= -->
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
          title="Toggle Master Tempo / Key Lock"
          class="px-2 py-1 rounded text-[8px] font-black border transition-all cursor-pointer"
          :class="deck.keyLock
            ? (color === 'cyan' ? 'bg-cyan-950/60 border-cyan-400 text-cyan-300' : 'bg-rose-950/60 border-rose-400 text-rose-300')
            : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
        >
          KEY LOCK
        </button>

        <button
          @click="$emit('open')"
          class="px-2.5 py-1 bg-zinc-800/90 hover:bg-zinc-700 active:scale-95 text-[9px] font-black rounded-lg border border-zinc-700 text-zinc-200 transition shrink-0 cursor-pointer flex items-center gap-1 shadow-xs"
        >
          <span>📂</span> LOAD
        </button>
      </div>
    </div>

    <!-- ================= 2. DJAY PRO WAVE SCRUBBER ================= -->
    <div class="flex flex-col gap-1 mb-1.5 shrink-0">
      <!-- Header row -->
      <div class="flex items-center justify-between text-[7px] font-bold text-zinc-400 px-1">
        <div class="flex items-center gap-1.5 font-mono">
          <span
            class="px-1 py-0.5 rounded text-[7px] font-black"
            :class="color === 'cyan' ? 'bg-cyan-950/70 text-cyan-300 border border-cyan-500/30' : 'bg-rose-950/70 text-rose-300 border border-rose-500/30'"
          >
            {{ deck.currentTime }}
          </span>
          <span class="text-[6.5px] text-zinc-500 tracking-wider">/ {{ deck.totalTime }}</span>
        </div>

        <!-- Legend -->
        <div class="flex items-center gap-2.5 text-[7.5px] font-sans font-semibold">
          <div class="flex items-center gap-1" title="Kick Drum (ភ្លេងបុក)">
            <span class="w-1.5 h-1.5 rounded-full bg-red-500"></span>
            <span class="text-red-400">KICK</span>
          </div>
          <div class="flex items-center gap-1" title="Snare (ស្គរ)">
            <span class="w-1.5 h-1.5 rounded-full bg-green-500"></span>
            <span class="text-green-400">SNARE</span>
          </div>
          <div class="flex items-center gap-1" title="Clap (ខ្ទាស់ដៃ)">
            <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
            <span class="text-blue-400">CLAP</span>
          </div>
        </div>

        <div class="font-mono text-[7px] font-bold" :class="color === 'cyan' ? 'text-cyan-400' : 'text-rose-400'">
          {{ remainingTime }}
        </div>
      </div>

      <!-- Waveform Scrubber Track (Canvas) -->
      <div
        @mousedown="handleScrubberMouseDown"
        title="Click or drag to scrub/seek"
        class="h-12 bg-[#050609] border rounded-lg relative cursor-ew-resize group overflow-hidden select-none"
        :class="color === 'cyan' ? 'border-cyan-500/30 hover:border-cyan-400/80' : 'border-rose-500/30 hover:border-rose-400/80'"
      >
        <canvas
          ref="scrubberCanvasRef"
          class="w-full h-full block"
        ></canvas>

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

        <!-- Elapsed Color Wash -->
        <div
          class="absolute left-0 top-0 bottom-0 pointer-events-none z-10"
          :class="color === 'cyan' ? 'bg-gradient-to-r from-cyan-950/20 to-cyan-500/5' : 'bg-gradient-to-r from-rose-950/20 to-rose-500/5'"
          :style="{ width: `${deck.progress * 100}%` }"
        ></div>
      </div>
    </div>

    <!-- ================= 3. MAIN CENTER ================= -->
    <div class="grid grid-cols-12 gap-2.5 items-center flex-1 my-0.5">

      <!-- 3.1 LEFT: PADS -->
      <div class="col-span-4 bg-[#08090d] border border-zinc-800/90 rounded-xl p-2 flex flex-col justify-between h-[155px]">
        <div class="grid grid-cols-4 gap-0.5 bg-black/60 p-0.5 rounded-lg border border-zinc-800 mb-1">
          <button
            v-for="tab in (['HOT CUE', 'LOOP', 'STEMS', 'FX'] as TabMode[])" :key="tab"
            @click="activeTab = tab"
            class="text-[6.5px] font-black py-1 rounded transition-all cursor-pointer truncate text-center"
            :class="activeTab === tab
              ? (color === 'cyan' ? 'bg-cyan-500 text-black shadow-xs font-black' : 'bg-rose-500 text-white shadow-xs font-black')
              : 'text-zinc-500 hover:text-zinc-300'"
          >
            {{ tab.replace('HOT ', '') }}
          </button>
        </div>

        <div v-if="activeTab === 'HOT CUE'" class="grid grid-cols-2 gap-1.5 flex-1">
          <button
            v-for="pad in [1, 2, 3, 4]" :key="'cue-' + pad"
            class="rounded-lg border text-[9px] font-black active:scale-95 transition-all cursor-pointer flex flex-col items-center justify-center p-1 shadow-xs group"
            :class="color === 'cyan'
              ? 'bg-cyan-950/20 border-cyan-500/30 text-cyan-300 hover:bg-cyan-500/25 hover:border-cyan-400'
              : 'bg-rose-950/20 border-rose-500/30 text-rose-300 hover:bg-rose-500/25 hover:border-rose-400'"
          >
            <span>CUE {{ pad }}</span>
            <span class="text-[6.5px] text-zinc-500 group-hover:text-zinc-300 font-mono">0{{ pad }}:0{{ pad * 8 }}</span>
          </button>
        </div>

        <div v-else-if="activeTab === 'LOOP'" class="grid grid-cols-2 gap-1.5 flex-1">
          <button
            v-for="l in ['1/2', '1', '2', '4']" :key="'loop-' + l"
            @click="toggleLoop(l)"
            class="rounded-lg border text-[9px] font-black active:scale-95 transition-all cursor-pointer flex items-center justify-center shadow-xs"
            :class="activeLoop === l
              ? 'bg-amber-500 border-amber-300 text-black shadow-[0_0_8px_#f59e0b]'
              : 'bg-amber-950/20 border-amber-500/30 text-amber-300 hover:bg-amber-500/20'"
          >
            {{ l }} BEAT
          </button>
        </div>

        <div v-else-if="activeTab === 'STEMS'" class="grid grid-cols-2 gap-1.5 flex-1">
          <button
            @click="stems.vocal = !stems.vocal"
            class="rounded-lg border text-[8px] font-black cursor-pointer flex items-center justify-center transition-all"
            :class="stems.vocal ? 'bg-emerald-500/25 border-emerald-400 text-emerald-300 shadow-[0_0_6px_rgba(16,185,129,0.3)]' : 'bg-zinc-900 border-zinc-800 text-zinc-600'"
          >
            🎤 VOCAL
          </button>
          <button
            @click="stems.drum = !stems.drum"
            class="rounded-lg border text-[8px] font-black cursor-pointer flex items-center justify-center transition-all"
            :class="stems.drum ? 'bg-amber-500/25 border-amber-400 text-amber-300 shadow-[0_0_6px_rgba(245,158,11,0.3)]' : 'bg-zinc-900 border-zinc-800 text-zinc-600'"
          >
            🥁 DRUMS
          </button>
          <button
            @click="stems.bass = !stems.bass"
            class="rounded-lg border text-[8px] font-black cursor-pointer flex items-center justify-center transition-all"
            :class="stems.bass ? 'bg-indigo-500/25 border-indigo-400 text-indigo-300 shadow-[0_0_6px_rgba(99,102,241,0.3)]' : 'bg-zinc-900 border-zinc-800 text-zinc-600'"
          >
            🎸 BASS
          </button>
          <button
            @click="stems.melody = !stems.melody"
            class="rounded-lg border text-[8px] font-black cursor-pointer flex items-center justify-center transition-all"
            :class="stems.melody ? 'bg-sky-500/25 border-sky-400 text-sky-300 shadow-[0_0_6px_rgba(14,165,233,0.3)]' : 'bg-zinc-900 border-zinc-800 text-zinc-600'"
          >
            🎹 MELODY
          </button>
        </div>

        <div v-else class="grid grid-cols-2 gap-1.5 flex-1">
          <button
            v-for="fx in ['ECHO', 'REVERB', 'FLANGER', 'ROLL']" :key="fx"
            class="rounded-lg border border-purple-500/30 bg-purple-950/20 text-purple-300 text-[8px] font-black active:scale-95 transition-all cursor-pointer flex items-center justify-center hover:bg-purple-500/25"
          >
            {{ fx }}
          </button>
        </div>
      </div>

      <!-- 3.2 CENTER: TURNTABLE -->
      <div class="col-span-5 flex justify-center items-center">
        <Turntable :color="color" :deck="deck" />
      </div>

      <!-- 3.3 RIGHT: PITCH -->
      <div class="col-span-3 flex flex-col items-center justify-between bg-[#08090d] border border-zinc-800/90 p-1.5 rounded-xl h-[155px]">
        <button
          @click="$emit('sync')"
          title="Instant BPM Sync with Master Deck"
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
            title="Pitch Bend -"
          >
            -
          </button>

          <span
            @dblclick="$emit('resetPitch')"
            title="Double-click to reset"
            class="text-[7.5px] font-mono cursor-pointer font-bold"
            :class="deck.pitch === 0 ? 'text-zinc-500' : 'text-amber-400'"
          >
            {{ (deck.pitch >= 0 ? "+" : "") + deck.pitch.toFixed(1) }}%
          </span>

          <button
            @click="$emit('nudge', 0.2)"
            class="w-4 h-4 rounded bg-zinc-900 border border-zinc-800 text-[9px] font-black text-zinc-400 hover:text-white flex items-center justify-center cursor-pointer active:scale-90"
            title="Pitch Bend +"
          >
            +
          </button>
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