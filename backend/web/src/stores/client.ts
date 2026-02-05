import { ofetch } from "ofetch";
import { defineStore } from "pinia";
import { ref } from "vue";
import { useAuthStore } from "./auth";
import { appStorage } from "@/utils/storage";

export const useClientStore = defineStore("client", () => {
  const authStore = useAuthStore();

  const client = ref(createClient());

  function createClient() {
    console.log("Creating new client instance");
    return ofetch.create({
      headers: {
        "Content-Type": "application/json",
      },
      responseType: "json",
      onRequest: ({ options }) => {
        const token = appStorage.getToken();
        if (token) {
          options.headers.set("Authorization", `Bearer ${token}`);
        }
      },
      onResponseError: ({ response }) => {
        if (response.status === 401) {
          authStore.logout();
        }
      },
    });
  }

  return { client };
});
