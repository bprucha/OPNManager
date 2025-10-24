<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import {
        mdiFingerprint,
    } from '@mdi/js';

    let dialogRef: HTMLDialogElement;
    let secret = '';
    let title = '';
    let message = '';
    let isVisible = false;
    let icon = null as (string | null);

    const dispatch = createEventDispatcher();

    function onCancel() {
        dialogRef.close('cancel');
        isVisible = false;
        dispatch('cancel');
    }

    function onConfirm() {
        dialogRef.close('confirm');
        isVisible = false;
        dispatch('confirm');
    }

    /**
     * @param {string} newTitle
     * @param {string} newMessage
     * @return {Promise<string?>}
     */
    export function show(newTitle: string, newMessage: string, newIcon: string) {
        secret = ''
        return new Promise((resolve) => {
            title = newTitle;
            message = newMessage;
            isVisible = true;
            icon = newIcon;
            dialogRef.showModal();

            const handleClose = () => {
                if(dialogRef.returnValue === 'confirm') {
                    resolve(true);
                } else {
                    resolve(false);
                }
                // Clean up the event listener
                dialogRef.removeEventListener('close', handleClose);
            };
            dialogRef.addEventListener('close', handleClose);
        });
    }
</script>


<el-dialog>
  <dialog bind:this={dialogRef} id="dialog" aria-labelledby="dialog-title" class="fixed inset-0 size-auto max-h-none max-w-none overflow-y-auto bg-transparent backdrop:bg-transparent">
    <el-dialog-backdrop class="fixed inset-0 bg-gray-500/75 transition-opacity data-closed:opacity-0 data-enter:duration-300 data-enter:ease-out data-leave:duration-200 data-leave:ease-in"></el-dialog-backdrop>
    <div class="flex min-h-full items-center justify-center p-4 text-center focus:outline-none sm:items-center sm:p-0">
      <el-dialog-panel class="relative transform overflow-hidden rounded-lg bg-base-100 text-left shadow-xl transition-all data-closed:translate-y-4 data-closed:opacity-0 data-enter:duration-300 data-enter:ease-out data-leave:duration-200 data-leave:ease-in sm:my-8 sm:w-full sm:max-w-lg data-closed:sm:translate-y-0 data-closed:sm:scale-95">
        <div class="bg-base-100 px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
          <div class="sm:flex sm:items-start">
            <!-- {#if icon}
              <div class="mx-auto flex size-12 shrink-0 items-center justify-center sm:mx-0 sm:size-10">icon
                  <svg viewBox="0 0 24 24" style="border: 1px solid red;">
                      <path fill="currentColor" d={icon} />
                  </svg>
              </div>
            {/if} -->
            <div class="flex mt-3 text-left sm:mt-0 sm:ml-4 sm:text-left">
              <div class="mt-2">
                <h3 id="dialog-title" class="text-center text-base font-semibold">{title}</h3>
                <span class="label-text">{message}</span>
              </div>
            </div>
          </div>
        </div>
        <div class="bg-base-100 px-4 py-3 flex flex-row-reverse px-6">
            <button class="inline-flex w-auto btn btn-primary px-3 py-2 ml-3" on:click={onConfirm}>Confirm</button>
            <button class="inline-flex w-auto btn btn-secondary px-3 py-2" on:click={onCancel}>Cancel</button>
        </div>
      </el-dialog-panel>
    </div>
  </dialog>
</el-dialog>
