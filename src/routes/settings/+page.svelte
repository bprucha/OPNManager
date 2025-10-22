<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { LazyStore } from '@tauri-apps/plugin-store';
  import { goto } from "$app/navigation";
  import AppLayout from '../AppLayout.svelte';
  import SettingsForm from '$lib/components/forms/SettingsForm.svelte';
  import Login from '$lib/components/forms/Login.svelte';
  import { toasts } from '$lib/stores/toastStore';
  import { authStore } from '$lib/stores/authStore';
  import { checkStatus, authenticate, AuthMode, DecryptCipherData } from '@tauri-apps/plugin-biometric';
  import SecureDialog from '$lib/components/SecureDialog.svelte';
  import {
      mdiFingerprint,
  } from '@mdi/js';

  let verifyPinDialog: SecureDialog;
  let isBiometricLoginEnabled = false;
  let isBiometricLoginSupported = false;
  let isUpdatingBio = false;
  let apiKey = "";
  let apiSecret = "";
  let apiUrl = "";
  let port = 443;
  let pin = "";
  let currentPin = "";
  let newPin = "";
  let confirmNewPin = "";
  let activeTab: 'api' | 'pin' = 'api';
  let isFirstRun = false;
  let isUpdatingPin = false;

  const pinStore = new LazyStore('pin.json');
  let encryptedPinData: DecryptCipherData | undefined = undefined;

  onMount(async () => {
    if ($authStore.isLoggedIn) {
      await loadApiInfo();
    }
    isFirstRun = await invoke<boolean>("check_first_run");

    encryptedPinData = await pinStore.get<DecryptCipherData>('encryptedPinData');
    isBiometricLoginEnabled = encryptedPinData != null;

    if(import.meta.env.TAURI_ENV_PLATFORM === "android"
      //|| import.meta.env.TAURI_ENV_PLATFORM === "ios")
    ){
      try {
        const biometricStatus = await checkStatus();
        if(biometricStatus.isAvailable) {
          isBiometricLoginSupported = true;
        }
      } catch {

      }
    }
  });

  async function loadApiInfo() {
    try {
      const apiInfo = await invoke<{
        api_key: string;
        api_secret: string;
        api_url: string;
        port: number;
      }>("get_api_info");
      if (apiInfo) {
        apiKey = apiInfo.api_key;
        apiSecret = apiInfo.api_secret;
        apiUrl = apiInfo.api_url;
        port = apiInfo.port;
      }
    } catch (error) {
      console.error("Failed to fetch API info:", error);
      toasts.error("Failed to load settings. Please try again.");
    }
  }

