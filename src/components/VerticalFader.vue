<script setup lang="ts">
import { computed, ref } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: number;
    min?: number;
    max?: number;
    step?: number;
    height?: number; // in pixels
    color?: string;
    centerDetent?: boolean;
    resetValue?: number;
  }>(),
  {
    min: 0,
    max: 1,
    step: 0.01,
    height: 80,
    color: "#38bdf8",
    centerDetent: false,
    resetValue: 0,
  }
);

const emit = defineEmits<{
  (e: "update:modelValue", val: number): void;
}>();

const trackRef = ref<HTMLElement | null>(null);
const isDragging = ref(false);

// Calculate percentage (0 = bottom/min, 100 = top/max)
const percentage = computed(() => {
  const range = props.max - props.min;
  if (range === 0) return 0;
  const p = ((props.modelValue - props.min) / range) * 100;
  return Math.max(0, Math.min(100, p));
});

function updateFromY(clientY: number) {
  if (!trackRef.value) return;
  const rect = trackRef.value.getBoundingClientRect();
  const trackHeight = rect.height;
  if (trackHeight <= 0) return;

  // clientY: bottom is min, top is max
  const offsetFromTop = clientY - rect.top;
  const ratio = 1 - (offsetFromTop / trackHeight);
  const clampedRatio = Math.max(0, Math.min(1, ratio));

  let val = props.min + clampedRatio * (props.max - props.min);
  if (props.step > 0) {
    const inv = 1 / props.step;
    val = Math.round(val * inv) / inv;
  }

  // Magnetic center detent near 0
  if (props.centerDetent && Math.abs(val) <= (props.max - props.min) * 0.03) {
    val = 0;
  }

  val = Math.max(props.min, Math.min(props.max, val));
  emit("update:modelValue", val);
}

function handleMouseDown(e: MouseEvent) {
  e.preventDefault();
  isDragging.value = true;
  updateFromY(e.clientY);

  function onMouseMove(ev: MouseEvent) {
    updateFromY(ev.clientY);
  }

  function onMouseUp() {
    isDragging.value = false;
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  }

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
}

function handleDoubleClick() {
  emit("update:modelValue", props.resetValue);
}

function handleWheel(e: WheelEvent) {
  e.preventDefault();
  const stepVal = props.step || 0.01;
  const delta = e.deltaY < 0 ? stepVal * 2 : -stepVal * 2;
  const nextVal = Math.max(props.min, Math.min(props.max, props.modelValue + delta));
  emit("update:modelValue", nextVal);
}
</script>

<template>
  <div 
    class="relative flex items-center justify-center select-none cursor-pointer"
    :style="{ height: `${height}px`, width: '32px' }"
    @mousedown="handleMouseDown"
    @dblclick="handleDoubleClick"
    @wheel="handleWheel"
    title="Drag or Scroll to Adjust. Double-click to Reset"
  >
    <!-- Background Groove Track -->
    <div 
      ref="trackRef" 
      class="relative w-[5px] h-full rounded-full bg-[#050608] border border-zinc-800 shadow-[inset_0_1px_3px_rgba(0,0,0,1)] flex items-center justify-center"
    >
      <!-- Center Detent Notch Tick -->
      <div 
        v-if="centerDetent"
        class="absolute top-1/2 -translate-y-1/2 w-3 h-[1.5px] rounded-full pointer-events-none transition-colors"
        :class="Math.abs(modelValue) < 0.05 ? 'bg-amber-400 shadow-[0_0_4px_#fbbf24]' : 'bg-zinc-600/70'"
      ></div>

      <!-- Active Level Glow Bar (bottom to current thumb) -->
      <div 
        class="absolute bottom-0 w-full rounded-full transition-all duration-75 pointer-events-none opacity-60"
        :style="{
          height: `${percentage}%`,
          backgroundColor: color,
          boxShadow: `0 0 6px ${color}`
        }"
      ></div>
    </div>

    <!-- Hardware Fader Thumb Cap -->
    <div 
      class="absolute left-1/2 -translate-x-1/2 -translate-y-1/2 w-[22px] h-[13px] rounded-[3px] border shadow-[0_2px_6px_rgba(0,0,0,0.9),inset_0_1px_0_rgba(255,255,255,0.25)] pointer-events-none transition-transform flex items-center justify-center"
      :class="isDragging 
        ? 'bg-gradient-to-r from-[#4f5260] via-[#2a2b34] to-[#181920] border-zinc-400 scale-105' 
        : 'bg-gradient-to-r from-[#3e404b] via-[#202128] to-[#121317] border-zinc-600'"
      :style="{
        top: `${100 - percentage}%`,
      }"
    >
      <!-- Center Line Marker on Fader Cap -->
      <div 
        class="w-full h-[1.5px] rounded-full"
        :style="{
          backgroundColor: color,
          boxShadow: isDragging ? `0 0 6px ${color}` : 'none'
        }"
      ></div>
    </div>
  </div>
</template>
