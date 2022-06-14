<template>
  <div class="form-control">
    <label class="label h-10">
      <span class="label-text"> {{ messageLeft }} </span>
      <span class="label-text-alt"> {{ messageRight }} </span>
    </label>
    <progress class="progress" :value="value" :max="maxValue"></progress>
  </div>
</template>

<script setup lang="ts">
import { defineProps } from "vue";

defineProps<{
  messageLeft: string;
  messageRight: string;
  value: number;
  maxValue: number;
}>();
</script>
