const CACHE_VERSION = 'mars-landing-v2';
const PRECACHE = `${CACHE_VERSION}-precache`;
const RUNTIME = `${CACHE_VERSION}-runtime`;
const OFFLINE_URL = '/';

const PRECACHE_URLS = [
  '/',
  '/docs/',
  '/downloads/',
  '/blog/',
  '/manifest.json',
  '/icons/mars-logo-file.svg',
  '/icons/mars-logo-file.png',
  '/icons/mars-logo.png',
  '/icons/mars-orbit.svg',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .open(PRECACHE)
      .then((cache) => cache.addAll(PRECACHE_URLS))
      .catch(() => undefined),
  );
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => key !== PRECACHE && key !== RUNTIME)
            .map((key) => caches.delete(key)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

const isCacheable = (request, response) => {
  if (request.method !== 'GET') return false;
  if (new URL(request.url).origin !== self.location.origin) return false;
  return response && response.ok && (response.type === 'basic' || response.type === 'cors');
};

const staleWhileRevalidate = async (request) => {
  const cache = await caches.open(RUNTIME);
  const cached = await cache.match(request);
  const network = fetch(request)
    .then((response) => {
      if (isCacheable(request, response)) {
        cache.put(request, response.clone());
      }
      return response;
    })
    .catch(() => undefined);

  return cached || network || Response.error();
};

const networkFirst = async (request) => {
  const cache = await caches.open(RUNTIME);
  try {
    const response = await fetch(request);
    if (isCacheable(request, response)) {
      cache.put(request, response.clone());
    }
    return response;
  } catch {
    const cached = await cache.match(request);
    return cached || (request.mode === 'navigate' ? caches.match(OFFLINE_URL) : Response.error());
  }
};

self.addEventListener('fetch', (event) => {
  const { request } = event;
  if (request.method !== 'GET') return;

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;

  if (request.mode === 'navigate') {
    event.respondWith(networkFirst(request));
    return;
  }

  if (['style', 'script', 'image', 'font'].includes(request.destination)) {
    event.respondWith(staleWhileRevalidate(request));
    return;
  }

  event.respondWith(staleWhileRevalidate(request));
});
