<script setup lang="ts">
import { ref, watch } from "vue";
import Knob from "./Knob.vue";
import VerticalFader from "./VerticalFader.vue";
import type { DeckState } from "../types/dj";

const props = defineProps<{
  deckA: DeckState;
  deckB: DeckState;
  crossfader: number;
  masterVolume?: number;
}>();

const emit = defineEmits<{
  (e: "crossfade", val: number): void;
  (e: "volumeA", val: number): void;
  (e: "volumeB", val: number): void;
  (e: "masterVolume", val: number): void;
  (e: "eqA", high: number, mid: number, low: number, filter: number): void;
  (e: "eqB", high: number, mid: number, low: number, filter: number): void;
  (e: "gainA", val: number): void;
  (e: "gainB", val: number): void;
}>();

const cueHeadphoneA = ref(true);
const cueHeadphoneB = ref(false);

// ✅ Trim values (separate from deck.gain for smooth drag)
const trimA = ref(props.deckA.gain ?? 50);
const trimB = ref(props.deckB.gain ?? 50);

// ✅ EQ values
const eqAHigh = ref(props.deckA.eqHigh ?? 0);
const eqAMid = ref(props.deckA.eqMid ?? 0);
const eqALow = ref(props.deckA.eqLow ?? 0);
const eqAFilter = ref(props.deckA.filter ?? 0);

const eqBHigh = ref(props.deckB.eqHigh ?? 0);
const eqBMid = ref(props.deckB.eqMid ?? 0);
const eqBLow = ref(props.deckB.eqLow ?? 0);
const eqBFilter = ref(props.deckB.filter ?? 0);

// ✅ Watch props → update local state
watch(() => props.deckA.gain, (v) => { trimA.value = v ?? 50; });
watch(() => props.deckB.gain, (v) => { trimB.value = v ?? 50; });

watch(() => props.deckA.eqHigh, (v) => { eqAHigh.value = v ?? 0; });
watch(() => props.deckA.eqMid, (v) => { eqAMid.value = v ?? 0; });
watch(() => props.deckA.eqLow, (v) => { eqALow.value = v ?? 0; });
watch(() => props.deckA.filter, (v) => { eqAFilter.value = v ?? 0; });

watch(() => props.deckB.eqHigh, (v) => { eqBHigh.value = v ?? 0; });
watch(() => props.deckB.eqMid, (v) => { eqBMid.value = v ?? 0; });
watch(() => props.deckB.eqLow, (v) => { eqBLow.value = v ?? 0; });
watch(() => props.deckB.filter, (v) => { eqBFilter.value = v ?? 0; });

// ✅ Emit EQ changes
function emitEqA() {
  emit("eqA", eqAHigh.value, eqAMid.value, eqALow.value, eqAFilter.value);
}

function emitEqB() {
  emit("eqB", eqBHigh.value, eqBMid.value, eqBLow.value, eqBFilter.value);
}

// ✅ Trim changes
watch(trimA, (v) => emit("gainA", v));
watch(trimB, (v) => emit("gainB", v));

// ✅ EQ preset save/load
const presets = ref<{
  name: string;
  a: { high: number; mid: number; low: number; filter: number };
  b: { high: number; mid: number; low: number; filter: number };
}[]>([]);

const showPresetModal = ref(false);
const presetName = ref("");

function savePreset() {
  if (!presetName.value.trim()) return;
  presets.value.push({
    name: presetName.value.trim(),
    a: {
      high: eqAHigh.value,
      mid: eqAMid.value,
      low: eqALow.value,
      filter: eqAFilter.value,
    },
    b: {
      high: eqBHigh.value,
      mid: eqBMid.value,
      low: eqBLow.value,
      filter: eqBFilter.value,
    },
  });
  presetName.value = "";
  showPresetModal.value = false;
}

function loadPreset(preset: typeof presets.value[0]) {
  eqAHigh.value = preset.a.high;
  eqAMid.value = preset.a.mid;
  eqALow.value = preset.a.low;
  eqAFilter.value = preset.a.filter;

  eqBHigh.value = preset.b.high;
  eqBMid.value = preset.b.mid;
  eqBLow.value = preset.b.low;
  eqBFilter.value = preset.b.filter;

  emitEqA();
  emitEqB();
}

