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
  "/Alg-Trainers/src/saveload.js",
  "/Alg-Trainers/src/selection.js",
  "/Alg-Trainers/src/settings.js",
  "/Alg-Trainers/src/timer.js",
  "/Alg-Trainers/src/utils.js",
  '/Alg-Trainers/error.html',
  "https://fonts.googleapis.com/css2?family=Titillium+Web:ital,wght@0,200;0,300;0,400;0,600;0,700;0,900;1,200;1,300;1,400;1,600;1,700&display=swap",
  "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200",
  "https://colorjs.io/dist/color.global.js"
];
const trainerCache = "alg-trainer-cachev3";

const putInCache = async (request, response) => {
    const cache = await caches.open(trainerCache);
    await cache.put(request, response);
};

// const cacheFirst = async ({ request, fallbackUrl }) => {
//     console.log('trying to fetch ', request)
//     const responseFromCache = await caches.match(request);
//     if (responseFromCache) {
//         return responseFromCache;
//     }
//     console.log("cache didn't find ", request);
//     try {
//         const responseFromNetwork = await fetch(request);
//         putInCache(request, responseFromNetwork.clone());
//         return responseFromNetwork;
//     } catch (error) {
//         const fallbackResponse = await caches.match(fallbackUrl);
//         console.log('cacheFirst', fallbackUrl);
//         if (fallbackResponse) {
//             return fallbackResponse;
//         }
//         return new Response("Network error happened", {
//             status: 408,
//             headers: { "Content-Type": "text/plain" },
//         });
//     }
// };

self.addEventListener('fetch', (event) => {
    // Check if this is a request for an image
    if (event.request.destination === 'image') {
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
    } else {
      return;
    }
  });
  

// self.addEventListener("fetch", (event) => {
//     event.respondWith(cacheFirst({request: event.request, fallbackUrl: "/Alg-Trainers/error.html"}));
// });


var algsInTrainers = {
    'Megaminx-OLL-Trainer': 259, 
    'Megaminx-PLL-Trainer': 152, 
    'Pyraminx-L4E-Trainer': 35, 
    'Skewb-NS2-Trainer': 267
};

for (const [trainer, algs] of Object.entries(algsInTrainers)) {
    assets.push(`/Alg-Trainers/${trainer}/algsinfo.js`);
    assets.push(`/Alg-Trainers/${trainer}/scrambles.js`);
    assets.push(`/Alg-Trainers/${trainer}/icon.png`);
    assets.push(`/Alg-Trainers/${trainer}/index.html`);
    for (var i = 1; i <= algs; i++) {
        assets.push(`/Alg-Trainers/${trainer}/pic/${i}.svg`);
    }
}

self.addEventListener("install", installEvent => {
    console.log('Install-event');
    installEvent.waitUntil(
        caches.open(trainerCache).then(cache => {
            cache.addAll(assets);
        })
    )
})
