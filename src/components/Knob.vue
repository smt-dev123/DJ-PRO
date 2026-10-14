<script setup lang="ts">
import { computed, ref } from "vue";

const props = withDefaults(
  defineProps<{
    label: string;
    modelValue: number;
    color?: string;
    size?: "sm" | "md";
    centerDetent?: boolean;
    showValue?: boolean;
  }>(),
  {
    color: "#06b6d4",
    size: "sm",
    centerDetent: true,
    showValue: false,
  }
);

const emit = defineEmits<{
  (e: "update:modelValue", val: number): void;
}>();

const isDragging = ref(false);
let startY = 0;
let startVal = 0;

function handleMouseDown(e: MouseEvent) {
  e.preventDefault();
  isDragging.value = true;
  startY = e.clientY;
  startVal = props.modelValue;
  window.addEventListener("mousemove", handleMouseMove);
  window.addEventListener("mouseup", handleMouseUp);
}

function handleMouseMove(e: MouseEvent) {
  // ✅ Sensitivity: Shift = fine, default = normal
  const sensitivity = e.shiftKey ? 0.3 : 1.2;
  const diff = (startY - e.clientY) * sensitivity;
  let nextVal = Math.min(100, Math.max(-100, Math.round(startVal + diff)));

  // ✅ Magnetic snap to 0
  if (props.centerDetent && Math.abs(nextVal) <= 3) {
    nextVal = 0;
  }
  emit("update:modelValue", nextVal);
}

function handleMouseUp() {
  isDragging.value = false;
  window.removeEventListener("mousemove", handleMouseMove);
  window.removeEventListener("mouseup", handleMouseUp);
}

function handleWheel(e: WheelEvent) {
  e.preventDefault();
  const step = e.shiftKey ? 1 : 5;
  const delta = e.deltaY < 0 ? step : -step;
  const nextVal = Math.min(100, Math.max(-100, props.modelValue + delta));
  emit("update:modelValue", nextVal);
}

function handleDoubleClick() {
  emit("update:modelValue", 0);
}

const rotationDeg = computed(() => {
  return props.modelValue * 1.35;
});

const formattedVal = computed(() => {
  if (props.modelValue === 0) return "0";
  return props.modelValue > 0 ? `+${props.modelValue}` : `${props.modelValue}`;
});

// ✅ Check if value is changed from 0
const isActive = computed(() => Math.abs(props.modelValue) > 0);
</script>

<template>
  <div
    class="flex flex-col items-center select-none group relative"
    :title="`${label}: ${formattedVal} (Drag up/down, Scroll, Double-click to Reset, Shift = Fine)`"
  >
    <span
      class="text-[6.5px] font-black tracking-wider transition-colors uppercase leading-none mb-0.5"
      :class="isActive ? 'text-zinc-200' : 'text-zinc-400 group-hover:text-zinc-200'"
    >
      {{ label }}
    </span>

    <div
      @mousedown="handleMouseDown"
      @wheel="handleWheel"
      @dblclick="handleDoubleClick"
      class="rounded-full bg-gradient-to-b from-[#2e303b] via-[#1a1b22] to-[#0c0d12] border shadow-[0_1px_4px_rgba(0,0,0,0.9)] relative flex items-center justify-center cursor-ns-resize transition-transform"
      :class="[
        size === 'sm' ? 'w-5.5 h-5.5' : 'w-6.5 h-6.5',
        isDragging ? 'scale-105' : 'active:scale-95',
        isActive ? 'border-zinc-500' : 'border-zinc-700/80'
      ]"
    >
      <!-- Center Detent Notch -->
      <div
        class="absolute -top-0.5 w-[2px] h-[3px] rounded-full transition-opacity"
        :class="Math.abs(modelValue) < 3
          ? 'bg-amber-400 shadow-[0_0_3px_#fbbf24] opacity-90'
          : 'bg-zinc-600 opacity-40'"
      ></div>

      <!-- Inner Cap -->
      <div
        class="rounded-full bg-gradient-to-br from-[#181920] to-[#0a0b0e] border border-zinc-800 flex items-center justify-center shadow-inner relative"
        :class="size === 'sm' ? 'w-4 h-4' : 'w-5 h-5'"
      >
        <!-- Rotating Indicator -->
        <div
          class="absolute inset-0 flex justify-center pointer-events-none"
          :style="{ transform: `rotate(${rotationDeg}deg)` }"
        >
          <div
            class="w-[1.5px] rounded-full mt-0.5 transition-colors"
            :class="size === 'sm' ? 'h-1.5' : 'h-2'"
            :style="{
              backgroundColor: color,
              boxShadow: isActive ? `0 0 6px ${color}` : `0 0 3px ${color}`,
            }"
          ></div>
        </div>
      </div>
    </div>

    <span
      v-if="showValue"
      class="text-[6px] font-mono leading-none mt-0.5 transition-colors"
      :class="modelValue === 0 ? 'text-zinc-600' : 'text-zinc-300 font-bold'"
    >
      {{ formattedVal }}
    </span>
  </div>
</template>