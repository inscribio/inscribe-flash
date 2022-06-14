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
      <tbody v-if="true">
        <tr
          v-for="(dev, i) in devices.unique"
          :key="i"
          :class="dev.devnum == selected ? 'active' : ''"
        >
          <th>{{ i + 1 }}</th>
          <td>{{ dev.is_dfu ? "Bootloader" : "Runtime" }}</td>
          <td>{{ dev.devnum }}</td>
          <td>
            {{
              padZeros(toHex(dev.vid), 4) + ":" + padZeros(toHex(dev.pid), 4)
            }}
          </td>
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
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { defineProps, ref } from "vue";
import { useToast } from "vue-toastification";
import { useDevicesStore } from "@/stores/devices";

const props = defineProps<{
  scan: boolean;
  period: number;
  selected: number | null;
}>();

const toast = useToast();
const devices = useDevicesStore();
const error = ref(false);

const padZeros = (v: string, n: number) =>
  "0".repeat(Math.max(0, n - v.length)) + v;

const toHex = (v: number) => v.toString(16);

const scan = () => {
  const again = () => setTimeout(scan, props.period);

  if (!props.scan) return again();

  devices.scan().then((r) => {
    if (r != undefined) {
      if (!error.value) toast.error("Device scan failed: " + r);
      error.value = true;
    } else {
      error.value = false;
    }
    again();
  });
};

setTimeout(scan, 0);
</script>
