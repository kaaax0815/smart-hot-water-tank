const TOKEN_KEY = "hotwater_token";

export const appStorage = {
  login(token: string) {
    localStorage.setItem(TOKEN_KEY, token);
  },
  logout() {
    localStorage.removeItem(TOKEN_KEY);
  },
  isLoggedIn() {
    return !!localStorage.getItem(TOKEN_KEY);
  },
  getToken() {
    return localStorage.getItem(TOKEN_KEY);
  },
};
