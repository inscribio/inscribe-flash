<template>
  <div
    @dragenter="dragEnter"
    @dragleave="dragLeave"
    @dragover.prevent="dragOver"
    @dragend="dragEnd"
    @drop.prevent="drop"
    :class="dragover ? 'dragover' : ''"
  >
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { readBinaryFile } from "@tauri-apps/api/fs";
import { defineEmits, onMounted, onBeforeUnmount, ref } from "vue";

const emit = defineEmits<{
  (_e: "fileDrop", _v: Uint8Array): void;
}>();

const dragover = ref(false);
const unlisten = ref<UnlistenFn | null>(null);

const dragEnter = (e: DragEvent) => {
  if (e.dataTransfer == null) return;
  e.dataTransfer.dropEffect = "copy";
  dragover.value = true;
};

const dragLeave = () => (dragover.value = false);
const dragOver = () => (dragover.value = true);
const dragEnd = () => (dragover.value = false);

// Will not fire in tauri, so we use tauri://file-drop
const drop = () => (dragover.value = false);

const tauriFileDrop = async (e) => {
  const file = e.payload[0];
  if (file == null) throw Error("Dropped file is null");

  const data = await readBinaryFile(file);
  emit("fileDrop", data);
};

onMounted(async () => {
  unlisten.value = await listen("tauri://file-drop", tauriFileDrop);
});
onBeforeUnmount(() => {
  if (unlisten.value != null) unlisten.value();
  unlisten.value = null;
});
</script>
