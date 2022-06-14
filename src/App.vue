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

    <button class="btn" @click="findDevices">Find devices</button>

    <div class="overflow-x-auto">
      <table class="table w-full">
        <thead>
          <tr>
            <th class="w-12"></th>
            <th class="w-32">Mode</th>
            <th class="w-24">Devnum</th>
            <th class="w-32">VID:PID</th>
          </tr>
        </thead>
        <tbody v-if="true">
          <tr v-for="(dev, i) in deviceList" :key="i">
            <th>{{ i + 1 }}</th>
            <td>{{ dev.is_dfu ? "Bootloader" : "Runtime" }}</td>
            <td>{{ dev.devnum }}</td>
            <td>
              {{
                padZeros(toHex(dev.vid), 4) + ":" + padZeros(toHex(dev.pid), 4)
              }}
            </td>
          </tr>
        </tbody>
      </table>
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

type DfuListEntry = {
  is_dfu: boolean;
  vid: number;
  pid: number;
  devnum: number;
  alt: number;
};

const isDfuListEntry = (v: unknown): v is DfuListEntry => {
  if (typeof v != "object" || v == null) return false;
  const o = v as DfuListEntry;
  return (
    typeof o.is_dfu == "boolean" &&
    typeof o.vid == "number" &&
    typeof o.pid == "number" &&
    typeof o.devnum == "number" &&
    typeof o.alt == "number"
  );
};

const isDfuListEntryList = (v: unknown): v is DfuListEntry[] =>
  Array.isArray(v) && v.every(isDfuListEntry);

const deviceList = ref<DfuListEntry[]>([]);

const progressPercent = computed(() =>
  ((progressBytes.value * 100) / firmwareSize.value).toFixed(0)
);

const progressKiloBytes = computed(() =>
  (progressBytes.value / 1024).toFixed(1)
);

const padZeros = (v: string, n: number) =>
  "0".repeat(Math.max(0, n - v.length)) + v;

const toHex = (v: number) => v.toString(16);

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

const findDevices = () =>
  invoke("list").then((r) => {
    if (typeof r == "string") {
      toast.error("Could not list devices: " + r);
    } else if (!isDfuListEntryList(r)) {
      console.error("unexpected data", r);
    } else {
      deviceList.value = r;
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
