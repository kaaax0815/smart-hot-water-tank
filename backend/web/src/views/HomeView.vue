<template>
  <h1>Sensors</h1>
  <ul>
    <li v-for="sensor in data" :key="sensor.id">
      {{ sensor.name }}
    </li>
  </ul>
</template>

<script setup lang="ts">
import { useSensors } from "@/queries/sensors";
import { useAuthStore } from "@/stores/auth";
import { watchEffect } from "vue";
import { useRouter } from "vue-router";

const authStore = useAuthStore();
const router = useRouter();
const { data } = useSensors();

watchEffect(() => {
  if (!authStore.isLoggedIn) {
    router.push("/login");
  }
});
</script>
