// Runs in the sign-in web view. The sign-in pages live on other hosts; once the browser is on
// the web player's own host it has signed in. From there on nothing of the web player is
// shown, and the headers it sends to the service are reported so the app can take over.
(() => {
  if (window.top !== window || window.__maxSignIn) return;
  window.__maxSignIn = true;
  if (location.hostname !== 'play.hbomax.com') return;

  const hidden = document.createElement('style');
  hidden.textContent = 'html { visibility: hidden !important; background: #000 !important; }';
  window.__maxShowPage = () => hidden.remove();
  const hide = () => (document.head || document.documentElement).appendChild(hidden);
  if (document.documentElement) hide();
  else document.addEventListener('readystatechange', hide, { once: true });

  const report = (rawUrl, headers) => {
    try {
      const url = new URL(rawUrl, location.href);
      if (!url.hostname.endsWith('.api.hbomax.com') || !headers['x-disco-client']) return;
      window.ipc.postMessage(JSON.stringify({ api_origin: url.origin, headers }));
    } catch (_) {}
  };
  const headersOf = (headers) => {
    try { return Object.fromEntries(new Headers(headers || {}).entries()); } catch (_) { return {}; }
  };
  const originalFetch = window.fetch;
  window.fetch = function (input, init) {
    const url = typeof input === 'string' ? input : (input && input.url) || String(input);
    report(url, headersOf((init && init.headers) || (input && input.headers)));
    return originalFetch.apply(this, arguments);
  };
  const originalOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function (method, url) {
    this.__maxRequest = { url, headers: {} };
    return originalOpen.apply(this, arguments);
  };
  const originalSetHeader = XMLHttpRequest.prototype.setRequestHeader;
  XMLHttpRequest.prototype.setRequestHeader = function (name, value) {
    if (this.__maxRequest) this.__maxRequest.headers[String(name).toLowerCase()] = String(value);
    return originalSetHeader.apply(this, arguments);
  };
  const originalSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.send = function () {
    if (this.__maxRequest) report(this.__maxRequest.url, this.__maxRequest.headers);
    return originalSend.apply(this, arguments);
  };
})();
