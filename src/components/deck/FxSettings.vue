<script setup lang="ts">
import { ref, watch, computed } from "vue";

const props = defineProps<{
  fxType: string;
  params: any;
  fxAmount: number;
}>();

const emit = defineEmits<{
  (e: "update", params: any): void;
  (e: "updateAmount", amount: number): void;
  (e: "close"): void;
}>();

const localParams = ref({ ...props.params });
const localAmount = ref(props.fxAmount);

watch(() => props.params, (newParams) => {
  localParams.value = { ...newParams };
}, { deep: true });

watch(() => props.fxAmount, (v) => {
  localAmount.value = v;
});

function updateParam(key: string, value: any) {
  (localParams.value as any)[key] = value;
  emit("update", { ...localParams.value });
}

function updateAmount(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  localAmount.value = val;
  emit("updateAmount", val);
}

const amountPercent = computed(() => Math.round(localAmount.value * 100));

const TIME_DIVISIONS = [
  { label: "1/16", value: 0.0625 },
  { label: "1/8", value: 0.125 },
  { label: "1/4", value: 0.25 },
  { label: "3/8", value: 0.375 },
  { label: "1/2", value: 0.5 },
  { label: "3/4", value: 0.75 },
  { label: "1", value: 1.0 },
  { label: "1½", value: 1.5 },
  { label: "2", value: 2.0 },
  { label: "3", value: 3.0 },
  { label: "4", value: 4.0 },
];

const REVERB_ROOMS = [
  { label: "ROOM", value: "room" },
  { label: "HALL", value: "hall" },
  { label: "PLATE", value: "plate" },
  { label: "CATH", value: "cathedral" },
  { label: "CHAM", value: "chamber" },
];

const ROLL_DIVISIONS = [
  { label: "1/16", value: 0.0625 },
  { label: "1/8", value: 0.125 },
  { label: "1/4", value: 0.25 },
  { label: "1/2", value: 0.5 },
  { label: "1", value: 1.0 },
];

const FILTER_MODES = [
  { label: "LP", value: "lowpass" },
  { label: "HP", value: "highpass" },
  { label: "BP", value: "bandpass" },
];
</script>

