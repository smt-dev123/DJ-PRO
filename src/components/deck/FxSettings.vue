<script setup lang="ts">
import { ref, computed, watch } from "vue";

type FxType = "echo" | "reverb" | "delay" | "flanger" | "filter" | "roll";

const props = defineProps<{
  fxType: string;
  params: {
    echo_time_beats: number;
    echo_feedback: number;
    echo_mix: number;
    reverb_decay: number;
    reverb_damping: number;
    reverb_mix: number;
    delay_time_ms: number;
    delay_feedback: number;
    delay_mix: number;
    flanger_rate: number;
    flanger_depth: number;
    flanger_feedback: number;
    filter_resonance: number;
    filter_mode: "lowpass" | "highpass" | "bandpass";
    roll_beats: number;
  };
}>();

const emit = defineEmits<{
  (e: "update", params: any): void;
  (e: "close"): void;
}>();

// ✅ Local copy of params
const localParams = ref({ ...props.params });

// ✅ Sync ពេល props ផ្លាស់ប្តូរ
watch(
  () => props.params,
  (newParams) => {
    localParams.value = { ...newParams };
  },
  { deep: true }
);

function updateParam(key: string, value: number | string) {
  (localParams.value as any)[key] = value;
  emit("update", { ...localParams.value });
}

// ✅ Echo time presets (beats)
const ECHO_TIMES = [
  { label: "1/16", value: 0.0625 },
  { label: "1/8", value: 0.125 },
  { label: "1/4", value: 0.25 },
  { label: "3/8", value: 0.375 },
  { label: "1/2", value: 0.5 },
  { label: "3/4", value: 0.75 },
  { label: "1", value: 1.0 },
  { label: "2", value: 2.0 },
];

// ✅ Roll beats
const ROLL_BEATS = [
  { label: "1/16", value: 0.0625 },
  { label: "1/8", value: 0.125 },
  { label: "1/4", value: 0.25 },
  { label: "1/2", value: 0.5 },
  { label: "1", value: 1.0 },
];

// ✅ Filter modes
const FILTER_MODES = [
  { label: "LP", value: "lowpass" },
  { label: "HP", value: "highpass" },
  { label: "BP", value: "bandpass" },
];
</script>

