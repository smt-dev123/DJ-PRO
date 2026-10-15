<script setup lang="ts">
type TabMode = "HOT CUE" | "LOOP" | "STEMS" | "FX";

const props = defineProps<{
  modelValue: TabMode;
  color: "cyan" | "rose";
}>();

const emit = defineEmits<{
  (e: "update:modelValue", val: TabMode): void;
}>();

const TABS: TabMode[] = ["HOT CUE", "LOOP", "STEMS", "FX"];

function selectTab(tab: TabMode) {
  emit("update:modelValue", tab);
}
</script>

<template>
  <div class="grid grid-cols-4 gap-0.5 bg-black/60 p-0.5 rounded-lg border border-zinc-800">
    <button
      v-for="tab in TABS"
      :key="tab"
      @click="selectTab(tab)"
      class="text-[6.5px] font-black py-1 rounded transition-all cursor-pointer truncate text-center"
      :class="modelValue === tab
        ? (color === 'cyan'
            ? 'bg-cyan-500 text-black shadow-xs font-black'
            : 'bg-rose-500 text-white shadow-xs font-black')
        : 'text-zinc-500 hover:text-zinc-300'"
    >
      {{ tab.replace('HOT ', '') }}
    </button>
  </div>
</template>