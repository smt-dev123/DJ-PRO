<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import Deck from "./components/Deck.vue";
import Mixer from "./components/Mixer.vue";
import WaveformDisplay from "./components/WaveformDisplay.vue";
import TrackLibrary from "./components/TrackLibrary.vue";
import { useDjEngine } from "./composables/useDjEngine";
import { getCurrentWebview } from "@tauri-apps/api/webview";

const {
  deckA,
  deckB,
  crossfader,
  masterVolume,
  openTrack,
  loadDirectTrack,
  loadFileToDeck,
  seekDeck,
  togglePlay,
  triggerCue,
  syncDeck,
  toggleKeyLock,
  triggerHotCue,
  toggleLoop,
  toggleStem,
  setVolume,
  setDeckEq,
  setDeckGain,
  setDeckFxType,
  setDeckFxAmount,
  setDeckFxActive,
  setPitch,
  nudgePitch,
  resetPitch,
  setCrossfader,
  triggerFxOut,
  cancelFxOut,
} = useDjEngine();

const deckARef = ref<any>(null);
const deckBRef = ref<any>(null);
const libraryRef = ref<any>(null);

let unlistenDragDrop: (() => void) | null = null;

// Native OS File Drag & Drop (Tauri v2)
onMounted(async () => {
  try {
    const webview = getCurrentWebview();
    unlistenDragDrop = await webview.onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        const paths = event.payload.paths;
        const pos = event.payload.position;
        if (!paths || paths.length === 0) return;

        // Convert physical coordinates to CSS logical coordinates
        const scale = window.devicePixelRatio || 1;
        const cssX = pos.x / scale;
        const cssY = pos.y / scale;

        // Check if dropped inside Deck A
        if (deckARef.value?.$el) {
          const rect = deckARef.value.$el.getBoundingClientRect();
          if (
            cssX >= rect.left &&
            cssX <= rect.right &&
            cssY >= rect.top &&
            cssY <= rect.bottom
          ) {
            loadFileToDeck("A", paths[0]);
            return;
          }
        }

        // Check if dropped inside Deck B
        if (deckBRef.value?.$el) {
          const rect = deckBRef.value.$el.getBoundingClientRect();
          if (
            cssX >= rect.left &&
            cssX <= rect.right &&
            cssY >= rect.top &&
            cssY <= rect.bottom
          ) {
            loadFileToDeck("B", paths[0]);
            return;
          }
        }

        // Otherwise import into Track Library active playlist
        if (libraryRef.value) {
          libraryRef.value.addAudioFiles(paths);
        }
      }
    });
  } catch (err) {
    // Non-Tauri browser preview mode fallback
  }
});

onUnmounted(() => {
  if (unlistenDragDrop) {
    unlistenDragDrop();
  }
});
</script>

