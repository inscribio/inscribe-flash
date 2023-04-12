import { invoke } from "@tauri-apps/api/tauri";
import { defineStore } from "pinia";

const STM32_BOOTLOADER_VID_PID = [0x0483, 0xdf11];
const KEYBOARD_VID_PID = [0x16c0, 0x27db];

export type DfuListEntry = {
  is_dfu: boolean;
  vid: number;
  pid: number;
  devnum: number;
  alt: number;
};

export type LibwdiDevice = {
  vid: number;
  pid: number;
  is_composite: boolean;
  mi?: number;
  driver_version?: string;
  desc: string;
  device_id?: string;
  hardware_id?: string;
  compatible_id?: string;
  upper_filter?: string;
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

function arraysEqual<T>(a: Array<T>, b: Array<T>): boolean {
  return a.every((v, i) => v == b[i]);
}

export const useDevicesStore = defineStore("Devices", {
  state: () => {
    return {
      devices: [] as DfuListEntry[],
      noDriver: [] as LibwdiDevice[],
      driverInstallOngoing: false,
    };
  },

  getters: {
    unique(state): DfuListEntry[] {
      const devnums = new Set();
      return state.devices.filter((d) => {
        if (devnums.has(d.devnum)) return false;
        devnums.add(d.devnum);
        return true;
      });
    },
  },

  actions: {
    async scan() {
      const devices = await invoke("list");
      if (!isDfuListEntryList(devices)) {
        throw Error("Unexpected data");
      }
      this.devices = devices;

      if (!this.driverInstallOngoing && (await invoke("has_winusb"))) {
        this.noDriver = await invoke("winusb_candidates");
      }
    },

    isKeyboard(dev: DfuListEntry): boolean {
      return arraysEqual([dev.vid, dev.pid], KEYBOARD_VID_PID);
    },

    isBootloader(dev: DfuListEntry): boolean {
      return arraysEqual([dev.vid, dev.pid], STM32_BOOTLOADER_VID_PID);
    },
  },
});
