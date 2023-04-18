<template>
  <VueFinalModal
    class="flex justify-center items-center"
    content-class="modal-box"
    :modelValue="state != 'idle'"
    @closed="onClosed"
    :clickToClose="state == 'selecting' || state == 'done'"
    :escToClose="state == 'selecting' || state == 'done'"
  >
    <h1 class="font-semibold text-lg">
      {{ title }}
    </h1>

    <div class="py-4 flex flex-col justify-center items-start gap-4">
      <p class="prose">
        {{ description }}
      </p>

      <div class="ml-2" v-for="(dev, i) in candidateDevices" :key="i">
        <label class="flex justify-start items-center cursor-pointer gap-4">
          <input
            v-if="state == 'selecting'"
            type="checkbox"
            class="checkbox"
            :value="i"
            v-model="selectedDevices"
          />
          <template v-else>
            <input
              v-if="!selectedDevices.has(i)"
              type="checkbox"
              class="checkbox"
              :value="i"
              :disabled="true"
              :modelValue="false"
            />
            <Icon
              v-else-if="installedDevices[i] == undefined"
              icon="gg:spinner"
              class="text-2xl animate-spin"
            />
            <Icon
              v-else-if="installedDevices[i] == false"
              icon="ic:baseline-error"
              class="text-2xl"
            />
            <Icon v-else icon="ic:baseline-check-circle" class="text-2xl" />
          </template>

          <p class="prose">
            {{ `${dev.desc} [${vidPidString(dev.vid, dev.pid)}]` }}
          </p>
        </label>
      </div>

      <p class="prose" v-if="state == 'selecting' || state == 'preparing'">
        <b>NOTE:</b>
        Uses
        <a href="https://github.com/pbatard/libwdi" target="_blank">libwdi</a>.
        This operation will require administrator permissions. System popup
        window may appear and has to be accepted.
      </p>
    </div>

    <div class="modal-action">
      <button
        v-if="state == 'selecting'"
        class="btn btn-ghost"
        @click="() => confirm(false)"
      >
        Cancel
      </button>
      <button
        class="btn btn-accent min-w-[5rem]"
        :disabled="
          (state != 'selecting' && state != 'done') || selectedDevices.size == 0
        "
        @click="() => confirm(true)"
      >
        {{ state == "done" ? "Ok" : "Install" }}
      </button>
    </div>
  </VueFinalModal>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { computed, defineExpose, onMounted, onBeforeUnmount, ref } from "vue";
import { VueFinalModal } from "vue-final-modal";
import { assertUnreachable, sleep, vidPidString } from "@/utils";
import { useDevicesStore, LibwdiDevice } from "@/stores/devices";

const devices = useDevicesStore();

type State =
  | "idle" // Called install() but not yet verified that we will start
  | "selecting" // There are some devices, we're on the device selection view
  | "preparing" // Invoked winusb_install, waiting for "Started" event
  | "ongoing" // Installation is onging
  | "done";

const progressMessage = ref(0);
const candidateDevices = ref([] as LibwdiDevice[]);
const selectedDevices = ref(new Set() as Set<number>);
const installedDevices = ref({} as Record<number, boolean>);
const state = ref("idle" as State);

const resetState = () => {
  progressMessage.value = 0;
  candidateDevices.value = [];
  selectedDevices.value = new Set();
  installedDevices.value = {};
  state.value = "idle";
};

const install = async () => {
  try {
    await doInstall();
  } finally {
    resetState();
  }
};

defineExpose({ install });

// Messages that appear in our progress bar as time passes
const PROGRESS_MESSAGES = [
  "Installation can take some time...",
  "The installation process can take up to 5 minutes...",
  "You may also be asked to reboot for KMDF upgrades.",
  "If so, please watch for additional popup windows.", // 1 min
  "The reason driver installation may take time...",
  "...is because a System Restore point is created.",
  "Microsoft offers no means of checking progress...",
  "...so we can't say how long it'll take...", // 2 mins
  "Please continue to be patient...",
  "There's a 5 minute timeout eventually...",
  "...so if there's a problem, the process will abort.",
  "I've really seen an installation take 5 minutes...", // 3 mins
  "...on a Vista 64 machine with a very large disk.",
  "So how was your day...",
  "...before it got ruined by this endless installation?",
  "Seriously, what is taking this process so long?!", // 4 mins
  "Aborting in 45 seconds...",
  "Aborting in 30 seconds...",
  "Aborting in 15 seconds...",
];

