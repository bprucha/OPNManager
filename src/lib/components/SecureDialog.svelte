<script>
    import { createEventDispatcher } from 'svelte';
    import {
        mdiFingerprint,
    } from '@mdi/js';

    /** @type {HTMLDialogElement} */
    let dialogRef;
    let secret = '';
    let title = '';
    let message = '';
    let isVisible = false;

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
    export function show(newTitle, newMessage) {
        secret = ''
        return new Promise((resolve) => {
            title = newTitle;
            message = newMessage;
            isVisible = true;
            dialogRef.showModal();

            const handleClose = () => {
                if(dialogRef.returnValue === 'confirm') {
                    resolve(secret);
                } else {
                    resolve(null);
                }
                // const result = dialogRef.returnValue;
                // resolve(result);
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
    <div tabindex="0" class="flex min-h-full items-center justify-center p-4 text-center focus:outline-none sm:items-center sm:p-0">
      <el-dialog-panel class="relative transform overflow-hidden rounded-lg bg-base-100 text-left shadow-xl transition-all data-closed:translate-y-4 data-closed:opacity-0 data-enter:duration-300 data-enter:ease-out data-leave:duration-200 data-leave:ease-in sm:my-8 sm:w-full sm:max-w-lg data-closed:sm:translate-y-0 data-closed:sm:scale-95">
        <div class="bg-base-100 px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
          <div class="sm:flex sm:items-start">
            <div class="mx-auto flex size-12 shrink-0 items-center justify-center sm:mx-0 sm:size-10">
                <svg viewBox="0 0 24 24">
                    <path fill="currentColor" d={mdiFingerprint} />
                </svg>
            </div>
            <div class="mt-3 text-left sm:mt-0 sm:ml-4 sm:text-left">
              <h3 id="dialog-title" class="text-center text-base font-semibold">{title}</h3>
              <div class="mt-2">
                <div class="form-control">
                    <label class="label" for="secret">
                        <span class="label-text">{message}</span>
                    </label>
                    <input
                        id="secret"
                        bind:value={secret}
                        type="password"
                        placeholder="{message}"
                        class="input input-bordered w-full"
                        required
                    />
                </div>
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
