<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/stores/toastStore";
  import AppLayout from "../AppLayout.svelte";
  import { fetch } from '@tauri-apps/plugin-http';
  import { mdiTrashCan, mdiRefresh } from "@mdi/js";
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { authStore } from "$lib/stores/authStore";
  import Choices, {type InputChoice } from "choices.js";
  import "choices.js/public/assets/styles/choices.css";

  interface CombinedDevice {
    mac: string;
    ipv4_addresses: string[];
    ipv6_addresses: string[];
    intf: string;
    expired?: boolean;
    expires?: number;
    permanent?: boolean;
    device_type?: string;
    manufacturer: string;
    hostname: string;
    intf_description: string;
  }

  let devices: CombinedDevice[] = [];

  // Page state
  let loading = true;
  let error: string | null = null;
  let states = [];
  let total = 0;
  let currentPage = 1;
  let rowsPerPage = 25;
  let searchPhrase = "";
  let appliedSearchPhrase = "";
  let editModal = { isOpen: false, state: null };
  let addModal = { isOpen: false };
  let deleteModal = { isOpen: false, id: null as String | null };
  let newTunable = { state: '', value: '', descr: '' };
  let expandedRow: string | null = null; // Track which row is expanded on mobile
  let confirmKillMatchedStates: ConfirmDialog;
  let deviceChoice: HTMLSelectElement;
  
  // Close any modals on page unload
  onDestroy(() => {
    editModal.isOpen = false;
  });

  // Load firewall states
  async function loadStates() {
    loading = true;
    error = null;
    try {
      const response = await fetch(`${$authStore.baseUrl}/api/diagnostics/firewall/query_states`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...$authStore.authHeader,
        },
        body: JSON.stringify({
          current: currentPage,
          rowCount: rowsPerPage,
          ...(deviceChoice.value !== null && deviceChoice.value !== '' && { searchPhrase: deviceChoice.value }),
          sort: {}
        })
      });
      let result = await response.json();

      states = result.rows || [];
      total = result.total || 0;
      appliedSearchPhrase = deviceChoice.value;
    } catch (err) {
      console.error("Failed to load states:", err);
      error = err.toString();
      states = [];
      total = 0;
      
      // Display toast only if it's not a 404 (version compatibility) or 403 (permissions)
      // since we'll show these directly in the UI instead
      if (!error.includes('404') && !error.includes('403')) {
        toasts.error(`Failed to load states: ${err}`);
      }
    } finally {
      loading = false;
    }
  }

  // Open add modal
  async function handleResetState() {
    let confrimResult = await confirmKillMatchedStates.show("Confirm Reset State Table",
      `Select 'Confirm' to continue resetting the state table or 'Cancel' to abort. Resetting the state
      table will remove all entries from the corresponding tables. This means that all open connections
      will be broken and will have to be re-established. This may be necessary after making substantial
      changes to the firewall and/or NAT rules, especially if there are IP protocol mappings (e.g. for
      PPTP or IPv6) with open connections.
      The firewall will normally leave the state tables intact when changing rules.`, mdiTrashCan)
    if(confrimResult) {
      const response = await fetch(`${$authStore.baseUrl}/api/diagnostics/firewall/flush_states/`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...$authStore.authHeader,
        },
        body: "{}"
      });

      loadStates();
      toasts.success(`State table has been reset.`);
    }
  }
  
  async function handleResetSource() {
    let confrimResult = await confirmKillMatchedStates.show("Confirm Reset Source Tracking",
      `Select 'Confirm' to continue resetting source tracking or 'Cancel' to abort. Resetting the
      source tracking table will remove all source/destination associations. This means that the
      "sticky" source/destination association will be cleared for all clients.
      This does not clear active connection states, only source tracking.`, mdiTrashCan)
    if(confrimResult) {
      const response = await fetch(`${$authStore.baseUrl}/api/diagnostics/firewall/flush_sources/`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...$authStore.authHeader,
        },
        body: "{}"
      });

      loadStates();
      toasts.success(`Source tracking has been reset.`);
    }
  }

  // Close add modal
  function closeAddModal() {
    addModal.isOpen = false;
  }
  
  // Open delete confirmation modal
  function openDeleteModal(id: string) {
    deleteModal = {
      isOpen: true,
      id,
    };
  }
  
  // Close delete confirmation modal
  function closeDeleteModal() {
    deleteModal = {
      isOpen: false,
      id: null,
    };
  }
  
  // Handle deleting a state
  async function handleDeleteState() {
    if (!deleteModal.id) {
      toasts.error("No state selected for deletion");
      return;
    }
    
    try {
      console.log("Deleting state with ID:", deleteModal.id);
      // const result = await invoke("delete_state", {
      //   id: deleteModal.id
      // });
      
      // console.log("Delete state result:", result);
      
      // if (result.result === "deleted") {
      //   // Apply the changes immediately
      //   const applyResult = await invoke("apply_states");
      //   console.log("Apply result after delete:", applyResult);
        
      //   if (applyResult.status === "ok") {
      //     toasts.success("Tunable deleted and changes applied successfully");
      //   } else {
      //     toasts.success("Tunable deleted but could not apply changes");
      //   }
        
        closeDeleteModal();
        
        // Reload the states list
        loadStates();
      // } else {
      //   toasts.error("Failed to delete state");
      // }
    } catch (error) {
      console.error("Failed to delete state:", error);
      toasts.error(`Failed to delete state: ${error}`);
    }
  }
  
  // Handle batch delete for selected states
  async function handleBatchDelete() {
    if(deviceChoice.value === '') {
      toasts.error(`Search pharse cannot be blank.`);
      return;
    }
    let confrimResult = await confirmKillMatchedStates.show("Confirm Kill States", "Select 'Confirm' to continue killing the matched States or 'Cancel' to abort.", mdiTrashCan)
    if(confrimResult) {
      const response = await fetch(`${$authStore.baseUrl}/api/diagnostics/firewall/kill_states/`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...$authStore.authHeader,
        },
        body: JSON.stringify({
          filter: deviceChoice.value,
        })
      });

      loadStates();
      toasts.success(`States have been killed.`);
    }
  }
  
  // Handle pagination
  function handlePageChange(newPage: number) {
    currentPage = newPage;
    loadStates();
  }
  
  // Handle search
  function handleSearch() {
    currentPage = 1;
    loadStates();
  }
  
  // Initial load
  onMount(async () => {
    if ($authStore.isLoggedIn) {
      loading = true;

      await fetchDevices();

      const deviceChoices = new Choices(deviceChoice, {
        duplicateItemsAllowed: false,
        searchEnabled: true,
        searchChoices: true,
        addItems: false,
        removeItemButton: true,
        placeholderValue: 'Select a device to search',
        callbackOnCreateTemplates: function(strToEl, escapeForTemplate, getClassNames) {
          return {
            item: (
              options: any,
              choice: any,
              removeItemButton: boolean,
            ) => {
              let template = Choices.defaults.templates.item(options, choice, removeItemButton);
              return template;
            },
          }
        },
      });
      deviceChoice.value = '';

      loadStates();
      loading = false;
    }
  });

  async function fetchDevices() {
    try {
      devices = await invoke<CombinedDevice[]>("get_combined_devices");
    } catch (error) {
      console.error("Failed to fetch devices:", error);
      toasts.error("Failed to fetch devices. Please try again.");
    }
  }
