<template>
  <h1>Login</h1>
  <form @submit.prevent="handleLogin">
    <div>
      <label for="password">Password:</label>
      <input type="password" id="password" name="password" />
    </div>
    <button type="submit">Login</button>
  </form>
</template>

<script setup lang="ts">
import { useRouter } from "vue-router";
import { useAuthStore } from "@/stores/auth";

const router = useRouter();
const authStore = useAuthStore();

function handleLogin(event: Event) {
  const form = event.target as HTMLFormElement;
  const passwordInput = form.querySelector("#password") as HTMLInputElement;
  const password = passwordInput.value;

  authStore.login(password);
  router.push("/");
}
</script>
