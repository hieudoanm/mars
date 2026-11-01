const CACHE_NAME = 'mars-v1';
const ASSETS = [
  '/',
  '/manifest.json',
  '/branding/svg/mars-logo.svg',
  '/branding/png/mars-logo.png',
  '/branding/svg/mars-file-logo.svg',
  '/branding/png/mars-file-logo.png',
  '/branding/svg/mars-og.svg',
  '/branding/png/mars-og.png',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(ASSETS)),
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys
          .filter((key) => key !== CACHE_NAME)
          .map((key) => caches.delete(key)),
      ),
    ),
  );
});

self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.match(event.request).then((response) => response || fetch(event.request)),
  );
});
