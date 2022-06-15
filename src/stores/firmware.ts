import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { useToast } from "vue-toastification";

type FlashStage = "ready" | "init" | "erase" | "download";

type Progress = {
  Erase?: number;
  Download?: number;
};

const isProgress = (v: unknown): v is Progress =>
  typeof v == "object" && v != null && ("Erase" in v || "Download" in v);

export const useFirmwareStore = defineStore("Firmware", {
  state: () => {
    return {
      firmware: null as Uint8Array | null,
      filename: null as string | null,
      flashProgress: 0,
      flashStage: "ready" as FlashStage,
      unlisten: undefined as UnlistenFn | undefined,
    };
  },

  getters: {
    size(state): number | undefined {
      return state.firmware?.byteLength;
    },

    progressPercent(): number | undefined {
      if (this.size == undefined) return undefined;
      return (this.flashProgress * 100) / this.size;
    },

    firmwareOk(): boolean {
      const size = this.size;
      // Check for some sane size
      return size != undefined && size > 20 * 1024 && size < 64 * 1024;
    },
  },

  actions: {
    resetFirmware() {
      // TODO: same as in setFirmware
      if (this.flashStage != "ready") {
        const toast = useToast();
        toast.error("Cannot change firmware: flashing operation is ongoing");
        return;
      }
      this.firmware = null;
      this.filename = null;
    },

    setFirmware(bytes: Uint8Array, filename: string) {
      // TODO: this is here not to mess up progress, but overall this restriction is undeeded
      if (this.flashStage != "ready") {
        const toast = useToast();
        toast.error("Cannot change firmware: flashing operation is ongoing");
        return;
      }
      this.firmware = bytes;
      this.filename = filename;
    },

    async listenProgress() {
      this.unlisten = await listen("flash-progress", (e) => {
        const progress = e.payload;
        if (!isProgress(progress)) {
          throw Error("Unexpected flash-progress event: " + e);
        }

        if (progress.Erase) {
          this.flashProgress = progress.Erase;
          this.flashStage = "erase";
        } else if (progress.Download) {
          this.flashProgress = progress.Download;
          this.flashStage = "download";
        }
      });
    },

    async flash(devNum: number) {
      const toast = useToast();
      if (this.flashStage != "ready") {
        toast.error("Cannot flash: another flashing operation is ongoing");
        return;
      } else if (!this.firmwareOk) {
        toast.error("Cannot flash: invalid firmware");
        return;
      }

      this.flashProgress = 0;
      this.flashStage = "init";
      await this.listenProgress();

      const finish = (error?: string) => {
        if (this.unlisten != undefined) this.unlisten();
        this.flashStage = "ready";
        if (error != undefined) throw Error(error);
      };

      if (this.firmware == null) {
        finish("Firmware is null");
        return;
      }
      const firmware = Array.from(this.firmware);
      try {
        await invoke("flash", { firmware, devNum });
      } finally {
        finish();
      }
    },
  },
});
