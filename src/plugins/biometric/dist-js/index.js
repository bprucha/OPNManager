import { invoke } from '@tauri-apps/api/core';

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
var AuthMode;
(function (AuthMode) {
    AuthMode[AuthMode["PROMPT"] = 0] = "PROMPT";
    AuthMode[AuthMode["ENCRYPT"] = 1] = "ENCRYPT";
    AuthMode[AuthMode["DECRYPT"] = 2] = "DECRYPT";
})(AuthMode || (AuthMode = {}));
var BiometryType;
(function (BiometryType) {
    BiometryType[BiometryType["None"] = 0] = "None";
    // Apple TouchID or Android fingerprint
    BiometryType[BiometryType["TouchID"] = 1] = "TouchID";
    // Apple FaceID or Android face authentication
    BiometryType[BiometryType["FaceID"] = 2] = "FaceID";
    // Android iris authentication
    BiometryType[BiometryType["Iris"] = 3] = "Iris";
})(BiometryType || (BiometryType = {}));
/**
 * Checks if the biometric authentication is available.
 * @returns a promise resolving to an object containing all the information about the status of the biometry.
 */
async function checkStatus() {
    return await invoke('plugin:biometric|status');
}
/**
 * Prompts the user for authentication using the system interface (touchID, faceID or Android Iris).
 * Rejects if the authentication fails.
 *
 * ```javascript
 * import { authenticate } from "@tauri-apps/plugin-biometric";
 * await authenticate('Open your wallet');
 * ```
 * @param reason
 * @param options
 * @returns a promise resolving to an object containing the encrypted or decrypted data if any
 */
async function authenticate(reason, options) {
    return await invoke('plugin:biometric|authenticate', {
        reason,
        ...options
    });
}

export { AuthMode, BiometryType, authenticate, checkStatus };
