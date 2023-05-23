<template>
  <div
    class="antialiased mx-auto min-h-screen max-w-lg flex flex-col justify-between items-center pt-12 pb-14 gap-4"
  >
    <div class="w-full flex flex-col justify-center items-center gap-8">
      <div class="w-full max-w-md form-control">
        <FirmwareSelect class="w-full h-36" :disabled="ongoing" />

        <label class="label px-4">
          <span class="label-text">
            <span class=""> File: </span>
            {{ filename != "" ? filename : "-" }}
          </span>
          <span class="label-text-alt">
            {{ fw.size == null ? "" : (fw.size / 1024).toFixed(1) + " kB" }}
          </span>
        </label>
      </div>

      <div class="btn-group btn-group-vertical">
        <button
          class="btn btn-accent btn-wide gap-2"
          :disabled="!fw.firmwareOk"
          @click="onFlash"
        >
          <Icon icon="ic:round-flash-on" class="text-lg" />
          Flash
        </button>

        <button
          v-if="driver_install_needed"
          class="btn btn-secondary gap-2"
          @click="onInstallDrivers"
        >
          <Icon icon="ic:baseline-install-desktop" class="text-lg" />
          Install drivers
        </button>
      </div>
    </div>

    <div class="form-control">
      <label class="label">
        <h1 class="label-text">Detected devices</h1>
      </label>
      <DeviceList
        :scan="true"
        :period="scanPeriod"
        :selected="selected.dev?.devnum ?? null"
        @deviceSelected="onDeviceSelected"
      />
      <label class="label">
        <h1 class="label-text italic">{{ selected?.tooltip ?? "" }}</h1>
      </label>
    </div>

    <DriversInstall ref="installer" />

    <ProgressBar
      class="w-full fixed bottom-0 translate-y-1/1 pb-4 px-10"
      :messageLeft="progress.msg"
      :messageRight="progress.note"
      :value="progress.value"
      :maxValue="progress.max"
    />

    <AboutApp class="absolute top-3 right-3" />

    <ModalsContainer />
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { basename } from "@tauri-apps/api/path";
import { computed, ref, watch } from "vue";
import { ModalsContainer } from "vue-final-modal";
import { useToast } from "vue-toastification";
import { sleep } from "@/utils";
import { useFirmwareStore } from "@/stores/firmware";
import { useDevicesStore, DfuListEntry, LibwdiDevice } from "@/stores/devices";
import FirmwareSelect from "@/components/FirmwareSelect.vue";
import DeviceList from "@/components/DeviceList.vue";
import ProgressBar from "@/components/ProgressBar.vue";
import DriversInstall from "@/components/DriversInstall.vue";
import AboutApp from "@/components/AboutApp.vue";

const toast = useToast();
const fw = useFirmwareStore();
const devices = useDevicesStore();

const DEFAULT_SCAN_PERIOD = 750;

const DETACH_SCAN_PERIOD = 250;
const DETACH_MAX_WAIT = 6000;
const DETACH_STEPS = Math.ceil(DETACH_MAX_WAIT / DETACH_SCAN_PERIOD);
const DETACH_SUB_STEPS = 5;
const DETACH_HITS_REQUIRED = 3;

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
const detachSubStep = ref(null as number | null);
const progress = ref<ProgressInfo>(defaultProgress);
const done = ref(false);
const filename = ref("");
const installer = ref<InstanceType<typeof DriversInstall> | null>(null);
const selectedDevnum = ref(null as number | null);

watch(
  () => fw.filename,
  async (name) => {
    try {
      filename.value = await basename(name ?? "");
    } catch {
      filename.value = "";
    }
  }
);

const ongoing = computed(
  () => fw.flashStage != "ready" || detachSubStep.value != null
);

const driver_install_needed = computed(() => devices.noDriver.length > 0);

