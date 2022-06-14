<template>
  <div
    class="mx-auto h-screen max-w-xl flex flex-col justify-center items-center gap-4 antialiased"
  >
    <div class="flex items-center">
      <FileUpload
        class="btn-group-start"
        accept="*"
        method="readAsArrayBuffer"
        @loaded="onFirmwareUpload"
        :disabled="ongoing"
      >
        <div class="flex justify-center items-center gap-2">
          <Icon icon="ic:baseline-upload" class="text-lg" />
          <div>Upload firmware</div>
        </div>
      </FileUpload>

      <button
        class="btn btn-accent btn-group-end"
        :disabled="!fw.firmwareOk"
        @click="onFlash"
      >
        Flash
      </button>
    </div>

    <ProgressBar
      class="w-full max-w-lg p-2"
      :messageLeft="progress.msg"
      :messageRight="progress.note"
      :value="progress.value"
      :maxValue="progress.max"
    />

    <div class="form-control">
      <label class="label">
        <h1 class="label-text">Detected devices</h1>
      </label>
      <DeviceList
        :scan="true"
        :period="scanPeriod"
        :selected="selected.dev?.devnum ?? null"
        :selectedTooltip="selected.tooltip ?? null"
      />
      <label class="label">
        <h1 class="label-text italic">{{ selected?.tooltip ?? "" }}</h1>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { computed, ref, watch } from "vue";
import { useToast } from "vue-toastification";
import { useFirmwareStore } from "@/stores/firmware";
import { useDevicesStore, DfuListEntry } from "@/stores/devices";
import FileUpload from "@/components/FileUpload.vue";
import DeviceList from "@/components/DeviceList.vue";
import ProgressBar from "@/components/ProgressBar.vue";

const toast = useToast();
const fw = useFirmwareStore();
const devices = useDevicesStore();

const STM32_BOOTLOADER_VID_PID = [0x0483, 0xdf11];
const KEYBOARD_VID_PID = [0x16c0, 0x27db];

const DEFAULT_SCAN_PERIOD = 750;

const DETACH_SCAN_PERIOD = 250;
const DETACH_MAX_WAIT = 6000;
const DETACH_STEPS = Math.ceil(DETACH_MAX_WAIT / DETACH_SCAN_PERIOD);
const DETACH_HITS_REQUIRED = 3;

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

type ProgressInfo = {
  msg: string;
  note: string;
  value: number;
  max: number;
};
const defaultProgress = {
  msg: "",
  note: "",
  value: 0,
  max: 1,
};

const scanPeriod = ref(DEFAULT_SCAN_PERIOD);
const detachStep = ref(null as number | null);
const progress = ref<ProgressInfo>(defaultProgress);
const done = ref(false);

const ongoing = computed(
  () => fw.flashStage != "ready" || detachStep.value != null
);

const getProgress = () => {
  if (detachStep.value != null) {
    return {
      msg: "Detaching...",
      note: "",
      value: detachStep.value,
      max: DETACH_STEPS,
    };
  }

  const flashProgress = () => ({
    note: (fw.flashProgress / 1024).toFixed(1) + " kB",
    value: fw.flashProgress,
    max: fw.size ?? 1,
  });

  switch (fw.flashStage) {
    case "init":
      return { msg: "Preparing...", ...flashProgress() };
    case "erase":
      return { msg: "Erasing...", ...flashProgress() };
    case "download":
      return { msg: "Flashing...", ...flashProgress() };
  }

  return !done.value
    ? defaultProgress
    : {
        msg: "Done",
        note: "",
        value: 1,
        max: 1,
      };
};

watch(
  () => [detachStep.value, fw.flashStage, fw.flashProgress],
  () => (progress.value = getProgress())
);

const selected = computed(() => {
  const keyboards = devices.unique.filter(devices.isKeyboard);
  const bootloaders = devices.unique.filter(devices.isBootloader);

  if (keyboards.length == 0 && bootloaders.length == 0) {
    return { dev: null, tooltip: "No compatible devices" };
  } else if (bootloaders.length == 1) {
    return {
      dev: bootloaders[0],
      tooltip: "DFU bootloader ready for flashing",
    };
  } else if (bootloaders.length > 0) {
    // Select highest devnum (most recently connected)
    bootloaders.sort((a, b) => b.devnum - a.devnum);
    const dev = bootloaders[0];
    const devnum = dev.devnum;
    return {
      dev,
      tooltip: `Found ${bootloaders.length} active DFU bootloaders: will use devnum=${devnum}`,
    };
  } else if (keyboards.length == 1) {
    return {
      dev: keyboards[0],
      tooltip: "Keyboard will be detached before flashing",
    };
  } else {
    // Select highest devnum (most recently connected)
    keyboards.sort((a, b) => b.devnum - a.devnum);
    const dev = keyboards[0];
    const devnum = dev.devnum;
    return {
      dev,
      tooltip: `Found ${keyboards.length} active keyboards: will use devnum=${devnum}`,
    };
  }
});

const onFirmwareUpload = (file: File, d: ArrayBuffer | string) => {
  const data = d as ArrayBuffer;
  fw.setFirmware(new Uint8Array(data));
};

const detach = async (dev: DfuListEntry) => {
  // Note now which devices with the same vid:pid exist now to ignore later.
  const ignore = devices.unique
    .filter((d) => d.vid == dev.vid && d.pid == dev.pid)
    .map((d) => d.devnum);
  const considered = () =>
    devices.unique.filter((d) => !ignore.includes(d.devnum));

  const result = await invoke("detach", { devNum: dev.devnum });

  if (typeof result == "string") {
    toast.error("Could not detach device: " + result);
    return null;
  }

  // Find a non-ignored bootloader
  const find = () => {
    const bootloaders = considered().filter(devices.isBootloader);
    // Sort by newest
    bootloaders.sort((a, b) => b.devnum - a.devnum);
    return bootloaders[0];
  };

  // Wait for the device to appear
  scanPeriod.value = DETACH_SCAN_PERIOD;
  detachStep.value = 0;

  let lastFound = null as DfuListEntry | null;
  let hits = 0;

  for (let i = 0; i < DETACH_STEPS; i++) {
    const found = find();

    if (found == undefined) hits = 0;
    else if (found.devnum == lastFound?.devnum) hits += 1;
    else hits = 0;

    if (hits >= DETACH_HITS_REQUIRED) break;

    lastFound = found;
    detachStep.value = i + 1;

    if (i + 1 < DETACH_STEPS) await sleep(scanPeriod.value);
  }

  scanPeriod.value = DEFAULT_SCAN_PERIOD;
  detachStep.value = null;

  if (hits >= DETACH_HITS_REQUIRED && lastFound != null) {
    return lastFound;
  } else {
    toast.error("Could not find device after detaching");
    return null;
  }
};

const onFlash = async () => {
  let dev = selected.value.dev;
  if (dev == null) return;

  done.value = false;

  // Detach and change device if needed
  if (!dev.is_dfu) {
    dev = await detach(dev);
    if (dev == null) return;
  }

  fw.flash(dev.devnum);
  done.value = true;
};
</script>