const title = computed(() =>
  state.value == "selecting"
    ? "Install missing USB drivers?"
    : "Installing drivers…"
);

const description = computed(() => {
  const s = state.value;
  return s == "idle"
    ? ""
    : s == "selecting"
    ? "Some device drivers are missing. Do you want to install WinUSB drivers for the following devices?"
    : s == "preparing"
    ? "Preparing installation..."
    : s == "ongoing"
    ? PROGRESS_MESSAGES[progressMessage.value]
    : s == "done"
    ? "Installation complete"
    : assertUnreachable(s);
});

type InstallResult = { Ok?: null; Err?: string };

type Progress = "Started" | { Device: [LibwdiDevice, InstallResult] };

const isProgress = (v: unknown): v is Progress =>
  v == "Started" || (typeof v == "object" && v != null && "Device" in v);

const isOk = (r: InstallResult): boolean => r.Ok !== undefined;

const unlisten = ref<UnlistenFn | null>(null);
onMounted(async () => {
  unlisten.value = await listen("winusb-progress", onWinusbProgress);
});
onBeforeUnmount(() => {
  if (unlisten.value != null) unlisten.value();
  unlisten.value = null;
});

const devicesEq = (a: LibwdiDevice, b: LibwdiDevice): boolean =>
  a.vid == b.vid &&
  a.pid == b.pid &&
  a.desc == b.desc &&
  a.device_id == b.device_id;

// Listen for progress events
const onWinusbProgress = async (e: { payload?: Progress }) => {
  const progress = e.payload;
  if (!isProgress(progress)) {
    console.log("Unexpected progress event", e);
    return;
  }

  if (progress == "Started") {
    if (state.value == "preparing") {
      state.value = "ongoing";
      updateProgressMessages();
    } else {
      console.log(`Error: received ${progress} event in state ${state.value}`);
    }
  } else if (progress.Device) {
    for (const i of selectedDevices.value) {
      if (devicesEq(progress.Device[0], candidateDevices.value[i])) {
        installedDevices.value[i] = isOk(progress.Device[1]);
      }
    }
  } else {
    console.log("Unexpected progress event", e);
  }
};

const updateProgressMessages = async () => {
  // Change libwdi progress messages every 15 seconds
  for (let i = 0; i < PROGRESS_MESSAGES.length; i++) {
    if (state.value == "done" || state.value == "idle") return;
    progressMessage.value = i;
    await sleep(15 * 1000);
  }
};

const doInstall = async () => {
  // Freeze the list of devices for the whole installation process
  candidateDevices.value = [...devices.noDriver];
  if (candidateDevices.value.length == 0)
    throw Error("No candidate devices for driver installation");

  // Show modal and wait for devices choice and confirmation
  state.value = "selecting";
  const confirmed = await waitConfirm();

  if (confirmed == false) return;

  const devs = [...selectedDevices.value.values()].map(
    (i) => candidateDevices.value[i]
  );
  if (devs.length == 0) throw Error("No devices selected for installation");

  devices.driverInstallOngoing = true;
  state.value = "preparing";

  try {
    await invoke("winusb_install", { devices: devs });
  } finally {
    state.value = "done";
    devices.driverInstallOngoing = false;

    // Block to allow user to observe results before clicking "Done"
    await waitConfirm();
  }
};

// FIXME: any way to block on a promise until confirm is invoked instead ugly of polling?
let confirmed = null as null | boolean;
const waitConfirm = async () => {
  while (confirmed == null) {
    await sleep(50);
  }
  const answer = confirmed;
  confirmed = null;
  return answer;
};

const confirm = (ok: boolean) => {
  confirmed = ok;
};

const onClosed = () => {
  if (confirmed == null) confirmed = false;
};
</script>
