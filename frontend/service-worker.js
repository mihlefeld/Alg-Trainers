const SHELL_PREFIX = "shell";
const CATALOG_PREFIX = "catalog";
const RUNTIME_PREFIX = "runtime";
const CACHE_LIMIT = 80;

async function versionInfo() {
  try {
    const response = await fetch("/version.json", { cache: "no-store" });
    if (response.ok) return await response.json();
  } catch {}
  return { version: "dev", catalogVersion: "dev" };
}

self.addEventListener("install", event => {
  event.waitUntil((async () => {
    const info = await versionInfo();
    const manifest = await fetch("/offline-manifest.json", { cache: "no-store" }).then(r => r.json());
    const shell = await caches.open(`${SHELL_PREFIX}-${info.version}`);
    const catalog = await caches.open(`${CATALOG_PREFIX}-${info.catalogVersion}`);
    await Promise.all(manifest.assets.map(async entry => {
      const response = await fetch(entry.url, { cache: "no-store" });
      if (!response.ok) throw new Error(`Unable to cache ${entry.url}`);
      if (entry.url.startsWith("/api/trainers")) {
        await catalog.put(entry.url, response);
      } else {
        await shell.put(entry.url, response);
        if (entry.url === "/" || entry.url === "/index.html") {
          await catalog.put(entry.url, response.clone());
        }
      }
    }));
    await self.skipWaiting();
  })());
});

self.addEventListener("activate", event => {
  event.waitUntil((async () => {
    await self.clients.claim();
    const info = await versionInfo();
    const names = await caches.keys();
    await Promise.all(names.map(async name => {
      const belongsToApp = name.startsWith(`${SHELL_PREFIX}-`) ||
        name.startsWith(`${CATALOG_PREFIX}-`) ||
        name.startsWith(`${RUNTIME_PREFIX}-`);
      const active = name === `${SHELL_PREFIX}-${info.version}` ||
        name === `${CATALOG_PREFIX}-${info.catalogVersion}` ||
        name === `${RUNTIME_PREFIX}-${info.version}`;
      if (belongsToApp && !active) {
        await caches.delete(name);
      }
    }));
  })());
});

self.addEventListener("fetch", event => {
  const request = event.request;
  const url = new URL(request.url);
  if (request.method !== "GET" || url.pathname.startsWith("/api/auth") ||
      url.pathname === "/api/me" || url.pathname === "/api/sync" ||
      url.pathname === "/version.json" || url.pathname === "/offline-manifest.json") {
    return;
  }
  if (request.mode === "navigate") {
    event.respondWith((async () => {
      try {
        return await fetch(request);
      } catch {
        for (const name of await caches.keys()) {
          if (!name.startsWith(`${SHELL_PREFIX}-`)) continue;
          const shell = await caches.open(name);
          const root = await shell.match("/");
          if (root) return root;
          const index = await shell.match("/index.html");
          if (index) return index;
        }
        return new Response("The app is not available offline yet.", {
          status: 503,
          headers: { "Content-Type": "text/plain" },
        });
      }
    })());
    return;
  }
  if (url.pathname.startsWith("/api/trainers")) {
    event.respondWith((async () => {
      const cache = await caches.open(`${CATALOG_PREFIX}-${(await versionInfo()).catalogVersion}`);
      const cached = await cache.match(request);
      if (cached) return cached;
      try {
        const response = await fetch(request);
        if (response.ok) await cache.put(request, response.clone());
        return response;
      } catch {
        return cached || new Response("Offline catalog unavailable", { status: 503 });
      }
    })());
    return;
  }
  event.respondWith((async () => {
    const cached = await caches.match(request);
    if (cached) return cached;
    try {
      const response = await fetch(request);
      if (response.ok) {
        const cache = await caches.open(`${RUNTIME_PREFIX}-${(await versionInfo()).version}`);
        await cache.put(request, response.clone());
        const keys = await cache.keys();
        if (keys.length > CACHE_LIMIT) await cache.delete(keys[0]);
      }
      return response;
    } catch {
      return new Response("Offline", { status: 503 });
    }
  })());
});

self.addEventListener("message", event => {
  if (event.data?.type === "SKIP_WAITING") self.skipWaiting();
});
