use std::{
    fs,
    path::{Path, PathBuf},
};

const SW_SCHEMA_VERSION: &str = "sw-v1";
pub const FRONTEND_VERSION: &str = env!("FRONTEND_CARGO_VERSION");

pub struct PwaAssets {
    pub version: String,
    pub service_worker_js: String,
}

struct Fnv1a128 {
    state: u128,
}

impl Fnv1a128 {
    fn new() -> Self {
        Self {
            state: 0x6c62272e07bb014262b821756295c58d,
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        const PRIME: u128 = 0x0000000001000000000000000000013b;
        for &byte in bytes {
            self.state ^= byte as u128;
            self.state = self.state.wrapping_mul(PRIME);
        }
    }

    fn update_file(&mut self, label: &str, path: &Path) {
        self.update(label.as_bytes());
        self.update(&[0]);
        if let Ok(bytes) = fs::read(path) {
            self.update(&bytes);
        }
        self.update(&[0xff]);
    }

    fn finish_hex(self) -> String {
        format!("{:032x}", self.state)
    }
}

fn sorted_files_recursive(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                paths.push(path);
            }
        }
    }
    paths.sort();
    paths
}

fn sorted_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file())
        .collect();
    paths.sort();
    paths
}

pub fn build_pwa_assets(
    frontend_dist: &Path,
    style_dir: &Path,
    catalog_root: &Path,
    trainer_ids: &[String],
    version_override: Option<&str>,
) -> PwaAssets {
    let version_tag = version_override
        .filter(|v| !v.is_empty())
        .unwrap_or(FRONTEND_VERSION);
    let mut hasher = Fnv1a128::new();
    hasher.update(SW_SCHEMA_VERSION.as_bytes());
    hasher.update(b"|version:");
    hasher.update(version_tag.as_bytes());

    let mut precache_urls = vec![
        "/".to_owned(),
        "/manifest.json".to_owned(),
        "/icon.png".to_owned(),
        "/mobile.png".to_owned(),
        "/api/trainers".to_owned(),
    ];

    hasher.update_file("manifest.json", Path::new("manifest.json"));
    hasher.update_file("icon.png", Path::new("icon.png"));
    hasher.update_file("mobile.png", Path::new("mobile.png"));

    for path in sorted_files_recursive(frontend_dist) {
        let Ok(relative) = path.strip_prefix(frontend_dist) else {
            continue;
        };
        if relative.components().any(|component| {
            component
                .as_os_str()
                .to_str()
                .is_some_and(|segment| segment.starts_with('.'))
        }) {
            continue;
        }
        let rel_url = relative
            .components()
            .filter_map(|component| component.as_os_str().to_str())
            .collect::<Vec<_>>()
            .join("/");
        if rel_url.is_empty() {
            continue;
        }
        hasher.update_file(&format!("dist/{rel_url}"), &path);
        if rel_url != "index.html" {
            precache_urls.push(format!("/{rel_url}"));
        }
    }

    for path in sorted_files(style_dir) {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".css") {
            continue;
        }
        hasher.update_file(&format!("style/{name}"), &path);
        precache_urls.push(format!("/style/{name}"));
    }

    hasher.update_file("static/index.json", &catalog_root.join("index.json"));
    let mut sorted_trainers = trainer_ids.to_vec();
    sorted_trainers.sort();
    for trainer_id in &sorted_trainers {
        precache_urls.push(format!("/catalog/{trainer_id}/icon.png"));
        let trainer_dir = catalog_root.join(trainer_id);
        for path in sorted_files(&trainer_dir) {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                hasher.update_file(&format!("static/{trainer_id}/{name}"), &path);
            }
        }
    }

    let content_hash = hasher.finish_hex();
    let version = format!("{version_tag}-{}", &content_hash[..12]);

    let precache_json = serde_json::to_string(&precache_urls).unwrap_or_else(|_| "[]".to_owned());
    let version_json = serde_json::to_string(&version).unwrap_or_else(|_| "\"v1\"".to_owned());

    let service_worker_js = render_service_worker(&version_json, &precache_json);
    PwaAssets {
        version,
        service_worker_js,
    }
}