<template>
  <div
    class="h-screen w-screen bg-[#06070a] text-zinc-200 select-none flex flex-col p-2.5 font-mono overflow-hidden"
  >
    <!-- ================= 1. TOP HEADER APP BAR ================= -->
    <header
      class="flex items-center justify-between border-b border-zinc-800/80 pb-2 mb-2 shrink-0"
    >
      <!-- App Brand & Title -->
      <div class="flex items-center gap-2.5">
        <div class="relative flex items-center justify-center">
          <span
            class="w-2.5 h-2.5 rounded-full bg-cyan-400 shadow-[0_0_10px_#22d3ee]"
          ></span>
          <span
            class="absolute w-4 h-4 rounded-full bg-cyan-400/20 animate-ping"
          ></span>
        </div>
        <h1
          class="text-sm font-black tracking-widest text-zinc-100 uppercase flex items-center gap-1.5"
        >
          DJ PRO
          <span
            class="text-cyan-400 text-[8.5px] px-1.5 py-0.5 border border-cyan-500/40 rounded-md bg-cyan-950/40 font-bold"
          >
            STUDIO EDITION
          </span>
        </h1>
      </div>

      <!-- Center Master Tempo / Clock Bar -->
      <div
        class="hidden sm:flex items-center gap-4 bg-[#0a0b10] border border-zinc-800/80 px-3 py-1 rounded-full text-[9px]"
      >
        <div class="flex items-center gap-1.5">
          <span class="text-zinc-500">MASTER CLOCK:</span>
          <span class="font-black text-white font-mono"
            >{{
              deckA.isPlaying
                ? deckA.bpm.toFixed(1)
                : deckB.isPlaying
                  ? deckB.bpm.toFixed(1)
                  : "126.0"
            }}
            BPM</span
          >
        </div>
        <span class="text-zinc-700">|</span>
        <div class="flex items-center gap-1.5">
          <span class="text-zinc-500">QUANTIZE:</span>
          <span class="text-emerald-400 font-bold">1 BEAT</span>
        </div>
      </div>

      <!-- Right Audio Engine Status -->
      <div class="flex items-center gap-2">
        <div
          class="text-[9px] text-zinc-400 bg-[#0c0d13] border border-zinc-800 px-3 py-1 rounded-full flex items-center gap-1.5 shadow-xs"
        >
          <span
            class="w-1.5 h-1.5 rounded-full bg-emerald-400 shadow-[0_0_6px_#34d399]"
          ></span>
          <span>AUDIO ENGINE:</span>
          <span class="text-emerald-400 font-bold"
            >WASAPI REALTIME (48kHz)</span
          >
        </div>
      </div>
    </header>

    <!-- ================= 2. WAVEFORM OVERVIEW SECTION ================= -->
    <section class="mb-2 shrink-0">
      <WaveformDisplay
        :deck-a="deckA"
        :deck-b="deckB"
        @seek-a="seekDeck('A', $event)"
        @seek-b="seekDeck('B', $event)"
      />
    </section>

    <!-- ================= 3. HARDWARE CONSOLE (DECK A | MIXER | DECK B) ================= -->
    <main class="flex items-stretch gap-2.5 mb-2.5 h-[340px] shrink-0">
      <!-- Deck A (Left) -->
      <Deck
        ref="deckARef"
        class="flex-1 min-w-0"
        :deck="deckA"
        color="cyan"
        @fx-out="(fxType, duration) => triggerFxOut('A', fxType, duration)"
        @cancel-fx-out="cancelFxOut('A')"
        @open="openTrack('A')"
        @play="togglePlay('A')"
        @cue="triggerCue('A')"
        @sync="syncDeck('A')"
        @key-lock="toggleKeyLock('A')"
        @pitch="setPitch('A', $event)"
        @nudge="nudgePitch('A', $event)"
        @reset-pitch="resetPitch('A')"
        @load-track="loadDirectTrack('A', $event)"
        @load-file="loadFileToDeck('A', $event)"
        @seek="seekDeck('A', $event)"
        @hot-cue="triggerHotCue('A', $event)"
        @toggle-loop="toggleLoop('A', $event)"
        @toggle-stem="(stem, active) => toggleStem('A', stem, active)"
        @fx-type="setDeckFxType('A', $event)"
        @fx-amount="setDeckFxAmount('A', $event)"
        @fx-active="setDeckFxActive('A', $event)"
      />

      <!-- Center Hardware Mixer -->
      <Mixer
        class="w-[280px] lg:w-[310px] shrink-0"
        :deck-a="deckA"
        :deck-b="deckB"
        :crossfader="crossfader"
        :master-volume="masterVolume"
        @crossfade="setCrossfader"
        @volume-a="setVolume('A', $event)"
        @volume-b="setVolume('B', $event)"
        @eq-a="(h, m, l, f) => setDeckEq('A', h, m, l, f)"
        @eq-b="(h, m, l, f) => setDeckEq('B', h, m, l, f)"
        @gain-a="setDeckGain('A', $event)"
        @gain-b="setDeckGain('B', $event)"
      />

      <!-- Deck B (Right) -->
      <Deck
        ref="deckBRef"
        class="flex-1 min-w-0"
        :deck="deckB"
        color="rose"
        @fx-out="(fxType, duration) => triggerFxOut('B', fxType, duration)"
        @cancel-fx-out="cancelFxOut('B')"
        @open="openTrack('B')"
        @play="togglePlay('B')"
        @cue="triggerCue('B')"
        @sync="syncDeck('B')"
        @key-lock="toggleKeyLock('B')"
        @pitch="setPitch('B', $event)"
        @nudge="nudgePitch('B', $event)"
        @reset-pitch="resetPitch('B')"
        @load-track="loadDirectTrack('B', $event)"
        @load-file="loadFileToDeck('B', $event)"
        @seek="seekDeck('B', $event)"
        @hot-cue="triggerHotCue('B', $event)"
        @toggle-loop="toggleLoop('B', $event)"
        @toggle-stem="(stem, active) => toggleStem('B', stem, active)"
        @fx-type="setDeckFxType('B', $event)"
        @fx-amount="setDeckFxAmount('B', $event)"
        @fx-active="setDeckFxActive('B', $event)"
      />
    </main>

    <!-- ================= 4. MUSIC LIBRARY BROWSER ================= -->
    <section class="flex-1 min-h-0 overflow-hidden">
      <TrackLibrary ref="libraryRef" @load-to-deck="loadDirectTrack" />
    </section>
  </div>
</template>