async function handleApiSubmit(event: CustomEvent<{profileName: string, apiKey: string, apiSecret: string, apiUrl: string, port: number, pin: string}>) {
  const { profileName, apiKey, apiSecret, apiUrl, port, pin } = event.detail;
  try {
    if (isFirstRun) {
      await invoke("save_initial_config", { profileName, apiKey, apiSecret, apiUrl, port: Number(port), pin });
      isFirstRun = false;
      authStore.setConfigured(true);
      toasts.success("Initial configuration saved successfully!");
    } else {
      // Just save the new API info directly without validation
      // This allows users to update revoked API keys with new ones
      await invoke("update_api_info", { 
        profileName, 
        apiKey, 
        apiSecret, 
        apiUrl, 
        port: Number(port),
        isDefault: true
      });
      
      // Clear the traffic cache after updating API info
      await invoke("clear_traffic_cache");
      
      toasts.success("API information updated successfully!");
    }
    await loadApiInfo();
  } catch (error) {
    console.error("Failed to update API info:", error);
    toasts.error(`Failed to update API info: ${error}`);
  }
}

  async function handlePinSubmit() {
    if (isUpdatingPin) return; // Prevent multiple submissions
    
    // Client-side validation
    if (newPin !== confirmNewPin) {
      toasts.error("New PINs do not match.");
      return;
    }

    if (!/^\d+$/.test(newPin)) {
      toasts.error("PIN must contain only numbers.");
      return;
    }
    
    if (newPin.length < 4) {
      toasts.error("PIN must be at least 4 digits long.");
      return;
    }

    if (currentPin === newPin) {
      toasts.error("New PIN must be different from current PIN.");
      return;
    }

    isUpdatingPin = true; // Set this before the async operation
    
    try {
      // Provide helpful message while processing
      toasts.info("Updating PIN and re-encrypting credentials... This may take a moment.");
      
      // Use setTimeout to give the UI a chance to update first
      setTimeout(async () => {
        try {
          console.log("Starting PIN update process");
          
          // Proceed with the update
          await invoke("update_pin", { 
            currentPin, 
            newPin, 
            confirmNewPin 
          });
          
          // Clear form values
          currentPin = "";
          newPin = "";
          confirmNewPin = "";
          
          toasts.success("PIN updated successfully! Please log in with your new PIN.");
          
          // Give the UI time to show the success message
          setTimeout(() => {
            handleLogout();
          }, 2000);
        } catch (error) {
          console.error("Failed to update PIN:", error);
          isUpdatingPin = false; // Important to reset this in case of error
          
          // Display the exact error message from the backend
          if (error instanceof Error) {
            toasts.error(`Failed to update PIN: ${error.message}`);
          } else if (typeof error === 'string') {
            toasts.error(`Failed to update PIN: ${error}`);
          } else {
            toasts.error("Failed to update PIN: An unknown error occurred.");
          }
        }
      }, 100); // Small delay to allow the UI to update first
    } catch (error) {
      isUpdatingPin = false; // Reset in case of error
      console.error("Unexpected error:", error);
      toasts.error("An unexpected error occurred.");
    }
  }

  function handleLogout() {
    authStore.logout();
    goto('/');
  }

  async function toggleBiometricLogin(): Promise<void> {
    // A basic call to backend Rust. The Biometric response hangs sometimes
    // without another backend call to keep things going.
    async function biometricRefresh() {
      let isFirstRun = await invoke<boolean>("check_first_run");
      if(isUpdatingBio) {
        setTimeout(biometricRefresh, 1000)
      }
    }
    try {
      if(isBiometricLoginEnabled) {
        pinStore.delete('encryptedPinData')
        toasts.success("Biometric authentication disabled");
        isBiometricLoginEnabled = false
      } else {
        const verifyPin = await verifyPinDialog.show(
                      'Verify PIN',
                      'Verify your PIN to enable Biometric Login.'
                  )
        if(verifyPin) {
          isUpdatingBio = true;
          biometricRefresh();
          const pinVerified = await invoke("verify_pin", {pin: verifyPin});
          if(pinVerified) {
            try {
              let encryptedPinData = await authenticate('Continue to enable Biometric Login', {
                allowDeviceCredential: false,
                cancelTitle: "Cancel",
                fallbackTitle: 'Sorry, authentication failed',
                title: 'OPNManager Authentication',
                confirmationRequired: true,
                mode: AuthMode.ENCRYPT,
                cipherKey: "OPNManagerKey",
                cipherData: {
                  data: verifyPin,
                },
              });

              await pinStore.set('encryptedPinData', encryptedPinData);
              await pinStore.save();
              isBiometricLoginEnabled = true

              toasts.success("Biometric Login enabled");
            } catch(error: any) {
              if(error.code != 'userCancel') {
                toasts.error(`Biometric authentication failed: ${error?.message}`);
              }
            }
          } else {
            toasts.error("PIN verification failed. Cannot enable Biometric Login.");
          }
        }
      }
    } catch(error) {
      toasts.error(`Biometric authentication failed`);
    } finally {
      isUpdatingBio = false;
    }
  }

  function setActiveTab(tab: 'api' | 'pin') {
    activeTab = tab;
  }

  function handleFormError(event: CustomEvent<{ message: string }>) {
    toasts.error(event.detail.message);
  }

  function handleFormSuccess(event: CustomEvent<{ message: string }>) {
    toasts.success(event.detail.message);
  }
  
</script>

