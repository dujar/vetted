/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** WalletConnect Cloud projectId — operator-provided; empty keeps the app read-only. */
  readonly VITE_WALLETCONNECT_PROJECT_ID?: string;
}
