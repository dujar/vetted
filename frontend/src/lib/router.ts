/**
 * Hash router — three routes, no dependency (verify.md loose end 2 fix: the
 * scaffold has no router). Hash over history routing because the app deploys
 * as a static bundle to Cloudflare Pages, where `/swap` deep links would 404
 * without SPA-fallback config; `#/swap` works everywhere. Deep links per
 * journeys.md:9: `#/?addr=0x…` and `#/?addr=0xA…&addr=0xB…` (compare).
 */
import { useEffect, useState } from "react";

export interface Route {
  path: string;
  query: URLSearchParams;
}

/**
 * Query precedence: the hash's own query when present (`#/?addr=…`), else the
 * bare `location.search` — journeys.md:9's documented entry format
 * (`?addr=0x…`, compare `?addr=0xA…&addr=0xB…`) carries no hash at all and
 * must still land on the scan (review round 1, blocking fix).
 */
export function parseHash(hash: string, search = ""): Route {
  const raw = hash.replace(/^#/, "");
  const [pathPart, hashSearch] = raw.includes("?") ? raw.split("?") : [raw, undefined];
  const path = pathPart === "" ? "/" : pathPart;
  return { path, query: new URLSearchParams(hashSearch ?? search) };
}

function currentRoute(): Route {
  return parseHash(window.location.hash, window.location.search);
}

export function useHashRoute(): Route {
  const [route, setRoute] = useState<Route>(currentRoute);
  useEffect(() => {
    const onChange = () => setRoute(currentRoute());
    window.addEventListener("hashchange", onChange);
    return () => window.removeEventListener("hashchange", onChange);
  }, []);
  return route;
}

export function navigate(to: string): void {
  window.location.hash = to;
}