<template>
  <div class="bg-[#0a0c14] border border-purple-500/40 rounded-lg p-2 shadow-xl">
    <!-- Header -->
    <div class="flex items-center justify-between mb-2 pb-1.5 border-b border-purple-500/30">
      <span class="text-[9px] font-black text-purple-300 uppercase tracking-wider">
        ⚙ FX SETTINGS — {{ fxType.toUpperCase() }}
      </span>
      <button
        @click="$emit('close')"
        class="text-[10px] text-zinc-500 hover:text-zinc-300 cursor-pointer"
      >
        ✕
      </button>
    </div>

    <!-- ==================== ECHO ==================== -->
    <div v-if="fxType === 'echo'" class="flex flex-col gap-2">
      <!-- Time (Beat divisions) -->
      <div>
        <label class="text-[7.5px] font-black text-zinc-400 uppercase">TIME</label>
        <div class="grid grid-cols-8 gap-0.5 mt-1">
          <button
            v-for="t in ECHO_TIMES"
            :key="t.value"
            @click="updateParam('echo_time_beats', t.value)"
            class="py-1 rounded text-[6.5px] font-black border transition-all cursor-pointer"
            :class="localParams.echo_time_beats === t.value
              ? 'bg-purple-500 border-purple-300 text-white'
              : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
          >
            {{ t.label }}
          </button>
        </div>
      </div>

      <!-- Feedback -->
      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">FEEDBACK</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ Math.round(localParams.echo_feedback * 100) }}%
          </span>
        </div>
        <input
          type="range" min="0" max="0.95" step="0.01"
          :value="localParams.echo_feedback"
          @input="updateParam('echo_feedback', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${localParams.echo_feedback * 100}%, #3f3f46 ${localParams.echo_feedback * 100}%)`
          }"
        />
      </div>

      <!-- Mix -->
      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">MIX (DRY/WET)</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ Math.round(localParams.echo_mix * 100) }}%
          </span>
        </div>
        <input
          type="range" min="0" max="1" step="0.01"
          :value="localParams.echo_mix"
          @input="updateParam('echo_mix', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${localParams.echo_mix * 100}%, #3f3f46 ${localParams.echo_mix * 100}%)`
          }"
        />
      </div>
    </div>

    <!-- ==================== REVERB ==================== -->
    <div v-else-if="fxType === 'reverb'" class="flex flex-col gap-2">
      <!-- Decay -->
      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">DECAY (ROOM SIZE)</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ Math.round(localParams.reverb_decay * 100) }}%
          </span>
        </div>
        <input
          type="range" min="0" max="1" step="0.01"
          :value="localParams.reverb_decay"
          @input="updateParam('reverb_decay', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${localParams.reverb_decay * 100}%, #3f3f46 ${localParams.reverb_decay * 100}%)`
          }"
        />
      </div>

      <!-- Damping -->
      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">DAMPING</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ Math.round(localParams.reverb_damping * 100) }}%
          </span>
        </div>
        <input
          type="range" min="0" max="1" step="0.01"
          :value="localParams.reverb_damping"
          @input="updateParam('reverb_damping', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${localParams.reverb_damping * 100}%, #3f3f46 ${localParams.reverb_damping * 100}%)`
          }"
        />
      </div>

      <!-- Mix -->
      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">MIX</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ Math.round(localParams.reverb_mix * 100) }}%
          </span>
        </div>
        <input
          type="range" min="0" max="1" step="0.01"
          :value="localParams.reverb_mix"
          @input="updateParam('reverb_mix', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${localParams.reverb_mix * 100}%, #3f3f46 ${localParams.reverb_mix * 100}%)`
          }"
        />
      </div>
    </div>

    <!-- ==================== DELAY ==================== -->
    <div v-else-if="fxType === 'delay'" class="flex flex-col gap-2">
      <!-- Time (ms) -->
      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">TIME</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ Math.round(localParams.delay_time_ms) }} ms
          </span>
        </div>
        <input
          type="range" min="10" max="2000" step="1"
          :value="localParams.delay_time_ms"
          @input="updateParam('delay_time_ms', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${(localParams.delay_time_ms / 2000) * 100}%, #3f3f46 ${(localParams.delay_time_ms / 2000) * 100}%)`
          }"
        />
      </div>

      <!-- Feedback -->
      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">FEEDBACK</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ Math.round(localParams.delay_feedback * 100) }}%
          </span>
        </div>
        <input
          type="range" min="0" max="0.95" step="0.01"
          :value="localParams.delay_feedback"
          @input="updateParam('delay_feedback', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${localParams.delay_feedback * 100}%, #3f3f46 ${localParams.delay_feedback * 100}%)`
          }"
        />
      </div>

      <!-- Mix -->
      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">MIX</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ Math.round(localParams.delay_mix * 100) }}%
          </span>
        </div>
        <input
          type="range" min="0" max="1" step="0.01"
          :value="localParams.delay_mix"
          @input="updateParam('delay_mix', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${localParams.delay_mix * 100}%, #3f3f46 ${localParams.delay_mix * 100}%)`
          }"
        />
      </div>
    </div>

    <!-- ==================== FLANGER ==================== -->
    <div v-else-if="fxType === 'flanger'" class="flex flex-col gap-2">
      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">RATE (HZ)</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ localParams.flanger_rate.toFixed(2) }} Hz
          </span>
        </div>
        <input
          type="range" min="0.1" max="5" step="0.01"
          :value="localParams.flanger_rate"
          @input="updateParam('flanger_rate', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${(localParams.flanger_rate / 5) * 100}%, #3f3f46 ${(localParams.flanger_rate / 5) * 100}%)`
          }"
        />
      </div>

      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">DEPTH</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ Math.round(localParams.flanger_depth * 100) }}%
          </span>
        </div>
        <input
          type="range" min="0" max="1" step="0.01"
          :value="localParams.flanger_depth"
          @input="updateParam('flanger_depth', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${localParams.flanger_depth * 100}%, #3f3f46 ${localParams.flanger_depth * 100}%)`
          }"
        />
      </div>

      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">FEEDBACK</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ Math.round(localParams.flanger_feedback * 100) }}%
          </span>
        </div>
        <input
          type="range" min="0" max="0.9" step="0.01"
          :value="localParams.flanger_feedback"
          @input="updateParam('flanger_feedback', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${localParams.flanger_feedback * 100}%, #3f3f46 ${localParams.flanger_feedback * 100}%)`
          }"
        />
      </div>
    </div>

    <!-- ==================== FILTER ==================== -->
    <div v-else-if="fxType === 'filter'" class="flex flex-col gap-2">
      <!-- Mode -->
      <div>
        <label class="text-[7.5px] font-black text-zinc-400 uppercase">MODE</label>
        <div class="grid grid-cols-3 gap-1 mt-1">
          <button
            v-for="m in FILTER_MODES"
            :key="m.value"
            @click="updateParam('filter_mode', m.value)"
            class="py-1 rounded text-[8px] font-black border transition-all cursor-pointer"
            :class="localParams.filter_mode === m.value
              ? 'bg-purple-500 border-purple-300 text-white'
              : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
          >
            {{ m.label }}
          </button>
        </div>
      </div>

      <!-- Resonance -->
      <div>
        <div class="flex items-center justify-between">
          <label class="text-[7.5px] font-black text-zinc-400 uppercase">RESONANCE (Q)</label>
          <span class="text-[7px] font-mono text-purple-300">
            {{ localParams.filter_resonance.toFixed(2) }}
          </span>
        </div>
        <input
          type="range" min="0.1" max="10" step="0.1"
          :value="localParams.filter_resonance"
          @input="updateParam('filter_resonance', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1 rounded-full appearance-none cursor-pointer mt-1"
          :style="{
            background: `linear-gradient(to right, #a855f7 ${(localParams.filter_resonance / 10) * 100}%, #3f3f46 ${(localParams.filter_resonance / 10) * 100}%)`
          }"
        />
      </div>
    </div>

    <!-- ==================== ROLL ==================== -->
    <div v-else-if="fxType === 'roll'" class="flex flex-col gap-2">
      <div>
        <label class="text-[7.5px] font-black text-zinc-400 uppercase">LENGTH</label>
        <div class="grid grid-cols-5 gap-0.5 mt-1">
          <button
            v-for="r in ROLL_BEATS"
            :key="r.value"
            @click="updateParam('roll_beats', r.value)"
            class="py-1 rounded text-[7px] font-black border transition-all cursor-pointer"
            :class="localParams.roll_beats === r.value
              ? 'bg-purple-500 border-purple-300 text-white'
              : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
          >
            {{ r.label }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>