</script>

<AppLayout>
  <style>
    [data-theme="dark"] .choices__inner {
      background-color: #191e24;
      border: 1px solid #3b3e40;
    }
    [data-theme="dark"] .choices__input {
      background-color: #191e24;
    }
    [data-theme="dark"] .choices__input--cloned {
      background-color: #191e24 !important;
    }
    [data-theme="dark"] .choices__item--choice {
      background-color: #191e24 !important;
    }
    [data-theme="dark"] .choices__button {
      filter: brightness(0) invert(1);
    }
    .choices {
      width: 100%;
    }
    
    /* Light Theme */
    :root {
      --chart-grid-color: #e5e5e5;
    }
    /* Dark Theme */
    :root[data-theme="dark"] {
      --chart-grid-color: #3b3e40;
    }
  </style>
  <ConfirmDialog bind:this={confirmKillMatchedStates} />
  <div class="p-4">
    <div class="mb-6">
      <h1 class="text-2xl font-bold mb-2">Firewall States</h1>
      <p class="text-sm opacity-75">
        System state allows you to terminate active connections. This could be used to terminate existing
        connections that were active before enabling a blocking rule that would otherwise prevent these
        connections from being established.
      </p>
    </div>
    
    <!-- Actions and Search Bar -->
    <div class="mb-6 space-y-3">
      <div class="flex justify-start">
        <div class="flex flex-grow">
        <select bind:this={deviceChoice} id="searchFilter" on:change={() => {loadStates()}}>
          {#each devices as device}
            {#if device.hostname || device.ipv4_addresses || device.ipv6_addresses}
              {#if device.ipv4_addresses}
                <option value="{device.ipv4_addresses}">{device.ipv4_addresses}{#if device.hostname}&nbsp;({device.hostname}){/if}</option>
              {/if}
              <!-- {#if device.ipv6_addresses}
                <option value="{device.ipv6_addresses}">{device.ipv6_addresses}{#if device.hostname}&nbsp;({device.hostname}){/if}</option>
              {/if} -->
            {/if}
          {/each}
        </select>
        </div>
        <button class="flex flex-initial items-center ml-2" aria-label="refresh" on:click={() => {fetchDevices(); loadStates();}}>
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="{mdiRefresh}" />
          </svg>
        </button>
      </div>
      
      <!-- Second row: Per page dropdown and Add button -->
      <div class="flex">
        <div class="flex form-control min-w-max">
          <select class="select select-bordered" bind:value={rowsPerPage} on:change={() => {currentPage = 1; loadStates()}}>
            <option value={10}>10 per page</option>
            <option value={25}>25 per page</option>
            <option value={50}>50 per page</option>
            <option value={100}>100 per page</option>
          </select>
        </div>
        <div class="flex flex-grow flex-wrap justify-end gap-2">
            <button class="btn btn-error" on:click={handleResetState}>
              <svg class="h-5 w-5 mr-1" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiTrashCan} />
              </svg>
              Reset state table
            </button>
            <button class="btn btn-error" on:click={handleResetSource}>
              <svg class="h-5 w-5 mr-1" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiTrashCan} />
              </svg>
              Reset source tracking
            </button>
        </div>
          <!-- <div class="flex-wrap justify-self-end" style="border: 1px solid blue">
            <button class="justify-self-end btn btn-primary m-2" on:click={openAddModal}>
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 5v14M5 12h14"></path>
              </svg>
              Reset state table
            </button>
            <button class="justify-self-end btn btn-primary m-2" on:click={openAddModal}>
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 5v14M5 12h14"></path>
              </svg>
              Reset source tracking
            </button>
          </div> -->
      </div>
    </div>
    
    <!-- Loading state -->
    {#if loading}
      <div class="flex justify-center items-center py-12">
        <div class="loading loading-spinner loading-lg"></div>
      </div>
    
    <!-- Error states -->
    {:else if error}
      <div class="alert {error.includes('404') ? 'alert-warning' : error.includes('403') ? 'alert-error' : 'alert-error'} shadow-lg mb-4">
        <div>
          <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          </svg>
          {#if error.includes('404')}
            <div>
              <h3 class="font-bold">Feature Not Available</h3>
              <div class="text-sm">Tunables management requires OPNsense 25.x or newer. Your firewall appears to be running an older version.</div>
            </div>
          {:else if error.includes('403')}
            <div>
              <h3 class="font-bold">Permission Denied</h3>
              <div class="text-sm">Your API credentials don't have sufficient permissions to manage states. Please check your API key permissions in OPNsense.</div>
            </div>
          {:else}
            <div>
              <h3 class="font-bold">Error Loading Tunables</h3>
              <div class="text-sm">{error}</div>
            </div>
          {/if}
        </div>
      </div>
    
    <!-- Tunables table for desktop view -->
    {:else if states.length > 0}
      <!-- Batch actions toolbar (shown when items are selected) -->
      
      <div class="bg-base-200 p-2 rounded-md mb-3 flex items-center justify-between">
        <span class="font-semibold">{total} item{total > 1 ? 's' : ''}{#if appliedSearchPhrase !== ''}&nbsp;matched{/if}</span>
        {#if appliedSearchPhrase !== ''}
          <div class="flex gap-2">
            <button 
              class="btn btn-sm btn-error" 
              on:click={handleBatchDelete}
            >
              Kill all matched states
            </button>
          </div>
        {/if}
      </div>

      <!-- Desktop table view (hidden on mobile) -->
      <div class="hidden md:block overflow-x-auto bg-base-100 rounded-lg shadow">
        <table class="table table-zebra w-full">
          <thead>
            <tr>
              <th>Tunable Name</th>
              <th>Type</th>
              <th>Value</th>
              <th>Default</th>
              <th class="hidden lg:table-cell">Description</th>
              <th class="w-24">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each states as state (state.id)}
              <tr>
                <td class="font-mono text-xs md:text-sm max-w-[200px] truncate">
                  <span title={state.state}>{state.state}</span>
                </td>
                <td>runtime</td>
                <td class="font-mono text-xs md:text-sm max-w-[120px] truncate">
                  <span title={state.value}>{state.value}</span>
                </td>
                <td class="font-mono text-xs md:text-sm max-w-[120px] truncate">
                  <span title={state.default_value || state.value}>{state.default_value || state.value}</span>
                </td>
                <td class="hidden lg:table-cell text-xs md:text-sm max-w-[200px] truncate">
                  <span title={state.descr || '-'}>{state.descr || '-'}</span>
                </td>
                <td>
                  <div class="flex gap-1">
                    <button
                      class="btn btn-sm btn-ghost btn-square"
                      title="Delete"
                      on:click={() => openDeleteModal(state.id)}
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M3 6h18"></path>
                        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                      </svg>
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      
      <!-- Mobile card view (shown on small screens) -->
      <div class="md:hidden space-y-4">
        {#each states as state (state.id)}
          <div class="card bg-base-100 shadow-md hover:shadow-lg transition-shadow">
            <div class="card-body p-4">
              <div class="flex flex-col">
                <div class="contents font-mono text-xs font-medium truncate mb-2">
                  <p class="contents text-purple-500">SRC:</p> {state.src_addr}:{state.src_port}
                </div>
                <div class="contents font-mono text-xs font-medium truncate mb-2">
                  <p class="contents text-purple-500">DST:</p> {state.dst_addr}:{state.dst_port}
                </div>
                {#if state.nat_addr}
                  <div class="contents font-mono text-xs font-medium truncate mb-2">
                    <p class="contents text-purple-500">NAT:</p> {state.nat_addr}:{state.nat_port}
                  </div>
                {/if}
                
                <div class="divider my-1"></div>
                
                <div class="flex flex-col gap-2 text-sm">
                  <div>
                    <div class="opacity-70 text-xs">State</div>
                    <div class="break-all text-sm">{state.state}</div>
                  </div>
                  <div>
                    <div class="opacity-70 text-xs">Firewall rule</div>
                    <div class="break-all text-sm">{state.descr}</div>
                  </div>
                  <div>
                    <div class="opacity-70 text-xs">Direction</div>
                    <div class="break-all text-sm">{state.direction}</div>
                  </div>
                  <div>
                    <div class="opacity-70 text-xs">Interface</div>
                    <div class="break-all text-sm">{state.interface}</div>
                  </div>
                  <div>
                    <div class="opacity-70 text-xs">Proto</div>
                    <div class="break-all text-sm">{state.proto}</div>
                  </div>
                  {#if state.Gateway}
                    <div>
                      <div class="opacity-70 text-xs">Gateway</div>
                      <div class="break-all text-sm">{state.Gateway}</div>
                    </div>
                  {/if}
                </div>
                
                <div class="card-actions justify-end mt-3">
                  <button 
                    class="btn btn-sm btn-error" 
                    on:click={() => openDeleteModal(state.id)}
                  >
                    Delete
                  </button>
                </div>
              </div>
            </div>
          </div>
        {/each}
      </div>
      
      <!-- Pagination and Record Count -->
      {#if total > 0}
        <div class="flex flex-col sm:flex-row justify-between items-center mt-6 gap-3">
          <div class="text-sm text-base-content/70">
            Showing {(currentPage - 1) * rowsPerPage + 1} to {Math.min(currentPage * rowsPerPage, total)} of {total} entries
          </div>
          
          {#if total > rowsPerPage}
            <div class="btn-group">
              <button 
                class="btn btn-sm" 
                disabled={currentPage === 1}
                on:click={() => handlePageChange(1)}
              >
                «
              </button>
              <button 
                class="btn btn-sm" 
                disabled={currentPage === 1}
                on:click={() => handlePageChange(currentPage - 1)}
              >
                ‹
              </button>
              
              {#each Array(Math.ceil(total / rowsPerPage)) as _, i}
                {#if i + 1 === currentPage || i + 1 === 1 || i + 1 === Math.ceil(total / rowsPerPage) || (i + 1 >= currentPage - 1 && i + 1 <= currentPage + 1)}
                  <button 
                    class="btn btn-sm" 
                    class:btn-active={i + 1 === currentPage}
                    on:click={() => handlePageChange(i + 1)}
                  >
                    {i + 1}
                  </button>
                {:else if i + 1 === currentPage - 2 || i + 1 === currentPage + 2}
                  <button class="btn btn-sm btn-disabled">...</button>
                {/if}
              {/each}
              
              <button 
                class="btn btn-sm" 
                disabled={currentPage === Math.ceil(total / rowsPerPage)}
                on:click={() => handlePageChange(currentPage + 1)}
              >
                ›
              </button>
              <button 
                class="btn btn-sm" 
                disabled={currentPage === Math.ceil(total / rowsPerPage)}
                on:click={() => handlePageChange(Math.ceil(total / rowsPerPage))}
              >
                »
              </button>
            </div>
          {/if}
        </div>
      {/if}
    
    <!-- Empty state -->
    {:else}
      <div class="text-center py-12">
        <p class="text-lg mb-2">No states found</p>
        {#if deviceChoice.value}
          <p class="text-sm opacity-75">Try adjusting your search criteria</p>
        {/if}
      </div>
    {/if}
  </div>
</AppLayout>