<template>
  <div class="bg-[#0a0c14] border-2 border-purple-500/50 rounded-xl p-3 shadow-2xl">
    <!-- Header -->
    <div class="flex items-center justify-between mb-3 pb-2 border-b border-purple-500/30">
      <div class="flex items-center gap-2">
        <span class="text-[11px] font-black text-purple-300 uppercase tracking-wider">
          ⚙ {{ fxType.toUpperCase() }} SETTINGS
        </span>
        <span class="px-1.5 py-0.5 rounded text-[7.5px] font-black uppercase bg-emerald-500/20 text-emerald-400 border border-emerald-500/40 flex items-center gap-1">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
          AUTO-SAVED
        </span>
      </div>
      <button
        @click="$emit('close')"
        class="w-6 h-6 rounded bg-zinc-900 hover:bg-red-500/40 text-zinc-400 hover:text-white text-[12px] font-black cursor-pointer"
      >
        ✕
      </button>
    </div>

    <!-- ✅ AMOUNT Slider (Universal) -->
    <div class="mb-3 pb-3 border-b border-zinc-800">
      <div class="flex items-center justify-between mb-1">
        <label class="text-[9px] font-black text-zinc-300 uppercase">AMOUNT (AMT)</label>
        <span class="text-[10px] font-mono text-purple-300 font-bold">{{ amountPercent }}%</span>
      </div>
      <input
        type="range" min="0" max="1" step="0.01"
        :value="localAmount"
        @input="updateAmount"
        class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
        :style="{ background: `linear-gradient(to right, #a855f7 ${amountPercent}%, #3f3f46 ${amountPercent}%)` }"
      />
    </div>

    <!-- ==================== ECHO ==================== -->
    <div v-if="fxType === 'echo'" class="flex flex-col gap-3">
      <div>
        <label class="text-[9px] font-black text-zinc-400 uppercase mb-1 block">TIME</label>
        <div class="grid grid-cols-6 gap-1">
          <button
            v-for="t in TIME_DIVISIONS"
            :key="t.value"
            @click="updateParam('echo_time_beats', t.value)"
            class="py-1.5 rounded text-[8px] font-black border transition-all cursor-pointer"
            :class="localParams.echo_time_beats === t.value
              ? 'bg-purple-500 border-purple-300 text-white'
              : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
          >{{ t.label }}</button>
        </div>
      </div>
      <div>
        <div class="flex items-center justify-between mb-1">
          <label class="text-[9px] font-black text-zinc-400 uppercase">FEEDBACK</label>
          <span class="text-[9px] font-mono text-purple-300 font-bold">{{ Math.round((localParams.echo_feedback || 0) * 100) }}%</span>
        </div>
        <input type="range" min="0" max="0.95" step="0.01"
          :value="localParams.echo_feedback"
          @input="updateParam('echo_feedback', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
          :style="{ background: `linear-gradient(to right, #a855f7 ${(localParams.echo_feedback || 0) * 100}%, #3f3f46 ${(localParams.echo_feedback || 0) * 100}%)` }" />
      </div>
    </div>

    <!-- ==================== REVERB ==================== -->
    <div v-else-if="fxType === 'reverb'" class="flex flex-col gap-3">
      <div>
        <label class="text-[9px] font-black text-zinc-400 uppercase mb-1 block">ROOM TYPE</label>
        <div class="grid grid-cols-5 gap-1">
          <button v-for="r in REVERB_ROOMS" :key="r.value"
            @click="updateParam('reverb_room', r.value)"
            class="py-2 rounded text-[7.5px] font-black border transition-all cursor-pointer"
            :class="localParams.reverb_room === r.value
              ? 'bg-purple-500 border-purple-300 text-white'
              : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
          >{{ r.label }}</button>
        </div>
      </div>
      <div>
        <div class="flex items-center justify-between mb-1">
          <label class="text-[9px] font-black text-zinc-400 uppercase">DECAY</label>
          <span class="text-[9px] font-mono text-purple-300 font-bold">{{ Math.round((localParams.reverb_decay || 0) * 100) }}%</span>
        </div>
        <input type="range" min="0" max="1" step="0.01"
          :value="localParams.reverb_decay"
          @input="updateParam('reverb_decay', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
          :style="{ background: `linear-gradient(to right, #a855f7 ${(localParams.reverb_decay || 0) * 100}%, #3f3f46 ${(localParams.reverb_decay || 0) * 100}%)` }" />
      </div>
      <div>
        <div class="flex items-center justify-between mb-1">
          <label class="text-[9px] font-black text-zinc-400 uppercase">DAMPING</label>
          <span class="text-[9px] font-mono text-purple-300 font-bold">{{ Math.round((localParams.reverb_damping || 0) * 100) }}%</span>
        </div>
        <input type="range" min="0" max="1" step="0.01"
          :value="localParams.reverb_damping"
          @input="updateParam('reverb_damping', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
          :style="{ background: `linear-gradient(to right, #a855f7 ${(localParams.reverb_damping || 0) * 100}%, #3f3f46 ${(localParams.reverb_damping || 0) * 100}%)` }" />
      </div>
    </div>

    <!-- ==================== DELAY ==================== -->
    <div v-else-if="fxType === 'delay'" class="flex flex-col gap-3">
      <div>
        <label class="text-[9px] font-black text-zinc-400 uppercase mb-1 block">TIME</label>
        <div class="grid grid-cols-6 gap-1">
          <button v-for="t in TIME_DIVISIONS" :key="t.value"
            @click="updateParam('delay_time_beats', t.value)"
            class="py-1.5 rounded text-[8px] font-black border transition-all cursor-pointer"
            :class="localParams.delay_time_beats === t.value
              ? 'bg-purple-500 border-purple-300 text-white'
              : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
          >{{ t.label }}</button>
        </div>
      </div>
      <div>
        <div class="flex items-center justify-between mb-1">
          <label class="text-[9px] font-black text-zinc-400 uppercase">FEEDBACK</label>
          <span class="text-[9px] font-mono text-purple-300 font-bold">{{ Math.round((localParams.delay_feedback || 0) * 100) }}%</span>
        </div>
        <input type="range" min="0" max="0.95" step="0.01"
          :value="localParams.delay_feedback"
          @input="updateParam('delay_feedback', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
          :style="{ background: `linear-gradient(to right, #a855f7 ${(localParams.delay_feedback || 0) * 100}%, #3f3f46 ${(localParams.delay_feedback || 0) * 100}%)` }" />
      </div>
    </div>

    <!-- ==================== FLANGER ==================== -->
    <div v-else-if="fxType === 'flanger'" class="flex flex-col gap-3">
      <div>
        <div class="flex items-center justify-between mb-1">
          <label class="text-[9px] font-black text-zinc-400 uppercase">RATE (HZ)</label>
          <span class="text-[9px] font-mono text-purple-300 font-bold">{{ (localParams.flanger_rate || 0).toFixed(2) }}</span>
        </div>
        <input type="range" min="0.1" max="5" step="0.01"
          :value="localParams.flanger_rate"
          @input="updateParam('flanger_rate', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
          :style="{ background: `linear-gradient(to right, #a855f7 ${((localParams.flanger_rate || 0.1) / 5) * 100}%, #3f3f46 ${((localParams.flanger_rate || 0.1) / 5) * 100}%)` }" />
      </div>
      <div>
        <div class="flex items-center justify-between mb-1">
          <label class="text-[9px] font-black text-zinc-400 uppercase">DEPTH</label>
          <span class="text-[9px] font-mono text-purple-300 font-bold">{{ Math.round((localParams.flanger_depth || 0) * 100) }}%</span>
        </div>
        <input type="range" min="0" max="1" step="0.01"
          :value="localParams.flanger_depth"
          @input="updateParam('flanger_depth', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
          :style="{ background: `linear-gradient(to right, #a855f7 ${(localParams.flanger_depth || 0) * 100}%, #3f3f46 ${(localParams.flanger_depth || 0) * 100}%)` }" />
      </div>
      <div>
        <div class="flex items-center justify-between mb-1">
          <label class="text-[9px] font-black text-zinc-400 uppercase">FEEDBACK</label>
          <span class="text-[9px] font-mono text-purple-300 font-bold">{{ Math.round((localParams.flanger_feedback || 0) * 100) }}%</span>
        </div>
        <input type="range" min="0" max="0.9" step="0.01"
          :value="localParams.flanger_feedback"
          @input="updateParam('flanger_feedback', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
          :style="{ background: `linear-gradient(to right, #a855f7 ${(localParams.flanger_feedback || 0) * 100}%, #3f3f46 ${(localParams.flanger_feedback || 0) * 100}%)` }" />
      </div>
    </div>

    <!-- ==================== FILTER ==================== -->
    <div v-else-if="fxType === 'filter'" class="flex flex-col gap-3">
      <div>
        <label class="text-[9px] font-black text-zinc-400 uppercase mb-1 block">MODE</label>
        <div class="grid grid-cols-3 gap-1">
          <button v-for="m in FILTER_MODES" :key="m.value"
            @click="updateParam('filter_mode', m.value)"
            class="py-2 rounded text-[10px] font-black border transition-all cursor-pointer"
            :class="localParams.filter_mode === m.value
              ? 'bg-purple-500 border-purple-300 text-white'
              : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
          >{{ m.label }}</button>
        </div>
      </div>
      <div>
        <div class="flex items-center justify-between mb-1">
          <label class="text-[9px] font-black text-zinc-400 uppercase">RESONANCE (Q)</label>
          <span class="text-[9px] font-mono text-purple-300 font-bold">{{ (localParams.filter_resonance || 0).toFixed(2) }}</span>
        </div>
        <input type="range" min="0.1" max="10" step="0.1"
          :value="localParams.filter_resonance"
          @input="updateParam('filter_resonance', parseFloat(($event.target as HTMLInputElement).value))"
          class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
          :style="{ background: `linear-gradient(to right, #a855f7 ${((localParams.filter_resonance || 0.1) / 10) * 100}%, #3f3f46 ${((localParams.filter_resonance || 0.1) / 10) * 100}%)` }" />
      </div>
    </div>

    <!-- ==================== ROLL ==================== -->
    <div v-else-if="fxType === 'roll'" class="flex flex-col gap-3">
      <div>
        <label class="text-[9px] font-black text-zinc-400 uppercase mb-1 block">LENGTH</label>
        <div class="grid grid-cols-5 gap-1">
          <button v-for="r in ROLL_DIVISIONS" :key="r.value"
            @click="updateParam('roll_beats', r.value)"
            class="py-2 rounded text-[9px] font-black border transition-all cursor-pointer"
            :class="localParams.roll_beats === r.value
              ? 'bg-purple-500 border-purple-300 text-white'
              : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
          >{{ r.label }}</button>
        </div>
      </div>
    </div>

    <div class="mt-3 pt-2 border-t border-purple-500/20 flex justify-end">
      <button @click="$emit('close')" class="px-3 py-1.5 bg-purple-500 hover:bg-purple-400 text-black text-[10px] font-black rounded-lg cursor-pointer">
        CLOSE
      </button>
    </div>
  </div>
</template>