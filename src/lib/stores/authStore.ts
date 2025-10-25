import { writable } from 'svelte/store';
import { invoke } from "@tauri-apps/api/core";

interface AuthState {
  isLoggedIn: boolean;
  isConfigured: boolean;
  baseUrl: string;
  authHeader: HeadersInit;
}

async function cleanupOnLogout() {
  console.log("Performing logout cleanup");
  
  try {

    await invoke("clear_traffic_cache").catch(err => {
      console.error("Error clearing traffic cache:", err);
    });

    await invoke("stop_log_polling").catch(err => {
      console.error("Error stopping log polling:", err);
    });

    await invoke("clear_pin").catch(err => {
      console.error("Error clearing PIN cache:", err);
    });
    
    console.log("Logout cleanup completed");
    return true;
  } catch (error) {
    console.error("Error during logout cleanup:", error);
    return false;
  }
}

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>({
    isLoggedIn: false,
    isConfigured: false,
    baseUrl: '',
    authHeader: {},
  });

  return {
    subscribe,
    login: async () => {
      let apiInfo = await invoke<{
        api_key: string;
        api_secret: string;
        api_url: string;
        port: number;
      }>("get_api_info");
      
      update(state => ({
        ...state,
        isLoggedIn: true,
        baseUrl: apiInfo.api_url + (
          (
            (apiInfo.api_url.startsWith('http:') && apiInfo.port == 80) ||
            (apiInfo.api_url.startsWith('https:') && apiInfo.port == 443))
          ? '' : `:${apiInfo.port}`),
        authHeader: {'Authorization': `Basic ${btoa(`${apiInfo.api_key}:${apiInfo.api_secret}`)}`}
      }))
    },
    logout: async () => {
      await cleanupOnLogout();
      update(state => ({ ...state, isLoggedIn: false }));
    },
    setConfigured: (value: boolean) => update(state => ({ ...state, isConfigured: value })),
    reset: () => set({ isLoggedIn: false, isConfigured: false, baseUrl: '', authHeader: {} })
  };
}

export const authStore = createAuthStore();