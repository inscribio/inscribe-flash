<template>
  <div class="overflow-x-auto">
    <table class="table table-compact w-full">
      <thead>
        <tr>
          <th class="w-12"></th>
          <th class="w-32">Mode</th>
          <th class="w-24">Devnum</th>
          <th class="w-32">VID:PID</th>
          <th class="w-32">Name</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="(dev, i) in devices.unique"
          :key="i"
          :class="dev.devnum == selected ? 'active' : ''"
        >
          <th>{{ i + 1 }}</th>
          <td>{{ dev.is_dfu ? "Bootloader" : "Runtime" }}</td>
          <td>{{ dev.devnum }}</td>
          <td>{{ vidPidString(dev.vid, dev.pid) }}</td>
          <td>
            {{
              devices.isKeyboard(dev)
                ? "Keyboard"
                : devices.isBootloader(dev)
                ? "Bootloader"
                : ""
            }}
          </td>
        </tr>

        <tr v-for="(dev, i) in devices.noDriver" :key="i">
          <th>{{ devices.unique.length + i + 1 }}</th>
          <td>No driver</td>
          <td>-</td>
          <td>{{ vidPidString(dev.vid, dev.pid) }}</td>
          <td>{{ dev.desc }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { defineProps, ref } from "vue";
import { useToast } from "vue-toastification";
import { vidPidString } from "@/utils";
import { useDevicesStore } from "@/stores/devices";

const props = defineProps<{
  scan: boolean;
  period: number;
  selected: number | null;
}>();

const toast = useToast();
const devices = useDevicesStore();
const error = ref(false);

const scan = () => {
  const again = () => setTimeout(scan, props.period);

  if (!props.scan) return again();

  devices
    .scan()
    .then(() => (error.value = false))
    .catch((e) => {
      // Ignore scan error if installation has just been started (during scan)
      if (devices.driverInstallOngoing) return;
      if (!error.value) toast.error("Device scan failed: " + e);
      error.value = true;
    })
    .finally(() => again());
};

setTimeout(scan, 0);
</script>
