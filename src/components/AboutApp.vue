<template>
  <button
    class="text-xl btn btn-circle btn-sm btn-ghost opacity-75 hover:opacity-100"
    @click="open = !open"
  >
    <Icon icon="material-symbols:info-outline" />

    <VueFinalModal
      class="flex justify-center items-center"
      content-class="modal-box"
      :modelValue="open"
      @closed="open = false"
    >
      <div class="m-2 flex justify-start items-center">
        <img :src="iconPath" alt="logo" class="w-40" />

        <div class="m-2 ml-10 flex flex-col justify-center items-start gap-2">
          <span class="font-semibold text-xl"
            >inscribe-flash
            <template v-if="version != ''"
              >| {{ "v" + version }}</template
            ></span
          >

          <a href="https://inscrib.io/" target="_blank" class="link"
            >inscrib.io</a
          >

          <a href="https://inscrib.io/inscribe/" target="_blank" class="link"
            >Configurator</a
          >

          <a
            href="https://github.com/inscribio/inscribe-flash/"
            target="_blank"
            class="link"
            >GitHub</a
          >
        </div>
      </div>
    </VueFinalModal>
  </button>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/tauri";
import { getVersion } from "@tauri-apps/api/app";
import { VueFinalModal } from "vue-final-modal";

const open = ref(false);
const version = ref("");

const iconPath = convertFileSrc("icons/icon.png");
getVersion().then((v) => (version.value = v));
</script>
