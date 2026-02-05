import { appStorage } from "@/utils/storage";
import { defineStore } from "pinia";
import { ref } from "vue";

export const useAuthStore = defineStore("auth", () => {
  const isLoggedIn = ref(appStorage.isLoggedIn());

  function login(token: string) {
    appStorage.login(token);
    isLoggedIn.value = true;
  }

  function logout() {
    appStorage.logout();
    isLoggedIn.value = false;
  }

  return { isLoggedIn, login, logout };
});
