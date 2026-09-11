---
topic: frontend-stack
version: vite 8.3.0 / react 19.3.0 / wagmi 3.7.7 / viem 2.56.3 / tailwindcss 4.3.3 / @reown/appkit 1.8.23
checked: 2026-09-11
sources:
  - https://registry.npmjs.org/<pkg>/latest (each version below read from the registry 2026-09-11)
  - https://docs.reown.com/appkit/react/core/installation.md
---

- Current npm latest (2026-09-11): vite 8.3.0, @vitejs/plugin-react 6.1.1, react 19.3.0,
  wagmi 3.7.7, @wagmi/core 3.6.5, viem 2.56.3, tailwindcss 4.3.3, @tailwindcss/vite 4.3.3,
  @reown/appkit 1.8.23, @reown/appkit-adapter-wagmi 1.8.23. If you are about to write a
  wagmi 2.x / Vite 5 / Tailwind 3 config from memory, all three majors have moved.
- THE TRAP: AppKit (Reown) docs state verbatim "AppKit is only compatible with wagmi 2.x.
  Ensure you are using a compatible version." `npm i wagmi` today installs 3.7.7, which
  AppKit 1.8.23 does NOT support. Two valid paths — pick one in scaffolding, don't mix:
  1) AppKit modal → pin `wagmi@2.19.5` (latest stable 2.x) + @reown/appkit +
     @reown/appkit-adapter-wagmi; 2) wagmi 3.7.7 → use its built-in connectors
     (wagmi/connectors incl. WalletConnect) and skip AppKit entirely.
- Web3Modal is dead branding: @web3modal/wagmi 5.1.11 is npm-DEPRECATED — "Web3Modal is
  now Reown AppKit" (upgrade guide docs.reown.com/appkit/upgrade-guide). Never generate
  @web3modal imports.
- AppKit wiring per docs.reown.com/appkit/react/core/installation: WagmiAdapter +
  `createAppKit({adapters, networks, projectId, metadata})`, then WagmiProvider +
  QueryClientProvider (react-query) wrap the app; docs site has llms.txt + `.md` per page.
- Tailwind v4 Vite integration: use the `@tailwindcss/vite` plugin (versioned in lockstep
  with tailwindcss, both 4.3.3) and CSS-first `@import "tailwindcss";` — the v3-era
  tailwind.config.js + PostCSS pipeline is NOT the current Vite path. Don't emit a
  postcss.config for Tailwind.
- Scaffold: `npm create vite@latest` — Reown docs explicitly warn create-react-app is
  deprecated and causes dependency issues.
- viem for all reads (public clients on 4663 + 421614, see robinhood-chain.md /
  arbitrum-sepolia.md for verified RPCs); wallet connect deferred to the swap beat only.
