var assets = [
  "/Alg-Trainers",
  "/Alg-Trainers/index.html",
  "/Alg-Trainers/template.html",
  "/Alg-Trainers/icon.png",
  "/Alg-Trainers/style/hint.css",
  "/Alg-Trainers/style/main.css",
  "/Alg-Trainers/style/selection.css",
  "/Alg-Trainers/style/settings.css",
  "/Alg-Trainers/style/timer.css",
  "/Alg-Trainers/src/main.js",
  "/Alg-Trainers/src/practice.js",
  "/Alg-Trainers/src/selection.js",
  "/Alg-Trainers/src/settings.js",
  "/Alg-Trainers/src/timedetails.js",
  "/Alg-Trainers/src/timer.js",
  "/Alg-Trainers/src/utils.js",
  '/Alg-Trainers/error.html',
  "https://fonts.googleapis.com/css2?family=Titillium+Web:ital,wght@0,200;0,300;0,400;0,600;0,700;0,900;1,200;1,300;1,400;1,600;1,700&display=swap",
  "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200",
  "https://colorjs.io/dist/color.global.js",
  "https://cdn.jsdelivr.net/npm/chart.js"
];
const trainerCache = "alg-trainer-cache-1.5.5";

function refreshCache() {
  caches.keys().then((keys) => {
    for (key of keys) {
      if (key != trainerCache) {
        caches.delete(key);
      }
    }
    caches.open(trainerCache).then((cache) => {
      cache.addAll(assets);
    })
  })
}

self.addEventListener('fetch', (event) => {
  event.respondWith(caches.open(trainerCache).then((cache) => {
    // Go to the cache first
    return cache.match(event.request.url).then((cachedResponse) => {
      // Return a cached response if we have one
      if (cachedResponse) {
        return cachedResponse;
      }

      // Otherwise, hit the network
      return fetch(event.request).then((fetchedResponse) => {
        // Add the network response to the cache for later visits
        cache.put(event.request, fetchedResponse.clone());

        // Return the network response
        return fetchedResponse;
      });
    });
  }));
});

var algsInTrainers = {
  '2x2-EG-Trainer': 160,
  '2x2-FH-Trainer': 172,
  '2x2-LS-Trainer': 486,
  '2x2-TCLL-Trainer': 86,
  '2x2-TEG-Trainer': 516,
  '3x3-BLD-UFR-Trainer': 378,
  '3x3-BLD-UF-Trainer': 440,
  '3x3-CMLL-Trainer': 42,
  '3x3-OH-PLL-Trainer': 21,
  '3x3-OH-CMLL-Trainer': 42,
  '3x3-OH-OLL-Trainer': 57,
  '3x3-OH-PLL-Trainer': 21,
  '3x3-OH-ZBLL-Trainer': 472,
  '3x3-OLL-Trainer': 57,
  '3x3-PLL-Trainer': 21,
  '3x3-ZBLL-Trainer': 472,
  '3x3-ZBLS-Trainer': 302,
  '5x5-L2E-Trainer': 15,
  'Megaminx-OLL-Trainer': 259, 
  'Megaminx-PLL-Trainer': 151, 
  'Octaminx-TCP-Trainer': 18,
  'Octaminx-L3T-Trainer': 12,
  'Pyraminx-L4E-Trainer': 35, 
  'Skewb-NS2-Trainer': 267,
  'Sq1-CPEP-Trainer': 57,
  'Sq1-PBL-Trainer': 967,
};

for (const [trainer, algs] of Object.entries(algsInTrainers)) {
  assets.push(`/Alg-Trainers/${trainer}/algs_info.json`);
  assets.push(`/Alg-Trainers/${trainer}/algsets_info.json`);
  assets.push(`/Alg-Trainers/${trainer}/algsinfo.js`);
  assets.push(`/Alg-Trainers/${trainer}/combined.json`);
  assets.push(`/Alg-Trainers/${trainer}/groups_info.json`);
  assets.push(`/Alg-Trainers/${trainer}/index.html`);
  assets.push(`/Alg-Trainers/${trainer}/icon.png`);
  assets.push(`/Alg-Trainers/${trainer}/scrambles.json`);
  assets.push(`/Alg-Trainers/${trainer}/selected_algsets.json`);
}

self.addEventListener("install", installEvent => {
    console.log('Install-event');
    installEvent.waitUntil(
        refreshCache()
    )
})
