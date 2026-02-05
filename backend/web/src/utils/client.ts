import { ofetch } from "ofetch";
import { appStorage } from "./storage";

export const client = ofetch.create({
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
      appStorage.logout();
    }
  },
});
