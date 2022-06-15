<template>
  <FileDropArea
    class="file-drop-area transition-colors duration-300 rounded-box bg-neutral/50 p-2"
    @fileDrop="onDrop"
  >
    <FileUpload
      :class="[
        'w-full h-full rounded-box border-2 border-dashed border-neutral-content/60',
        'bg-neutral/70 hover:bg-neutral hover:border-2 hover:border-dashed hover:border-neutral-content/60',
        'flex justify-center items-center gap-4',
      ]"
      accept="*"
      method="readAsArrayBuffer"
      @loaded="onUpload"
      :disabled="disabled || fakeLoading"
    >
      <div class="flex flex-col justify-center items-center gap-4">
        <Icon
          :icon="fakeLoading ? 'mdi:loading' : 'ic:baseline-upload'"
          class="text-2xl"
          :class="fakeLoading ? 'animate-spin' : ''"
        />
        <span class="normal-case"> Select firmware file or drag it here </span>
      </div>
    </FileUpload>
  </FileDropArea>
</template>

<script setup lang="ts">
import { defineProps, ref } from "vue";
import { useFirmwareStore } from "@/stores/firmware";
import FileUpload from "@/components/FileUpload.vue";
import FileDropArea from "@/components/FileDropArea.vue";

defineProps<{
  disabled: boolean;
}>();

const firmware = useFirmwareStore();

// Delay firmware upload to make the interface more responsive
const fakeLoading = ref(false);
const fakeWait = 200;

const delayedSetFirmware = (data: Uint8Array, filename: string) => {
  firmware.resetFirmware();
  fakeLoading.value = true;
  setTimeout(() => {
    fakeLoading.value = false;
    firmware.setFirmware(data, filename);
  }, fakeWait);
};

const onUpload = (file: File, d: ArrayBuffer | string) => {
  const data = d as ArrayBuffer;
  delayedSetFirmware(new Uint8Array(data), file.name);
};
const onDrop = (data: Uint8Array, filename: string) =>
  delayedSetFirmware(data, filename);
</script>

<style>
.file-drop-area.dragover {
  @apply bg-neutral;
}
</style>
