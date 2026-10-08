// Runs in every page: report each API call the top-level page makes (method, URL without
// query values, status). Bodies are read only for the calls named in CAPTURE, along with the
// request headers the page set, which the Rust client replays.
(() => {
  if (window.top !== window || window.__maxrsRecord) return;
  window.__maxrsRecord = true;
  const CAPTURE = { '/cms/routes/home': 'routes-home', '/v2_token': 'v2-token' };
  const captureName = (rawUrl) => {
    try { return CAPTURE[new URL(rawUrl, location.href).pathname]; } catch (_) { return undefined; }
  };
  const capture = (name, rawUrl, status, headers, body) => {
    const u = new URL(rawUrl, location.href);
    window.ipc.postMessage(JSON.stringify({
      capture: name, url: u.origin + u.pathname, href: u.href,
      query: Object.fromEntries(u.searchParams), status, requestHeaders: headers, body,
    }));
  };
  const headersOf = (h) => {
    try { return Object.fromEntries(new Headers(h || {}).entries()); } catch (_) { return {}; }
  };
  const report = (method, rawUrl, status) => {
    try {
      const u = new URL(rawUrl, location.href);
      const params = [...new Set(u.searchParams.keys())].sort();
      window.ipc.postMessage(JSON.stringify({
        api: true, method: String(method || 'GET').toUpperCase(),
        url: u.origin + u.pathname, params, status,
      }));
    } catch (_) {}
  };
  const origFetch = window.fetch;
  window.fetch = function (input, init) {
    const url = typeof input === 'string' ? input : (input && input.url) || String(input);
    const method = (init && init.method) || (input && input.method) || 'GET';
    const p = origFetch.apply(this, arguments);
    p.then((r) => report(method, url, r.status), () => report(method, url, 0));
    const name = captureName(url);
    if (name) {
      const names = headersOf((init && init.headers) || (input && input.headers));
      p.then((r) => r.clone().text().then((body) => capture(name, url, r.status, names, body)), () => {});
    }
    return p;
  };
  const origOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function (method, url) {
    this.__maxrsHeaders = {};
    this.addEventListener('loadend', () => {
      report(method, url, this.status);
      const name = captureName(url);
      if (name && (this.responseType === '' || this.responseType === 'text')) {
        capture(name, url, this.status, this.__maxrsHeaders, this.responseText);
      }
    });
    return origOpen.apply(this, arguments);
  };
  const origSetHeader = XMLHttpRequest.prototype.setRequestHeader;
  XMLHttpRequest.prototype.setRequestHeader = function (name, value) {
    if (this.__maxrsHeaders) this.__maxrsHeaders[String(name).toLowerCase()] = String(value);
    return origSetHeader.apply(this, arguments);
  };
})();