fn render_service_worker(version_json: &str, precache_json: &str) -> String {
    format!(
        r#"const CACHE_VERSION = {version_json};
const CACHE_NAME = "alg-trainers-v-" + CACHE_VERSION;
const PRECACHE_URLS = {precache_json};
const OFFLINE_TRAINERS_URL = "/api/offline/trainers";
const FONT_STYLESHEET_URLS = [
  "https://fonts.googleapis.com/css2?family=Titillium+Web:ital,wght@0,200;0,300;0,400;0,600;0,700;0,900;1,200;1,300;1,400;1,600;1,700&display=swap",
  "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200"
];

async function cacheUrlsInBatches(cache, urls, batchSize) {{
  for (let i = 0; i < urls.length; i += batchSize) {{
    const batch = urls.slice(i, i + batchSize);
    await Promise.all(batch.map(async (url) => {{
      const response = await fetch(url, {{ cache: "reload" }});
      if (!response.ok) {{
        throw new Error("Failed to precache " + url + ": " + response.status);
      }}
      await cache.put(url, response);
    }}));
  }}
}}

async function cacheAllTrainerBundles(cache) {{
  const response = await fetch(OFFLINE_TRAINERS_URL, {{ cache: "reload" }});
  if (!response.ok) {{
    throw new Error("Failed to fetch offline trainers bundle: " + response.status);
  }}
  const bundles = await response.json();
  if (!Array.isArray(bundles)) {{
    throw new Error("Invalid offline trainers bundle payload");
  }}
  const batchSize = 8;
  for (let i = 0; i < bundles.length; i += batchSize) {{
    const batch = bundles.slice(i, i + batchSize);
    await Promise.all(batch.map(async (bundle) => {{
      if (!bundle || typeof bundle.trainerId !== "string") return;
      const trainerUrl = "/api/trainers/" + encodeURIComponent(bundle.trainerId);
      const body = JSON.stringify(bundle);
      const cachedResponse = new Response(body, {{
        status: 200,
        headers: {{
          "Content-Type": "application/json",
          "Cache-Control": "public, max-age=31536000"
        }}
      }});
      await cache.put(trainerUrl, cachedResponse);
    }}));
  }}
}}

async function precacheFontsBestEffort(cache) {{
  try {{
    const fontUrls = new Set();
    for (const cssUrl of FONT_STYLESHEET_URLS) {{
      const response = await fetch(cssUrl, {{ mode: "cors", cache: "reload" }});
      if (!response.ok) continue;
      const cssText = await response.clone().text();
      await cache.put(cssUrl, response);
      const matches = cssText.matchAll(/url\((https:\/\/fonts\.gstatic\.com\/[^)]+)\)/g);
      for (const match of matches) {{
        fontUrls.add(match[1].replace(/^['"]|['"]$/g, ""));
      }}
    }}
    const list = Array.from(fontUrls);
    for (let i = 0; i < list.length; i += 8) {{
      await Promise.all(list.slice(i, i + 8).map(async (fontUrl) => {{
        try {{
          const fontRes = await fetch(fontUrl, {{ mode: "cors" }});
          if (fontRes.ok) {{
            await cache.put(fontUrl, fontRes);
          }}
        }} catch (_) {{}}
      }}));
    }}
  }} catch (_) {{
    // Font precaching is best-effort so offline/restricted networks never block install.
  }}
}}

self.addEventListener("install", (event) => {{
  // Intentionally do NOT call self.skipWaiting() so updates require explicit user approval.
  event.waitUntil((async () => {{
    const cache = await caches.open(CACHE_NAME);
    await cacheUrlsInBatches(cache, PRECACHE_URLS, 8);
    await cacheAllTrainerBundles(cache);
    await precacheFontsBestEffort(cache);
  }})());
}});

self.addEventListener("activate", (event) => {{
  event.waitUntil((async () => {{
    const keys = await caches.keys();
    await Promise.all(
      keys
        .filter((key) => key !== CACHE_NAME)
        .map((key) => caches.delete(key))
    );
    await self.clients.claim();
  }})());
}});

self.addEventListener("message", (event) => {{
  const data = event.data;
  if (!data) return;
  if (data.type === "SKIP_WAITING") {{
    self.skipWaiting();
  }} else if (data.type === "GET_VERSION" && event.ports && event.ports[0]) {{
    event.ports[0].postMessage({{ version: CACHE_VERSION, cacheName: CACHE_NAME }});
  }}
}});

function isNetworkOnlyPath(pathname) {{
  return (
    pathname === "/sw.js" ||
    pathname === "/api/me" ||
    pathname === "/api/sync" ||
    pathname.startsWith("/api/auth/") ||
    pathname === OFFLINE_TRAINERS_URL
  );
}}

self.addEventListener("fetch", (event) => {{
  const {{ request }} = event;
  if (request.method !== "GET") return;

  const url = new URL(request.url);

  if (url.origin === self.location.origin) {{
    if (isNetworkOnlyPath(url.pathname)) {{
      return;
    }}

    if (request.mode === "navigate") {{
      event.respondWith((async () => {{
        const cache = await caches.open(CACHE_NAME);
        const cachedShell = await cache.match("/");
        if (cachedShell) return cachedShell;
        return fetch(request);
      }})());
      return;
    }}

    // Cache-first (offline cache preferred) for all app assets, icons, and trainer data.
    event.respondWith((async () => {{
      const cache = await caches.open(CACHE_NAME);
      const cached = await cache.match(request) || (url.search === "" ? await cache.match(url.pathname) : null);
      if (cached) return cached;
      const response = await fetch(request);
      if (response.ok) {{
        cache.put(request, response.clone()).catch(() => {{}});
      }}
      return response;
    }})());
    return;
  }}

  if (url.hostname === "fonts.googleapis.com" || url.hostname === "fonts.gstatic.com") {{
    event.respondWith((async () => {{
      const cache = await caches.open(CACHE_NAME);
      const cached = await cache.match(request);
      if (cached) return cached;
      const response = await fetch(request);
      if (response.ok || response.type === "opaque") {{
        cache.put(request, response.clone()).catch(() => {{}});
      }}
      return response;
    }})());
  }}
}});
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwa_version_changes_with_override_and_includes_trainer_icons() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let dist = root.join("frontend/dist");
        let style = root.join("style");
        let static_dir = root.join("static");
        let trainers = vec!["3x3-PLL-Trainer".to_owned(), "2x2-EG-Trainer".to_owned()];

        let default_v = build_pwa_assets(&dist, &style, &static_dir, &trainers, None);
        let v1 = build_pwa_assets(&dist, &style, &static_dir, &trainers, Some("v1"));
        let v2 = build_pwa_assets(&dist, &style, &static_dir, &trainers, Some("v2"));

        assert!(default_v.version.starts_with(&format!("{FRONTEND_VERSION}-")));
        assert_ne!(v1.version, v2.version);
        assert!(v1.service_worker_js.contains("/catalog/3x3-PLL-Trainer/icon.png"));
        assert!(v1.service_worker_js.contains("/catalog/2x2-EG-Trainer/icon.png"));
        assert!(v1.service_worker_js.contains("/manifest.json"));
        assert!(v1.service_worker_js.contains("/api/offline/trainers"));
        assert!(!v1.service_worker_js.contains("self.skipWaiting();\n});"));
    }
}
