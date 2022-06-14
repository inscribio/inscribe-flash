<template>
  <button @click="fileInput?.click()" class="btn">
    <input
      type="file"
      class="hidden"
      ref="fileInput"
      :accept="accept"
      @input="onUpload"
    />
    <slot>
      <Icon icon="ic:baseline-upload" class="text-lg" />
    </slot>
  </button>
</template>

<script setup lang="ts">
import { defineProps, defineEmits, ref } from "vue";
import { useToast } from "vue-toastification";

const props = defineProps<{
  accept: string;
  method:
    | "readAsArrayBuffer"
    | "readAsBinaryString"
    | "readAsDataURL"
    | "readAsText";
}>();

const emit = defineEmits<{
  (_e: "loaded", _file: File, _data: ArrayBuffer | string): void;
}>();

// const toast = useToast();
const fileInput = ref<HTMLElement | null>(null);

const onUpload = (e: InputEvent) => {
  console.log("ON UPLOAD", e);
  if (e?.target == null) return;
  if (e.target.files.length < 1) throw Error("No file has been uploaded");

  const file = e.target.files[0];

  const reader = new FileReader();
  reader.addEventListener("error", () => {
    // toast.error("Failed to read the file");
  });
  reader.addEventListener("load", () => {
    emit("loaded", file, reader.result);
  });

  reader[props.method](file);
};
</script>