const getProgress = () => {
  if (detachSubStep.value != null) {
    return {
      msg: "Detaching...",
      note: "",
      value: detachSubStep.value,
      max: DETACH_STEPS * DETACH_SUB_STEPS,
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
  () => [detachSubStep.value, fw.flashStage, fw.flashProgress],
  () => (progress.value = getProgress())
);

const onDeviceSelected = (dev: DfuListEntry) => {
  if (selectedDevnum.value == dev.devnum) selectedDevnum.value = null;
  else selectedDevnum.value = dev.devnum;
};

const selected = computed(() => {
  const keyboards = devices.unique.filter(devices.isKeyboard);
  const bootloaders = devices.unique.filter(devices.isBootloader);

  const userSelected = [...keyboards, ...bootloaders].find(
    (dev) => dev.devnum == selectedDevnum.value
  );

  if (keyboards.length == 0 && bootloaders.length == 0) {
    return { dev: null, tooltip: "No compatible devices" };
  } else if (userSelected != undefined) {
    // Use user's choice when possible
    return {
      dev: userSelected,
      tooltip: `Will use selected devnum=${userSelected.devnum}`,
    };
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

const detach = async (dev: DfuListEntry) => {
  // Note now which devices with the same vid:pid exist now to ignore later.
  const ignore = devices.unique
    .filter((d) => d.vid == dev.vid && d.pid == dev.pid)
    .map((d) => d.devnum);
  const considered = () =>
    devices.unique.filter((d) => !ignore.includes(d.devnum));

  // Analogically for devices without drivers but use different key
  const noDriverDevKey = (dev: LibwdiDevice) =>
    `${dev.vid}:${dev.pid}:${dev.desc}:${dev.device_id ?? "-"}`;
  const ignoreNoDriver = devices.noDriver.map(noDriverDevKey);
  const consideredNoDriver = () =>
    devices.noDriver.filter((d) => !ignoreNoDriver.includes(noDriverDevKey(d)));

  const bootloadSecureMsg =
    "Did you press AllowBootloader key? (required in secure bootload mode)";

  try {
    // Wait for the device to appear
    scanPeriod.value = DETACH_SCAN_PERIOD;
    detachSubStep.value = 0;

    try {
      await invoke("detach", { devNum: dev.devnum });
    } catch (error) {
      console.log(error);
      toast.error("Could not detach device. " + bootloadSecureMsg);
      return null;
    }

    // Find a non-ignored bootloader
    const find = () => {
      const bootloaders = considered().filter(devices.isBootloader);
      // Sort by newest
      bootloaders.sort((a, b) => b.devnum - a.devnum);
      return bootloaders[0];
    };
    const findNoDriver = () => consideredNoDriver()[0];
    const tryNoDriverKey = (dev: LibwdiDevice | null) =>
      dev != null ? noDriverDevKey(dev) : null;
    const noDriverDevCmp = (found: LibwdiDevice, last: LibwdiDevice | null) =>
      noDriverDevKey(found) == tryNoDriverKey(last);

    let lastFound = null as DfuListEntry | null;
    let hits = 0;
    let lastFoundNoDriver = null as LibwdiDevice | null;
    let hitsNoDriver = 0;

    for (let i = 0; i < DETACH_STEPS; i++) {
      const found = find();

      if (found == undefined) hits = 0;
      else if (found.devnum == lastFound?.devnum) hits += 1;
      else hits = 0;

      const foundNoDriver = findNoDriver();
      if (foundNoDriver == undefined) hitsNoDriver = 0;
      else if (noDriverDevCmp(foundNoDriver, lastFoundNoDriver))
        hitsNoDriver += 1;
      else hitsNoDriver = 0;

      // If there is a device without drivers then install them
      if (hitsNoDriver >= DETACH_HITS_REQUIRED) {
        await installer.value?.install();
        hitsNoDriver = 0;
        // restart detach loop
        i = 0;
        // add this device to ignored ones
        ignoreNoDriver.push(noDriverDevKey(foundNoDriver));
      }

      if (hits >= DETACH_HITS_REQUIRED) break;

      lastFound = found;
      lastFoundNoDriver = foundNoDriver;

      for (let j = 0; j < DETACH_SUB_STEPS; j++) {
        detachSubStep.value += 1;
        await sleep(Math.floor(scanPeriod.value / DETACH_SUB_STEPS));
      }
    }

    if (hits >= DETACH_HITS_REQUIRED && lastFound != null) {
      return lastFound;
    } else {
      toast.error(
        "Could not find device after detaching. " + bootloadSecureMsg
      );
      return null;
    }
  } finally {
    scanPeriod.value = DEFAULT_SCAN_PERIOD;
    detachSubStep.value = null;
  }
};

const onFlash = async () => {
  if (driver_install_needed.value && selected.value.dev == null) {
    await installer.value?.install();
    toast.warning(
      "Some devices are missing USB drivers. Click INSTALL DRIVERS if you encounter problems."
    );
  }

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

const onInstallDrivers = async () => {
  try {
    await installer.value?.install();
  } catch (_error) {
    toast.error("Installation failed");
  }
};
</script>

<style>
.file-drop-area.dragover {
  @apply bg-neutral;
}
</style>
