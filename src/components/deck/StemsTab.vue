<script setup lang="ts">
import { ref } from "vue";

const stems = ref({
  vocal: true,
  drum: true,
  bass: true,
  melody: true,
});

const emit = defineEmits<{
  (e: "toggleStem", stem: keyof typeof stems.value, active: boolean): void;
}>();

function toggleStem(stem: keyof typeof stems.value) {
  stems.value[stem] = !stems.value[stem];
  emit("toggleStem", stem, stems.value[stem]);
}
</script>

<template>
  <div class="grid grid-cols-2 gap-1.5 flex-1">
    <button
      @click="toggleStem('vocal')"
      class="rounded-lg border text-[8px] font-black cursor-pointer flex items-center justify-center transition-all"
      :class="stems.vocal
        ? 'bg-emerald-500/25 border-emerald-400 text-emerald-300 shadow-[0_0_6px_rgba(16,185,129,0.3)]'
        : 'bg-zinc-900 border-zinc-800 text-zinc-600'"
    >
      🎤 VOCAL
    </button>
    <button
      @click="toggleStem('drum')"
      class="rounded-lg border text-[8px] font-black cursor-pointer flex items-center justify-center transition-all"
      :class="stems.drum
        ? 'bg-amber-500/25 border-amber-400 text-amber-300 shadow-[0_0_6px_rgba(245,158,11,0.3)]'
        : 'bg-zinc-900 border-zinc-800 text-zinc-600'"
    >
      🥁 DRUMS
    </button>
    <button
      @click="toggleStem('bass')"
      class="rounded-lg border text-[8px] font-black cursor-pointer flex items-center justify-center transition-all"
      :class="stems.bass
        ? 'bg-indigo-500/25 border-indigo-400 text-indigo-300 shadow-[0_0_6px_rgba(99,102,241,0.3)]'
        : 'bg-zinc-900 border-zinc-800 text-zinc-600'"
    >
      🎸 BASS
    </button>
    <button
      @click="toggleStem('melody')"
      class="rounded-lg border text-[8px] font-black cursor-pointer flex items-center justify-center transition-all"
      :class="stems.melody
        ? 'bg-sky-500/25 border-sky-400 text-sky-300 shadow-[0_0_6px_rgba(14,165,233,0.3)]'
        : 'bg-zinc-900 border-zinc-800 text-zinc-600'"
    >
      🎹 MELODY
    </button>
  </div>
</template>