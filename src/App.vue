<template>
  <div
    class="mx-auto h-screen w-96 flex flex-col justify-center items-center gap-4 antialiased"
  >
    <h1 class="text-xl font-bold">Check this out!</h1>

    <FileUpload
      accept="*"
      method="readAsArrayBuffer"
      @loaded="onFirmwareUpload"
    >
      <div class="flex justify-center items-center gap-2">
        <Icon icon="ic:baseline-upload" class="text-lg" />
        <div>Upload firmware</div>
      </div>
    </FileUpload>

    <div v-if="showProgress" class="form-control">
      <label class="label">
        <span class="label-text"> {{ progressMessage }} </span>
        <span class="label-text-alt"> {{ progressPercent }}% </span>
        <span class="label-text-alt"> {{ progressKiloBytes }} kB </span>
      </label>
      <progress
        class="progress w-72"
        :value="progressBytes"
        :max="firmwareSize"
      ></progress>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { computed, ref, watch } from "vue";
import { useToast } from "vue-toastification";
import FileUpload from "@/components/FileUpload.vue";

const toast = useToast();

const showProgress = ref(false);
const progressBytes = ref(0);
const firmwareSize = ref(1);
const progressMessage = ref("");

const progressPercent = computed(() =>
  ((progressBytes.value * 100) / firmwareSize.value).toFixed(0)
);

const progressKiloBytes = computed(() =>
  (progressBytes.value / 1024).toFixed(1)
);

type Progress = {
  Erase?: number;
  Download?: number;
};

listen("flash-progress", (e) => {
  const progress = e.payload as Progress;

  if (progress.Erase != undefined) {
    progressBytes.value = progress.Erase;
    progressMessage.value = "Erasing...";
  } else if (progress.Download != undefined) {
    progressBytes.value = progress.Download;
    progressMessage.value = "Flashing...";
  } else {
    console.error("Unexpected flash-progress event", e);
  }
});

const onFirmwareUpload = (file: File, d: ArrayBuffer | string) => {
  const data = d as ArrayBuffer;
  const bytes = Array.from(new Uint8Array(data));

  if (bytes.length == 0) {
    console.error("Firmware size is zero");
    return;
  }

  progressMessage.value = "Starting...";
  progressBytes.value = 0;
  firmwareSize.value = bytes.length;

  showProgress.value = true;

  invoke("flash", { firmware: bytes }).then((result) => {
    showProgress.value = false;
    console.log("Result", result);
  });
};
</script>