{#if $authStore.isLoggedIn || isFirstRun}
  <AppLayout>
    <div class="max-w-4xl mx-auto">
      <h2 class="text-2xl font-bold mb-6">Settings</h2>
      
      {#if !isFirstRun}
        <div class="tabs tabs-boxed mb-6">
          <button 
            type="button"
            class="tab {activeTab === 'api' ? 'tab-active' : ''}" 
            on:click={() => setActiveTab('api')}
            disabled={isUpdatingPin}
          >
            API Settings
          </button>
          <button 
            type="button"
            class="tab {activeTab === 'pin' ? 'tab-active' : ''}" 
            on:click={() => setActiveTab('pin')}
            disabled={isUpdatingPin}
          >
            Security
          </button>
        </div>
      {/if}

      {#if isFirstRun || activeTab === 'api'}
        <div class="bg-base-100 p-6 rounded-lg shadow-lg">
          <h3 class="text-xl font-semibold mb-4">
            {isFirstRun ? 'Initial Configuration' : 'API Configuration'}
          </h3>
          <SettingsForm 
            {apiKey}
            {apiSecret}
            {apiUrl}
            {port}
            {pin}
            showPin={isFirstRun}
            on:submit={handleApiSubmit}
            on:error={handleFormError}
          />
        </div>
      {:else if activeTab === 'pin'}
        <div class="bg-base-100 p-6 rounded-lg shadow-lg">
          <h3 class="text-xl font-semibold mb-4">Change PIN</h3>
          <form on:submit|preventDefault={handlePinSubmit} class="space-y-4">
            <div class="form-control">
              <label class="label" for="currentPin">
                <span class="label-text">Current PIN</span>
              </label>
              <input 
                id="currentPin"
                bind:value={currentPin}
                type="password"
                inputmode="numeric"
                pattern="\d*"
                placeholder="Enter current PIN"
                class="input input-bordered w-full"
                required
                disabled={isUpdatingPin}
              />
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="form-control">
                <label class="label" for="newPin">
                  <span class="label-text">New PIN</span>
                </label>
                <input 
                  id="newPin"
                  bind:value={newPin}
                  type="password"
                  inputmode="numeric"
                  pattern="\d*"
                  placeholder="Enter new PIN"
                  class="input input-bordered w-full"
                  required
                  disabled={isUpdatingPin}
                />
              </div>
              <div class="form-control">
                <label class="label" for="confirmNewPin">
                  <span class="label-text">Confirm New PIN</span>
                </label>
                <input 
                  id="confirmNewPin"
                  bind:value={confirmNewPin}
                  type="password"
                  inputmode="numeric"
                  pattern="\d*"
                  placeholder="Confirm new PIN"
                  class="input input-bordered w-full"
                  required
                  disabled={isUpdatingPin}
                />
              </div>
            </div>
            <div class="flex justify-end mt-6">
              <button type="submit" class="btn btn-primary" disabled={isUpdatingPin}>
                {#if isUpdatingPin}
                  <span class="loading loading-spinner loading-sm mr-2"></span>
                  Updating PIN...
                {:else}
                  Update PIN
                {/if}
              </button>
            </div>
            
            {#if isUpdatingPin}
              <div class="mt-4 text-sm text-info">
                <p>PIN update in progress. This might take a moment. Please wait...</p>
              </div>
            {/if}
          </form>
        </div>
        {#if isBiometricLoginSupported || isBiometricLoginEnabled}
          <div class="bg-base-100 p-6 rounded-lg shadow-lg mt-6">
            <h3 class="text-xl font-semibold mb-4">Biometric Login</h3>
            <SecureDialog bind:this={verifyPinDialog} />

            <div class="flex items-center form-control">
              <div class="size-14 shrink-0 items-center justify-center mr-4">
                  <svg viewBox="0 0 24 24">
                      <path fill="currentColor" d={mdiFingerprint} />
                  </svg>
              </div>
              <button type="submit" class="btn btn-primary" disabled={isUpdatingBio || isUpdatingPin} on:click={toggleBiometricLogin}>
                {#if isBiometricLoginEnabled}
                  {#if isUpdatingBio}
                    <span class="loading loading-spinner loading-sm mr-2"></span>
                    Disabling
                  {:else}
                    Disable
                  {/if}
                {:else}
                  {#if isUpdatingBio}
                    <span class="loading loading-spinner loading-sm mr-2"></span>
                    Enabling
                  {:else}
                    Enable
                  {/if}
                {/if}
              </button>
            </div>
          </div>
        {/if}
      {/if}
    </div>
  </AppLayout>
{:else}
  <Login />
{/if}

