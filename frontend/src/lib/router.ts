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

export function parseHash(hash: string): Route {
  const raw = hash.replace(/^#/, "");
  const [pathPart, search = ""] = raw.split("?");
  const path = pathPart === "" ? "/" : pathPart;
  return { path, query: new URLSearchParams(search) };
}

function currentRoute(): Route {
  return parseHash(window.location.hash);
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
