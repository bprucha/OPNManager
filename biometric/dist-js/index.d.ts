export declare enum AuthMode {
    PROMPT = 0,
    ENCRYPT = 1,
    DECRYPT = 2
}
export declare enum BiometryType {
    None = 0,
    TouchID = 1,
    FaceID = 2,
    Iris = 3
}
export interface Status {
    isAvailable: boolean;
    biometryType: BiometryType;
    error?: string;
    errorCode?: 'appCancel' | 'authenticationFailed' | 'invalidContext' | 'notInteractive' | 'passcodeNotSet' | 'systemCancel' | 'userCancel' | 'userFallback' | 'biometryLockout' | 'biometryNotAvailable' | 'biometryNotEnrolled';
}
interface EncryptCipherData {
    data: string;
}
interface DecryptCipherData {
    data: string;
    initializationVector: string;
}
export type CipherData = EncryptCipherData | DecryptCipherData;
interface BaseAuthOptions {
    allowDeviceCredential?: boolean;
    cancelTitle?: string;
    fallbackTitle?: string;
    title?: string;
    subtitle?: string;
    confirmationRequired?: boolean;
    maxAttemps?: number;
}
interface PromptAuthOptions extends BaseAuthOptions {
    mode: AuthMode.PROMPT;
}
interface EncryptAuthOptions extends BaseAuthOptions {
    mode: AuthMode.ENCRYPT;
    cipherKey: string;
    cipherData: EncryptCipherData;
}
interface DecryptAuthOptions extends BaseAuthOptions {
    mode: AuthMode.DECRYPT;
    cipherKey: string;
    cipherData: DecryptCipherData;
}
export type AuthOptions = BaseAuthOptions | PromptAuthOptions | EncryptAuthOptions | DecryptAuthOptions;
/**
 * Checks if the biometric authentication is available.
 * @returns a promise resolving to an object containing all the information about the status of the biometry.
 */
export declare function checkStatus(): Promise<Status>;
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
export declare function authenticate(reason: string, options?: AuthOptions): Promise<CipherData>;
export {};
