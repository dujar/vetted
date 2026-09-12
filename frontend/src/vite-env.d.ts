/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** WalletConnect Cloud projectId — operator-provided; empty keeps the app read-only. */
  readonly VITE_WALLETCONNECT_PROJECT_ID?: string;
  /** "mock" (fixture-backed, default) | "live" (step-4 worker). Default keeps the app working with no backend. */
  readonly VITE_API_MODE?: string;
  /** Live scan-backend worker base URL, e.g. https://vetted-scan-backend.dujar-coding.workers.dev */
  readonly VITE_API_URL?: string;
  /** Deployed Canonical Registry address (steps 3/7 set this at deploy time). Empty = registry runs fixture-backed. */
  readonly VITE_REGISTRY_ADDRESS?: string;
  /** Deployed guarded-swap guard address. Empty = live execution unavailable. */
  readonly VITE_GUARD_ADDRESS?: string;
}