function resetEq(deck: "A" | "B") {
  if (deck === "A") {
    eqAHigh.value = 0;
    eqAMid.value = 0;
    eqALow.value = 0;
    eqAFilter.value = 0;
    emitEqA();
  } else {
    eqBHigh.value = 0;
    eqBMid.value = 0;
    eqBLow.value = 0;
    eqBFilter.value = 0;
    emitEqB();
  }
}
</script>

<template>
  <div class="bg-[#0b0c11] border border-zinc-800/90 rounded-xl p-2 flex flex-col justify-between h-full select-none font-mono shadow-2xl transition-all overflow-hidden">

    <!-- ================= 1. MIXER TOP TITLE & TRIM BAR ================= -->
    <div class="w-full flex items-center justify-between border-b border-zinc-800/80 pb-1 px-1 shrink-0">
      <!-- CH 1 Badge + Trim -->
      <div class="flex items-center gap-1.5">
        <span class="w-1.5 h-1.5 rounded-full bg-cyan-400 shadow-[0_0_4px_#22d3ee]"></span>
        <span class="text-[8px] font-black text-cyan-400">CH 1</span>
        <Knob
          label="TRIM"
          v-model="trimA"
          color="#38bdf8"
          size="sm"
          :show-value="true"
        />
      </div>

      <!-- Center Title -->
      <div class="flex flex-col items-center">
        <span class="text-[7.5px] font-black text-zinc-300 tracking-widest uppercase leading-none">MIXER</span>
        <span class="text-[6px] text-amber-400 font-bold tracking-tighter">STUDIO PRO</span>
      </div>

      <!-- CH 2 Trim + Badge -->
      <div class="flex items-center gap-1.5">
        <Knob
          label="TRIM"
          v-model="trimB"
          color="#fb7185"
          size="sm"
          :show-value="true"
        />
        <span class="text-[8px] font-black text-rose-400">CH 2</span>
        <span class="w-1.5 h-1.5 rounded-full bg-rose-400 shadow-[0_0_4px_#f43f5e]"></span>
      </div>
    </div>

    <!-- ================= 2. 4-BAND EQ SECTION ================= -->
    <div class="grid grid-cols-2 gap-1 w-full bg-[#08090d] border border-zinc-800/80 rounded-xl p-1 my-0.5 shrink-0">

      <!-- CH 1 EQ Column -->
      <div class="flex flex-col items-center justify-between gap-1 relative">
        <!-- Reset button -->
        <button
          @click="resetEq('A')"
          class="absolute top-0 right-0 text-[5.5px] font-black text-zinc-600 hover:text-cyan-400 transition-colors"
          title="Reset EQ"
        >RST</button>

        <Knob
          label="HI"
          v-model="eqAHigh"
          color="#06b6d4"
          size="sm"
          :show-value="true"
          @update:model-value="emitEqA"
        />
        <Knob
          label="MID"
          v-model="eqAMid"
          color="#06b6d4"
          size="sm"
          :show-value="true"
          @update:model-value="emitEqA"
        />
        <Knob
          label="LOW"
          v-model="eqALow"
          color="#06b6d4"
          size="sm"
          :show-value="true"
          @update:model-value="emitEqA"
        />
        <Knob
          label="FILTER"
          v-model="eqAFilter"
          color="#a855f7"
          size="sm"
          :show-value="true"
          @update:model-value="emitEqA"
        />
      </div>

      <!-- CH 2 EQ Column -->
      <div class="flex flex-col items-center justify-between gap-1 border-l border-zinc-800/80 pl-1 relative">
        <button
          @click="resetEq('B')"
          class="absolute top-0 right-0 text-[5.5px] font-black text-zinc-600 hover:text-rose-400 transition-colors"
          title="Reset EQ"
        >RST</button>

        <Knob
          label="HI"
          v-model="eqBHigh"
          color="#f43f5e"
          size="sm"
          :show-value="true"
          @update:model-value="emitEqB"
        />
        <Knob
          label="MID"
          v-model="eqBMid"
          color="#f43f5e"
          size="sm"
          :show-value="true"
          @update:model-value="emitEqB"
        />
        <Knob
          label="LOW"
          v-model="eqBLow"
          color="#f43f5e"
          size="sm"
          :show-value="true"
          @update:model-value="emitEqB"
        />
        <Knob
          label="FILTER"
          v-model="eqBFilter"
          color="#a855f7"
          size="sm"
          :show-value="true"
          @update:model-value="emitEqB"
        />
      </div>
    </div>

    <!-- ✅ Preset Save/Load Bar -->
    <div class="w-full flex items-center justify-between gap-1 my-0.5 shrink-0">
      <button
        @click="showPresetModal = true"
        class="flex-1 py-0.5 bg-zinc-900 hover:bg-cyan-950/40 border border-zinc-800 hover:border-cyan-500/50 rounded text-[6.5px] font-black text-zinc-400 hover:text-cyan-300 transition-all cursor-pointer"
        title="Save EQ Preset"
      >
        💾 SAVE EQ
      </button>

      <select
        v-if="presets.length > 0"
        @change="(e) => {
          const idx = parseInt((e.target as HTMLSelectElement).value);
          if (idx >= 0) loadPreset(presets[idx]);
        }"
        class="flex-1 py-0.5 bg-zinc-900 border border-zinc-800 rounded text-[6.5px] font-black text-zinc-400 cursor-pointer"
      >
        <option value="-1">LOAD PRESET</option>
        <option v-for="(p, idx) in presets" :key="idx" :value="idx">{{ p.name }}</option>
      </select>
    </div>

    <!-- ================= 3. FADERS & VU METER ================= -->
    <div class="grid grid-cols-12 gap-1 items-center bg-[#08090d] border border-zinc-800/80 rounded-xl p-1 my-0.5 shrink-0">

      <!-- 3.1 CH 1 FADER & CUE -->
      <div class="col-span-5 flex flex-col items-center justify-between h-[80px]">
        <button
          @click="cueHeadphoneA = !cueHeadphoneA"
          title="Headphone Monitor CH 1"
          class="w-full py-0.5 rounded text-[6.5px] font-black border transition-all cursor-pointer flex items-center justify-center gap-1"
          :class="cueHeadphoneA
            ? 'bg-amber-400 border-amber-300 text-black shadow-[0_0_6px_#f59e0b]'
            : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
        >
          🎧 CUE 1
        </button>

        <div class="flex items-center justify-center my-0.5">
          <VerticalFader
            :model-value="deckA.volume"
            :min="0"
            :max="1"
            :step="0.01"
            :height="48"
            :reset-value="0.8"
            color="#06b6d4"
            @update:model-value="$emit('volumeA', $event)"
          />
        </div>

        <span class="text-[6px] text-cyan-400 font-bold font-mono leading-none">
          {{ Math.round(deckA.volume * 100) }}%
        </span>
      </div>

      <!-- 3.2 VU METER -->
      <div class="col-span-2 flex items-center justify-center gap-0.5 h-[80px] bg-black/60 border border-zinc-800/80 rounded-lg p-0.5">
        <div class="flex flex-col-reverse justify-between h-full w-1 bg-[#050608] p-[0.5px] rounded">
          <div
            v-for="i in 10" :key="'vu-a-' + i"
            class="w-full h-1 rounded-[0.5px] transition-all duration-75"
            :class="[
              deckA.isPlaying && (deckA.volume * (1 - crossfader) * 10) >= i
                ? (i >= 9
                    ? 'bg-rose-500 shadow-[0_0_3px_#f43f5e]'
                    : (i >= 7 ? 'bg-amber-400 shadow-[0_0_2px_#fbbf24]' : 'bg-emerald-400 shadow-[0_0_2px_#34d399]'))
                : 'bg-zinc-800/30'
            ]"
          ></div>
        </div>

        <div class="flex flex-col justify-between h-full text-[4.5px] text-zinc-600 font-bold select-none py-0.5">
          <span>+</span>
          <span>0</span>
          <span>-</span>
        </div>

        <div class="flex flex-col-reverse justify-between h-full w-1 bg-[#050608] p-[0.5px] rounded">
          <div
            v-for="i in 10" :key="'vu-b-' + i"
            class="w-full h-1 rounded-[0.5px] transition-all duration-75"
            :class="[
              deckB.isPlaying && (deckB.volume * crossfader * 10) >= i
                ? (i >= 9
                    ? 'bg-rose-500 shadow-[0_0_3px_#f43f5e]'
                    : (i >= 7 ? 'bg-amber-400 shadow-[0_0_2px_#fbbf24]' : 'bg-emerald-400 shadow-[0_0_2px_#34d399]'))
                : 'bg-zinc-800/30'
            ]"
          ></div>
        </div>
      </div>

      <!-- 3.3 CH 2 FADER & CUE -->
      <div class="col-span-5 flex flex-col items-center justify-between h-[80px]">
        <button
          @click="cueHeadphoneB = !cueHeadphoneB"
          title="Headphone Monitor CH 2"
          class="w-full py-0.5 rounded text-[6.5px] font-black border transition-all cursor-pointer flex items-center justify-center gap-1"
          :class="cueHeadphoneB
            ? 'bg-amber-400 border-amber-300 text-black shadow-[0_0_6px_#f59e0b]'
            : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
        >
          🎧 CUE 2
        </button>

        <div class="flex items-center justify-center my-0.5">
          <VerticalFader
            :model-value="deckB.volume"
            :min="0"
            :max="1"
            :step="0.01"
            :height="48"
            :reset-value="0.8"
            color="#f43f5e"
            @update:model-value="$emit('volumeB', $event)"
          />
        </div>

        <span class="text-[6px] text-rose-400 font-bold font-mono leading-none">
          {{ Math.round(deckB.volume * 100) }}%
        </span>
      </div>

    </div>

    <!-- ================= 4. CROSSFADER ================= -->
    <div class="w-full bg-[#08090d] px-2 py-1.5 rounded-xl border border-zinc-800/90 flex flex-col items-center gap-0.5 shrink-0">
      <div class="w-full flex justify-between items-center text-[7px] font-black px-1 leading-none">
        <span class="text-cyan-400">◄ A</span>
        <span class="text-zinc-500 tracking-wider">X-FADER</span>
        <span class="text-rose-400">B ►</span>
      </div>

      <div class="relative w-full flex items-center justify-center">
        <div class="absolute left-1/2 -translate-x-1/2 top-0 bottom-0 w-[1.5px] bg-zinc-600/80 pointer-events-none z-10"></div>

        <input
          type="range" min="0" max="1" step="0.01"
          :value="crossfader"
          @input="$emit('crossfade', parseFloat(($event.target as HTMLInputElement).value))"
          class="dj-slider-horizontal w-full"
        />
      </div>
    </div>

    <!-- ================= PRESET MODAL ================= -->
    <div
      v-if="showPresetModal"
      class="fixed inset-0 bg-black/75 backdrop-blur-xs flex items-center justify-center z-50 p-4"
    >
      <div class="bg-[#12141c] border border-zinc-700 rounded-xl p-4 w-80 shadow-2xl">
        <h3 class="text-xs font-black text-zinc-100 uppercase mb-3 flex items-center gap-2">
          <span>💾</span> Save EQ Preset
        </h3>
        <input
          v-model="presetName"
          @keyup.enter="savePreset"
          placeholder="Preset name (e.g. Club Bass Boost)"
          class="w-full bg-black/60 border border-zinc-700 rounded-lg px-3 py-1.5 text-xs text-zinc-100 placeholder:text-zinc-600 focus:outline-none focus:border-cyan-400 mb-4 font-sans"
          autofocus
        />
        <div class="flex justify-end gap-2">
          <button
            @click="showPresetModal = false"
            class="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-xs font-bold rounded-lg text-zinc-400 cursor-pointer transition-colors"
          >
            Cancel
          </button>
          <button
            @click="savePreset"
            class="px-3 py-1.5 bg-cyan-500 hover:bg-cyan-400 text-black text-xs font-bold rounded-lg cursor-pointer transition-colors"
          >
            Save
          </button>
        </div>
      </div>
    </div>

  </div>
</template>